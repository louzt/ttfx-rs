// Slice effect — faithful TTE reimplementation
// Text split vertically, halves slide in from opposite edges with in_out_expo

pub const NAME: &str = "slice";
pub const DESCRIPTION: &str =
    "Slices the input in half and slides it into place from opposite directions.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};

struct SliceChar {
    final_y: usize,
    final_x: usize,
    start_y: f64,
    cur_y: f64,
    original_ch: char,
    final_color: Rgb,
    progress: f64,
    speed: f64,
    done: bool,
}

pub struct SliceEffect {
    chars: Vec<SliceChar>,
    width: usize,
    height: usize,
}

impl SliceEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;
        let dm: usize = 2;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("FFFFFF"),
            ],
            12,
        );

        let movement_speed = 0.25;
        let center_col = width / 2;

        let mut chars = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let final_color =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Diagonal);
                // Left half slides from top, right half slides from bottom
                let start_y = if x < center_col {
                    -(height as f64)
                } else {
                    (height * 2) as f64
                };
                let dist = (y as f64 - start_y).abs().max(1.0);
                let speed = (movement_speed / dist) / dm as f64;

                chars.push(SliceChar {
                    final_y: y,
                    final_x: x,
                    start_y,
                    cur_y: start_y,
                    original_ch: grid.cells[y][x].ch,
                    final_color,
                    progress: 0.0,
                    speed,
                    done: false,
                });
            }
        }

        SliceEffect {
            chars,
            width,
            height,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let mut all_done = true;

        for ch in &mut self.chars {
            if ch.done {
                continue;
            }
            ch.progress += ch.speed;
            if ch.progress >= 1.0 {
                ch.progress = 1.0;
                ch.done = true;
            }
            let eased = easing::in_out_expo(ch.progress);
            ch.cur_y = ch.start_y + (ch.final_y as f64 - ch.start_y) * eased;
            if !ch.done {
                all_done = false;
            }
        }

        // Render
        for row in &mut grid.cells {
            for cell in row {
                cell.visible = false;
            }
        }

        for ch in &self.chars {
            let ry = ch.cur_y.round() as isize;
            if ry >= 0 && (ry as usize) < self.height && ch.final_x < self.width {
                let cell = &mut grid.cells[ry as usize][ch.final_x];
                cell.visible = true;
                cell.ch = ch.original_ch;
                cell.fg = Some(ch.final_color.to_crossterm());
            }
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
            return true;
        }
        false
    }
}
