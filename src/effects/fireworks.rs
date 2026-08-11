// Fireworks effect — shells launch together, explode outward, fall into place

pub const NAME: &str = "fireworks";
pub const DESCRIPTION: &str = "Characters launch and explode like fireworks and fall into place.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

const SHELL_COLORS: [Rgb; 5] = [
    Rgb {
        r: 0x88,
        g: 0xF7,
        b: 0xE2,
    },
    Rgb {
        r: 0x44,
        g: 0xD4,
        b: 0x92,
    },
    Rgb {
        r: 0xF5,
        g: 0xEB,
        b: 0x67,
    },
    Rgb {
        r: 0xFF,
        g: 0xA1,
        b: 0x5C,
    },
    Rgb {
        r: 0xFA,
        g: 0x23,
        b: 0x3E,
    },
];

const FIREWORK_SYMBOL: char = 'o';

#[derive(Clone, Copy, PartialEq, Debug)]
enum FWPhase {
    Waiting,
    Launch,
    Explode,
    Bloom,
    Fall,
    Done,
}

struct FWChar {
    final_y: usize,
    final_x: usize,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    final_color: Rgb,
    shell_color: Rgb,
    launch_start_y: f64,
    launch_start_x: f64,
    origin_y: f64,
    origin_x: f64,
    explode_target_y: f64,
    explode_target_x: f64,
    bloom_control_y: f64,
    bloom_control_x: f64,
    bloom_target_y: f64,
    bloom_target_x: f64,
    fall_control_y: f64,
    fall_control_x: f64,
    explode_speed_base: f64,
    phase: FWPhase,
    progress: f64,
    speed: f64,
}

struct Shell {
    char_indices: Vec<usize>,
}

pub struct FireworksEffect {
    chars: Vec<FWChar>,
    shells: std::collections::VecDeque<Shell>,
    launch_delay: usize,
    delay_counter: usize,
    base_launch_delay: usize,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn aspect_dist(dy: f64, dx: f64) -> f64 {
    (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0)
}

fn quad_bezier(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let x = u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0;
    let y = u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1;
    (x, y)
}

fn bezier_aspect_length(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64)) -> f64 {
    let mut len = 0.0;
    let mut prev = p0;
    for i in 1..=20 {
        let t = i as f64 / 20.0;
        let cur = quad_bezier(p0, p1, p2, t);
        len += aspect_dist(cur.1 - prev.1, cur.0 - prev.0);
        prev = cur;
    }
    len.max(1.0)
}

impl FireworksEffect {
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

        let original_chars: Vec<Vec<char>> = grid
            .cells
            .iter()
            .map(|row| row.iter().map(|c| c.ch).collect())
            .collect();

