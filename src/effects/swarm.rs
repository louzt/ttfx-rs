// Swarm effect — grouped swarm movement through areas before settling

pub const NAME: &str = "swarm";
pub const DESCRIPTION: &str = "Characters are grouped into swarms and move around the terminal before settling into position.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;

struct SwarmChar {
    final_y: usize,
    final_x: usize,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    final_color: Rgb,
    swarm_idx: usize,
    progress: f64,
    speed: f64,
    done: bool,
}

struct SwarmGroup {
    targets: Vec<(f64, f64)>, // waypoints
    target_idx: usize,
    progress: f64,
    speed: f64,
    cur_y: f64,
    cur_x: f64,
    settling: bool,
}

pub struct SwarmEffect {
    chars: Vec<SwarmChar>,
    swarms: Vec<SwarmGroup>,
    dm: usize,
    width: usize,
    height: usize,
}

impl SwarmEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height, dm) = (grid.width, grid.height, 2usize);
        let final_gradient = Gradient::new(&[Rgb::from_hex("31b900"), Rgb::from_hex("f0ff65")], 12);
        let mut rng = rand::thread_rng();
        let total = width * height;
        let swarm_size = ((total as f64 * 0.1) as usize).max(1);

        let mut chars = Vec::with_capacity(total);
        let mut indices: Vec<usize> = (0..total).collect();
        indices.shuffle(&mut rng);

        for y in 0..height {
            for x in 0..width {
                let fc = final_gradient.color_at_coord(
                    y,
                    x,
                    height,
                    width,
                    GradientDirection::Horizontal,
                );
                chars.push(SwarmChar {
                    final_y: y,
                    final_x: x,
                    cur_y: 0.0,
                    cur_x: 0.0,
                    original_ch: grid.cells[y][x].ch,
                    final_color: fc,
                    swarm_idx: 0,
                    progress: 0.0,
                    speed: 0.0,
                    done: false,
                });
            }
        }

        let mut swarms = Vec::new();
        let mut pos = 0;
        let mut swarm_id = 0;
        while pos < total {
            let size = swarm_size.min(total - pos);
            let area_count = rng.gen_range(2..=4);
            let mut targets = Vec::new();
            for _ in 0..area_count {
                let ty = rng.gen_range(0..height.max(1)) as f64;
                let tx = rng.gen_range(0..width.max(1)) as f64;
                targets.push((ty, tx));
            }
            let start_y = rng.gen_range(0..height.max(1)) as f64;
            let start_x = rng.gen_range(0..width.max(1)) as f64;

            for &ci in &indices[pos..pos + size] {
                chars[ci].swarm_idx = swarm_id;
                chars[ci].cur_y = start_y;
                chars[ci].cur_x = start_x;
            }

            swarms.push(SwarmGroup {
                targets,
                target_idx: 0,
                progress: 0.0,
                speed: 0.4 / dm as f64,
                cur_y: start_y,
                cur_x: start_x,
                settling: false,
            });
            swarm_id += 1;
            pos += size;
        }

        SwarmEffect {
            chars,
            swarms,
            dm,
            width,
            height,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let dm = self.dm;
        // Move swarm groups through waypoints
        for sg in &mut self.swarms {
            if sg.settling {
                continue;
            }
            if sg.target_idx >= sg.targets.len() {
                sg.settling = true;
                continue;
            }
            let (ty, tx) = sg.targets[sg.target_idx];
            let dist = ((ty - sg.cur_y).powi(2) + (tx - sg.cur_x).powi(2)).sqrt();
            if dist < 1.0 {
                sg.target_idx += 1;
            } else {
                let step = sg.speed;
                sg.cur_y += (ty - sg.cur_y) / dist * step * dist.min(3.0);
                sg.cur_x += (tx - sg.cur_x) / dist * step * dist.min(3.0);
            }
        }

        let mut all_done = true;
        for ch in &mut self.chars {
            if ch.done {
                continue;
            }
            let sg = &self.swarms[ch.swarm_idx];
            if sg.settling {
                ch.progress += 0.45
                    / ((ch.final_y as f64 - ch.cur_y).powi(2)
                        + (ch.final_x as f64 - ch.cur_x).powi(2))
                    .sqrt()
                    .max(1.0)
                    / dm as f64;
                if ch.progress >= 1.0 {
                    ch.progress = 1.0;
                    ch.done = true;
                }
                let start_y = sg.cur_y;
                let start_x = sg.cur_x;
                let eased = easing::in_out_quad(ch.progress);
                ch.cur_y = start_y + (ch.final_y as f64 - start_y) * eased;
                ch.cur_x = start_x + (ch.final_x as f64 - start_x) * eased;
            } else {
                ch.cur_y = sg.cur_y + (rand::thread_rng().gen_range(-1.0..1.0));
                ch.cur_x = sg.cur_x + (rand::thread_rng().gen_range(-1.0..1.0));
            }
            if !ch.done {
                all_done = false;
            }
        }

        for row in &mut grid.cells {
            for cell in row {
                cell.visible = false;
            }
        }
        let base_color = Rgb::from_hex("31a0d4");
        let flash_color = Rgb::from_hex("f2ea79");

        for ch in &self.chars {
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
            if ch.done {
                cell.fg = Some(ch.final_color.to_crossterm());
            } else if self.swarms[ch.swarm_idx].settling {
                cell.fg = Some(Rgb::lerp(flash_color, ch.final_color, ch.progress).to_crossterm());
            } else {
                cell.fg = Some(base_color.to_crossterm());
            }
        }
        all_done
    }
}
