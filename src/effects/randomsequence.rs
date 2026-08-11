// RandomSequence effect — chars revealed in random order. Each revealed
// char plays a gradient animation from `starting_color` (#000000) to its
// position-based final color. Matches TTE's `Gradient(start, final, steps=7)`
// which yields 8 colors at 8 frames each.

pub const NAME: &str = "randomsequence";
pub const DESCRIPTION: &str = "Prints the input data in a random sequence.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;

struct CharAnim {
    y: usize,
    x: usize,
    original_ch: char,
    gradient_colors: Vec<Rgb>,
    step: usize,
    hold: usize,
    frames_per_step: usize,
    active: bool,
    done: bool,
    final_color: Rgb,
}

impl CharAnim {
    fn tick(&mut self) {
        if !self.active || self.done {
            return;
        }
        self.hold += 1;
        if self.hold >= self.frames_per_step {
            self.hold = 0;
            self.step += 1;
            if self.step >= self.gradient_colors.len() {
                self.step = self.gradient_colors.len() - 1;
                self.done = true;
            }
        }
    }

    fn current_color(&self) -> Rgb {
        if self.gradient_colors.is_empty() {
            return self.final_color;
        }
        self.gradient_colors[self.step]
    }
}

pub struct RandomSequenceEffect {
    chars: Vec<CharAnim>,
    reveal_order: Vec<usize>,
    reveal_pos: usize,
    chars_per_tick: usize,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

impl RandomSequenceEffect {
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
        let starting_color = Rgb::new(0, 0, 0);

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

        let frames_per_step = 8usize;
        // TTE Gradient(start, final, steps=7) yields 8 colors. Match exactly.
        let gradient_total_colors = 8usize;

        let mut chars: Vec<CharAnim> = Vec::new();
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
                // Spectrum: t = 0/7, 1/7, ..., 6/7, then final stop.
                let mut gradient_colors = Vec::with_capacity(gradient_total_colors);
                for i in 0..(gradient_total_colors - 1) {
                    let t = i as f64 / (gradient_total_colors - 1) as f64;
                    gradient_colors.push(Rgb::lerp(starting_color, final_color, t));
                }
                gradient_colors.push(final_color);

                chars.push(CharAnim {
                    y,
                    x,
                    original_ch: ch,
                    gradient_colors,
                    step: 0,
                    hold: 0,
                    frames_per_step,
                    active: false,
                    done: false,
                    final_color,
                });
            }
        }

        let mut reveal_order: Vec<usize> = (0..chars.len()).collect();
        let mut rng = rand::thread_rng();
        reveal_order.shuffle(&mut rng);

        // TTE: max(int(speed * len(input_characters)), 1) where speed = 0.007.
        let chars_per_tick = ((0.007 * chars.len() as f64) as usize).max(1);

        RandomSequenceEffect {
            chars,
            reveal_order,
            reveal_pos: 0,
            chars_per_tick,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let end = (self.reveal_pos + self.chars_per_tick).min(self.reveal_order.len());
        for i in self.reveal_pos..end {
            let idx = self.reveal_order[i];
            self.chars[idx].active = true;
        }
        self.reveal_pos = end;

        let all_revealed = self.reveal_pos >= self.reveal_order.len();
        let mut all_done = all_revealed;
        for ca in &mut self.chars {
            ca.tick();
            if ca.active && !ca.done {
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

        for ca in &self.chars {
            if !ca.active {
                continue;
            }
            if ca.y < grid.height && ca.x < grid.width {
                let cell = &mut grid.cells[ca.y][ca.x];
                cell.visible = true;
                cell.ch = ca.original_ch;
                cell.fg = Some(ca.current_color().to_crossterm());
            }
        }

        if all_done {
            for ca in &self.chars {
                if ca.y < grid.height && ca.x < grid.width {
                    let cell = &mut grid.cells[ca.y][ca.x];
                    cell.visible = true;
                    cell.ch = ca.original_ch;
                    cell.fg = Some(ca.final_color.to_crossterm());
                }
            }
            return true;
        }

        false
    }
}

#[cfg(test)]
#[path = "../tests/effects/randomsequence.rs"]
mod tests;
