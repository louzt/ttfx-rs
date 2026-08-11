// Bubbles effect — faithful TTE reimplementation
// Characters grouped into bubbles, float upward, pop, and settle

pub const NAME: &str = "bubbles";
pub const DESCRIPTION: &str = "Characters are formed into bubbles that float down and pop.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

const BUBBLE_COLORS: [Rgb; 4] = [
    Rgb {
        r: 0xd3,
        g: 0x3a,
        b: 0xff,
    },
    Rgb {
        r: 0x73,
        g: 0x95,
        b: 0xc4,
    },
    Rgb {
        r: 0x43,
        g: 0xc2,
        b: 0xa7,
    },
    Rgb {
        r: 0x02,
        g: 0xff,
        b: 0x7f,
    },
];

const POP_COLOR: Rgb = Rgb {
    r: 0xff,
    g: 0xff,
    b: 0xff,
};

#[derive(Clone, Copy, PartialEq)]
enum BubbleCharPhase {
    Floating,
    PopOut,
    Settling,
    Done,
}

struct BubbleChar {
    final_y: usize,
    final_x: usize,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    final_color: Rgb,
    bubble_color: Rgb,
    angle: f64,
    phase: BubbleCharPhase,
    pop_frame: usize,
    pop_out_start_y: f64,
    pop_out_start_x: f64,
    pop_out_end_y: f64,
    pop_out_end_x: f64,
    pop_out_progress: f64,
    pop_out_speed: f64,
    pop_out_done: bool,
    settle_start_y: f64,
    settle_start_x: f64,
    settle_progress: f64,
    settle_speed: f64,
}

struct Bubble {
    char_indices: Vec<usize>,
    radius: f64,
    lowest_row: f64,
    anchor_start_y: f64,
    anchor_start_x: f64,
    anchor_end_y: f64,
    anchor_end_x: f64,
    anchor_progress: f64,
    anchor_speed: f64,
    cur_anchor_y: f64,
    cur_anchor_x: f64,
    active: bool,
    landed: bool,
}

pub struct BubblesEffect {
    chars: Vec<BubbleChar>,
    bubbles: std::collections::VecDeque<Bubble>,
    active_bubbles: Vec<Bubble>,
    bubble_delay: usize,
    steps_since_last_bubble: usize,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn circle_point(anchor_x: f64, anchor_y: f64, radius: f64, angle: f64) -> (f64, f64) {
    let x_off = radius * angle.cos();
    let x = anchor_x + 2.0 * x_off;
    let y = anchor_y + radius * angle.sin();
    (y, x)
}

impl BubblesEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;

        let final_gradient = Gradient::new(&[Rgb::from_hex("d33aff"), Rgb::from_hex("02ff7f")], 12);

        let mut rng = rand::thread_rng();

        let original_chars: Vec<Vec<char>> = grid
            .cells
            .iter()
            .map(|row| row.iter().map(|c| c.ch).collect())
            .collect();

        let mut chars: Vec<BubbleChar> = Vec::new();
        let mut unbubbled: Vec<usize> = Vec::new();

