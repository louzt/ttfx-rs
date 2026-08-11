// MiddleOut effect — chars start at canvas center, expand horizontally to
// their column at center row (phase 1), then vertically to their final row
// (phase 2). Color blends from `starting_color` to per-char final color.

pub const NAME: &str = "middleout";
pub const DESCRIPTION: &str =
    "Text expands in a single row or column in the middle of the canvas then out.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    Center,
    Full,
    Complete,
}

struct MiddleChar {
    final_y: usize,
    final_x: usize,
    mid_y: f64,
    mid_x: f64,
    original_ch: char,
    cur_y: f64,
    cur_x: f64,
    final_color: Rgb,
    progress: f64,
    speed_center: f64,
    speed_full: f64,
    done_p1: bool,
    done_p2: bool,
}

pub struct MiddleOutEffect {
    chars: Vec<MiddleChar>,
    center_y: f64,
    center_x: f64,
    starting_color: Rgb,
    phase: Phase,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn aspect_dist(dy: f64, dx: f64) -> f64 {
    (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0)
}

impl MiddleOutEffect {
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

        let center_y = height as f64 / 2.0;
        let center_x = width as f64 / 2.0;
        let center_speed = 0.6;
        let full_speed = 0.6;

        let mut chars: Vec<MiddleChar> = Vec::new();
        for y in 0..height {
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
                // Vertical expand (TTE default): mid waypoint = (final_x, center_row).
                let mid_y = center_y;
                let mid_x = x as f64;

                let d1 = aspect_dist(mid_y - center_y, mid_x - center_x);
                let speed_center = center_speed / d1;
                let d2 = aspect_dist(y as f64 - mid_y, x as f64 - mid_x);
                let speed_full = full_speed / d2;

                chars.push(MiddleChar {
                    final_y: y,
                    final_x: x,
                    mid_y,
                    mid_x,
                    original_ch: ch,
                    cur_y: center_y,
                    cur_x: center_x,
                    final_color,
                    progress: 0.0,
                    speed_center,
                    speed_full,
                    done_p1: false,
                    done_p2: false,
                });
            }
        }

        MiddleOutEffect {
            chars,
            center_y,
            center_x,
            starting_color,
            phase: Phase::Center,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        match self.phase {
            Phase::Center => {
                let mut all_done = true;
                for ch in &mut self.chars {
                    if ch.done_p1 {
                        continue;
                    }
                    ch.progress = (ch.progress + ch.speed_center).min(1.0);
                    if ch.progress >= 1.0 {
                        ch.done_p1 = true;
                    }
                    let eased = easing::in_out_sine(ch.progress);
                    ch.cur_y = self.center_y + (ch.mid_y - self.center_y) * eased;
                    ch.cur_x = self.center_x + (ch.mid_x - self.center_x) * eased;
                    if !ch.done_p1 {
                        all_done = false;
                    }
                }
                if all_done {
                    self.phase = Phase::Full;
                    for ch in &mut self.chars {
                        ch.progress = 0.0;
                    }
                }
            }
            Phase::Full => {
                let mut all_done = true;
                for ch in &mut self.chars {
                    if ch.done_p2 {
                        continue;
                    }
                    ch.progress = (ch.progress + ch.speed_full).min(1.0);
                    if ch.progress >= 1.0 {
                        ch.done_p2 = true;
                    }
                    let eased = easing::in_out_sine(ch.progress);
                    ch.cur_y = ch.mid_y + (ch.final_y as f64 - ch.mid_y) * eased;
                    ch.cur_x = ch.mid_x + (ch.final_x as f64 - ch.mid_x) * eased;
                    if !ch.done_p2 {
                        all_done = false;
                    }
                }
                if all_done {
                    self.phase = Phase::Complete;
                }
            }
            Phase::Complete => {}
        }

        for (y, row) in grid.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                cell.visible = false;
                cell.ch = self.original_chars[y][x];
                cell.fg = None;
            }
        }

        for ch in &self.chars {
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
            // Color blend follows the full-phase progress (matching TTE's
            // full_scene which only plays during phase 2). During the center
            // phase chars stay in starting_color.
            let t = if self.phase == Phase::Center {
                0.0
            } else if self.phase == Phase::Full {
                ch.progress
            } else {
                1.0
            };
            cell.fg = Some(Rgb::lerp(self.starting_color, ch.final_color, t).to_crossterm());
        }

        if self.phase == Phase::Complete {
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
#[path = "../tests/effects/middleout.rs"]
mod tests;
