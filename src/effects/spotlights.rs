// Spotlights effect — faithful TTE reimplementation
// Spotlights search canvas, illuminate nearby chars, then expand to reveal all

pub const NAME: &str = "spotlights";
pub const DESCRIPTION: &str = "Spotlights search the text area, illuminating characters, before converging in the center and expanding.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

pub struct SpotlightsEffect {
    // Spotlight positions
    spots: Vec<(f64, f64)>,
    spot_targets: Vec<Vec<(f64, f64)>>,
    spot_target_idx: Vec<usize>,
    spot_progress: Vec<f64>,
    spot_speed: Vec<f64>,
    // Character colors
    final_colors: Vec<Vec<Rgb>>,
    // Phases
    search_frames: usize,
    frame: usize,
    beam_width: f64,
    beam_falloff: f64,
    expanding: bool,
    expand_radius: f64,
    dm: usize,
    width: usize,
    height: usize,
    done: bool,
}

impl SpotlightsEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;
        let dm: usize = 2;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("ab48ff"),
                Rgb::from_hex("e7b2b2"),
                Rgb::from_hex("fffebd"),
            ],
            12,
        );

        let mut rng = rand::thread_rng();
        let spot_count = 3;
        let beam_width = (width.min(height) as f64 / 2.0).max(2.0);
        let search_frames = 550 * dm;

        let mut final_colors = Vec::with_capacity(height);
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                row.push(final_gradient.color_at_coord(
                    y,
                    x,
                    height,
                    width,
                    GradientDirection::Vertical,
                ));
            }
            final_colors.push(row);
        }

        // Create spotlight waypoint chains
        let mut spots = Vec::new();
        let mut spot_targets = Vec::new();
        let mut spot_target_idx = Vec::new();
        let mut spot_progress = Vec::new();
        let mut spot_speed = Vec::new();

        for _ in 0..spot_count {
            let sy = rng.gen_range(0..height.max(1)) as f64;
            let sx = rng.gen_range(0..width.max(1)) as f64;
            spots.push((sy, sx));

            // 10 waypoints
            let mut targets = Vec::new();
            for _ in 0..10 {
                let ty = rng.gen_range(0..height.max(1)) as f64;
                let tx = rng.gen_range(0..width.max(1)) as f64;
                targets.push((ty, tx));
            }
            spot_targets.push(targets);
            spot_target_idx.push(0);
            spot_progress.push(0.0);
            let spd: f64 = rng.gen_range(0.35..0.75);
            spot_speed.push(spd / dm as f64);
        }

        SpotlightsEffect {
            spots,
            spot_targets,
            spot_target_idx,
            spot_progress,
            spot_speed,
            final_colors,
            search_frames,
            frame: 0,
            beam_width,
            beam_falloff: 0.3,
            expanding: false,
            expand_radius: 0.0,
            dm,
            width,
            height,
            done: false,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        if self.done {
            return true;
        }
        self.frame += 1;

        let search_done = self.frame >= self.search_frames;

        if !search_done {
            // Move spotlights along waypoints
            for i in 0..self.spots.len() {
                let tidx = self.spot_target_idx[i];
                if tidx >= self.spot_targets[i].len() {
                    self.spot_target_idx[i] = 0;
                    continue;
                }
                let (ty, tx) = self.spot_targets[i][tidx];
                let (sy, sx) = self.spots[i];
                let dist = ((ty - sy).powi(2) + (tx - sx).powi(2)).sqrt();
                if dist < 0.5 {
                    self.spot_target_idx[i] = (tidx + 1) % self.spot_targets[i].len();
                } else {
                    let step = self.spot_speed[i];
                    self.spots[i].0 += (ty - sy) / dist * step * dist.min(3.0);
                    self.spots[i].1 += (tx - sx) / dist * step * dist.min(3.0);
                }
            }
        } else {
            // Expanding phase
            self.expanding = true;
            self.expand_radius += 1.0;
            let max_dim = self.width.max(self.height) as f64 * 1.5;
            if self.expand_radius > max_dim {
                self.done = true;
            }
            // Move all spots to center
            let cy = self.height as f64 / 2.0;
            let cx = self.width as f64 / 2.0;
            for spot in &mut self.spots {
                spot.0 += (cy - spot.0) * 0.1;
                spot.1 += (cx - spot.1) * 0.1;
            }
        }

        // Render: calculate brightness per cell
        let dark = Rgb::new(20, 20, 20);
        let radius = if self.expanding {
            self.expand_radius
        } else {
            self.beam_width
        };

        for y in 0..self.height.min(grid.height) {
            for x in 0..self.width.min(grid.width) {
                let ch = grid.cells[y][x].ch;
                let cell = &mut grid.cells[y][x];
                cell.visible = true;
                cell.ch = ch;

                // Find minimum distance to any spotlight
                let mut min_dist = f64::MAX;
                for spot in &self.spots {
                    let d = ((y as f64 - spot.0).powi(2) + (x as f64 - spot.1).powi(2)).sqrt();
                    if d < min_dist {
                        min_dist = d;
                    }
                }

                let final_color = self.final_colors[y][x];

                if min_dist <= radius {
                    let falloff_start = radius * (1.0 - self.beam_falloff);
                    let brightness = if min_dist <= falloff_start {
                        1.0
                    } else {
                        let falloff_width = radius - falloff_start;
                        (1.0 - (min_dist - falloff_start) / falloff_width).max(0.2)
                    };
                    cell.fg = Some(final_color.adjust_brightness(brightness).to_crossterm());
                } else {
                    cell.fg = Some(dark.to_crossterm());
                }
            }
        }

        if self.done {
            for y in 0..self.height.min(grid.height) {
                for x in 0..self.width.min(grid.width) {
                    let cell = &mut grid.cells[y][x];
                    cell.visible = true;
                    cell.fg = Some(self.final_colors[y][x].to_crossterm());
                }
            }
        }

        self.done
    }
}
