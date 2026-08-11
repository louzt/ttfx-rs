// Crumble effect — weaken, dust fall, vacuum up, reset to position

pub const NAME: &str = "crumble";
pub const DESCRIPTION: &str =
    "Characters lose color and crumble into dust, vacuumed up, and reformed.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::VecDeque;

const DUST_SYMBOLS: [char; 3] = ['*', '.', ','];

#[derive(Clone, Copy, PartialEq, Debug)]
enum CrumblePhase {
    Pending,
    Weakening,
    Falling,
    Landed,
    Vacuuming,
    VacuumComplete,
    Resetting,
    Done,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    Falling,
    Vacuuming,
    Resetting,
    Complete,
}

struct CrumbleChar {
    final_y: usize,
    final_x: usize,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    final_color: Rgb,
    weak_color: Rgb,
    dust_color: Rgb,
    phase: CrumblePhase,
    weaken_frame: usize,
    weaken_total: usize,
    fall_progress: f64,
    fall_speed: f64,
    fall_start_y: f64,
    fall_end_y: f64,
    vacuum_progress: f64,
    vacuum_speed: f64,
    vacuum_start: (f64, f64),
    vacuum_ctrl: (f64, f64),
    vacuum_end: (f64, f64),
    reset_progress: f64,
    reset_speed: f64,
    reset_start_y: f64,
    flash_frame: usize,
    flash_total: usize,
    strengthen_frame: usize,
    strengthen_total: usize,
}

pub struct CrumbleEffect {
    chars: Vec<CrumbleChar>,
    pending: VecDeque<usize>,
    unvacuumed: VecDeque<usize>,
    stage: Stage,
    fall_delay: usize,
    min_fall_delay: usize,
    max_fall_delay: usize,
    fall_group_maxsize: usize,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn quad_bezier(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let x = u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0;
    let y = u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1;
    (x, y)
}

fn bezier_length(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64)) -> f64 {
    let mut len = 0.0;
    let mut prev = p0;
    for i in 1..=20 {
        let t = i as f64 / 20.0;
        let cur = quad_bezier(p0, p1, p2, t);
        let dx = cur.0 - prev.0;
        let dy = cur.1 - prev.1;
        len += (dx * dx + (dy * 2.0).powi(2)).sqrt();
        prev = cur;
    }
    len
}

