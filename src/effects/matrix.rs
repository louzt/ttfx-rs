// Matrix effect — digital rain columns that resolve to text

pub const NAME: &str = "matrix";
pub const DESCRIPTION: &str = "Matrix digital rain effect.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Rain,
    Fill,
    Resolve,
    Done,
}

struct RainColumn {
    x: usize,
    head: isize,
    length: usize,
    speed_counter: usize,
    speed: usize,
    active: bool,
    full: bool,
    hold: usize,
}

struct MatrixChar {
    final_ch: char,
    resolved: bool,
    resolve_step: usize,
}

const RESOLVE_TOTAL: usize = 24;

pub struct MatrixEffect {
    columns: Vec<RainColumn>,
    chars: Vec<Vec<MatrixChar>>,
    phase: Phase,
    frame: usize,
    width: usize,
    height: usize,
    rain_time: usize,
    column_delay: usize,
    resolve_delay_counter: usize,
    pending_resolve: Vec<(usize, usize)>,
    rain_gradient: Vec<Rgb>,
    highlight_color: Rgb,
    final_gradient: Gradient,
    final_gradient_text_top: usize,
    final_gradient_text_left: usize,
    final_gradient_text_h: usize,
    final_gradient_text_w: usize,
    rain_symbols: Vec<char>,
}

