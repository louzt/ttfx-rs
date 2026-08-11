// Rings effect — chars disperse from input positions to wander locally, then
// condense onto concentric spinning rings. After cycling between disperse and
// spin phases, every char travels back home. Characters with no ring slot
// drift off-canvas during the disperse phase and reappear during final.

pub const NAME: &str = "rings";
pub const DESCRIPTION: &str = "Characters are dispersed and form into spinning rings.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    Start,
    Disperse,
    Spin,
    Final,
    Complete,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum SubPhase {
    Idle,      // sitting at home with final color (Start phase)
    Initial,   // ring char: home → first wander target (speed 0.3, out_cubic)
    Wander,    // ring char: looped pick-and-move within ring_gap rect (speed 0.14)
    Condense,  // ring char: cur → ring slot start (speed 0.1)
    Orbit,     // ring char: traversing ring waypoints (speed = ring.rotation_speed)
    External,  // non-ring char: home → off-canvas (speed 0.8, out_sine)
    OffCanvas, // non-ring char: arrived off-canvas, invisible
    Home,      // any char: cur → home (speed 0.8, out_quad)
    Settled,   // home reached
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ColorAnim {
    Solid,
    ToFinal, // ring → final
    ToRing,  // final → ring
}

struct CharState {
    home_y: usize,
    home_x: usize,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    final_color: Rgb,
    ring_color: Rgb,
    is_ring: bool,
    visible: bool,

    // Ring assignment (only meaningful when is_ring)
    ring_idx: usize,
    slot: usize, // current ring slot index (advances on each Orbit waypoint completion)

    // Wander origin (the ring slot's coord; disperse waypoints are within ring_gap of this)
    wander_cy: f64,
    wander_cx: f64,
    wander_r: f64,

    // Current motion
    sub: SubPhase,
    sx: f64,
    sy: f64,
    tx: f64,
    ty: f64,
    progress: f64,
    ease: fn(f64) -> f64,
    speed: f64, // cells per frame

    // Color animation: linear lerp over color_total frames
    color_anim: ColorAnim,
    color_frame: usize,
    color_total: usize,
}

struct Ring {
    coords: Vec<(f64, f64)>, // (y, x) waypoints around the circle
    rotation_speed: f64,
    ring_color: Rgb,
    clockwise: bool,
}

pub struct RingsEffect {
    chars: Vec<CharState>,
    rings: Vec<Ring>,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,

    phase: Phase,
    initial_phase_remaining: usize,
    disperse_remaining: usize,
    spin_remaining: usize,
    cycles_remaining: usize,

    spin_duration: usize,
    disperse_duration: usize,
    ring_gap: f64,
}

fn dist(sy: f64, sx: f64, ty: f64, tx: f64) -> f64 {
    ((ty - sy).powi(2) + (tx - sx).powi(2)).sqrt()
}

fn pick_wander_target(rng: &mut impl Rng, cy: f64, cx: f64, r: f64) -> (f64, f64) {
    // TTE picks a coord within a rect of half-side ring_gap around origin.
    let ty = cy + rng.gen_range(-r..=r);
    let tx = cx + rng.gen_range(-r..=r);
    (ty, tx)
}

impl RingsEffect {
    pub fn new(grid: &Grid) -> Self {
        let mut rng = rand::thread_rng();
        let width = grid.width;
        let height = grid.height;
        let center_y = (height as f64 - 1.0) / 2.0;
        let center_x = (width as f64 - 1.0) / 2.0;

        let original_chars: Vec<Vec<char>> = grid
            .cells
            .iter()
            .map(|row| row.iter().map(|c| c.ch).collect())
            .collect();

        // Text bounds for the final gradient.
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

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("ab48ff"),
                Rgb::from_hex("e7b2b2"),
                Rgb::from_hex("fffebd"),
            ],
            12,
        );
        let ring_color_palette = [
            Rgb::from_hex("ab48ff"),
            Rgb::from_hex("e7b2b2"),
            Rgb::from_hex("fffebd"),
        ];

        // TTE: ring_gap = max(round(min(canvas.top, canvas.right) * 0.1), 1)
        let ring_gap_int = ((width.min(height) as f64 * 0.1).round() as usize).max(1);
        let ring_gap = ring_gap_int as f64;

        // Build rings: for radius in 1..max(W, H) step ring_gap, generate
        // 7*r unique points on the circle. Stop when <25% are in canvas.
        let mut rings: Vec<Ring> = Vec::new();
        let max_radius = width.max(height);
        let mut r = 1usize;
        while r < max_radius {
            let n_points = (7 * r).max(1);
            let mut coords: Vec<(f64, f64)> = Vec::with_capacity(n_points);
            let mut seen = std::collections::HashSet::new();
            for k in 0..n_points {
                let theta = (k as f64) * std::f64::consts::TAU / n_points as f64;
                // TTE doubles x-distance from origin to correct for terminal
                // cell aspect ratio (cells are ~2:1 height:width).
                let cy = center_y + (r as f64) * theta.sin();
                let cx = center_x + 2.0 * (r as f64) * theta.cos();
                let key = (cy.round() as i64, cx.round() as i64);
                if seen.insert(key) {
                    coords.push((cy, cx));
                }
            }
            if coords.is_empty() {
                r += ring_gap_int;
                continue;
            }
            let in_canvas = coords
                .iter()
                .filter(|(cy, cx)| {
                    let ry = cy.round();
                    let rx = cx.round();
                    ry >= 0.0 && ry < height as f64 && rx >= 0.0 && rx < width as f64
                })
                .count();
            if (in_canvas as f64) / (coords.len() as f64) < 0.25 {
                break;
            }
            let rotation_speed = rng.gen_range(0.25..=1.0);
            let ring_color = ring_color_palette[rings.len() % ring_color_palette.len()];
            let clockwise = rings.len() % 2 == 1;
            rings.push(Ring {
                coords,
                rotation_speed,
                ring_color,
                clockwise,
            });
            r += ring_gap_int;
        }

        // Collect non-space chars in shuffled order.
        let mut pending: Vec<(usize, usize, char, Rgb)> = Vec::new();
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
                    GradientDirection::Vertical,
                );
                pending.push((y, x, ch, final_color));
            }
        }
        pending.shuffle(&mut rng);

        // Assign chars round-robin into ring slots; remaining are non-ring.
        let mut chars: Vec<CharState> = Vec::new();
        let mut idx = 0usize;
        'outer: for (ring_i, ring) in rings.iter().enumerate() {
            for slot_i in 0..ring.coords.len() {
                if idx >= pending.len() {
                    break 'outer;
                }
                let (hy, hx, ch, fc) = pending[idx];
                let (sy, sx) = ring.coords[slot_i];
                chars.push(CharState {
                    home_y: hy,
                    home_x: hx,
                    cur_y: hy as f64,
                    cur_x: hx as f64,
                    original_ch: ch,
                    final_color: fc,
                    ring_color: ring.ring_color,
                    is_ring: true,
                    visible: true,
                    ring_idx: ring_i,
                    slot: slot_i,
                    wander_cy: sy,
                    wander_cx: sx,
                    wander_r: ring_gap,
                    sub: SubPhase::Idle,
                    sx: hx as f64,
                    sy: hy as f64,
                    tx: hx as f64,
                    ty: hy as f64,
                    progress: 0.0,
                    ease: easing::linear,
                    speed: 0.0,
                    color_anim: ColorAnim::Solid,
                    color_frame: 0,
                    color_total: 1,
                });
                idx += 1;
            }
        }
        // Non-ring chars
        while idx < pending.len() {
            let (hy, hx, ch, fc) = pending[idx];
            chars.push(CharState {
                home_y: hy,
                home_x: hx,
                cur_y: hy as f64,
                cur_x: hx as f64,
                original_ch: ch,
                final_color: fc,
                ring_color: Rgb::new(0, 0, 0),
                is_ring: false,
                visible: true,
                ring_idx: 0,
                slot: 0,
                wander_cy: 0.0,
                wander_cx: 0.0,
                wander_r: 0.0,
                sub: SubPhase::Idle,
                sx: hx as f64,
                sy: hy as f64,
                tx: hx as f64,
                ty: hy as f64,
                progress: 0.0,
                ease: easing::linear,
                speed: 0.0,
                color_anim: ColorAnim::Solid,
                color_frame: 0,
                color_total: 1,
            });
            idx += 1;
        }

        RingsEffect {
            chars,
            rings,
            width,
            height,
            original_chars,
            phase: Phase::Start,
            initial_phase_remaining: 100,
            disperse_remaining: 200,
            spin_remaining: 200,
            cycles_remaining: 3,
            spin_duration: 200,
            disperse_duration: 200,
            ring_gap,
        }
    }

    fn begin_initial_disperse(&mut self) {
        let mut rng = rand::thread_rng();
        let h = self.height as f64;
        let w = self.width as f64;
        for ch in &mut self.chars {
            if ch.is_ring {
                let (ty, tx) =
                    pick_wander_target(&mut rng, ch.wander_cy, ch.wander_cx, ch.wander_r);
                ch.sx = ch.cur_x;
                ch.sy = ch.cur_y;
                ch.tx = tx;
                ch.ty = ty;
                ch.progress = 0.0;
                ch.ease = easing::out_cubic;
                ch.speed = 0.3;
                ch.sub = SubPhase::Initial;
                ch.color_anim = ColorAnim::ToFinal;
                ch.color_frame = 0;
                ch.color_total = 80; // Gradient(8 colors) × 10 frames
            } else {
                let (ty, tx) = match rng.gen_range(0..4) {
                    0 => (
                        -1.0 - rng.gen_range(0.0..h.max(1.0)),
                        rng.gen_range(0.0..w.max(1.0)),
                    ),
                    1 => (
                        h + rng.gen_range(0.0..h.max(1.0)),
                        rng.gen_range(0.0..w.max(1.0)),
                    ),
                    2 => (
                        rng.gen_range(0.0..h.max(1.0)),
                        -1.0 - rng.gen_range(0.0..w.max(1.0)),
                    ),
                    _ => (
                        rng.gen_range(0.0..h.max(1.0)),
                        w + rng.gen_range(0.0..w.max(1.0)),
                    ),
                };
                ch.sx = ch.cur_x;
                ch.sy = ch.cur_y;
                ch.tx = tx;
                ch.ty = ty;
                ch.progress = 0.0;
                ch.ease = easing::out_sine;
                ch.speed = 0.8;
                ch.sub = SubPhase::External;
                ch.color_anim = ColorAnim::Solid;
            }
        }
    }

    fn begin_subsequent_disperse(&mut self) {
        let mut rng = rand::thread_rng();
        for ch in &mut self.chars {
            if !ch.is_ring {
                continue; // non-ring chars stay off-canvas (invisible)
            }
            // Pick a new wander target near current position.
            ch.wander_cy = ch.cur_y;
            ch.wander_cx = ch.cur_x;
            let (ty, tx) = pick_wander_target(&mut rng, ch.wander_cy, ch.wander_cx, ch.wander_r);
            ch.sx = ch.cur_x;
            ch.sy = ch.cur_y;
            ch.tx = tx;
            ch.ty = ty;
            ch.progress = 0.0;
            ch.ease = easing::linear;
            ch.speed = 0.14;
            ch.sub = SubPhase::Wander;
            ch.color_anim = ColorAnim::ToFinal;
            ch.color_frame = 0;
            ch.color_total = 80;
        }
    }

    fn begin_spin(&mut self) {
        for ch in &mut self.chars {
            if !ch.is_ring {
                continue;
            }
            let ring = &self.rings[ch.ring_idx];
            let (ty, tx) = ring.coords[ch.slot];
            ch.sx = ch.cur_x;
            ch.sy = ch.cur_y;
            ch.tx = tx;
            ch.ty = ty;
            ch.progress = 0.0;
            ch.ease = easing::linear;
            ch.speed = 0.1;
            ch.sub = SubPhase::Condense;
            ch.color_anim = ColorAnim::ToRing;
            ch.color_frame = 0;
            ch.color_total = 24; // Gradient(8 colors) × 3 frames
        }
    }

    fn begin_final(&mut self) {
        for ch in &mut self.chars {
            ch.visible = true;
            ch.sx = ch.cur_x;
            ch.sy = ch.cur_y;
            ch.tx = ch.home_x as f64;
            ch.ty = ch.home_y as f64;
            ch.progress = 0.0;
            ch.ease = easing::out_quad;
            ch.speed = 0.8;
            ch.sub = SubPhase::Home;
            if ch.is_ring {
                ch.color_anim = ColorAnim::ToFinal;
                ch.color_frame = 0;
                ch.color_total = 80;
            } else {
                ch.color_anim = ColorAnim::Solid;
            }
        }
    }

    fn update_motion(&mut self) {
        let mut rng = rand::thread_rng();
        for ch in &mut self.chars {
            match ch.sub {
                SubPhase::Idle | SubPhase::Settled | SubPhase::OffCanvas => {}
                SubPhase::Initial
                | SubPhase::Wander
                | SubPhase::Condense
                | SubPhase::External
                | SubPhase::Home => {
                    let d = dist(ch.sy, ch.sx, ch.ty, ch.tx).max(1e-9);
                    ch.progress = (ch.progress + ch.speed / d).min(1.0);
                    let e = (ch.ease)(ch.progress);
                    ch.cur_y = ch.sy + (ch.ty - ch.sy) * e;
                    ch.cur_x = ch.sx + (ch.tx - ch.sx) * e;
                    if ch.progress >= 1.0 {
                        // Transition on arrival
                        match ch.sub {
                            SubPhase::Initial | SubPhase::Wander => {
                                let (ty, tx) = pick_wander_target(
                                    &mut rng,
                                    ch.wander_cy,
                                    ch.wander_cx,
                                    ch.wander_r,
                                );
                                ch.sx = ch.cur_x;
                                ch.sy = ch.cur_y;
                                ch.tx = tx;
                                ch.ty = ty;
                                ch.progress = 0.0;
                                ch.ease = easing::linear;
                                ch.speed = 0.14;
                                ch.sub = SubPhase::Wander;
                            }
                            SubPhase::Condense => {
                                let ring = &self.rings[ch.ring_idx];
                                let next_slot = if ring.clockwise {
                                    (ch.slot + ring.coords.len() - 1) % ring.coords.len()
                                } else {
                                    (ch.slot + 1) % ring.coords.len()
                                };
                                ch.slot = next_slot;
                                let (ty, tx) = ring.coords[next_slot];
                                ch.sx = ch.cur_x;
                                ch.sy = ch.cur_y;
                                ch.tx = tx;
                                ch.ty = ty;
                                ch.progress = 0.0;
                                ch.ease = easing::linear;
                                ch.speed = ring.rotation_speed;
                                ch.sub = SubPhase::Orbit;
                            }
                            SubPhase::External => {
                                ch.visible = false;
                                ch.sub = SubPhase::OffCanvas;
                            }
                            SubPhase::Home => {
                                ch.cur_y = ch.home_y as f64;
                                ch.cur_x = ch.home_x as f64;
                                ch.sub = SubPhase::Settled;
                            }
                            _ => {}
                        }
                    }
                }
                SubPhase::Orbit => {
                    let ring = &self.rings[ch.ring_idx];
                    let d = dist(ch.sy, ch.sx, ch.ty, ch.tx).max(1e-9);
                    ch.progress = (ch.progress + ch.speed / d).min(1.0);
                    ch.cur_y = ch.sy + (ch.ty - ch.sy) * ch.progress;
                    ch.cur_x = ch.sx + (ch.tx - ch.sx) * ch.progress;
                    if ch.progress >= 1.0 {
                        let next_slot = if ring.clockwise {
                            (ch.slot + ring.coords.len() - 1) % ring.coords.len()
                        } else {
                            (ch.slot + 1) % ring.coords.len()
                        };
                        ch.slot = next_slot;
                        let (ty, tx) = ring.coords[next_slot];
                        ch.sx = ch.cur_x;
                        ch.sy = ch.cur_y;
                        ch.tx = tx;
                        ch.ty = ty;
                        ch.progress = 0.0;
                        ch.speed = ring.rotation_speed;
                    }
                }
            }
        }
    }

    fn update_color(&mut self) {
        for ch in &mut self.chars {
            if matches!(ch.color_anim, ColorAnim::ToFinal | ColorAnim::ToRing)
                && ch.color_frame < ch.color_total
            {
                ch.color_frame += 1;
            }
        }
    }

    fn current_color(ch: &CharState) -> Rgb {
        match ch.color_anim {
            ColorAnim::Solid => ch.final_color,
            ColorAnim::ToFinal => {
                let t = (ch.color_frame as f64 / ch.color_total as f64).min(1.0);
                Rgb::lerp(ch.ring_color, ch.final_color, t)
            }
            ColorAnim::ToRing => {
                let t = (ch.color_frame as f64 / ch.color_total as f64).min(1.0);
                Rgb::lerp(ch.final_color, ch.ring_color, t)
            }
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        // Phase transitions
        match self.phase {
            Phase::Start => {
                if self.initial_phase_remaining == 0 {
                    self.phase = Phase::Disperse;
                    self.disperse_remaining = self.disperse_duration;
                    self.begin_initial_disperse();
                } else {
                    self.initial_phase_remaining -= 1;
                }
            }
            Phase::Disperse => {
                if self.disperse_remaining == 0 {
                    self.phase = Phase::Spin;
                    self.spin_remaining = self.spin_duration;
                    self.cycles_remaining = self.cycles_remaining.saturating_sub(1);
                    self.begin_spin();
                } else {
                    self.disperse_remaining -= 1;
                }
            }
            Phase::Spin => {
                if self.spin_remaining == 0 {
                    if self.cycles_remaining == 0 {
                        self.phase = Phase::Final;
                        self.begin_final();
                    } else {
                        self.phase = Phase::Disperse;
                        self.disperse_remaining = self.disperse_duration;
                        self.begin_subsequent_disperse();
                    }
                } else {
                    self.spin_remaining -= 1;
                }
            }
            Phase::Final => {
                let all_done = self.chars.iter().all(|c| c.sub == SubPhase::Settled);
                if all_done {
                    self.phase = Phase::Complete;
                }
            }
            Phase::Complete => {}
        }

        if self.phase != Phase::Complete {
            self.update_motion();
            self.update_color();
        }

        // Reset every cell to its original char & clear color/visibility.
        for (y, row) in grid.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                cell.visible = false;
                cell.ch = self.original_chars[y][x];
                cell.fg = None;
            }
        }

        for ch in &self.chars {
            if !ch.visible {
                continue;
            }
            let ry = ch.cur_y.round();
            let rx = ch.cur_x.round();
            if ry < 0.0 || rx < 0.0 {
                continue;
            }
            let (ry, rx) = (ry as usize, rx as usize);
            if ry >= self.height || rx >= self.width {
                continue;
            }
            let cell = &mut grid.cells[ry][rx];
            cell.visible = true;
            cell.ch = ch.original_ch;
            cell.fg = Some(Self::current_color(ch).to_crossterm());
        }

        if self.phase == Phase::Complete {
            for ch in &self.chars {
                if ch.home_y < self.height && ch.home_x < self.width {
                    let cell = &mut grid.cells[ch.home_y][ch.home_x];
                    cell.visible = true;
                    cell.ch = ch.original_ch;
                    cell.fg = Some(ch.final_color.to_crossterm());
                }
            }
            return true;
        }

        false
    }
}

#[cfg(test)]
#[path = "../tests/effects/rings.rs"]
mod tests;
