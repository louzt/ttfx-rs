// Unstable effect — rumble jitter, explosion, reassembly

pub const NAME: &str = "unstable";
pub const DESCRIPTION: &str = "Spawn characters jumbled, explode them to the edge of the canvas, then reassemble them in the correct layout.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Rumble,
    Explode,
    Hold,
    Reassemble,
    Done,
}

struct UnstableChar {
    final_y: usize,
    final_x: usize,
    cur_y: f64,
    cur_x: f64,
    explode_y: f64,
    explode_x: f64,
    original_ch: char,
    final_color: Rgb,
    phase: Phase,
    progress: f64,
    rumble_color_step: usize,
}

pub struct UnstableEffect {
    chars: Vec<UnstableChar>,
    frame: usize,
    dm: usize,
    width: usize,
    height: usize,
    global_phase: Phase,
    rumble_steps: usize,
    hold_counter: usize,
    rumble_gradient: Vec<Rgb>,
    final_gradient_spec: Vec<Rgb>,
}

impl UnstableEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height, dm) = (grid.width, grid.height, 2usize);
        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("FFFFFF"),
            ],
            12,
        );
        let unstable_color = Rgb::from_hex("ff9200");

        let mut chars = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let fc =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Vertical);
                chars.push(UnstableChar {
                    final_y: y,
                    final_x: x,
                    cur_y: y as f64,
                    cur_x: x as f64,
                    explode_y: 0.0,
                    explode_x: 0.0,
                    original_ch: grid.cells[y][x].ch,
                    final_color: fc,
                    phase: Phase::Rumble,
                    progress: 0.0,
                    rumble_color_step: 0,
                });
            }
        }

        // Build rumble color gradient: final_color → unstable_color (12 steps)
        let rumble_grad: Vec<Rgb> = (0..12)
            .map(|i| {
                let t = i as f64 / 11.0;
                Rgb::lerp(Rgb::from_hex("8A008A"), unstable_color, t)
            })
            .collect();

        let final_spec: Vec<Rgb> = (0..12)
            .map(|i| {
                let t = i as f64 / 11.0;
                Rgb::lerp(unstable_color, Rgb::from_hex("8A008A"), t)
            })
            .collect();

        UnstableEffect {
            chars,
            frame: 0,
            dm,
            width,
            height,
            global_phase: Phase::Rumble,
            rumble_steps: 150 * dm,
            hold_counter: 0,
            rumble_gradient: rumble_grad,
            final_gradient_spec: final_spec,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        self.frame += 1;
        let dm = self.dm;
        let mut rng = rand::thread_rng();

        match self.global_phase {
            Phase::Rumble => {
                let max_rumble = self.rumble_steps;
                let jitter_intensity = (self.frame as f64 / max_rumble as f64).min(1.0) * 2.0;

                for ch in &mut self.chars {
                    ch.cur_y =
                        ch.final_y as f64 + rng.gen_range(-jitter_intensity..jitter_intensity);
                    ch.cur_x =
                        ch.final_x as f64 + rng.gen_range(-jitter_intensity..jitter_intensity);
                    let step = ((self.frame as f64 / max_rumble as f64) * 11.0) as usize;
                    ch.rumble_color_step = step.min(11);
                }

                if self.frame >= max_rumble {
                    // Assign explosion targets (random edge positions)
                    for ch in &mut self.chars {
                        let edge = rng.gen_range(0..4);
                        let (ey, ex) = match edge {
                            0 => (
                                rng.gen_range(0..self.height) as f64,
                                -(rng.gen_range(5..20) as f64),
                            ),
                            1 => (
                                rng.gen_range(0..self.height) as f64,
                                (self.width + rng.gen_range(5..20)) as f64,
                            ),
                            2 => (
                                -(rng.gen_range(5..20) as f64),
                                rng.gen_range(0..self.width) as f64,
                            ),
                            _ => (
                                (self.height + rng.gen_range(5..20)) as f64,
                                rng.gen_range(0..self.width) as f64,
                            ),
                        };
                        ch.explode_y = ey;
                        ch.explode_x = ex;
                        ch.phase = Phase::Explode;
                        ch.progress = 0.0;
                    }
                    self.global_phase = Phase::Explode;
                }
            }
            Phase::Explode => {
                let mut all_exploded = true;
                for ch in &mut self.chars {
                    if ch.phase != Phase::Explode {
                        continue;
                    }
                    ch.progress += 1.0 / (dm as f64 * 20.0);
                    if ch.progress >= 1.0 {
                        ch.progress = 1.0;
                        ch.phase = Phase::Hold;
                    } else {
                        all_exploded = false;
                    }
                    let eased = easing::out_expo(ch.progress);
                    ch.cur_y = ch.final_y as f64 + (ch.explode_y - ch.final_y as f64) * eased;
                    ch.cur_x = ch.final_x as f64 + (ch.explode_x - ch.final_x as f64) * eased;
                }
                if all_exploded {
                    self.global_phase = Phase::Hold;
                    self.hold_counter = 0;
                }
            }
            Phase::Hold => {
                self.hold_counter += 1;
                if self.hold_counter >= 30 * dm {
                    for ch in &mut self.chars {
                        ch.phase = Phase::Reassemble;
                        ch.progress = 0.0;
                    }
                    self.global_phase = Phase::Reassemble;
                }
            }
            Phase::Reassemble => {
                let mut all_done = true;
                for ch in &mut self.chars {
                    if ch.phase == Phase::Done {
                        continue;
                    }
                    ch.progress += 1.0 / (dm as f64 * 25.0);
                    if ch.progress >= 1.0 {
                        ch.progress = 1.0;
                        ch.phase = Phase::Done;
                        ch.cur_y = ch.final_y as f64;
                        ch.cur_x = ch.final_x as f64;
                    } else {
                        all_done = false;
                        let eased = easing::out_expo(ch.progress);
                        ch.cur_y = ch.explode_y + (ch.final_y as f64 - ch.explode_y) * eased;
                        ch.cur_x = ch.explode_x + (ch.final_x as f64 - ch.explode_x) * eased;
                    }
                }
                if all_done {
                    self.global_phase = Phase::Done;
                }
            }
            Phase::Done => return true,
        }

        // Render
        for row in &mut grid.cells {
            for cell in row {
                cell.visible = false;
            }
        }
        let unstable_color = Rgb::from_hex("ff9200");
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
            match ch.phase {
                Phase::Rumble => {
                    cell.fg = Some(self.rumble_gradient[ch.rumble_color_step].to_crossterm());
                }
                Phase::Explode | Phase::Hold => {
                    cell.fg = Some(unstable_color.to_crossterm());
                }
                Phase::Reassemble => {
                    let step = (ch.progress * 11.0) as usize;
                    cell.fg = Some(self.final_gradient_spec[step.min(11)].to_crossterm());
                }
                Phase::Done => {
                    cell.fg = Some(ch.final_color.to_crossterm());
                }
            }
        }
        false
    }
}