impl MatrixEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height) = (grid.width, grid.height);
        let final_gradient = Gradient::new(&[Rgb::from_hex("92be92"), Rgb::from_hex("336b33")], 12);
        let highlight_color = Rgb::from_hex("dbffdb");

        let rain_grad: Vec<Rgb> = (0..12)
            .map(|i| {
                let t = i as f64 / 11.0;
                Rgb::lerp(Rgb::from_hex("92be92"), Rgb::from_hex("185318"), t)
            })
            .collect();

        let rain_symbols: Vec<char> = "ﾊﾐﾋｰｳｼﾅﾓﾆｻﾜﾂｵﾘｱﾎﾃﾏｹﾒｴｶｷﾑﾕﾗｾﾈｽﾀﾇﾍ012345789:.<>*+=-"
            .chars()
            .collect();

        let mut text_top = usize::MAX;
        let mut text_bottom = 0usize;
        let mut text_left = usize::MAX;
        let mut text_right = 0usize;
        for y in 0..height {
            for x in 0..width {
                if grid.cells[y][x].ch != ' ' {
                    text_top = text_top.min(y);
                    text_bottom = text_bottom.max(y);
                    text_left = text_left.min(x);
                    text_right = text_right.max(x);
                }
            }
        }
        if text_top == usize::MAX {
            text_top = 0;
            text_bottom = height.saturating_sub(1);
            text_left = 0;
            text_right = width.saturating_sub(1);
        }
        let final_gradient_text_h = text_bottom.saturating_sub(text_top).max(1);
        let final_gradient_text_w = text_right.saturating_sub(text_left).max(1);

        let mut chars = Vec::new();
        for y in 0..height {
            let mut row = Vec::new();
            for x in 0..width {
                row.push(MatrixChar {
                    final_ch: grid.cells[y][x].ch,
                    resolved: false,
                    resolve_step: 0,
                });
            }
            chars.push(row);
        }

        let mut rng = rand::thread_rng();
        let mut columns: Vec<RainColumn> = (0..width)
            .map(|x| RainColumn {
                x,
                head: -(rng.gen_range(0..height.max(1)) as isize),
                length: rng.gen_range(1..height.max(2)),
                speed: rng.gen_range(2..=15),
                speed_counter: 0,
                active: false,
                full: false,
                hold: 0,
            })
            .collect();

        let mut order: Vec<usize> = (0..width).collect();
        order.shuffle(&mut rng);
        for (i, &idx) in order.iter().enumerate() {
            columns[idx].speed_counter = i * rng.gen_range(3..=9);
        }

        let mut pending: Vec<(usize, usize)> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                pending.push((y, x));
            }
        }
        pending.shuffle(&mut rng);

        MatrixEffect {
            columns,
            chars,
            phase: Phase::Rain,
            frame: 0,
            width,
            height,
            rain_time: 15 * 60,
            column_delay: 0,
            resolve_delay_counter: 0,
            pending_resolve: pending,
            rain_gradient: rain_grad,
            highlight_color,
            final_gradient,
            final_gradient_text_top: text_top,
            final_gradient_text_left: text_left,
            final_gradient_text_h,
            final_gradient_text_w,
            rain_symbols,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        self.frame += 1;
        let mut rng = rand::thread_rng();

        for col in &mut self.columns {
            if !col.active {
                if col.speed_counter == 0 {
                    col.active = true;
                } else {
                    col.speed_counter -= 1;
                }
            }
        }

        match self.phase {
            Phase::Rain | Phase::Fill => {
                let fill = self.phase == Phase::Fill;
                let target_length = self.height + 5;
                for col in &mut self.columns {
                    if !col.active {
                        continue;
                    }
                    if col.full {
                        if col.hold > 0 {
                            col.hold -= 1;
                        }
                        continue;
                    }
                    if fill && col.length < target_length {
                        col.length += 1;
                    }
                    col.speed_counter += 1;
                    let spd = if fill {
                        (col.speed / 3).max(1)
                    } else {
                        col.speed
                    };
                    if col.speed_counter >= spd {
                        col.speed_counter = 0;
                        col.head += 1;
                        if col.head >= self.height as isize {
                            if fill {
                                if col.length >= target_length {
                                    col.full = true;
                                    col.hold = rng.gen_range(20..=45);
                                } else {
                                    // Hold head at the bottom while length
                                    // catches up — column is "filling in" from
                                    // above as the trail extends.
                                    col.head = self.height as isize - 1;
                                }
                            } else {
                                col.head = -(rng.gen_range(0..self.height / 2 + 1) as isize);
                                let lo = self.height / 10 + 1;
                                let hi = self.height.max(lo + 1);
                                col.length = rng.gen_range(lo..hi);
                            }
                        }
                    }
                }
                if self.phase == Phase::Rain && self.frame >= self.rain_time {
                    self.phase = Phase::Fill;
                    // Don't disturb existing head/length — each column simply
                    // starts growing its trail and accelerating its fall from
                    // wherever it was when rain ended. This matches TTE's
                    // smooth handover (active rain columns continue their
                    // motion; fill happens by extension, not relocation).
                    for col in &mut self.columns {
                        col.full = false;
                        col.hold = 0;
                        col.speed_counter = 0;
                        col.active = true;
                    }
                }
                if fill && self.columns.iter().all(|c| c.full && c.hold == 0) {
                    self.phase = Phase::Resolve;
                }
            }
            Phase::Resolve => {
                // TTE resolves 1-4 chars per column per `resolve_delay` (=3
                // frames). For a width-W canvas that averages ~W*0.83/frame.
                let resolve_per_frame = ((self.width as f64 * 0.8) as usize).max(1);
                for _ in 0..resolve_per_frame {
                    if let Some((y, x)) = self.pending_resolve.pop() {
                        self.chars[y][x].resolved = true;
                    }
                }
                for row in &mut self.chars {
                    for ch in row {
                        if ch.resolved && ch.resolve_step < RESOLVE_TOTAL {
                            ch.resolve_step += 1;
                        }
                    }
                }
                if self.pending_resolve.is_empty()
                    && self
                        .chars
                        .iter()
                        .all(|row| row.iter().all(|c| c.resolve_step >= RESOLVE_TOTAL))
                {
                    self.phase = Phase::Done;
                }
            }
            Phase::Done => return true,
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &mut grid.cells[y][x];
                if self.phase == Phase::Resolve || self.phase == Phase::Done {
                    let mc = &self.chars[y][x];
                    if mc.resolved {
                        cell.visible = true;
                        cell.ch = mc.final_ch;
                        let t = mc.resolve_step as f64 / RESOLVE_TOTAL as f64;
                        let ry = y.saturating_sub(self.final_gradient_text_top);
                        let rx = x.saturating_sub(self.final_gradient_text_left);
                        let fc = self.final_gradient.color_at_coord(
                            ry,
                            rx,
                            self.final_gradient_text_h,
                            self.final_gradient_text_w,
                            GradientDirection::Radial,
                        );
                        cell.fg = Some(Rgb::lerp(self.highlight_color, fc, t).to_crossterm());
                    } else {
                        // Keep the stable rain symbol from the rain/fill phase;
                        // only swap occasionally (matches TTE's symbol_swap_chance).
                        cell.visible = true;
                        if cell.ch == ' ' || rng.r#gen::<f64>() < 0.005 {
                            cell.ch = *self.rain_symbols.choose(&mut rng).unwrap_or(&'0');
                        }
                        let depth = (y as f64 / self.height.max(1) as f64 * 11.0) as usize;
                        cell.fg = Some(self.rain_gradient[depth.min(11)].to_crossterm());
                    }
                } else {
                    cell.visible = false;
                    let col = &self.columns[x];
                    if !col.active {
                        continue;
                    }
                    let tail = col.head - col.length as isize;
                    if (y as isize) <= col.head && (y as isize) > tail {
                        cell.visible = true;
                        if y as isize == col.head {
                            cell.ch = *self.rain_symbols.choose(&mut rng).unwrap_or(&'0');
                            cell.fg = Some(self.highlight_color.to_crossterm());
                        } else {
                            if rng.r#gen::<f64>() < 0.005 {
                                cell.ch = *self.rain_symbols.choose(&mut rng).unwrap_or(&'0');
                            }
                            let dist_from_head = (col.head - y as isize) as f64 / col.length as f64;
                            let idx = (dist_from_head * 11.0) as usize;
                            cell.fg = Some(self.rain_gradient[idx.min(11)].to_crossterm());
                        }
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
#[path = "../tests/effects/matrix.rs"]
mod tests;
