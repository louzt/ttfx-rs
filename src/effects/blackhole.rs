// Blackhole effect — faithful TTE reimplementation
//
// All characters appear as a scattered starfield. A subset forms a rotating
// blackhole ring that consumes the remaining stars into its center. The ring
// collapses with an unstable-point animation, then all characters explode
// outward and settle into their final positions.

pub const NAME: &str = "blackhole";
pub const DESCRIPTION: &str = "Characters are consumed by a black hole and explode outwards.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;

const BLACKHOLE_COLOR: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 255,
};

const EXPLOSION_COLORS: [Rgb; 6] = [
    Rgb {
        r: 0xff,
        g: 0xcc,
        b: 0x0d,
    },
    Rgb {
        r: 0xff,
        g: 0x73,
        b: 0x26,
    },
    Rgb {
        r: 0xff,
        g: 0x19,
        b: 0x4d,
    },
    Rgb {
        r: 0xbf,
        g: 0x26,
        b: 0x69,
    },
    Rgb {
        r: 0x70,
        g: 0x2a,
        b: 0x8c,
    },
    Rgb {
        r: 0x04,
        g: 0x9d,
        b: 0xbf,
    },
];

const STAR_SYMBOLS: [char; 7] = ['*', '\'', '`', '¤', '•', '°', '·'];
const UNSTABLE_SYMBOLS: [char; 7] = ['◦', '◎', '◉', '●', '◉', '◎', '◦'];
// 7 symbols × 3 frames each × 3 cycles
const UNSTABLE_TOTAL: usize = 7 * 3 * 3;

struct BHChar {
    final_y: usize,
    final_x: usize,
    original_ch: char,
    final_color: Rgb,
    cur_y: f64,
    cur_x: f64,
    star_symbol: char,
    star_color: Rgb,
    scatter_y: f64,
    scatter_x: f64,
    is_ring: bool,
    ring_slot: usize,
    // Forming (ring)
    form_started: bool,
    form_progress: f64,
    form_speed: f64,
    // Consuming (non-ring)
    consume_progress: f64,
    consume_speed: f64,
    // Collapse angle (set when entering collapse)
    collapse_angle: f64,
    // Explosion
    explode_y: f64,
    explode_x: f64,
    explode_progress: f64,
    explode_speed: f64,
    explode_color: Rgb,
    // Settling
    settle_progress: f64,
    settle_speed: f64,
}

#[derive(PartialEq)]
enum Phase {
    Forming,
    Consuming,
    Collapsing,
    Exploding,
    Settling,
    Done,
}

pub struct BlackholeEffect {
    chars: Vec<BHChar>,
    ring_indices: Vec<usize>,
    non_ring_indices: Vec<usize>,
    ring_positions: Vec<(f64, f64)>,
    center_y: f64,
    center_x: f64,
    radius: f64,
    width: usize,
    height: usize,
    phase: Phase,
    // Forming
    formation_delay: usize,
    formation_timer: usize,
    next_ring_to_place: usize,
    // Rotation (during consuming)
    rotation_angle: f64,
    angular_speed: f64,
    // Collapse sub-phases: 0=expand, 1=collapse, 2=unstable
    collapse_sub: u8,
    collapse_progress: f64,
    unstable_frame: usize,
}