        for y in (0..height).rev() {
            for x in 0..width {
                let original_ch = grid.cells[y][x].ch;
                if original_ch == ' ' {
                    continue;
                }
                let final_color =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Diagonal);
                let idx = chars.len();
                chars.push(BubbleChar {
                    final_y: y,
                    final_x: x,
                    cur_y: 0.0,
                    cur_x: 0.0,
                    original_ch,
                    final_color,
                    bubble_color: BUBBLE_COLORS[rng.gen_range(0..BUBBLE_COLORS.len())],
                    angle: 0.0,
                    phase: BubbleCharPhase::Floating,
                    pop_frame: 0,
                    pop_out_start_y: 0.0,
                    pop_out_start_x: 0.0,
                    pop_out_end_y: 0.0,
                    pop_out_end_x: 0.0,
                    pop_out_progress: 0.0,
                    pop_out_speed: 0.0,
                    pop_out_done: false,
                    settle_start_y: 0.0,
                    settle_start_x: 0.0,
                    settle_progress: 0.0,
                    settle_speed: 0.0,
                });
                unbubbled.push(idx);
            }
        }

        let mut bubbles: std::collections::VecDeque<Bubble> = std::collections::VecDeque::new();
        let mut pos = 0;
        while pos < unbubbled.len() {
            let remaining = unbubbled.len() - pos;
            let size = if remaining < 5 {
                remaining
            } else {
                rng.gen_range(5..=remaining.min(20))
            };
            let char_indices: Vec<usize> = unbubbled[pos..pos + size].to_vec();
            pos += size;

            let radius = ((size / 5).max(1)) as f64;

            let lowest_row = char_indices
                .iter()
                .map(|&i| chars[i].final_y as f64)
                .fold(0.0_f64, f64::max);

            let anchor_start_x = rng.gen_range(0..width.max(1)) as f64;
            let anchor_start_y = -10.0;
            let anchor_end_x = rng.gen_range(0..width.max(1)) as f64;
            let anchor_end_y = lowest_row;

            let dy = anchor_end_y - anchor_start_y;
            let dx = anchor_end_x - anchor_start_x;
            let dist = ((2.0 * dy).powi(2) + dx.powi(2)).sqrt().max(1.0);
            let anchor_speed = 0.5 / dist;

            let n = char_indices.len() as f64;
            for (i, &ci) in char_indices.iter().enumerate() {
                let angle = (i as f64) * std::f64::consts::TAU / n;
                chars[ci].angle = angle;
                let (cy, cx) = circle_point(anchor_start_x, anchor_start_y, radius, angle);
                chars[ci].cur_y = cy;
                chars[ci].cur_x = cx;
            }

            bubbles.push_back(Bubble {
                char_indices,
                radius,
                lowest_row,
                anchor_start_y,
                anchor_start_x,
                anchor_end_y,
                anchor_end_x,
                anchor_progress: 0.0,
                anchor_speed,
                cur_anchor_y: anchor_start_y,
                cur_anchor_x: anchor_start_x,
                active: false,
                landed: false,
            });
        }

        BubblesEffect {
            chars,
            bubbles,
            active_bubbles: Vec::new(),
            bubble_delay: 20,
            steps_since_last_bubble: 0,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        if !self.bubbles.is_empty() && self.steps_since_last_bubble >= self.bubble_delay {
            if let Some(mut next) = self.bubbles.pop_front() {
                next.active = true;
                self.active_bubbles.push(next);
                self.steps_since_last_bubble = 0;
            }
        }
        self.steps_since_last_bubble += 1;

        let mut landed_indices: Vec<usize> = Vec::new();
        for (bi, bubble) in self.active_bubbles.iter_mut().enumerate() {
            if bubble.landed {
                continue;
            }
            bubble.anchor_progress += bubble.anchor_speed;
            if bubble.anchor_progress > 1.0 {
                bubble.anchor_progress = 1.0;
            }
            bubble.cur_anchor_y = bubble.anchor_start_y
                + (bubble.anchor_end_y - bubble.anchor_start_y) * bubble.anchor_progress;
            bubble.cur_anchor_x = bubble.anchor_start_x
                + (bubble.anchor_end_x - bubble.anchor_start_x) * bubble.anchor_progress;

            let mut hit = false;
            for &ci in &bubble.char_indices {
                let (cy, cx) = circle_point(
                    bubble.cur_anchor_x,
                    bubble.cur_anchor_y,
                    bubble.radius,
                    self.chars[ci].angle,
                );
                self.chars[ci].cur_y = cy;
                self.chars[ci].cur_x = cx;
                if (cy.round() as i64) >= bubble.lowest_row.round() as i64 {
                    hit = true;
                }
            }

            if hit || bubble.anchor_progress >= 1.0 {
                bubble.landed = true;
                landed_indices.push(bi);
            }
        }

        for &bi in &landed_indices {
            let bubble = &self.active_bubbles[bi];
            for &ci in &bubble.char_indices {
                let ch = &mut self.chars[ci];
                ch.phase = BubbleCharPhase::PopOut;
                ch.pop_frame = 0;
                ch.pop_out_start_y = ch.cur_y;
                ch.pop_out_start_x = ch.cur_x;
                let (ey, ex) = circle_point(
                    bubble.cur_anchor_x,
                    bubble.cur_anchor_y,
                    bubble.radius + 3.0,
                    ch.angle,
                );
                ch.pop_out_end_y = ey;
                ch.pop_out_end_x = ex;
                let dy = ey - ch.pop_out_start_y;
                let dx = ex - ch.pop_out_start_x;
                let dist = ((2.0 * dy).powi(2) + dx.powi(2)).sqrt().max(1.0);
                ch.pop_out_speed = 0.3 / dist;
                ch.pop_out_progress = 0.0;
                ch.pop_out_done = false;
            }
        }
        self.active_bubbles.retain(|b| !b.landed);

        for ch in &mut self.chars {
            match ch.phase {
                BubbleCharPhase::Floating => {}
                BubbleCharPhase::PopOut => {
                    ch.pop_frame += 1;
                    if !ch.pop_out_done {
                        ch.pop_out_progress += ch.pop_out_speed;
                        if ch.pop_out_progress >= 1.0 {
                            ch.pop_out_progress = 1.0;
                            ch.pop_out_done = true;
                        }
                        let eased = easing::out_expo(ch.pop_out_progress);
                        ch.cur_y =
                            ch.pop_out_start_y + (ch.pop_out_end_y - ch.pop_out_start_y) * eased;
                        ch.cur_x =
                            ch.pop_out_start_x + (ch.pop_out_end_x - ch.pop_out_start_x) * eased;
                    }
                    if ch.pop_frame >= 18 && ch.pop_out_done {
                        ch.phase = BubbleCharPhase::Settling;
                        ch.settle_start_y = ch.cur_y;
                        ch.settle_start_x = ch.cur_x;
                        let dy = ch.final_y as f64 - ch.settle_start_y;
                        let dx = ch.final_x as f64 - ch.settle_start_x;
                        let dist = ((2.0 * dy).powi(2) + dx.powi(2)).sqrt().max(1.0);
                        ch.settle_speed = 0.3 / dist;
                        ch.settle_progress = 0.0;
                    }
                }
                BubbleCharPhase::Settling => {
                    ch.settle_progress += ch.settle_speed;
                    if ch.settle_progress >= 1.0 {
                        ch.settle_progress = 1.0;
                        ch.phase = BubbleCharPhase::Done;
                    }
                    let eased = easing::in_out_expo(ch.settle_progress);
                    ch.cur_y = ch.settle_start_y + (ch.final_y as f64 - ch.settle_start_y) * eased;
                    ch.cur_x = ch.settle_start_x + (ch.final_x as f64 - ch.settle_start_x) * eased;
                }
                BubbleCharPhase::Done => {}
            }
        }

        for (y, row) in grid.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                cell.visible = false;
                cell.ch = self.original_chars[y][x];
                cell.fg = None;
            }
        }

        for bubble in &self.active_bubbles {
            for &ci in &bubble.char_indices {
                let ch = &self.chars[ci];
                let ry = ch.cur_y.round() as isize;
                let rx = ch.cur_x.round() as isize;
                if ry < 0 || rx < 0 {
                    continue;
                }
                let (ry, rx) = (ry as usize, rx as usize);
                if ry >= self.height || rx >= self.width {
                    continue;
                }
                let cell = &mut grid.cells[ry][rx];
                cell.visible = true;
                cell.ch = ch.original_ch;
                cell.fg = Some(ch.bubble_color.to_crossterm());
            }
        }

        for ch in &self.chars {
            if ch.phase == BubbleCharPhase::Floating {
                continue;
            }
            let ry = ch.cur_y.round() as isize;
            let rx = ch.cur_x.round() as isize;
            if ry < 0 || rx < 0 {
                continue;
            }
            let (ry, rx) = (ry as usize, rx as usize);
            if ry >= self.height || rx >= self.width {
                continue;
            }
            let cell = &mut grid.cells[ry][rx];
            cell.visible = true;
            match ch.phase {
                BubbleCharPhase::PopOut => {
                    if ch.pop_frame < 9 {
                        cell.ch = '*';
                        cell.fg = Some(POP_COLOR.to_crossterm());
                    } else if ch.pop_frame < 18 {
                        cell.ch = '\'';
                        cell.fg = Some(POP_COLOR.to_crossterm());
                    } else {
                        cell.ch = ch.original_ch;
                        cell.fg = Some(POP_COLOR.to_crossterm());
                    }
                }
                BubbleCharPhase::Settling => {
                    cell.ch = ch.original_ch;
                    let t = ch.settle_progress;
                    cell.fg = Some(Rgb::lerp(POP_COLOR, ch.final_color, t).to_crossterm());
                }
                BubbleCharPhase::Done => {
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.final_color.to_crossterm());
                }
                _ => {}
            }
        }

        let all_done = self.bubbles.is_empty()
            && self.active_bubbles.is_empty()
            && self.chars.iter().all(|c| c.phase == BubbleCharPhase::Done);

        if all_done {
            for ch in &self.chars {
                if ch.final_y < self.height && ch.final_x < self.width {
                    let cell = &mut grid.cells[ch.final_y][ch.final_x];
                    cell.visible = true;
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.final_color.to_crossterm());
                }
            }
        }
        all_done
    }
}

#[cfg(test)]
#[path = "../tests/effects/bubbles.rs"]
mod tests;
