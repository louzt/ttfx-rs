// ColorShift effect — animated gradient cycle with traveling wave

pub const NAME: &str = "colorshift";
pub const DESCRIPTION: &str = "Display a gradient that shifts colors across the terminal.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Cycling,
    Transitioning,
    Done,
}

pub struct ColorShiftEffect {
    width: usize,
    height: usize,
    spectrum: Vec<Rgb>,
    gradient_frames: usize,
    cycles: usize,
    current_cycle: usize,
    spectrum_offset: usize,
    step_counter: usize,
    phase: Phase,
    trans_frame: usize,
    trans_total: usize,
    final_colors: Vec<Vec<Rgb>>,
    radial_shift: Vec<Vec<usize>>,
    original: Vec<Vec<char>>,
}

impl ColorShiftEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height) = (grid.width, grid.height);

        let rainbow_stops = [
            Rgb::from_hex("e81416"),
            Rgb::from_hex("ffa500"),
            Rgb::from_hex("faeb36"),
            Rgb::from_hex("79c314"),
            Rgb::from_hex("487de7"),
            Rgb::from_hex("4b369d"),
            Rgb::from_hex("70369d"),
        ];
        let spectrum_gradient = Gradient::new(&rainbow_stops, 12);
        let mut spectrum = spectrum_gradient.spectrum().to_vec();
        let mut rev = spectrum.clone();
        rev.reverse();
        rev.pop();
        spectrum.extend(rev);

        let final_gradient = Gradient::new(&rainbow_stops, 12);

        let mut text_top = usize::MAX;
        let mut text_bottom = 0usize;
        let mut text_left = usize::MAX;
        let mut text_right = 0usize;
        let mut has_text = false;
        for y in 0..height {
            for x in 0..width {
                if grid.cells[y][x].ch != ' ' {
                    has_text = true;
                    text_top = text_top.min(y);
                    text_bottom = text_bottom.max(y);
                    text_left = text_left.min(x);
                    text_right = text_right.max(x);
                }
            }
        }
        if !has_text {
            text_top = 0;
            text_bottom = height.saturating_sub(1);
            text_left = 0;
            text_right = width.saturating_sub(1);
        }
        let text_h = (text_bottom - text_top + 1) as f64;
        let text_w = (text_right - text_left + 1) as f64;
        let center_y = text_h / 2.0;
        let center_x = text_w / 2.0;
        let max_dist = (text_w * text_w + (text_h * 2.0).powi(2)).sqrt();
        let half_max = (max_dist / 2.0).max(1.0);

        let spec_len = spectrum.len();
        let mut radial_shift = vec![vec![0usize; width]; height];
        let mut final_colors = vec![vec![Rgb::new(0, 0, 0); width]; height];
        for y in 0..height {
            for x in 0..width {
                let ry = ((y as isize - text_top as isize) + 1) as f64;
                let rx = ((x as isize - text_left as isize) + 1) as f64;
                let dy = ry - center_y;
                let dx = rx - center_x;
                let dist = (dx * dx + (dy * 2.0).powi(2)).sqrt();
                let norm = (dist / half_max).clamp(0.0, 1.0);
                radial_shift[y][x] = ((spec_len as f64 * norm) as usize) % spec_len.max(1);

                let fy = y.saturating_sub(text_top);
                let fx = x.saturating_sub(text_left);
                final_colors[y][x] = final_gradient.color_at_coord(
                    fy,
                    fx,
                    (text_bottom - text_top).max(1),
                    (text_right - text_left).max(1),
                    GradientDirection::Vertical,
                );
            }
        }

        let mut original = Vec::with_capacity(height);
        for y in 0..height {
            let row: Vec<char> = (0..width).map(|x| grid.cells[y][x].ch).collect();
            original.push(row);
        }

        let gradient_frames = 2usize;
        let trans_total = 9 * gradient_frames;

        ColorShiftEffect {
            width,
            height,
            spectrum,
            gradient_frames,
            cycles: 3,
            current_cycle: 0,
            spectrum_offset: 0,
            step_counter: 0,
            phase: Phase::Cycling,
            trans_frame: 0,
            trans_total,
            final_colors,
            radial_shift,
            original,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let spec_len = self.spectrum.len().max(1);

        match self.phase {
            Phase::Cycling => {
                self.step_counter += 1;
                if self.step_counter >= self.gradient_frames {
                    self.step_counter = 0;
                    self.spectrum_offset += 1;
                    if self.spectrum_offset >= spec_len {
                        self.spectrum_offset = 0;
                        self.current_cycle += 1;
                        if self.current_cycle >= self.cycles {
                            self.phase = Phase::Transitioning;
                            self.trans_frame = 0;
                        }
                    }
                }
            }
            Phase::Transitioning => {
                self.trans_frame += 1;
                if self.trans_frame >= self.trans_total {
                    self.phase = Phase::Done;
                }
            }
            Phase::Done => {}
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &mut grid.cells[y][x];
                cell.visible = true;
                cell.ch = self.original[y][x];

                let shift = self.radial_shift[y][x];
                match self.phase {
                    Phase::Cycling => {
                        let idx = (self.spectrum_offset + shift) % spec_len;
                        cell.fg = Some(self.spectrum[idx].to_crossterm());
                    }
                    Phase::Transitioning => {
                        let last_cycle_color = self.spectrum[(spec_len - 1 + shift) % spec_len];
                        let t = self.trans_frame as f64 / self.trans_total as f64;
                        let c = Rgb::lerp(last_cycle_color, self.final_colors[y][x], t);
                        cell.fg = Some(c.to_crossterm());
                    }
                    Phase::Done => {
                        cell.fg = Some(self.final_colors[y][x].to_crossterm());
                    }
                }
            }
        }

        self.phase == Phase::Done
    }
}

#[cfg(test)]
#[path = "../tests/effects/colorshift.rs"]
mod tests;
