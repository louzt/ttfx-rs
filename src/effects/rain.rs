// Rain effect — chars fall from the canvas top, row-by-row from the visual
// bottom upward. Each row's chars are popped randomly 1–2 per frame and fall
// to their input position with an in_quart easing. After landing, each char
// fades from its raindrop color to its position-based final color.

pub const NAME: &str = "rain";
pub const DESCRIPTION: &str = "Rain characters from the top of the canvas.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

const RAIN_SYMBOLS: [char; 5] = ['o', '.', ',', '*', '|'];

const RAIN_COLORS: [&str; 8] = [
    "00315C", "004C8F", "0075DB", "3F91D9", "78B9F2", "9AC8F5", "B8D8F8", "E3EFFC",
];

struct RainChar {
    final_y: usize,
    final_x: usize,
    start_y: f64,
    cur_y: f64,
    original_ch: char,
    final_color: Rgb,
    rain_color: Rgb,
    rain_symbol: char,
    progress: f64,
    speed: f64,
    active: bool,
    landed: bool,
    fade_frame: usize,
    fade_total: usize,
    fade_done: bool,
}

pub struct RainEffect {
    chars: Vec<RainChar>,
    groups: std::collections::VecDeque<Vec<usize>>, // bottom-row first
    pending: Vec<usize>,                            // remaining indices in the current group
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn aspect_dist(dy: f64) -> f64 {
    (2.0 * dy).abs().max(1.0)
}

impl RainEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("488bff"),
                Rgb::from_hex("b2e7de"),
                Rgb::from_hex("57eaf7"),
            ],
            12,
        );
        let rain_palette: Vec<Rgb> = RAIN_COLORS.iter().map(|h| Rgb::from_hex(h)).collect();

        let original_chars: Vec<Vec<char>> = grid
            .cells
            .iter()
            .map(|row| row.iter().map(|c| c.ch).collect())
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
        let text_h = text_bottom.saturating_sub(text_top).max(1);
        let text_w = text_right.saturating_sub(text_left).max(1);

        let mut rng = rand::thread_rng();
        let mut chars: Vec<RainChar> = Vec::new();
        let mut by_row: Vec<Vec<usize>> = vec![Vec::new(); height];
        for (y, row_bucket) in by_row.iter_mut().enumerate() {
            for x in 0..width {
                let ch = grid.cells[y][x].ch;
                if ch == ' ' {
                    continue;
                }
                let ry = y.saturating_sub(text_top);
                let rx = x.saturating_sub(text_left);
                let final_color = final_gradient.color_at_coord(
                    ry,
                    rx,
                    text_h,
                    text_w,
                    GradientDirection::Diagonal,
                );
                let rain_color = rain_palette[rng.gen_range(0..rain_palette.len())];
                let rain_symbol = RAIN_SYMBOLS[rng.gen_range(0..RAIN_SYMBOLS.len())];
                let speed_val: f64 = rng.gen_range(0.33..=0.57);
                // TTE starts each char at canvas.top (visual top edge of
                // canvas). In rtte top-down that's row 0.
                let start_y = 0.0;
                let dy = y as f64 - start_y;
                let speed = speed_val / aspect_dist(dy);

                let idx = chars.len();
                row_bucket.push(idx);
                chars.push(RainChar {
                    final_y: y,
                    final_x: x,
                    start_y,
                    cur_y: start_y,
                    original_ch: ch,
                    final_color,
                    rain_color,
                    rain_symbol,
                    progress: 0.0,
                    speed,
                    active: false,
                    landed: false,
                    fade_frame: 0,
                    // TTE fade scene = Gradient(raindrop, final, steps=7) over
                    // 8 colors × 3 frames each = 24 frames.
                    fade_total: 8 * 3,
                    fade_done: false,
                });
            }
        }

        // ROW order: visual bottom first (matches TTE's `min(group_by_row.keys())`
        // = canvas.bottom in bottom-up coords).
        let mut groups: std::collections::VecDeque<Vec<usize>> = std::collections::VecDeque::new();
        for y in (0..height).rev() {
            if !by_row[y].is_empty() {
                groups.push_back(std::mem::take(&mut by_row[y]));
            }
        }

        let pending = groups.pop_front().unwrap_or_default();

        RainEffect {
            chars,
            groups,
            pending,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let mut rng = rand::thread_rng();

        // Refill pending from the next group when current is empty.
        if self.pending.is_empty() {
            if let Some(next) = self.groups.pop_front() {
                self.pending = next;
            }
        }

        if !self.pending.is_empty() {
            let n = rng.gen_range(1..=2);
            for _ in 0..n {
                if self.pending.is_empty() {
                    break;
                }
                let i = rng.gen_range(0..self.pending.len());
                let idx = self.pending.swap_remove(i);
                self.chars[idx].active = true;
            }
        }

        for ch in &mut self.chars {
            if !ch.active {
                continue;
            }
            if !ch.landed {
                ch.progress = (ch.progress + ch.speed).min(1.0);
                if ch.progress >= 1.0 {
                    ch.landed = true;
                }
                let eased = easing::in_quart(ch.progress);
                ch.cur_y = ch.start_y + (ch.final_y as f64 - ch.start_y) * eased;
            } else if !ch.fade_done {
                ch.fade_frame += 1;
                if ch.fade_frame >= ch.fade_total {
                    ch.fade_done = true;
                }
            }
        }

        for (y, row) in grid.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                cell.visible = false;
                cell.ch = self.original_chars[y][x];
                cell.fg = None;
            }
        }

        for ch in &self.chars {
            if !ch.active {
                continue;
            }
            if ch.landed {
                if ch.final_y < self.height && ch.final_x < self.width {
                    let cell = &mut grid.cells[ch.final_y][ch.final_x];
                    cell.visible = true;
                    cell.ch = ch.original_ch;
                    let t = if ch.fade_done {
                        1.0
                    } else {
                        ch.fade_frame as f64 / ch.fade_total as f64
                    };
                    cell.fg = Some(Rgb::lerp(ch.rain_color, ch.final_color, t).to_crossterm());
                }
            } else {
                let ry = ch.cur_y.round() as isize;
                if ry >= 0 && (ry as usize) < self.height && ch.final_x < self.width {
                    let cell = &mut grid.cells[ry as usize][ch.final_x];
                    cell.visible = true;
                    cell.ch = ch.rain_symbol;
                    cell.fg = Some(ch.rain_color.to_crossterm());
                }
            }
        }

        let activations_done = self.pending.is_empty() && self.groups.is_empty();
        let all_settled = self.chars.iter().all(|c| !c.active || c.fade_done);
        activations_done && all_settled
    }
}

#[cfg(test)]
#[path = "../tests/effects/rain.rs"]
mod tests;
