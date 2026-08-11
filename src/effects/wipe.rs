// Wipe effect — faithful TTE reimplementation
// Diagonal wipe revealing characters with gradient animation

pub const NAME: &str = "wipe";
pub const DESCRIPTION: &str = "Wipes the text across the terminal to reveal characters.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};

struct WipeChar {
    y: usize,
    x: usize,
    original_ch: char,
    final_color: Rgb,
    // Gradient animation
    colors: Vec<Rgb>,
    step: usize,
    hold: usize,
    frames_per_step: usize,
    active: bool,
    done: bool,
}

impl WipeChar {
    fn tick(&mut self) {
        if !self.active || self.done {
            return;
        }
        self.hold += 1;
        if self.hold >= self.frames_per_step {
            self.hold = 0;
            self.step += 1;
            if self.step >= self.colors.len() {
                self.step = self.colors.len() - 1;
                self.done = true;
            }
        }
    }
}

pub struct WipeEffect {
    chars: Vec<WipeChar>,
    groups: Vec<Vec<usize>>,
    total_groups: usize,
    easer_step: f64,
    easer_speed: f64,
    activated_up_to: usize,
    width: usize,
    height: usize,
}

impl WipeEffect {
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
        let start_color = Rgb::from_hex("833ab4");
        let frames_per_step = 3 * dm;

        let max_diag = width + height;
        let mut groups: Vec<Vec<usize>> = vec![Vec::new(); max_diag];
        let mut chars = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let final_color =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Vertical);
                // Gradient: start_color → final_color (12 steps)
                let steps = 12;
                let mut colors = Vec::with_capacity(steps);
                for i in 0..steps {
                    let t = (i + 1) as f64 / steps as f64;
                    colors.push(Rgb::lerp(start_color, final_color, t));
                }

                let idx = chars.len();
                let diag = x + y;
                if diag < max_diag {
                    groups[diag].push(idx);
                }

                chars.push(WipeChar {
                    y,
                    x,
                    original_ch: grid.cells[y][x].ch,
                    final_color,
                    colors,
                    step: 0,
                    hold: 0,
                    frames_per_step,
                    active: false,
                    done: false,
                });
            }
        }

        groups.retain(|g| !g.is_empty());
        let total_groups = groups.len();
        let easer_speed = 1.0 / (total_groups as f64 * dm as f64).max(1.0);

        WipeEffect {
            chars,
            groups,
            total_groups,
            easer_step: 0.0,
            easer_speed,
            activated_up_to: 0,
            width,
            height,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        // Advance easer
        self.easer_step += self.easer_speed;
        if self.easer_step > 1.0 {
            self.easer_step = 1.0;
        }

        let eased = easing::in_out_circ(self.easer_step);
        let target = (eased * self.total_groups as f64).round() as usize;
        let target = target.min(self.total_groups);

        while self.activated_up_to < target {
            for &idx in &self.groups[self.activated_up_to] {
                self.chars[idx].active = true;
            }
            self.activated_up_to += 1;
        }

        // Tick animations
        let mut all_done = self.activated_up_to >= self.total_groups;
        for ch in &mut self.chars {
            ch.tick();
            if ch.active && !ch.done {
                all_done = false;
            }
        }

        // Render
        for ch in &self.chars {
            if ch.y < grid.height && ch.x < grid.width {
                let cell = &mut grid.cells[ch.y][ch.x];
                if ch.active {
                    cell.visible = true;
                    cell.ch = ch.original_ch;
                    if ch.done {
                        cell.fg = Some(ch.final_color.to_crossterm());
                    } else {
                        cell.fg = Some(ch.colors[ch.step].to_crossterm());
                    }
                }
            }
        }

        if all_done {
            for ch in &self.chars {
                if ch.y < grid.height && ch.x < grid.width {
                    let cell = &mut grid.cells[ch.y][ch.x];
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
