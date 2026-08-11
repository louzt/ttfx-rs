// Scattered effect — chars start at random canvas coords and converge to their
// input position with `in_out_back` easing. The color animation is synced to
// motion distance: at progress 0 the gradient's first stop (#ff9048) shows;
// at progress 1 the position-based final color shows.

pub const NAME: &str = "scattered";
pub const DESCRIPTION: &str = "Text is scattered across the canvas and moves into position.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

struct CharMotion {
    final_y: usize,
    final_x: usize,
    original_ch: char,
    start_y: f64,
    start_x: f64,
    cur_y: f64,
    cur_x: f64,
    speed: f64,    // progress increment per tick
    progress: f64, // 0.0 → 1.0
    done: bool,
    color_steps: Vec<Rgb>, // 11 colors: spectrum[0] → final_color
    final_color: Rgb,
}

pub struct ScatteredEffect {
    chars: Vec<CharMotion>,
    hold_frames: usize,
    hold_count: usize,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn aspect_dist(dy: f64, dx: f64) -> f64 {
    // TTE uses double_row_diff=True for path-length calculations, treating
    // a row step as twice the visual unit of a column step.
    (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0)
}

impl ScatteredEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;

        let original_chars: Vec<Vec<char>> = grid
            .cells
            .iter()
            .map(|row| row.iter().map(|c| c.ch).collect())
            .collect();

        // Text bounds for the final gradient.
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
        }
        let text_h = text_bottom.saturating_sub(text_top).max(1);
        let text_w = text_right.saturating_sub(text_left).max(1);

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("ff9048"),
                Rgb::from_hex("ab9dff"),
                Rgb::from_hex("bdffea"),
            ],
            12,
        );
        // TTE's per-character gradient: Gradient(spectrum[0], final, steps=10)
        // → 11 colors. spectrum[0] is the first stop of the final gradient.
        let start_color = Rgb::from_hex("ff9048");

        let mut rng = rand::thread_rng();
        let movement_speed: f64 = 0.5;

        let mut chars = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let original_ch = grid.cells[y][x].ch;
                let ry = y.saturating_sub(text_top);
                let rx = x.saturating_sub(text_left);
                let final_color = final_gradient.color_at_coord(
                    ry,
                    rx,
                    text_h,
                    text_w,
                    GradientDirection::Vertical,
                );

                // TTE's canvas.random_coord(): uniform over the entire canvas.
                let start_y = if height > 1 {
                    rng.gen_range(0..height) as f64
                } else {
                    0.0
                };
                let start_x = if width > 1 {
                    rng.gen_range(0..width) as f64
                } else {
                    0.0
                };

                let dy = y as f64 - start_y;
                let dx = x as f64 - start_x;
                let speed = movement_speed / aspect_dist(dy, dx);

                let mut color_steps = Vec::with_capacity(11);
                for i in 0..11 {
                    let t = i as f64 / 10.0;
                    color_steps.push(Rgb::lerp(start_color, final_color, t));
                }

                chars.push(CharMotion {
                    final_y: y,
                    final_x: x,
                    original_ch,
                    start_y,
                    start_x,
                    cur_y: start_y,
                    cur_x: start_x,
                    speed,
                    progress: 0.0,
                    done: false,
                    color_steps,
                    final_color,
                });
            }
        }

        ScatteredEffect {
            chars,
            hold_frames: 25,
            hold_count: 0,
            width,
            height,
            original_chars,
        }
    }

    fn reset_grid(&self, grid: &mut Grid) {
        for (y, row) in grid.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                cell.visible = false;
                cell.ch = self.original_chars[y][x];
                cell.fg = None;
            }
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        // Initial hold: chars stay at random start positions in start color.
        if self.hold_count < self.hold_frames {
            self.hold_count += 1;
            self.reset_grid(grid);
            for cm in &self.chars {
                let ry = cm.start_y.round() as usize;
                let rx = cm.start_x.round() as usize;
                if ry < self.height && rx < self.width {
                    let cell = &mut grid.cells[ry][rx];
                    cell.visible = true;
                    cell.ch = cm.original_ch;
                    cell.fg = Some(cm.color_steps[0].to_crossterm());
                }
            }
            return false;
        }

        // Movement phase.
        let mut all_done = true;
        for cm in &mut self.chars {
            if cm.done {
                continue;
            }
            cm.progress += cm.speed;
            if cm.progress >= 1.0 {
                cm.progress = 1.0;
                cm.done = true;
            }
            let eased = easing::in_out_back(cm.progress);
            cm.cur_y = cm.start_y + (cm.final_y as f64 - cm.start_y) * eased;
            cm.cur_x = cm.start_x + (cm.final_x as f64 - cm.start_x) * eased;
            if !cm.done {
                all_done = false;
            }
        }

        self.reset_grid(grid);
        for cm in &self.chars {
            let ry = cm.cur_y.round();
            let rx = cm.cur_x.round();
            if ry < 0.0 || rx < 0.0 {
                continue;
            }
            let (ry, rx) = (ry as usize, rx as usize);
            if ry >= self.height || rx >= self.width {
                continue;
            }
            let cell = &mut grid.cells[ry][rx];
            cell.visible = true;
            cell.ch = cm.original_ch;
            // Color synced to linear distance progress (TTE: SyncMetric.DISTANCE).
            let color_idx = (cm.progress * (cm.color_steps.len() - 1) as f64).round() as usize;
            let color_idx = color_idx.min(cm.color_steps.len() - 1);
            cell.fg = Some(cm.color_steps[color_idx].to_crossterm());
        }

        if all_done {
            for cm in &self.chars {
                if cm.final_y < grid.height && cm.final_x < grid.width {
                    let cell = &mut grid.cells[cm.final_y][cm.final_x];
                    cell.visible = true;
                    cell.ch = cm.original_ch;
                    cell.fg = Some(cm.final_color.to_crossterm());
                }
            }
            return true;
        }

        false
    }
}

#[cfg(test)]
#[path = "../tests/effects/scattered.rs"]
mod tests;