        let mut text_top = usize::MAX;
        let mut text_bottom = 0usize;
        let mut text_left = usize::MAX;
        let mut text_right = 0usize;
        let mut positions: Vec<(usize, usize)> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if grid.cells[y][x].ch != ' ' {
                    positions.push((y, x));
                    text_top = text_top.min(y);
                    text_bottom = text_bottom.max(y);
                    text_left = text_left.min(x);
                    text_right = text_right.max(x);
                }
            }
        }
        let text_h = text_bottom.saturating_sub(text_top).max(1);
        let text_w = text_right.saturating_sub(text_left).max(1);

        let mut rng = rand::thread_rng();

        let num_chars = positions.len();
        let firework_volume = ((num_chars as f64 * 0.05).round() as usize).max(1);
        let explode_distance = ((width as f64 * 0.2).round() as usize).clamp(1, 15) as f64;
        let bottom = (height as f64 - 1.0).max(0.0);

        let mut chars: Vec<FWChar> = Vec::with_capacity(num_chars);
        for &(y, x) in &positions {
            let ry = y.saturating_sub(text_top);
            let rx = x.saturating_sub(text_left);
            let final_color = final_gradient.color_at_coord(
                ry,
                rx,
                text_h,
                text_w,
                GradientDirection::Horizontal,
            );
            chars.push(FWChar {
                final_y: y,
                final_x: x,
                cur_y: bottom,
                cur_x: 0.0,
                original_ch: grid.cells[y][x].ch,
                final_color,
                shell_color: SHELL_COLORS[0],
                launch_start_y: bottom,
                launch_start_x: 0.0,
                origin_y: 0.0,
                origin_x: 0.0,
                explode_target_y: 0.0,
                explode_target_x: 0.0,
                bloom_control_y: 0.0,
                bloom_control_x: 0.0,
                bloom_target_y: 0.0,
                bloom_target_x: 0.0,
                fall_control_y: 0.0,
                fall_control_x: 0.0,
                explode_speed_base: 0.3,
                phase: FWPhase::Waiting,
                progress: 0.0,
                speed: 0.0,
            });
        }

        let mut shells: std::collections::VecDeque<Shell> = std::collections::VecDeque::new();
        let mut idx_iter = 0usize;
        while idx_iter < chars.len() {
            let end = (idx_iter + firework_volume).min(chars.len());
            let shell_indices: Vec<usize> = (idx_iter..end).collect();

            let first_y = chars[shell_indices[0]].final_y;
            let origin_x = rng.gen_range(0..width.max(1)) as f64;
            let origin_y = if first_y == 0 {
                0.0
            } else {
                rng.gen_range(0..=first_y) as f64
            };
            let shell_color = SHELL_COLORS[rng.gen_range(0..SHELL_COLORS.len())];

            for &ci in &shell_indices {
                let ch = &mut chars[ci];
                ch.shell_color = shell_color;
                ch.launch_start_x = origin_x;
                ch.launch_start_y = bottom;
                ch.cur_x = origin_x;
                ch.cur_y = bottom;
                ch.origin_x = origin_x;
                ch.origin_y = origin_y;

                let angle: f64 = rng.gen_range(0.0..std::f64::consts::TAU);
                let dist: f64 = rng.gen_range(1.0..=explode_distance.max(1.0));
                ch.explode_target_x = origin_x + angle.cos() * dist;
                ch.explode_target_y = origin_y + angle.sin() * (dist / 2.0);
                ch.explode_speed_base = rng.gen_range(0.2..=0.4);

                // Bloom control: extrapolated past the burst point along the
                // origin→burst ray by explode_distance/2 (TTE behavior).
                let bdy = ch.explode_target_y - origin_y;
                let bdx = ch.explode_target_x - origin_x;
                let burst_len = (bdx * bdx + bdy * bdy).sqrt();
                let extra = explode_distance / 2.0;
                let ratio = if burst_len > 0.0 {
                    (burst_len + extra) / burst_len
                } else {
                    1.0
                };
                ch.bloom_control_x = origin_x + bdx * ratio;
                ch.bloom_control_y = origin_y + bdy * ratio;
                // Bloom target: 7 rows visually below the bloom control,
                // clamped to canvas bottom. (TTE: max(1, row - 7) in
                // bottom-up coords ↔ min(height-1, y + 7) in rtte top-down.)
                ch.bloom_target_x = ch.bloom_control_x;
                ch.bloom_target_y = (ch.bloom_control_y + 7.0).min(bottom);
                // Settle bezier control: column of bloom target, canvas bottom.
                ch.fall_control_x = ch.bloom_target_x;
                ch.fall_control_y = bottom;
            }

            shells.push_back(Shell {
                char_indices: shell_indices,
            });
            idx_iter = end;
        }

        let base_launch_delay = 45usize;
        let initial_delay = (base_launch_delay as f64 * rng.gen_range(0.5..=1.5)) as usize;

        FireworksEffect {
            chars,
            shells,
            launch_delay: initial_delay,
            delay_counter: 0,
            base_launch_delay,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let mut rng = rand::thread_rng();

        if self.delay_counter == 0 {
            if let Some(shell) = self.shells.pop_back() {
                for &ci in &shell.char_indices {
                    let ch = &mut self.chars[ci];
                    ch.phase = FWPhase::Launch;
                    ch.progress = 0.0;
                    let dy = ch.origin_y - ch.launch_start_y;
                    let dx = ch.origin_x - ch.launch_start_x;
                    ch.speed = 0.35 / aspect_dist(dy, dx);
                }
                self.launch_delay =
                    (self.base_launch_delay as f64 * rng.gen_range(0.5..=1.5)) as usize;
                self.delay_counter = self.launch_delay;
            }
        } else {
            self.delay_counter -= 1;
        }

        for ch in &mut self.chars {
            match ch.phase {
                FWPhase::Waiting | FWPhase::Done => {}
                FWPhase::Launch => {
                    ch.progress = (ch.progress + ch.speed).min(1.0);
                    let eased = easing::out_expo(ch.progress);
                    ch.cur_y = ch.launch_start_y + (ch.origin_y - ch.launch_start_y) * eased;
                    ch.cur_x = ch.launch_start_x + (ch.origin_x - ch.launch_start_x) * eased;
                    if ch.progress >= 1.0 {
                        ch.phase = FWPhase::Explode;
                        ch.progress = 0.0;
                        let dy = ch.explode_target_y - ch.origin_y;
                        let dx = ch.explode_target_x - ch.origin_x;
                        ch.speed = ch.explode_speed_base / aspect_dist(dy, dx);
                    }
                }
                FWPhase::Explode => {
                    ch.progress = (ch.progress + ch.speed).min(1.0);
                    let eased = easing::out_circ(ch.progress);
                    ch.cur_y = ch.origin_y + (ch.explode_target_y - ch.origin_y) * eased;
                    ch.cur_x = ch.origin_x + (ch.explode_target_x - ch.origin_x) * eased;
                    if ch.progress >= 1.0 {
                        ch.phase = FWPhase::Bloom;
                        ch.progress = 0.0;
                        let p0 = (ch.explode_target_x, ch.explode_target_y);
                        let p1 = (ch.bloom_control_x, ch.bloom_control_y);
                        let p2 = (ch.bloom_target_x, ch.bloom_target_y);
                        ch.speed = ch.explode_speed_base / bezier_aspect_length(p0, p1, p2);
                    }
                }
                FWPhase::Bloom => {
                    ch.progress = (ch.progress + ch.speed).min(1.0);
                    let eased = easing::out_circ(ch.progress);
                    let p = quad_bezier(
                        (ch.explode_target_x, ch.explode_target_y),
                        (ch.bloom_control_x, ch.bloom_control_y),
                        (ch.bloom_target_x, ch.bloom_target_y),
                        eased,
                    );
                    ch.cur_x = p.0;
                    ch.cur_y = p.1;
                    if ch.progress >= 1.0 {
                        ch.phase = FWPhase::Fall;
                        ch.progress = 0.0;
                        let p0 = (ch.bloom_target_x, ch.bloom_target_y);
                        let p1 = (ch.fall_control_x, ch.fall_control_y);
                        let p2 = (ch.final_x as f64, ch.final_y as f64);
                        ch.speed = 0.6 / bezier_aspect_length(p0, p1, p2);
                    }
                }
                FWPhase::Fall => {
                    ch.progress = (ch.progress + ch.speed).min(1.0);
                    let eased = easing::in_out_quart(ch.progress);
                    let p = quad_bezier(
                        (ch.bloom_target_x, ch.bloom_target_y),
                        (ch.fall_control_x, ch.fall_control_y),
                        (ch.final_x as f64, ch.final_y as f64),
                        eased,
                    );
                    ch.cur_x = p.0;
                    ch.cur_y = p.1;
                    if ch.progress >= 1.0 {
                        ch.phase = FWPhase::Done;
                        ch.cur_y = ch.final_y as f64;
                        ch.cur_x = ch.final_x as f64;
                    }
                }
            }
        }

        for (y, row) in grid.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                cell.visible = false;
                cell.ch = self.original_chars[y][x];
                cell.fg = None;
            }
        }

        let white = Rgb::new(255, 255, 255);
        for ch in &self.chars {
            if ch.phase == FWPhase::Waiting {
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
            match ch.phase {
                FWPhase::Launch => {
                    cell.ch = FIREWORK_SYMBOL;
                    let blink = if (ch.progress * 60.0) as usize % 3 == 0 {
                        white
                    } else {
                        ch.shell_color
                    };
                    cell.fg = Some(blink.to_crossterm());
                }
                FWPhase::Explode => {
                    cell.ch = ch.original_ch;
                    let t = ch.progress;
                    let color = if t < 0.5 {
                        Rgb::lerp(ch.shell_color, white, t * 2.0)
                    } else {
                        Rgb::lerp(white, ch.shell_color, (t - 0.5) * 2.0)
                    };
                    cell.fg = Some(color.to_crossterm());
                }
                FWPhase::Bloom => {
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.shell_color.to_crossterm());
                }
                FWPhase::Fall => {
                    cell.ch = ch.original_ch;
                    cell.fg =
                        Some(Rgb::lerp(ch.shell_color, ch.final_color, ch.progress).to_crossterm());
                }
                FWPhase::Done => {
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.final_color.to_crossterm());
                }
                _ => {}
            }
        }

        self.shells.is_empty() && self.chars.iter().all(|c| c.phase == FWPhase::Done)
    }
}

#[cfg(test)]
#[path = "../tests/effects/fireworks.rs"]
mod tests;
