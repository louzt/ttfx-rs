// Spray effect — faithful TTE reimplementation
// All characters start at origin point, spray outward to positions

pub const NAME: &str = "spray";
pub const DESCRIPTION: &str = "Draws the characters spawning at varying rates from a single point.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;

struct SprayChar {
    final_y: usize,
    final_x: usize,
    start_y: f64,
    start_x: f64,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    final_color: Rgb,
    start_color: Rgb,
    progress: f64,
    speed: f64,
    active: bool,
    done: bool,
}

pub struct SprayEffect {
    chars: Vec<SprayChar>,
    pending: Vec<usize>,
    volume: f64,
    width: usize,
    height: usize,
}

impl SprayEffect {
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

        let mut rng = rand::thread_rng();
        // Origin: east edge (default "e")
        let origin_y = height as f64 / 2.0;
        let origin_x = (width.saturating_sub(1)) as f64;

        let mut chars = Vec::with_capacity(width * height);
        let spec_len = final_gradient.spectrum().len().max(1);

        for y in 0..height {
            for x in 0..width {
                let final_color =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Vertical);
                let speed_val: f64 = rng.gen_range(0.6..1.4);
                let dist = ((y as f64 - origin_y).powi(2) + (x as f64 - origin_x).powi(2))
                    .sqrt()
                    .max(1.0);
                let speed = (speed_val / dist) / dm as f64;
                let start_color = final_gradient.spectrum()[rng.gen_range(0..spec_len)];

                chars.push(SprayChar {
                    final_y: y,
                    final_x: x,
                    start_y: origin_y,
                    start_x: origin_x,
                    cur_y: origin_y,
                    cur_x: origin_x,
                    original_ch: grid.cells[y][x].ch,
                    final_color,
                    start_color,
                    progress: 0.0,
                    speed,
                    active: false,
                    done: false,
                });
            }
        }

        let mut pending: Vec<usize> = (0..chars.len()).collect();
        pending.shuffle(&mut rng);

        SprayEffect {
            chars,
            pending,
            volume: 0.005,
            width,
            height,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        // Activate chars
        let activate_count = ((self.chars.len() as f64 * self.volume) as usize).max(1);
        for _ in 0..activate_count {
            if let Some(idx) = self.pending.pop() {
                self.chars[idx].active = true;
            }
        }

        // Tick
        let mut all_done = self.pending.is_empty();
        for ch in &mut self.chars {
            if !ch.active || ch.done {
                continue;
            }
            ch.progress += ch.speed;
            if ch.progress >= 1.0 {
                ch.progress = 1.0;
                ch.done = true;
            }
            let eased = easing::out_expo(ch.progress);
            ch.cur_y = ch.start_y + (ch.final_y as f64 - ch.start_y) * eased;
            ch.cur_x = ch.start_x + (ch.final_x as f64 - ch.start_x) * eased;
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
            if !ch.active {
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
            cell.ch = ch.original_ch;
            let t = ch.progress;
            cell.fg = Some(Rgb::lerp(ch.start_color, ch.final_color, t).to_crossterm());
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
