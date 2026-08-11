// Pour effect — chars fall from canvas top into their final positions.
// Activation order: ROW_BOTTOM_TO_TOP, alternating left-to-right / right-to-left
// per row (so the pouring snakes back and forth as it climbs).

pub const NAME: &str = "pour";
pub const DESCRIPTION: &str = "Pours the characters into position from the given direction.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

struct PourChar {
    final_y: usize,
    final_x: usize,
    start_y: f64,
    cur_y: f64,
    original_ch: char,
    final_color: Rgb,
    progress: f64,
    speed: f64,
    active: bool,
    done: bool,
}

pub struct PourEffect {
    chars: Vec<PourChar>,
    pending: Vec<usize>,
    pour_speed: usize,
    gap: usize,
    gap_counter: usize,
    starting_color: Rgb,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn aspect_dist(dy: f64) -> f64 {
    (2.0 * dy).abs().max(1.0)
}

impl PourEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("FFFFFF"),
            ],
            12,
        );
        let starting_color = Rgb::from_hex("ffffff");

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
        let mut chars: Vec<PourChar> = Vec::new();
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
                    GradientDirection::Vertical,
                );
                let speed_val: f64 = rng.gen_range(0.4..=0.6);
                let start_y = 0.0;
                let dy = y as f64 - start_y;
                let speed = speed_val / aspect_dist(dy);
                let idx = chars.len();
                chars.push(PourChar {
                    final_y: y,
                    final_x: x,
                    start_y,
                    cur_y: start_y,
                    original_ch: ch,
                    final_color,
                    progress: 0.0,
                    speed,
                    active: false,
                    done: false,
                });
                row_bucket.push(idx);
            }
        }
        for row in &mut by_row {
            row.sort_by_key(|&i| chars[i].final_x);
        }

        // Build pending list: ROW_BOTTOM_TO_TOP, alternating direction per row.
        let mut pending: Vec<usize> = Vec::new();
        let mut group_idx = 0usize;
        for y in (0..height).rev() {
            let row = &by_row[y];
            if row.is_empty() {
                continue;
            }
            if group_idx % 2 == 0 {
                pending.extend(row.iter().copied());
            } else {
                pending.extend(row.iter().rev().copied());
            }
            group_idx += 1;
        }

        PourEffect {
            chars,
            pending,
            pour_speed: 2,
            gap: 1,
            gap_counter: 0,
            starting_color,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        if !self.pending.is_empty() {
            if self.gap_counter == 0 {
                for _ in 0..self.pour_speed {
                    if self.pending.is_empty() {
                        break;
                    }
                    let idx = self.pending.remove(0);
                    self.chars[idx].active = true;
                }
                self.gap_counter = self.gap;
            } else {
                self.gap_counter -= 1;
            }
        }

        let mut all_done = self.pending.is_empty();
        for ch in &mut self.chars {
            if !ch.active || ch.done {
                continue;
            }
            ch.progress = (ch.progress + ch.speed).min(1.0);
            if ch.progress >= 1.0 {
                ch.done = true;
            }
            let eased = easing::in_quad(ch.progress);
            ch.cur_y = ch.start_y + (ch.final_y as f64 - ch.start_y) * eased;
            if !ch.done {
                all_done = false;
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
            let ry = ch.cur_y.round() as isize;
            if ry >= 0 && (ry as usize) < self.height && ch.final_x < self.width {
                let cell = &mut grid.cells[ry as usize][ch.final_x];
                cell.visible = true;
                cell.ch = ch.original_ch;
                cell.fg = Some(
                    Rgb::lerp(self.starting_color, ch.final_color, ch.progress).to_crossterm(),
                );
            }
        }

        if all_done {
            for ch in &self.chars {
                if ch.final_y < grid.height && ch.final_x < grid.width {
                    let cell = &mut grid.cells[ch.final_y][ch.final_x];
                    cell.visible = true;
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.final_color.to_crossterm());
                }
            }
            return true;
        }
        false
    }
}

#[cfg(test)]
#[path = "../tests/effects/pour.rs"]
mod tests;
