// OrbittingVolley — 4 launchers orbit the canvas perimeter, firing volleys
// of characters inward to build the text from the center outward.

pub const NAME: &str = "orbittingvolley";
pub const DESCRIPTION: &str = "Four launchers orbit the canvas firing volleys of characters inward to build the input text from the center out.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use std::collections::VecDeque;

const LAUNCHER_SYMBOL: char = '█';

struct OVChar {
    final_y: usize,
    final_x: usize,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    final_color: Rgb,
    progress: f64,
    speed: f64,
    active: bool,
    done: bool,
    start_y: f64,
    start_x: f64,
}

pub struct OrbittingVolleyEffect {
    chars: Vec<OVChar>,
    pending: Vec<VecDeque<usize>>, // one queue per launcher (0..4)
    orbit_progress: f64,
    orbit_speed: f64,
    launcher_positions: [(f64, f64); 4],
    launcher_color: Rgb,
    volley_delay: usize,
    delay_count: usize,
    volley_per_launcher: usize,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn aspect_dist(dy: f64, dx: f64) -> f64 {
    (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0)
}

fn launcher_positions(progress: f64, width: usize, height: usize) -> [(f64, f64); 4] {
    let p = progress.clamp(0.0, 1.0);
    let last_x = (width as f64 - 1.0).max(0.0);
    let last_y = (height as f64 - 1.0).max(0.0);
    // Top: top-left → top-right
    let top = (0.0, p * last_x);
    // Right: top-right → bottom-right
    let right = (p * last_y, last_x);
    // Bottom: bottom-right → bottom-left
    let bottom = (last_y, (1.0 - p) * last_x);
    // Left: bottom-left → top-left
    let left = ((1.0 - p) * last_y, 0.0);
    [top, right, bottom, left]
}

impl OrbittingVolleyEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;

        let final_gradient = Gradient::new(&[Rgb::from_hex("FFA15C"), Rgb::from_hex("44D492")], 12);
        let launcher_color = *final_gradient.spectrum().last().unwrap();

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
        let text_cy = (text_top + text_bottom) as f64 / 2.0;
        let text_cx = (text_left + text_right) as f64 / 2.0;

        let mut text_positions: Vec<(usize, usize)> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if grid.cells[y][x].ch != ' ' {
                    text_positions.push((y, x));
                }
            }
        }
        // Sort center-to-outside (matches TTE's CENTER_TO_OUTSIDE grouping).
        text_positions.sort_by(|a, b| {
            let da = ((a.0 as f64 - text_cy).powi(2) + (a.1 as f64 - text_cx).powi(2)).sqrt();
            let db = ((b.0 as f64 - text_cy).powi(2) + (b.1 as f64 - text_cx).powi(2)).sqrt();
            da.partial_cmp(&db).unwrap()
        });

        let mut chars: Vec<OVChar> = Vec::with_capacity(text_positions.len());
        for &(y, x) in &text_positions {
            let ry = y.saturating_sub(text_top);
            let rx = x.saturating_sub(text_left);
            let final_color =
                final_gradient.color_at_coord(ry, rx, text_h, text_w, GradientDirection::Radial);
            chars.push(OVChar {
                final_y: y,
                final_x: x,
                cur_y: 0.0,
                cur_x: 0.0,
                original_ch: grid.cells[y][x].ch,
                final_color,
                progress: 0.0,
                speed: 0.0,
                active: false,
                done: false,
                start_y: 0.0,
                start_x: 0.0,
            });
        }

        // Round-robin assign chars to launchers (TTE's `cycle(launchers)`).
        let mut pending: Vec<VecDeque<usize>> = (0..4).map(|_| VecDeque::new()).collect();
        for (i, _) in chars.iter().enumerate() {
            pending[i % 4].push_back(i);
        }

        let num_chars = chars.len();
        // TTE: max(int(volley_size * num_chars / 4), 1). volley_size default = 0.03.
        let volley_per_launcher = ((num_chars as f64 * 0.03 / 4.0) as usize).max(1);

        // TTE main launcher: speed=0.8 along top edge (length = width-1).
        let orbit_speed = if width > 1 {
            0.8 / (width as f64 - 1.0)
        } else {
            1.0
        };
        let launcher_positions = launcher_positions(0.0, width, height);

        OrbittingVolleyEffect {
            chars,
            pending,
            orbit_progress: 0.0,
            orbit_speed,
            launcher_positions,
            launcher_color,
            volley_delay: 30,
            delay_count: 0,
            volley_per_launcher,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        // Advance orbit; wrap on completion (TTE re-activates the perimeter
        // path, snapping back to the start position).
        self.orbit_progress += self.orbit_speed;
        if self.orbit_progress >= 1.0 {
            self.orbit_progress -= 1.0;
        }
        self.launcher_positions = launcher_positions(self.orbit_progress, self.width, self.height);

        let any_pending = self.pending.iter().any(|q| !q.is_empty());

        if any_pending {
            if self.delay_count == 0 {
                for li in 0..4 {
                    let (ly, lx) = self.launcher_positions[li];
                    for _ in 0..self.volley_per_launcher {
                        if let Some(idx) = self.pending[li].pop_front() {
                            let ch = &mut self.chars[idx];
                            ch.active = true;
                            ch.start_y = ly;
                            ch.start_x = lx;
                            ch.cur_y = ly;
                            ch.cur_x = lx;
                            let dy = ch.final_y as f64 - ly;
                            let dx = ch.final_x as f64 - lx;
                            ch.speed = 1.5 / aspect_dist(dy, dx);
                        }
                    }
                }
                self.delay_count = self.volley_delay;
            } else {
                self.delay_count -= 1;
            }
        }

        let mut all_done = !any_pending;
        for ch in &mut self.chars {
            if !ch.active || ch.done {
                continue;
            }
            ch.progress = (ch.progress + ch.speed).min(1.0);
            if ch.progress >= 1.0 {
                ch.done = true;
            }
            let eased = easing::out_sine(ch.progress);
            ch.cur_y = ch.start_y + (ch.final_y as f64 - ch.start_y) * eased;
            ch.cur_x = ch.start_x + (ch.final_x as f64 - ch.start_x) * eased;
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
            cell.fg = Some(ch.final_color.to_crossterm());
        }

        if !all_done {
            for &(ly, lx) in &self.launcher_positions {
                let ry = ly.round() as isize;
                let rx = lx.round() as isize;
                if ry < 0 || rx < 0 {
                    continue;
                }
                let (ry, rx) = (ry as usize, rx as usize);
                if ry < self.height && rx < self.width {
                    let cell = &mut grid.cells[ry][rx];
                    cell.visible = true;
                    cell.ch = LAUNCHER_SYMBOL;
                    cell.fg = Some(self.launcher_color.to_crossterm());
                }
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
        }
        all_done
    }
}

#[cfg(test)]
#[path = "../tests/effects/orbittingvolley.rs"]
mod tests;
