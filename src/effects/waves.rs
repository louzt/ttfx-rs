// Waves effect — faithful TTE reimplementation
// Wave symbols cascade across characters with gradient, then settle to final color

pub const NAME: &str = "waves";
pub const DESCRIPTION: &str = "Waves travel across the terminal leaving behind the characters.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};

const WAVE_SYMBOLS: [char; 15] = [
    '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█', '▇', '▆', '▅', '▄', '▃', '▂', '▁',
];

struct WaveChar {
    y: usize,
    x: usize,
    original_ch: char,
    final_color: Rgb,
    // Wave phase
    wave_frame: usize,
    wave_total: usize,
    wave_active: bool,
    wave_done: bool,
    // Final phase
    final_active: bool,
    final_frame: usize,
    final_total: usize,
    final_done: bool,
}

pub struct WavesEffect {
    chars: Vec<WaveChar>,
    groups: Vec<Vec<usize>>, // column groups L→R
    easer_step: f64,
    easer_speed: f64,
    activated_up_to: usize,
    total_groups: usize,
    wave_gradient: Gradient,
    wave_length: usize,
    dm: usize,
    width: usize,
    height: usize,
}

impl WavesEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;
        let dm: usize = 2;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("ffb102"),
                Rgb::from_hex("31a0d4"),
                Rgb::from_hex("f0ff65"),
            ],
            12,
        );
        let wave_gradient = Gradient::new(
            &[
                Rgb::from_hex("f0ff65"),
                Rgb::from_hex("ffb102"),
                Rgb::from_hex("31a0d4"),
                Rgb::from_hex("ffb102"),
                Rgb::from_hex("f0ff65"),
            ],
            6,
        );

        let wave_count = 7;
        let wave_length = 2 * dm;
        let wave_total = WAVE_SYMBOLS.len() * wave_count * wave_length;
        let final_total = 10 * dm;

        let mut chars = Vec::with_capacity(width * height);
        let mut groups: Vec<Vec<usize>> = vec![Vec::new(); width];

        for y in 0..height {
            for (x, group) in groups.iter_mut().enumerate() {
                let final_color =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Diagonal);
                let idx = chars.len();
                group.push(idx);

                chars.push(WaveChar {
                    y,
                    x,
                    original_ch: grid.cells[y][x].ch,
                    final_color,
                    wave_frame: 0,
                    wave_total,
                    wave_active: false,
                    wave_done: false,
                    final_active: false,
                    final_frame: 0,
                    final_total,
                    final_done: false,
                });
            }
        }

        groups.retain(|g| !g.is_empty());
        let total_groups = groups.len();
        let easer_speed = 1.0 / (total_groups as f64 * dm as f64).max(1.0);

        WavesEffect {
            chars,
            groups,
            easer_step: 0.0,
            easer_speed,
            activated_up_to: 0,
            total_groups,
            wave_gradient,
            wave_length: wave_length.max(1),
            dm,
            width,
            height,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        // Activate groups
        self.easer_step += self.easer_speed;
        if self.easer_step > 1.0 {
            self.easer_step = 1.0;
        }
        let eased = easing::in_out_sine(self.easer_step);
        let target = (eased * self.total_groups as f64).round() as usize;
        let target = target.min(self.total_groups);

        while self.activated_up_to < target {
            for &idx in &self.groups[self.activated_up_to] {
                self.chars[idx].wave_active = true;
            }
            self.activated_up_to += 1;
        }

        // Tick
        let mut all_done = self.activated_up_to >= self.total_groups;
        for ch in &mut self.chars {
            if ch.wave_active && !ch.wave_done {
                ch.wave_frame += 1;
                if ch.wave_frame >= ch.wave_total {
                    ch.wave_done = true;
                    ch.final_active = true;
                }
            }
            if ch.final_active && !ch.final_done {
                ch.final_frame += 1;
                if ch.final_frame >= ch.final_total {
                    ch.final_done = true;
                }
            }
            if !ch.final_done {
                all_done = false;
            }
        }

        // Render
        let spec_len = self.wave_gradient.spectrum().len().max(1);

        for ch in &self.chars {
            if ch.y >= grid.height || ch.x >= grid.width {
                continue;
            }
            let cell = &mut grid.cells[ch.y][ch.x];

            if ch.final_done {
                cell.visible = true;
                cell.ch = ch.original_ch;
                cell.fg = Some(ch.final_color.to_crossterm());
            } else if ch.final_active {
                cell.visible = true;
                cell.ch = ch.original_ch;
                let t = ch.final_frame as f64 / ch.final_total as f64;
                let last_wave = self.wave_gradient.spectrum()[spec_len - 1];
                cell.fg = Some(Rgb::lerp(last_wave, ch.final_color, t).to_crossterm());
            } else if ch.wave_active {
                cell.visible = true;
                let sym_idx = (ch.wave_frame / self.wave_length) % WAVE_SYMBOLS.len();
                cell.ch = WAVE_SYMBOLS[sym_idx];
                let color_idx = (ch.wave_frame * spec_len / ch.wave_total.max(1)).min(spec_len - 1);
                cell.fg = Some(self.wave_gradient.spectrum()[color_idx].to_crossterm());
            }
        }

        all_done
    }
}
