// VHSTape effect — glitch waves, snow noise, then redraw

pub const NAME: &str = "vhstape";
pub const DESCRIPTION: &str =
    "Lines of characters glitch left and right and lose detail like an old VHS tape.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Glitch,
    Noise,
    Redraw,
    Done,
}

struct VHSChar {
    final_y: usize,
    final_x: usize,
    offset_x: f64,
    original_ch: char,
    final_color: Rgb,
    glitching: bool,
    glitch_hold: usize,
    redraw_step: usize,
}

pub struct VHSTapeEffect {
    chars: Vec<Vec<VHSChar>>,
    phase: Phase,
    frame: usize,
    dm: usize,
    width: usize,
    height: usize,
    total_glitch_time: usize,
    glitch_wave_y: isize,
    glitch_wave_dir: isize,
    noise_frame: usize,
    noise_duration: usize,
    redraw_row: usize,
    glitch_line_colors: Vec<Rgb>,
    noise_colors: Vec<Rgb>,
    final_gradient: Gradient,
}

impl VHSTapeEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height, dm) = (grid.width, grid.height, 2usize);
        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("ab48ff"),
                Rgb::from_hex("e7b2b2"),
                Rgb::from_hex("fffebd"),
            ],
            12,
        );
        let glitch_line_colors = vec![
            Rgb::from_hex("ffffff"),
            Rgb::from_hex("ff0000"),
            Rgb::from_hex("00ff00"),
            Rgb::from_hex("0000ff"),
            Rgb::from_hex("ffffff"),
        ];
        let noise_colors = vec![
            Rgb::from_hex("1e1e1f"),
            Rgb::from_hex("3c3b3d"),
            Rgb::from_hex("6d6c70"),
            Rgb::from_hex("a2a1a6"),
            Rgb::from_hex("cbc9cf"),
            Rgb::from_hex("ffffff"),
        ];

        let mut chars = Vec::new();
        for y in 0..height {
            let mut row = Vec::new();
            for x in 0..width {
                let fc =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Vertical);
                row.push(VHSChar {
                    final_y: y,
                    final_x: x,
                    offset_x: 0.0,
                    original_ch: grid.cells[y][x].ch,
                    final_color: fc,
                    glitching: false,
                    glitch_hold: 0,
                    redraw_step: 0,
                });
            }
            chars.push(row);
        }

        VHSTapeEffect {
            chars,
            phase: Phase::Glitch,
            frame: 0,
            dm,
            width,
            height,
            total_glitch_time: 600 * dm,
            glitch_wave_y: -1,
            glitch_wave_dir: 1,
            noise_frame: 0,
            noise_duration: 120 * dm,
            redraw_row: 0,
            glitch_line_colors,
            noise_colors,
            final_gradient,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        self.frame += 1;
        let dm = self.dm;
        let mut rng = rand::thread_rng();

        match self.phase {
            Phase::Glitch => {
                // Glitch wave propagation
                if rng.r#gen::<f64>() < 0.15 || self.glitch_wave_y >= 0 {
                    if self.glitch_wave_y < 0 {
                        self.glitch_wave_y = if self.glitch_wave_dir > 0 {
                            0
                        } else {
                            self.height as isize - 1
                        };
                    }
                    // Apply glitch to 3 rows around wave
                    for dy in -1..=1 {
                        let gy = self.glitch_wave_y + dy;
                        if gy >= 0 && (gy as usize) < self.height {
                            let row = gy as usize;
                            let offset = rng.gen_range(-8.0..8.0);
                            for ch in &mut self.chars[row] {
                                ch.offset_x = offset;
                                ch.glitching = true;
                                ch.glitch_hold = rng.gen_range(1..15) * dm;
                            }
                        }
                    }
                    self.glitch_wave_y += self.glitch_wave_dir;
                    if self.glitch_wave_y < 0 || self.glitch_wave_y >= self.height as isize {
                        self.glitch_wave_y = -1;
                        self.glitch_wave_dir = -self.glitch_wave_dir;
                    }
                }

                // Random line glitches
                if rng.r#gen::<f64>() < 0.05 {
                    let row = rng.gen_range(0..self.height);
                    let offset = rng.gen_range(-25.0..25.0);
                    for ch in &mut self.chars[row] {
                        ch.offset_x = offset;
                        ch.glitching = true;
                        ch.glitch_hold = rng.gen_range(1..50) * dm;
                    }
                }

                // Decay glitch holds
                for row in &mut self.chars {
                    for ch in row {
                        if ch.glitching {
                            if ch.glitch_hold > 0 {
                                ch.glitch_hold -= 1;
                            } else {
                                ch.glitching = false;
                                ch.offset_x *= 0.7;
                                if ch.offset_x.abs() < 0.5 {
                                    ch.offset_x = 0.0;
                                }
                            }
                        }
                    }
                }

                if self.frame >= self.total_glitch_time {
                    self.phase = Phase::Noise;
                    self.noise_frame = 0;
                }
            }
            Phase::Noise => {
                self.noise_frame += 1;
                if self.noise_frame >= self.noise_duration {
                    self.phase = Phase::Redraw;
                    self.redraw_row = 0;
                }
            }
            Phase::Redraw => {
                // Redraw rows from top to bottom
                let rows_per_frame = (2 * dm).max(1);
                for _ in 0..rows_per_frame {
                    if self.redraw_row < self.height {
                        for ch in &mut self.chars[self.redraw_row] {
                            ch.redraw_step = 7; // mark as redrawn
                            ch.offset_x = 0.0;
                            ch.glitching = false;
                        }
                        self.redraw_row += 1;
                    }
                }
                // Advance redraw animation for already started
                for row in &mut self.chars {
                    for ch in row {
                        if ch.redraw_step > 0 && ch.redraw_step < 7 {
                            ch.redraw_step += 1;
                        }
                    }
                }
                if self.redraw_row >= self.height {
                    self.phase = Phase::Done;
                }
            }
            Phase::Done => return true,
        }

        // Render
        let snow_chars = ['#', '*', '.', ':'];
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &mut grid.cells[y][x];
                match self.phase {
                    Phase::Noise => {
                        cell.visible = true;
                        if rng.r#gen::<f64>() < 0.3 {
                            cell.ch = snow_chars[rng.gen_range(0..snow_chars.len())];
                            let nc = self.noise_colors[rng.gen_range(0..self.noise_colors.len())];
                            cell.fg = Some(nc.to_crossterm());
                        } else {
                            cell.ch = ' ';
                            cell.fg = Some(
                                Rgb {
                                    r: 30,
                                    g: 30,
                                    b: 31,
                                }
                                .to_crossterm(),
                            );
                        }
                    }
                    Phase::Redraw | Phase::Done => {
                        let ch = &self.chars[y][x];
                        if ch.redraw_step >= 7 {
                            cell.visible = true;
                            cell.ch = ch.original_ch;
                            cell.fg = Some(ch.final_color.to_crossterm());
                        } else if ch.redraw_step > 0 {
                            cell.visible = true;
                            cell.ch = '█';
                            cell.fg = Some(Rgb::from_hex("ffffff").to_crossterm());
                        } else {
                            // Still noise
                            cell.visible = true;
                            cell.ch = snow_chars[rng.gen_range(0..snow_chars.len())];
                            let nc = self.noise_colors[rng.gen_range(0..self.noise_colors.len())];
                            cell.fg = Some(nc.to_crossterm());
                        }
                    }
                    _ => {
                        // Glitch phase
                        let ch = &self.chars[y][x];
                        let sx = (x as f64 + ch.offset_x).round() as isize;
                        cell.visible = true;
                        if sx < 0 || sx >= self.width as isize {
                            cell.ch = ' ';
                            cell.fg = None;
                        } else {
                            let src_x = sx as usize;
                            cell.ch = self.chars[y][src_x].original_ch;
                            if ch.glitching {
                                let gc = self.glitch_line_colors
                                    [rng.gen_range(0..self.glitch_line_colors.len())];
                                cell.fg = Some(gc.to_crossterm());
                            } else {
                                cell.fg = Some(ch.final_color.to_crossterm());
                            }
                        }
                    }
                }
            }
        }
        false
    }
}