impl CrumbleEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height) = (grid.width, grid.height);
        let final_gradient = Gradient::new(&[Rgb::from_hex("5CE1FF"), Rgb::from_hex("FF8C00")], 12);

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

        let bottom = (height as f64) - 1.0;
        let center = (width as f64 / 2.0, height as f64 / 2.0);

        let mut chars: Vec<CrumbleChar> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let ch = grid.cells[y][x].ch;
                if ch == ' ' {
                    continue;
                }
                let ry = y.saturating_sub(text_top);
                let rx = x.saturating_sub(text_left);
                let final_color = final_gradient.color_at_coord(
                    ry,
                    rx,
                    text_h,
                    text_w,
                    GradientDirection::Diagonal,
                );
                let weak_color = final_color.adjust_brightness(0.65);
                let dust_color = final_color.adjust_brightness(0.55);

                let fall_dist = (bottom - y as f64).abs().max(1.0);
                let fall_speed = 0.65 / (2.0 * fall_dist);

                let v_start = (x as f64, bottom);
                let v_end = (x as f64, 0.0);
                let v_len = bezier_length(v_start, center, v_end).max(1.0);
                let vacuum_speed = 1.0 / v_len;

                let reset_dist = (y as f64).max(1.0);
                let reset_speed = 1.0 / (2.0 * reset_dist);

                chars.push(CrumbleChar {
                    final_y: y,
                    final_x: x,
                    cur_y: y as f64,
                    cur_x: x as f64,
                    original_ch: ch,
                    final_color,
                    weak_color,
                    dust_color,
                    phase: CrumblePhase::Pending,
                    weaken_frame: 0,
                    weaken_total: 9 * 4,
                    fall_progress: 0.0,
                    fall_speed,
                    fall_start_y: y as f64,
                    fall_end_y: bottom,
                    vacuum_progress: 0.0,
                    vacuum_speed,
                    vacuum_start: v_start,
                    vacuum_ctrl: center,
                    vacuum_end: v_end,
                    reset_progress: 0.0,
                    reset_speed,
                    reset_start_y: 0.0,
                    flash_frame: 0,
                    flash_total: 6 * 4,
                    strengthen_frame: 0,
                    strengthen_total: 9 * 4,
                });
            }
        }

        let mut rng = rand::thread_rng();
        let mut indices: Vec<usize> = (0..chars.len()).collect();
        indices.shuffle(&mut rng);
        let pending: VecDeque<usize> = indices.into_iter().collect();
        let mut vac_indices: Vec<usize> = (0..chars.len()).collect();
        vac_indices.shuffle(&mut rng);
        let unvacuumed: VecDeque<usize> = vac_indices.into_iter().collect();

        CrumbleEffect {
            chars,
            pending,
            unvacuumed,
            stage: Stage::Falling,
            fall_delay: 12,
            min_fall_delay: 9,
            max_fall_delay: 12,
            fall_group_maxsize: 1,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let mut rng = rand::thread_rng();

        match self.stage {
            Stage::Falling => {
                if !self.pending.is_empty() {
                    if self.fall_delay == 0 {
                        let group_size = rng.gen_range(1..=self.fall_group_maxsize.max(1));
                        for _ in 0..group_size {
                            if let Some(i) = self.pending.pop_front() {
                                self.chars[i].phase = CrumblePhase::Weakening;
                            }
                        }
                        self.fall_delay = rng.gen_range(self.min_fall_delay..=self.max_fall_delay);
                        if rng.gen_range(1..=10) > 4 {
                            self.fall_group_maxsize += 1;
                            self.min_fall_delay = self.min_fall_delay.saturating_sub(1);
                            self.max_fall_delay = self.max_fall_delay.saturating_sub(1);
                        }
                    } else {
                        self.fall_delay -= 1;
                    }
                }
                let any_active = self
                    .chars
                    .iter()
                    .any(|c| matches!(c.phase, CrumblePhase::Weakening | CrumblePhase::Falling));
                if self.pending.is_empty() && !any_active {
                    self.stage = Stage::Vacuuming;
                }
            }
            Stage::Vacuuming => {
                if !self.unvacuumed.is_empty() {
                    let n = rng.gen_range(3..=10);
                    for _ in 0..n {
                        if let Some(i) = self.unvacuumed.pop_front() {
                            let ch = &mut self.chars[i];
                            ch.phase = CrumblePhase::Vacuuming;
                            ch.vacuum_progress = 0.0;
                            ch.vacuum_start = (ch.cur_x, ch.cur_y);
                        }
                    }
                }
                let any_vacuuming = self
                    .chars
                    .iter()
                    .any(|c| c.phase == CrumblePhase::Vacuuming);
                if self.unvacuumed.is_empty() && !any_vacuuming {
                    self.stage = Stage::Resetting;
                    for ch in &mut self.chars {
                        ch.phase = CrumblePhase::Resetting;
                        ch.reset_progress = 0.0;
                        ch.reset_start_y = ch.cur_y;
                        ch.flash_frame = 0;
                        ch.strengthen_frame = 0;
                    }
                }
            }
            Stage::Resetting => {
                let all_done = self.chars.iter().all(|c| c.phase == CrumblePhase::Done);
                if all_done {
                    self.stage = Stage::Complete;
                }
            }
            Stage::Complete => {}
        }

        for ch in &mut self.chars {
            match ch.phase {
                CrumblePhase::Pending | CrumblePhase::Done => {}
                CrumblePhase::Weakening => {
                    ch.weaken_frame += 1;
                    if ch.weaken_frame >= ch.weaken_total {
                        ch.phase = CrumblePhase::Falling;
                        ch.fall_progress = 0.0;
                    }
                }
                CrumblePhase::Falling => {
                    ch.fall_progress = (ch.fall_progress + ch.fall_speed).min(1.0);
                    let eased = easing::out_bounce(ch.fall_progress);
                    ch.cur_y = ch.fall_start_y + (ch.fall_end_y - ch.fall_start_y) * eased;
                    if ch.fall_progress >= 1.0 {
                        ch.phase = CrumblePhase::Landed;
                    }
                }
                CrumblePhase::Landed => {}
                CrumblePhase::Vacuuming => {
                    ch.vacuum_progress = (ch.vacuum_progress + ch.vacuum_speed).min(1.0);
                    let eased = easing::out_quint(ch.vacuum_progress);
                    let p = quad_bezier(ch.vacuum_start, ch.vacuum_ctrl, ch.vacuum_end, eased);
                    ch.cur_x = p.0;
                    ch.cur_y = p.1;
                    if ch.vacuum_progress >= 1.0 {
                        ch.phase = CrumblePhase::VacuumComplete;
                    }
                }
                CrumblePhase::VacuumComplete => {}
                CrumblePhase::Resetting => {
                    if ch.reset_progress < 1.0 {
                        ch.reset_progress = (ch.reset_progress + ch.reset_speed).min(1.0);
                        ch.cur_y = ch.reset_start_y
                            + (ch.final_y as f64 - ch.reset_start_y) * ch.reset_progress;
                        ch.cur_x = ch.final_x as f64;
                    } else if ch.flash_frame < ch.flash_total {
                        ch.flash_frame += 1;
                    } else if ch.strengthen_frame < ch.strengthen_total {
                        ch.strengthen_frame += 1;
                    } else {
                        ch.phase = CrumblePhase::Done;
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
            match ch.phase {
                CrumblePhase::Pending => {
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.weak_color.to_crossterm());
                }
                CrumblePhase::Weakening => {
                    cell.ch = ch.original_ch;
                    let t = ch.weaken_frame as f64 / ch.weaken_total as f64;
                    let c = Rgb::lerp(ch.weak_color, ch.dust_color, t);
                    cell.fg = Some(c.to_crossterm());
                }
                CrumblePhase::Falling => {
                    let dist = (ch.cur_y - ch.fall_start_y).abs();
                    let idx = (dist as usize) % DUST_SYMBOLS.len();
                    cell.ch = DUST_SYMBOLS[idx];
                    cell.fg = Some(ch.dust_color.to_crossterm());
                }
                CrumblePhase::Landed => {
                    cell.ch = DUST_SYMBOLS[0];
                    cell.fg = Some(ch.dust_color.to_crossterm());
                }
                CrumblePhase::Vacuuming | CrumblePhase::VacuumComplete => {
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.dust_color.to_crossterm());
                }
                CrumblePhase::Resetting => {
                    cell.ch = ch.original_ch;
                    if ch.reset_progress < 1.0 {
                        cell.fg = Some(ch.dust_color.to_crossterm());
                    } else if ch.flash_frame < ch.flash_total {
                        let t = ch.flash_frame as f64 / ch.flash_total as f64;
                        let c = Rgb::lerp(ch.final_color, Rgb::new(255, 255, 255), t);
                        cell.fg = Some(c.to_crossterm());
                    } else {
                        let t = ch.strengthen_frame as f64 / ch.strengthen_total as f64;
                        let c = Rgb::lerp(Rgb::new(255, 255, 255), ch.final_color, t);
                        cell.fg = Some(c.to_crossterm());
                    }
                }
                CrumblePhase::Done => {
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.final_color.to_crossterm());
                }
            }
        }

        self.stage == Stage::Complete
    }
}

#[cfg(test)]
#[path = "../tests/effects/crumble.rs"]
mod tests;
