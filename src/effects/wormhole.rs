// Wormhole effect — text is shown briefly, then characters are pulled toward
// the center with increasing speed. Once consumed, they rapidly expand back
// outward from the center to their original positions with a rainbow gradient
// burst that settles into the final colors.

pub const NAME: &str = "wormhole";
pub const DESCRIPTION: &str =
    "Characters are pulled into a wormhole and violently restored with a flash.";
pub const EXTRA_EFFECT: bool = true;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

// Hold: show original text before animation starts
const HOLD_FRAMES: usize = 60;

#[derive(PartialEq)]
enum Phase {
    Hold,
    Pulling,
    Expanding,
    Done,
}

struct WHChar {
    final_y: usize,
    final_x: usize,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    hold_color: Rgb,
    final_color: Rgb,
    accent_color: Rgb,
    pull_progress: f64,
    pull_speed: f64,
    expand_progress: f64,
    expand_speed: f64,
}

pub struct WormholeEffect {
    chars: Vec<WHChar>,
    center_y: f64,
    center_x: f64,
    width: usize,
    height: usize,
    phase: Phase,
    hold_frame: usize,
}

/// Map an angle (radians) to a warp-exit accent: white → light blue.
fn warp_accent(angle: f64) -> Rgb {
    let tau = std::f64::consts::TAU;
    let t = ((angle + std::f64::consts::PI) / tau).fract();
    // light blue (130,210,255) → white (255,255,255)
    Rgb::new((130.0 + 125.0 * t) as u8, (210.0 + 45.0 * t) as u8, 255)
}

impl WormholeEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height) = (grid.width, grid.height);
        let center_y = height as f64 / 2.0;
        let center_x = width as f64 / 2.0;

        let hold_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("80E0FF"),
            ],
            9,
        );
        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("80E0FF"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("8A008A"),
            ],
            9,
        );

        let mut rng = rand::thread_rng();
        let mut chars = Vec::new();

        for (y, x) in grid.char_positions() {
            let hold_color =
                hold_gradient.color_at_coord(y, x, height, width, GradientDirection::Radial);
            let final_color =
                final_gradient.color_at_coord(y, x, height, width, GradientDirection::Radial);
            let dist = ((y as f64 - center_y).powi(2) + (x as f64 - center_x).powi(2))
                .sqrt()
                .max(1.0);
            let pull_speed = rng.gen_range(0.08..0.14) / dist;
            let expand_speed = rng.gen_range(0.15..0.25) / dist;

            // Accent color based on angle from center — creates rainbow burst
            let angle = (y as f64 - center_y).atan2(x as f64 - center_x);
            let accent_color = warp_accent(angle);

            chars.push(WHChar {
                final_y: y,
                final_x: x,
                cur_y: y as f64,
                cur_x: x as f64,
                original_ch: grid.cells[y][x].ch,
                hold_color,
                final_color,
                accent_color,
                pull_progress: 0.0,
                pull_speed,
                expand_progress: 0.0,
                expand_speed,
            });
        }

        WormholeEffect {
            chars,
            center_y,
            center_x,
            width,
            height,
            phase: if grid.char_positions().is_empty() {
                Phase::Done
            } else {
                Phase::Hold
            },
            hold_frame: 0,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        if self.phase == Phase::Done {
            for row in &mut grid.cells {
                for cell in row {
                    cell.ch = ' ';
                    cell.fg = None;
                    cell.visible = true;
                }
            }
            for ch in &self.chars {
                let cell = &mut grid.cells[ch.final_y][ch.final_x];
                cell.ch = ch.original_ch;
                cell.fg = Some(ch.final_color.to_crossterm());
            }
            return true;
        }

        match self.phase {
            Phase::Hold => self.tick_hold(),
            Phase::Pulling => self.tick_pulling(),
            Phase::Expanding => self.tick_expanding(),
            Phase::Done => {}
        }

        self.render(grid);
        false
    }

    fn tick_hold(&mut self) {
        self.hold_frame += 1;
        if self.hold_frame >= HOLD_FRAMES {
            self.phase = Phase::Pulling;
        }
    }

    fn tick_pulling(&mut self) {
        let (cy, cx) = (self.center_y, self.center_x);
        for ch in &mut self.chars {
            if ch.pull_progress >= 1.0 {
                continue;
            }
            ch.pull_progress = (ch.pull_progress + ch.pull_speed).min(1.0);
            let t = easing::in_expo(ch.pull_progress);
            ch.cur_y = ch.final_y as f64 + (cy - ch.final_y as f64) * t;
            ch.cur_x = ch.final_x as f64 + (cx - ch.final_x as f64) * t;
        }

        if self.chars.iter().all(|ch| ch.pull_progress >= 1.0) {
            for ch in &mut self.chars {
                ch.cur_y = self.center_y;
                ch.cur_x = self.center_x;
                ch.expand_progress = 0.0;
            }
            self.phase = Phase::Expanding;
        }
    }

    fn tick_expanding(&mut self) {
        let (cy, cx) = (self.center_y, self.center_x);
        for ch in &mut self.chars {
            if ch.expand_progress >= 1.0 {
                continue;
            }
            ch.expand_progress = (ch.expand_progress + ch.expand_speed).min(1.0);
            let t = easing::out_expo(ch.expand_progress);
            ch.cur_y = cy + (ch.final_y as f64 - cy) * t;
            ch.cur_x = cx + (ch.final_x as f64 - cx) * t;
        }

        if self.chars.iter().all(|ch| ch.expand_progress >= 1.0) {
            self.phase = Phase::Done;
        }
    }

    fn render(&self, grid: &mut Grid) {
        for row in &mut grid.cells {
            for cell in row {
                cell.visible = false;
            }
        }

        match self.phase {
            Phase::Hold => {
                for ch in &self.chars {
                    let cell = &mut grid.cells[ch.final_y][ch.final_x];
                    cell.visible = true;
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.hold_color.to_crossterm());
                }
            }
            Phase::Pulling => {
                for ch in &self.chars {
                    if ch.pull_progress >= 1.0 {
                        continue;
                    }
                    let ry = ch.cur_y.round() as isize;
                    let rx = ch.cur_x.round() as isize;
                    if ry < 0 || rx < 0 || ry as usize >= self.height || rx as usize >= self.width {
                        continue;
                    }
                    let cell = &mut grid.cells[ry as usize][rx as usize];
                    cell.visible = true;
                    cell.ch = ch.original_ch;
                    let brightness = 1.0 - ch.pull_progress;
                    cell.fg = Some(ch.hold_color.adjust_brightness(brightness).to_crossterm());
                }
            }
            Phase::Expanding | Phase::Done => {
                let white = Rgb::new(255, 255, 255);
                for ch in &self.chars {
                    let ry = ch.cur_y.round() as isize;
                    let rx = ch.cur_x.round() as isize;
                    if ry < 0 || rx < 0 || ry as usize >= self.height || rx as usize >= self.width {
                        continue;
                    }
                    let cell = &mut grid.cells[ry as usize][rx as usize];
                    cell.visible = true;
                    cell.ch = ch.original_ch;
                    // Gradient: white → rainbow accent → final color
                    let p = ch.expand_progress;
                    let color = if p < 0.35 {
                        Rgb::lerp(white, ch.accent_color, p / 0.35)
                    } else {
                        Rgb::lerp(ch.accent_color, ch.final_color, (p - 0.35) / 0.65)
                    };
                    cell.fg = Some(color.to_crossterm());
                }
            }
        }
    }
}