impl BlackholeEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height) = (grid.width, grid.height);
        // Geometric center of the canvas (matches TTE's canvas.center, which
        // is at the half-cell point between the four central cells).
        let center_y = (height as f64 - 1.0) / 2.0;
        let center_x = (width as f64 - 1.0) / 2.0;
        // Matches TTE: radius = min(width*0.3, height*0.20), min 3.
        // ASPECT stretches x separately in circle_positions.
        let radius = (width as f64 * 0.3)
            .min(height as f64 * 0.20)
            .max(3.0)
            .round();

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("FFFFFF"),
            ],
            9,
        );
        let starfield_gradient =
            Gradient::new(&[Rgb::from_hex("4a4a4d"), Rgb::from_hex("ffffff")], 6);

        let mut rng = rand::thread_rng();
        let mut positions = grid.char_positions();
        positions.shuffle(&mut rng);

        let n_ring = ((radius as usize) * 3).min(positions.len());
        let ring_positions = circle_positions(center_y, center_x, radius, n_ring);

        let mut chars = Vec::with_capacity(positions.len());
        let mut ring_indices = Vec::new();
        let mut non_ring_indices = Vec::new();

        for (i, &(y, x)) in positions.iter().enumerate() {
            let is_ring = i < n_ring;
            let final_color =
                final_gradient.color_at_coord(y, x, height, width, GradientDirection::Diagonal);
            let star_symbol = STAR_SYMBOLS[rng.gen_range(0..STAR_SYMBOLS.len())];
            let star_color = starfield_gradient.at(rng.gen_range(0.0..1.0));

            // Ring chars start at input positions; non-ring scatter randomly
            let (scatter_y, scatter_x) = if is_ring {
                (y as f64, x as f64)
            } else {
                (
                    rng.gen_range(0..height.max(1)) as f64,
                    rng.gen_range(0..width.max(1)) as f64,
                )
            };

            let ring_slot = if is_ring { i } else { 0 };

            let form_speed = if is_ring {
                let (ry, rx) = ring_positions[ring_slot];
                let dist = ((scatter_y - ry).powi(2) + (scatter_x - rx).powi(2))
                    .sqrt()
                    .max(1.0);
                0.7 / dist
            } else {
                0.0
            };

            let consume_speed = if !is_ring {
                let dist = ((scatter_y - center_y).powi(2) + (scatter_x - center_x).powi(2))
                    .sqrt()
                    .max(1.0);
                rng.gen_range(0.17..0.30) / dist
            } else {
                0.0
            };

            let idx = chars.len();
            if is_ring {
                ring_indices.push(idx);
            } else {
                non_ring_indices.push(idx);
            }

            chars.push(BHChar {
                final_y: y,
                final_x: x,
                original_ch: grid.cells[y][x].ch,
                final_color,
                cur_y: scatter_y,
                cur_x: scatter_x,
                star_symbol,
                star_color,
                scatter_y,
                scatter_x,
                is_ring,
                ring_slot,
                form_started: false,
                form_progress: 0.0,
                form_speed,
                consume_progress: 0.0,
                consume_speed,
                collapse_angle: 0.0,
                explode_y: 0.0,
                explode_x: 0.0,
                explode_progress: 0.0,
                explode_speed: 0.0,
                explode_color: EXPLOSION_COLORS[rng.gen_range(0..EXPLOSION_COLORS.len())],
                settle_progress: 0.0,
                settle_speed: 0.0,
            });
        }

        let formation_delay = 100usize.checked_div(n_ring).unwrap_or(6).max(6);
        // TTE uses linear speed 0.45 along the elliptical ring path.
        // Convert to angular speed: ω = 2π·v / C, where C ≈ π·r·(1+ASPECT).
        let angular_speed = 0.9 / (radius * (1.0 + ASPECT)).max(1.0);

        BlackholeEffect {
            chars,
            ring_indices,
            non_ring_indices,
            ring_positions,
            center_y,
            center_x,
            radius,
            width,
            height,
            phase: if positions.is_empty() {
                Phase::Done
            } else {
                Phase::Forming
            },
            formation_delay,
            formation_timer: 0,
            next_ring_to_place: 0,
            rotation_angle: 0.0,
            angular_speed,
            collapse_sub: 0,
            collapse_progress: 0.0,
            unstable_frame: 0,
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
            Phase::Forming => self.tick_forming(),
            Phase::Consuming => self.tick_consuming(),
            Phase::Collapsing => self.tick_collapsing(),
            Phase::Exploding => self.tick_exploding(),
            Phase::Settling => self.tick_settling(),
            Phase::Done => {}
        }

        self.render(grid);
        false
    }

    fn tick_forming(&mut self) {
        // Launch ring chars one at a time with delay
        if self.next_ring_to_place < self.ring_indices.len() {
            if self.formation_timer == 0 {
                let idx = self.ring_indices[self.next_ring_to_place];
                self.chars[idx].form_started = true;
                self.next_ring_to_place += 1;
                self.formation_timer = self.formation_delay;
            } else {
                self.formation_timer -= 1;
            }
        }

        // Move launched ring chars toward ring positions
        let ring_positions = &self.ring_positions;
        for &ri in &self.ring_indices {
            let ch = &mut self.chars[ri];
            if !ch.form_started || ch.form_progress >= 1.0 {
                continue;
            }
            ch.form_progress = (ch.form_progress + ch.form_speed).min(1.0);
            let t = easing::in_out_sine(ch.form_progress);
            let (ry, rx) = ring_positions[ch.ring_slot];
            ch.cur_y = ch.scatter_y + (ry - ch.scatter_y) * t;
            ch.cur_x = ch.scatter_x + (rx - ch.scatter_x) * t;
        }

        // All placed and arrived → start consuming
        if self.next_ring_to_place >= self.ring_indices.len()
            && self
                .ring_indices
                .iter()
                .all(|&ri| self.chars[ri].form_progress >= 1.0)
        {
            self.phase = Phase::Consuming;
        }
    }

    fn tick_consuming(&mut self) {
        // Rotate ring chars
        self.rotation_angle += self.angular_speed;
        let n_ring = self.ring_indices.len();
        let tau = std::f64::consts::TAU;
        for &ri in &self.ring_indices {
            let ch = &mut self.chars[ri];
            let base_angle = tau * ch.ring_slot as f64 / n_ring.max(1) as f64;
            let angle = base_angle + self.rotation_angle;
            ch.cur_y = self.center_y + self.radius * angle.sin();
            ch.cur_x = self.center_x + self.radius * angle.cos() * ASPECT;
        }

        // Pull non-ring chars toward center
        let (cy, cx) = (self.center_y, self.center_x);
        for &ni in &self.non_ring_indices {
            let ch = &mut self.chars[ni];
            if ch.consume_progress >= 1.0 {
                continue;
            }
            ch.consume_progress = (ch.consume_progress + ch.consume_speed).min(1.0);
            let t = easing::in_expo(ch.consume_progress);
            ch.cur_y = ch.scatter_y + (cy - ch.scatter_y) * t;
            ch.cur_x = ch.scatter_x + (cx - ch.scatter_x) * t;
        }

        // All consumed → collapse (or explode if no ring)
        if self
            .non_ring_indices
            .iter()
            .all(|&ni| self.chars[ni].consume_progress >= 1.0)
        {
            if self.ring_indices.is_empty() {
                self.prepare_explosion();
                self.phase = Phase::Exploding;
            } else {
                self.prepare_collapse();
                self.phase = Phase::Collapsing;
            }
        }
    }

    fn prepare_collapse(&mut self) {
        let n_ring = self.ring_indices.len();
        let tau = std::f64::consts::TAU;
        for &ri in &self.ring_indices {
            let ch = &mut self.chars[ri];
            let base_angle = tau * ch.ring_slot as f64 / n_ring.max(1) as f64;
            ch.collapse_angle = base_angle + self.rotation_angle;
        }
        self.collapse_sub = 0;
        self.collapse_progress = 0.0;
        self.unstable_frame = 0;
    }

    fn tick_collapsing(&mut self) {
        match self.collapse_sub {
            0 => {
                // Expand: ring → ring+3
                self.collapse_progress += 0.2 / 3.0_f64.max(1.0);
                let t = easing::in_expo(self.collapse_progress.min(1.0));
                for &ri in &self.ring_indices {
                    let ch = &mut self.chars[ri];
                    let r = self.radius + 3.0 * t;
                    ch.cur_y = self.center_y + r * ch.collapse_angle.sin();
                    ch.cur_x = self.center_x + r * ch.collapse_angle.cos() * ASPECT;
                }
                if self.collapse_progress >= 1.0 {
                    self.collapse_progress = 0.0;
                    self.collapse_sub = 1;
                }
            }
            1 => {
                // Collapse: ring+3 → center
                let expanded_radius = self.radius + 3.0;
                self.collapse_progress += 0.3 / expanded_radius.max(1.0);
                let t = easing::in_expo(self.collapse_progress.min(1.0));
                for &ri in &self.ring_indices {
                    let ch = &mut self.chars[ri];
                    let r = expanded_radius * (1.0 - t);
                    ch.cur_y = self.center_y + r * ch.collapse_angle.sin();
                    ch.cur_x = self.center_x + r * ch.collapse_angle.cos() * ASPECT;
                }
                if self.collapse_progress >= 1.0 {
                    self.collapse_sub = 2;
                }
            }
            _ => {
                // Unstable point at center
                self.unstable_frame += 1;
                if self.unstable_frame >= UNSTABLE_TOTAL {
                    self.prepare_explosion();
                    self.phase = Phase::Exploding;
                }
            }
        }
    }

    fn prepare_explosion(&mut self) {
        let mut rng = rand::thread_rng();
        let tau = std::f64::consts::TAU;
        for ch in &mut self.chars {
            ch.cur_y = self.center_y;
            ch.cur_x = self.center_x;
            let angle = rng.gen_range(0.0..tau);
            ch.explode_y = ch.final_y as f64 + angle.sin() * 3.0;
            ch.explode_x = ch.final_x as f64 + angle.cos() * 3.0;
            let dist = ((ch.explode_y - self.center_y).powi(2)
                + (ch.explode_x - self.center_x).powi(2))
            .sqrt()
            .max(1.0);
            ch.explode_speed = rng.gen_range(0.3..0.4) / dist;
            ch.explode_progress = 0.0;
            ch.explode_color = EXPLOSION_COLORS[rng.gen_range(0..EXPLOSION_COLORS.len())];
        }
    }

    fn tick_exploding(&mut self) {
        let (cy, cx) = (self.center_y, self.center_x);
        for ch in &mut self.chars {
            if ch.explode_progress >= 1.0 {
                continue;
            }
            ch.explode_progress = (ch.explode_progress + ch.explode_speed).min(1.0);
            let t = easing::out_expo(ch.explode_progress);
            ch.cur_y = cy + (ch.explode_y - cy) * t;
            ch.cur_x = cx + (ch.explode_x - cx) * t;
        }

        if self.chars.iter().all(|ch| ch.explode_progress >= 1.0) {
            self.prepare_settling();
            self.phase = Phase::Settling;
        }
    }

    fn prepare_settling(&mut self) {
        let mut rng = rand::thread_rng();
        for ch in &mut self.chars {
            let dist = ((ch.final_y as f64 - ch.explode_y).powi(2)
                + (ch.final_x as f64 - ch.explode_x).powi(2))
            .sqrt()
            .max(1.0);
            ch.settle_speed = rng.gen_range(0.04..0.06) / dist;
            ch.settle_progress = 0.0;
        }
    }

    fn tick_settling(&mut self) {
        for ch in &mut self.chars {
            if ch.settle_progress >= 1.0 {
                continue;
            }
            ch.settle_progress = (ch.settle_progress + ch.settle_speed).min(1.0);
            let t = easing::in_cubic(ch.settle_progress);
            ch.cur_y = ch.explode_y + (ch.final_y as f64 - ch.explode_y) * t;
            ch.cur_x = ch.explode_x + (ch.final_x as f64 - ch.explode_x) * t;
        }

        if self.chars.iter().all(|ch| ch.settle_progress >= 1.0) {
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
            Phase::Forming | Phase::Consuming => {
                // Non-ring chars: star symbols fading to black
                for &ni in &self.non_ring_indices {
                    let ch = &self.chars[ni];
                    if ch.consume_progress >= 1.0 {
                        continue;
                    }
                    let ry = ch.cur_y.round() as isize;
                    let rx = ch.cur_x.round() as isize;
                    if ry < 0 || rx < 0 || ry as usize >= self.height || rx as usize >= self.width {
                        continue;
                    }
                    let cell = &mut grid.cells[ry as usize][rx as usize];
                    cell.visible = true;
                    cell.ch = ch.star_symbol;
                    let fade = Rgb::lerp(ch.star_color, Rgb::new(0, 0, 0), ch.consume_progress);
                    cell.fg = Some(fade.to_crossterm());
                }
                // Ring chars on top
                for &ri in &self.ring_indices {
                    let ch = &self.chars[ri];
                    let ry = ch.cur_y.round() as isize;
                    let rx = ch.cur_x.round() as isize;
                    if ry < 0 || rx < 0 || ry as usize >= self.height || rx as usize >= self.width {
                        continue;
                    }
                    let cell = &mut grid.cells[ry as usize][rx as usize];
                    cell.visible = true;
                    if ch.form_started {
                        cell.ch = '*';
                        cell.fg = Some(BLACKHOLE_COLOR.to_crossterm());
                    } else {
                        cell.ch = ch.star_symbol;
                        cell.fg = Some(ch.star_color.to_crossterm());
                    }
                }
            }
            Phase::Collapsing => {
                // Ring chars
                for &ri in &self.ring_indices {
                    let ch = &self.chars[ri];
                    let ry = ch.cur_y.round() as isize;
                    let rx = ch.cur_x.round() as isize;
                    if ry < 0 || rx < 0 || ry as usize >= self.height || rx as usize >= self.width {
                        continue;
                    }
                    let cell = &mut grid.cells[ry as usize][rx as usize];
                    cell.visible = true;
                    cell.ch = '*';
                    cell.fg = Some(BLACKHOLE_COLOR.to_crossterm());
                }
                // Unstable point at center
                if self.collapse_sub == 2 {
                    let cy = self.center_y.round() as usize;
                    let cx = self.center_x.round() as usize;
                    if cy < self.height && cx < self.width {
                        let cell = &mut grid.cells[cy][cx];
                        cell.visible = true;
                        let sym_idx = (self.unstable_frame / 3) % UNSTABLE_SYMBOLS.len();
                        cell.ch = UNSTABLE_SYMBOLS[sym_idx];
                        let color_idx = (self.unstable_frame / 3) % EXPLOSION_COLORS.len();
                        cell.fg = Some(EXPLOSION_COLORS[color_idx].to_crossterm());
                    }
                }
            }
            Phase::Exploding => {
                for ch in &self.chars {
                    self.render_char(grid, ch, ch.original_ch, ch.explode_color);
                }
            }
            Phase::Settling | Phase::Done => {
                for ch in &self.chars {
                    let color = Rgb::lerp(ch.explode_color, ch.final_color, ch.settle_progress);
                    self.render_char(grid, ch, ch.original_ch, color);
                }
            }
        }
    }

    fn render_char(&self, grid: &mut Grid, ch: &BHChar, symbol: char, color: Rgb) {
        let ry = ch.cur_y.round() as isize;
        let rx = ch.cur_x.round() as isize;
        if ry < 0 || rx < 0 || ry as usize >= self.height || rx as usize >= self.width {
            return;
        }
        let cell = &mut grid.cells[ry as usize][rx as usize];
        cell.visible = true;
        cell.ch = symbol;
        cell.fg = Some(color.to_crossterm());
    }
}

/// TTE's `find_coords_on_circle` doubles the x-distance from origin to
/// correct for terminal cells being ~2:1 height:width. Match exactly.
const ASPECT: f64 = 2.0;

fn circle_positions(center_y: f64, center_x: f64, radius: f64, n: usize) -> Vec<(f64, f64)> {
    let tau = std::f64::consts::TAU;
    (0..n)
        .map(|i| {
            let angle = tau * i as f64 / n.max(1) as f64;
            (
                center_y + radius * angle.sin(),
                center_x + radius * angle.cos() * ASPECT,
            )
        })
        .collect()
}
