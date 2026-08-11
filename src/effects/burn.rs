// Burn effect — faithful TTE reimplementation
// Fire spreads through text with burn symbols, smoke particles

pub const NAME: &str = "burn";
pub const DESCRIPTION: &str = "Burns vertically in the canvas.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;
use std::collections::VecDeque;

const BURN_CHARS: [char; 9] = ['\'', '.', '▖', '▙', '█', '▜', '▀', '▝', '.'];
const SMOKE_SYMBOLS: [char; 6] = ['.', ',', '\'', '`', '#', '*'];

#[derive(Clone, Copy, PartialEq, Debug)]
enum BurnPhase {
    Waiting,
    Burning,
    Final,
    Done,
}

struct BurnChar {
    y: usize,
    x: usize,
    original_ch: char,
    final_color: Rgb,
    phase: BurnPhase,
    frame: usize,
    burn_total: usize,
    final_total: usize,
}

struct SmokeParticle {
    cur_y: f64,
    cur_x: f64,
    start_y: f64,
    start_x: f64,
    end_y: f64,
    end_x: f64,
    progress: f64,
    speed: f64,
    symbol: char,
    anim_frame: usize,
    anim_total: usize,
    active: bool,
}

pub struct BurnEffect {
    chars: Vec<Vec<BurnChar>>,
    char_link_order: VecDeque<(usize, usize)>,
    smoke: Vec<SmokeParticle>,
    burn_gradient: Gradient,
    smoke_gradient: Gradient,
    starting_color: Rgb,
    smoke_chance: f64,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn unlinked_neighbors(
    y: usize,
    x: usize,
    linked: &[Vec<bool>],
    is_text: &[Vec<bool>],
    height: usize,
    width: usize,
) -> Vec<(usize, usize)> {
    let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let mut out = Vec::new();
    for (dy, dx) in &dirs {
        let ny = y as isize + dy;
        let nx = x as isize + dx;
        if ny < 0 || nx < 0 {
            continue;
        }
        let (ny, nx) = (ny as usize, nx as usize);
        if ny < height && nx < width && is_text[ny][nx] && !linked[ny][nx] {
            out.push((ny, nx));
        }
    }
    out
}

impl BurnEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height) = (grid.width, grid.height);
        let final_gradient = Gradient::new(&[Rgb::from_hex("00c3ff"), Rgb::from_hex("ffff1c")], 12);
        let burn_gradient = Gradient::new(
            &[
                Rgb::from_hex("ffffff"),
                Rgb::from_hex("fff75d"),
                Rgb::from_hex("fe650d"),
                Rgb::from_hex("8A003C"),
                Rgb::from_hex("510100"),
            ],
            10,
        );
        let smoke_gradient = Gradient::new(&[Rgb::from_hex("504F4F"), Rgb::from_hex("C7C7C7")], 9);
        let starting_color = Rgb::from_hex("837373");

        let original_chars: Vec<Vec<char>> = grid
            .cells
            .iter()
            .map(|row| row.iter().map(|c| c.ch).collect())
            .collect();

        let mut text_positions: Vec<(usize, usize)> = Vec::new();
        let mut text_top = usize::MAX;
        let mut text_bottom = 0usize;
        let mut text_left = usize::MAX;
        let mut text_right = 0usize;
        for y in 0..height {
            for x in 0..width {
                if grid.cells[y][x].ch != ' ' {
                    text_positions.push((y, x));
                    text_top = text_top.min(y);
                    text_bottom = text_bottom.max(y);
                    text_left = text_left.min(x);
                    text_right = text_right.max(x);
                }
            }
        }
        let text_h = text_bottom.saturating_sub(text_top).max(1);
        let text_w = text_right.saturating_sub(text_left).max(1);

        let mut chars: Vec<Vec<BurnChar>> = Vec::with_capacity(height);
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let ch = grid.cells[y][x].ch;
                let ry = y.saturating_sub(text_top).min(text_h);
                let rx = x.saturating_sub(text_left).min(text_w);
                let fc = final_gradient.color_at_coord(
                    ry,
                    rx,
                    text_h,
                    text_w,
                    GradientDirection::Vertical,
                );
                row.push(BurnChar {
                    y,
                    x,
                    original_ch: ch,
                    final_color: fc,
                    phase: BurnPhase::Waiting,
                    frame: 0,
                    burn_total: burn_gradient.spectrum().len() * 4,
                    final_total: 9 * 4,
                });
            }
            chars.push(row);
        }

        let mut in_bounds = vec![vec![false; width]; height];
        if !text_positions.is_empty() {
            for row in in_bounds.iter_mut().take(text_bottom + 1).skip(text_top) {
                for cell in row.iter_mut().take(text_right + 1).skip(text_left) {
                    *cell = true;
                }
            }
        }

        let mut rng = rand::thread_rng();
        let mut char_link_order: VecDeque<(usize, usize)> = VecDeque::new();
        if !text_positions.is_empty() {
            let mut linked = vec![vec![false; width]; height];
            let start = text_positions[rng.gen_range(0..text_positions.len())];
            linked[start.0][start.1] = true;
            char_link_order.push_back(start);
            let mut edge_chars: Vec<(usize, usize)> = vec![start];
            while !edge_chars.is_empty() {
                let i = rng.gen_range(0..edge_chars.len());
                let current = edge_chars.swap_remove(i);
                let mut neighbors =
                    unlinked_neighbors(current.0, current.1, &linked, &in_bounds, height, width);
                if neighbors.is_empty() {
                    continue;
                }
                let j = rng.gen_range(0..neighbors.len());
                let next = neighbors.swap_remove(j);
                linked[next.0][next.1] = true;
                char_link_order.push_back(next);
                if !neighbors.is_empty() {
                    edge_chars.push(current);
                }
                if !unlinked_neighbors(next.0, next.1, &linked, &in_bounds, height, width)
                    .is_empty()
                {
                    edge_chars.push(next);
                }
            }
        }

        let mut smoke: Vec<SmokeParticle> = Vec::with_capacity(500);
        for _ in 0..500 {
            smoke.push(SmokeParticle {
                cur_y: 0.0,
                cur_x: 0.0,
                start_y: 0.0,
                start_x: 0.0,
                end_y: 0.0,
                end_x: 0.0,
                progress: 0.0,
                speed: 0.0,
                symbol: '.',
                anim_frame: 0,
                anim_total: 10 * 10,
                active: false,
            });
        }

        BurnEffect {
            chars,
            char_link_order,
            smoke,
            burn_gradient,
            smoke_gradient,
            starting_color,
            smoke_chance: 0.5,
            width,
            height,
            original_chars,
        }
    }

    fn emit_smoke(&mut self, oy: f64, ox: f64, rng: &mut impl Rng) {
        if rng.r#gen::<f64>() > self.smoke_chance {
            return;
        }
        let slot = self.smoke.iter().position(|s| !s.active);
        let idx = match slot {
            Some(i) => i,
            None => return,
        };
        let target_x = ox + rng.gen_range(-4.0..=4.0);
        let target_y = -1.0;
        let dy = target_y - oy;
        let dx = target_x - ox;
        let dist = ((2.0 * dy).powi(2) + dx.powi(2)).sqrt().max(1.0);
        let speed = 0.5 / dist;
        let s = &mut self.smoke[idx];
        s.cur_y = oy;
        s.cur_x = ox;
        s.start_y = oy;
        s.start_x = ox;
        s.end_y = target_y;
        s.end_x = target_x;
        s.progress = 0.0;
        s.speed = speed;
        s.symbol = SMOKE_SYMBOLS[rng.gen_range(0..SMOKE_SYMBOLS.len())];
        s.anim_frame = 0;
        s.active = true;
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let mut rng = rand::thread_rng();

        let activate_count = rng.gen_range(2..=4);
        for _ in 0..activate_count {
            if let Some((y, x)) = self.char_link_order.pop_front() {
                let ch = &mut self.chars[y][x];
                if ch.original_ch == ' ' {
                    continue;
                }
                if ch.phase == BurnPhase::Waiting {
                    ch.phase = BurnPhase::Burning;
                    ch.frame = 0;
                }
            } else {
                break;
            }
        }

        let mut smoke_emissions: Vec<(f64, f64)> = Vec::new();
        for row in &mut self.chars {
            for ch in row {
                match ch.phase {
                    BurnPhase::Waiting => {}
                    BurnPhase::Burning => {
                        ch.frame += 1;
                        if ch.frame >= ch.burn_total {
                            ch.phase = BurnPhase::Final;
                            ch.frame = 0;
                            smoke_emissions.push((ch.y as f64, ch.x as f64));
                        }
                    }
                    BurnPhase::Final => {
                        ch.frame += 1;
                        if ch.frame >= ch.final_total {
                            ch.phase = BurnPhase::Done;
                        }
                    }
                    BurnPhase::Done => {}
                }
            }
        }
        for (oy, ox) in smoke_emissions {
            self.emit_smoke(oy, ox, &mut rng);
        }

        for s in &mut self.smoke {
            if !s.active {
                continue;
            }
            s.progress = (s.progress + s.speed).min(1.0);
            s.cur_y = s.start_y + (s.end_y - s.start_y) * s.progress;
            s.cur_x = s.start_x + (s.end_x - s.start_x) * s.progress;
            s.anim_frame += 1;
            if s.anim_frame >= s.anim_total {
                s.active = false;
            }
        }

        for (y, row) in grid.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                cell.visible = false;
                cell.ch = self.original_chars[y][x];
                cell.fg = None;
            }
        }

        let burn_spec = self.burn_gradient.spectrum();
        let burn_spec_len = burn_spec.len().max(1);
        let smoke_spec = self.smoke_gradient.spectrum();
        let smoke_spec_len = smoke_spec.len().max(1);

        for row in &self.chars {
            for ch in row {
                if ch.original_ch == ' ' && ch.phase == BurnPhase::Waiting {
                    continue;
                }
                if ch.y >= grid.height || ch.x >= grid.width {
                    continue;
                }
                let cell = &mut grid.cells[ch.y][ch.x];
                cell.visible = true;
                match ch.phase {
                    BurnPhase::Waiting => {
                        cell.ch = ch.original_ch;
                        cell.fg = Some(self.starting_color.to_crossterm());
                    }
                    BurnPhase::Burning => {
                        let ci = (ch.frame / 4).min(burn_spec_len - 1);
                        let sym_idx =
                            (ci * BURN_CHARS.len() / burn_spec_len).min(BURN_CHARS.len() - 1);
                        cell.ch = BURN_CHARS[sym_idx];
                        cell.fg = Some(burn_spec[ci].to_crossterm());
                    }
                    BurnPhase::Final => {
                        cell.ch = ch.original_ch;
                        let t = ch.frame as f64 / ch.final_total as f64;
                        let fire_end = burn_spec[burn_spec_len - 1];
                        cell.fg = Some(Rgb::lerp(fire_end, ch.final_color, t).to_crossterm());
                    }
                    BurnPhase::Done => {
                        cell.ch = ch.original_ch;
                        cell.fg = Some(ch.final_color.to_crossterm());
                    }
                }
            }
        }

        for s in &self.smoke {
            if !s.active {
                continue;
            }
            let ry = s.cur_y.round() as isize;
            let rx = s.cur_x.round() as isize;
            if ry < 0 || rx < 0 {
                continue;
            }
            let (ry, rx) = (ry as usize, rx as usize);
            if ry >= self.height || rx >= self.width {
                continue;
            }
            let ci = (s.anim_frame / 10).min(smoke_spec_len - 1);
            let cell = &mut grid.cells[ry][rx];
            cell.visible = true;
            cell.ch = s.symbol;
            cell.fg = Some(smoke_spec[ci].to_crossterm());
        }

        let chars_done = self.chars.iter().all(|row| {
            row.iter()
                .all(|c| c.original_ch == ' ' || c.phase == BurnPhase::Done)
        });
        let smoke_done = self.smoke.iter().all(|s| !s.active);
        let queue_done = self.char_link_order.is_empty();
        queue_done && chars_done && smoke_done
    }
}

#[cfg(test)]
#[path = "../tests/effects/burn.rs"]
mod tests;
