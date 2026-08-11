// Sweep effect — faithful TTE reimplementation
// Two-phase sweep: initial gray shimmer R→L, then color sweep L→R

pub const NAME: &str = "sweep";
pub const DESCRIPTION: &str =
    "Sweep across the canvas to reveal uncolored text, reverse sweep to color the text.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

const SWEEP_SYMBOLS: [char; 4] = ['█', '▓', '▒', '░'];
const GRAY_SHADES: [Rgb; 5] = [
    Rgb {
        r: 160,
        g: 160,
        b: 160,
    },
    Rgb {
        r: 128,
        g: 128,
        b: 128,
    },
    Rgb {
        r: 64,
        g: 64,
        b: 64,
    },
    Rgb {
        r: 32,
        g: 32,
        b: 32,
    },
    Rgb {
        r: 16,
        g: 16,
        b: 16,
    },
];

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    InitialSweep,
    SecondSweep,
    Complete,
}

struct SweepChar {
    y: usize,
    x: usize,
    original_ch: char,
    final_color: Rgb,
    // Scene frames for current phase
    phase: Phase,
    frame_idx: usize,
    hold: usize,
    phase1_done: bool,
    phase2_done: bool,
    active_p1: bool,
    active_p2: bool,
    gray_color: Rgb,
}

pub struct SweepEffect {
    chars: Vec<SweepChar>,
    // Phase 1 groups: columns R→L
    groups_p1: Vec<Vec<usize>>,
    // Phase 2 groups: columns L→R
    groups_p2: Vec<Vec<usize>>,
    phase: Phase,
    easer_step: f64,
    easer_speed: f64,
    activated_up_to: usize,
    total_groups: usize,
    dm: usize,
    width: usize,
    height: usize,
    final_gradient: Gradient,
}

