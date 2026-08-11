// Slide effect — faithful TTE reimplementation
// Characters slide in from outside terminal, grouped by row, with in_out_quad easing

pub const NAME: &str = "slide";
pub const DESCRIPTION: &str = "Slide characters into view from outside the terminal.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};

struct SlideChar {
    final_y: usize,
    final_x: usize,
    start_x: f64,
    cur_x: f64,
    original_ch: char,
    final_color: Rgb,
    progress: f64,
    speed: f64,
    active: bool,
    done: bool,
}

pub struct SlideEffect {
    chars: Vec<SlideChar>,
    groups: Vec<Vec<usize>>, // row groups
    gap: usize,
    gap_counter: usize,
    activated_up_to: usize,
    width: usize,
    height: usize,
}

impl SlideEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;
        let dm: usize = 2;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("833ab4"),
                Rgb::from_hex("fd1d1d"),
                Rgb::from_hex("fcb045"),
            ],
            12,
        );

        let movement_speed = 0.8;
        let gap = 2 * dm;

        let mut chars = Vec::with_capacity(width * height);
        let mut groups: Vec<Vec<usize>> = vec![Vec::new(); height];

        for (y, group) in groups.iter_mut().enumerate() {
            for x in 0..width {
                let final_color =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Vertical);
                // Start from left edge (negative)
                let start_x = -(width as f64) + x as f64 * 0.1;
                let dist = (x as f64 - start_x).abs().max(1.0);
                let speed = (movement_speed / dist) / dm as f64;

                let idx = chars.len();
                group.push(idx);

                chars.push(SlideChar {
                    final_y: y,
                    final_x: x,
                    start_x,
                    cur_x: start_x,
                    original_ch: grid.cells[y][x].ch,
                    final_color,
                    progress: 0.0,
                    speed,
                    active: false,
                    done: false,
                });
            }
        }

        SlideEffect {
            chars,
            groups,
            gap,
            gap_counter: 0,
            activated_up_to: 0,
            width,
            height,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        // Activate groups with gap
        if self.activated_up_to < self.groups.len() {
            if self.gap_counter == 0 {
                for &idx in &self.groups[self.activated_up_to] {
                    self.chars[idx].active = true;
                }
                self.activated_up_to += 1;
                self.gap_counter = self.gap;
            } else {
                self.gap_counter -= 1;
            }
        }

        // Tick movement
        let mut all_done = self.activated_up_to >= self.groups.len();
        for ch in &mut self.chars {
            if !ch.active || ch.done {
                continue;
            }
            ch.progress += ch.speed;
            if ch.progress >= 1.0 {
                ch.progress = 1.0;
                ch.done = true;
            }
            let eased = easing::in_out_quad(ch.progress);
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
            let rx = ch.cur_x.round() as isize;
            let ry = ch.final_y;
            if rx >= 0 && (rx as usize) < self.width && ry < self.height {
                let cell = &mut grid.cells[ry][rx as usize];
                cell.visible = true;
                cell.ch = ch.original_ch;
                let t = ch.progress;
                let start_c = Rgb::from_hex("833ab4");
                cell.fg = Some(Rgb::lerp(start_c, ch.final_color, t).to_crossterm());
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
