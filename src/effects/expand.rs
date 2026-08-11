// Expand effect — chars start at canvas center, ease out to their final positions
// while their color blends from the gradient's first stop to their position-based final color.

pub const NAME: &str = "expand";
pub const DESCRIPTION: &str = "Expands the text from a single point.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};

struct ExpandChar {
    final_y: usize,
    final_x: usize,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    final_color: Rgb,
    progress: f64,
    speed: f64,
    done: bool,
}

pub struct ExpandEffect {
    chars: Vec<ExpandChar>,
    center_y: f64,
    center_x: f64,
    start_color: Rgb,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

impl ExpandEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;

        let stops = [
            Rgb::from_hex("8A008A"),
            Rgb::from_hex("00D1FF"),
            Rgb::from_hex("FFFFFF"),
        ];
        let final_gradient = Gradient::new(&stops, 12);
        let start_color = final_gradient.spectrum()[0];

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
        let movement_speed = 0.35;

        let mut chars: Vec<ExpandChar> = Vec::new();
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
                let dy = y as f64 - center_y;
                let dx = x as f64 - center_x;
                let aspect_dist = (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0);
                let speed = movement_speed / aspect_dist;

                chars.push(ExpandChar {
                    final_y: y,
                    final_x: x,
                    cur_y: center_y,
                    cur_x: center_x,
                    original_ch: ch,
                    final_color,
                    progress: 0.0,
                    speed,
                    done: false,
                });
            }
        }

        ExpandEffect {
            chars,
            center_y,
            center_x,
            start_color,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let mut all_done = true;

        for ch in &mut self.chars {
            if ch.done {
                continue;
            }
            ch.progress = (ch.progress + ch.speed).min(1.0);
            if ch.progress >= 1.0 {
                ch.done = true;
            }
            let eased = easing::in_out_quart(ch.progress);
            ch.cur_y = self.center_y + (ch.final_y as f64 - self.center_y) * eased;
            ch.cur_x = self.center_x + (ch.final_x as f64 - self.center_x) * eased;
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
            cell.fg = Some(Rgb::lerp(self.start_color, ch.final_color, ch.progress).to_crossterm());
        }

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
#[path = "../tests/effects/expand.rs"]
mod tests;