impl SweepEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;
        let dm: usize = 2;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("FFFFFF"),
            ],
            8,
        );

        let mut rng = rand::thread_rng();

        let mut chars = Vec::with_capacity(width * height);
        // Phase 1: columns R→L
        let mut groups_p1: Vec<Vec<usize>> = vec![Vec::new(); width];
        // Phase 2: columns L→R
        let mut groups_p2: Vec<Vec<usize>> = vec![Vec::new(); width];

        for y in 0..height {
            for (x, g2) in groups_p2.iter_mut().enumerate() {
                let final_color =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Vertical);
                let gray_idx = rng.gen_range(0..GRAY_SHADES.len());

                let idx = chars.len();
                // R→L for phase 1
                let p1_col = width.saturating_sub(1).saturating_sub(x);
                groups_p1[p1_col].push(idx);
                // L→R for phase 2
                g2.push(idx);

                chars.push(SweepChar {
                    y,
                    x,
                    original_ch: grid.cells[y][x].ch,
                    final_color,
                    phase: Phase::InitialSweep,
                    frame_idx: 0,
                    hold: 0,
                    phase1_done: false,
                    phase2_done: false,
                    active_p1: false,
                    active_p2: false,
                    gray_color: GRAY_SHADES[gray_idx],
                });
            }
        }

        groups_p1.retain(|g| !g.is_empty());
        groups_p2.retain(|g| !g.is_empty());
        let total_groups = groups_p1.len();
        let easer_speed = 1.0 / (total_groups as f64 * dm as f64).max(1.0);

        SweepEffect {
            chars,
            groups_p1,
            groups_p2,
            phase: Phase::InitialSweep,
            easer_step: 0.0,
            easer_speed,
            activated_up_to: 0,
            total_groups,
            dm,
            width,
            height,
            final_gradient,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let frames_per_symbol = 5 * self.dm;
        let _total_scene_frames = SWEEP_SYMBOLS.len() * frames_per_symbol + 1;

        // Advance easer and activate groups
        self.easer_step += self.easer_speed;
        if self.easer_step > 1.0 {
            self.easer_step = 1.0;
        }

        let eased = easing::in_out_circ(self.easer_step);
        let target = (eased * self.total_groups as f64).round() as usize;
        let target = target.min(self.total_groups);

        match self.phase {
            Phase::InitialSweep => {
                while self.activated_up_to < target {
                    for &idx in &self.groups_p1[self.activated_up_to] {
                        self.chars[idx].active_p1 = true;
                    }
                    self.activated_up_to += 1;
                }
            }
            Phase::SecondSweep => {
                while self.activated_up_to < target {
                    for &idx in &self.groups_p2[self.activated_up_to] {
                        self.chars[idx].active_p2 = true;
                    }
                    self.activated_up_to += 1;
                }
            }
            Phase::Complete => {}
        }

        // Tick all chars
        let mut all_p1_done = true;
        let mut all_p2_done = true;

        for ch in &mut self.chars {
            match self.phase {
                Phase::InitialSweep => {
                    if ch.active_p1 && !ch.phase1_done {
                        ch.hold += 1;
                        if ch.hold >= frames_per_symbol {
                            ch.hold = 0;
                            ch.frame_idx += 1;
                        }
                        if ch.frame_idx > SWEEP_SYMBOLS.len() {
                            ch.phase1_done = true;
                        } else {
                            all_p1_done = false;
                        }
                    } else if !ch.active_p1 {
                        all_p1_done = false;
                    }
                }
                Phase::SecondSweep => {
                    if ch.active_p2 && !ch.phase2_done {
                        ch.hold += 1;
                        if ch.hold >= frames_per_symbol {
                            ch.hold = 0;
                            ch.frame_idx += 1;
                        }
                        if ch.frame_idx > SWEEP_SYMBOLS.len() {
                            ch.phase2_done = true;
                        } else {
                            all_p2_done = false;
                        }
                    } else if !ch.active_p2 {
                        all_p2_done = false;
                    }
                }
                Phase::Complete => {}
            }
        }

        // Phase transitions
        match self.phase {
            Phase::InitialSweep => {
                if all_p1_done && self.activated_up_to >= self.total_groups {
                    self.phase = Phase::SecondSweep;
                    self.easer_step = 0.0;
                    self.activated_up_to = 0;
                    self.total_groups = self.groups_p2.len();
                    self.easer_speed = 1.0 / (self.total_groups as f64 * self.dm as f64).max(1.0);
                    for ch in &mut self.chars {
                        ch.frame_idx = 0;
                        ch.hold = 0;
                    }
                }
            }
            Phase::SecondSweep => {
                if all_p2_done && self.activated_up_to >= self.total_groups {
                    self.phase = Phase::Complete;
                }
            }
            Phase::Complete => {}
        }

        // Render
        for ch in &self.chars {
            if ch.y >= grid.height || ch.x >= grid.width {
                continue;
            }
            let cell = &mut grid.cells[ch.y][ch.x];

            match self.phase {
                Phase::InitialSweep => {
                    if ch.active_p1 {
                        cell.visible = true;
                        if ch.phase1_done || ch.frame_idx >= SWEEP_SYMBOLS.len() {
                            cell.ch = ch.original_ch;
                            cell.fg = Some(Rgb::new(128, 128, 128).to_crossterm());
                        } else {
                            cell.ch = SWEEP_SYMBOLS[ch.frame_idx];
                            cell.fg = Some(ch.gray_color.to_crossterm());
                        }
                    }
                }
                Phase::SecondSweep => {
                    cell.visible = true;
                    if ch.active_p2 {
                        if ch.phase2_done || ch.frame_idx >= SWEEP_SYMBOLS.len() {
                            cell.ch = ch.original_ch;
                            cell.fg = Some(ch.final_color.to_crossterm());
                        } else {
                            cell.ch = SWEEP_SYMBOLS[ch.frame_idx];
                            let t = ch.frame_idx as f64 / SWEEP_SYMBOLS.len() as f64;
                            cell.fg = Some(
                                Rgb::lerp(Rgb::new(128, 128, 128), ch.final_color, t)
                                    .to_crossterm(),
                            );
                        }
                    } else {
                        cell.ch = ch.original_ch;
                        cell.fg = Some(Rgb::new(128, 128, 128).to_crossterm());
                    }
                }
                Phase::Complete => {
                    cell.visible = true;
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.final_color.to_crossterm());
                }
            }
        }

        self.phase == Phase::Complete
    }
}
