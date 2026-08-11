// BinaryPath effect — binary digits travel right-angle paths to final positions
//
// Each character's 8-bit binary representation travels as a trailing snake
// along a right-angle path from outside the canvas to the character's position.
// After all travel completes, a diagonal wipe brightens the final text.

pub const NAME: &str = "binarypath";
pub const DESCRIPTION: &str =
    "Binary representations of each character move towards the home coordinate of the character.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;

// Collapse: white -> dim (7 gradient steps, 3 frames each, in_quad easing)
const COLLAPSE_FRAMES: usize = 21;
// Brighten: dim -> final (10 gradient steps, 2 frames each, linear)
const BRIGHTEN_FRAMES: usize = 20;

struct BinaryDigit {
    symbol: char,
    color: Rgb,
    target_idx: usize, // next waypoint to reach (starts at 1)
    cy: f64,
    cx: f64,
    active: bool,
    arrived: bool,
}

struct BinaryRep {
    final_y: usize,
    final_x: usize,
    original_ch: char,
    final_color: Rgb,
    path: Vec<(f64, f64)>,
    digits: Vec<BinaryDigit>,
    next_release: usize,
    travel_complete: bool,
    source_visible: bool,
    collapse_step: usize,
    brighten_active: bool,
    brighten_step: usize,
}

#[derive(PartialEq)]
enum Phase {
    Travel,
    Wipe,
    Done,
}

pub struct BinaryPathEffect {
    reps: Vec<BinaryRep>,
    width: usize,
    height: usize,
    max_active: usize,
    pending: Vec<usize>,
    active_reps: Vec<usize>,
    phase: Phase,
    wipe_groups: Vec<Vec<usize>>,
    wipe_idx: usize,
}

impl BinaryPathEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height) = (grid.width, grid.height);
        let final_gradient = Gradient::new(&[Rgb::from_hex("00d500"), Rgb::from_hex("007500")], 12);
        let binary_colors = [
            Rgb::from_hex("044E29"),
            Rgb::from_hex("157e38"),
            Rgb::from_hex("45bf55"),
            Rgb::from_hex("95ed87"),
        ];

        let mut rng = rand::thread_rng();
        let mut reps = Vec::new();

        for (y, x) in grid.char_positions() {
            let fc = final_gradient.color_at_coord(y, x, height, width, GradientDirection::Radial);
            let ch = grid.cells[y][x].ch;
            let binary_string = format!("{:08b}", ch as u32);
            let path = generate_path(&mut rng, y, x, width, height);

            let digits: Vec<BinaryDigit> = binary_string
                .chars()
                .map(|bit| BinaryDigit {
                    symbol: bit,
                    color: binary_colors[rng.gen_range(0..binary_colors.len())],
                    target_idx: 1,
                    cy: path[0].0,
                    cx: path[0].1,
                    active: false,
                    arrived: false,
                })
                .collect();

            reps.push(BinaryRep {
                final_y: y,
                final_x: x,
                original_ch: ch,
                final_color: fc,
                path,
                digits,
                next_release: 0,
                travel_complete: false,
                source_visible: false,
                collapse_step: 0,
                brighten_active: false,
                brighten_step: 0,
            });
        }

        let total = reps.len();
        let max_active = (total as f64 * 0.08).max(1.0) as usize;
        let mut pending: Vec<usize> = (0..total).collect();
        pending.shuffle(&mut rng);

        // Diagonal wipe groups: top-right to bottom-left
        let max_diag = if width > 0 && height > 0 {
            height + width - 2
        } else {
            0
        };
        let mut wipe_groups: Vec<Vec<usize>> = (0..=max_diag).map(|_| Vec::new()).collect();
        for (i, rep) in reps.iter().enumerate() {
            let diag = rep.final_y + width.saturating_sub(1).saturating_sub(rep.final_x);
            if diag <= max_diag {
                wipe_groups[diag].push(i);
            }
        }
        wipe_groups.retain(|g| !g.is_empty());

        BinaryPathEffect {
            reps,
            width,
            height,
            max_active,
            pending,
            active_reps: Vec::new(),
            phase: Phase::Travel,
            wipe_groups,
            wipe_idx: 0,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        if self.phase == Phase::Done {
            // Reset all cells to spaces first — digits in flight may have
            // overwritten ch on space cells during earlier renders.
            for row in &mut grid.cells {
                for cell in row {
                    cell.ch = ' ';
                    cell.fg = None;
                    cell.visible = true;
                }
            }
            for rep in &self.reps {
                let cell = &mut grid.cells[rep.final_y][rep.final_x];
                cell.ch = rep.original_ch;
                cell.fg = Some(rep.final_color.to_crossterm());
            }
            return true;
        }

        if self.phase == Phase::Travel {
            self.tick_travel();
        }
        if self.phase == Phase::Wipe {
            self.tick_wipe();
        }

        self.render(grid);
        false
    }

    fn tick_travel(&mut self) {
        // Activate new groups up to max_active
        let active_count = self
            .active_reps
            .iter()
            .filter(|&&i| !self.reps[i].travel_complete)
            .count();
        let to_activate = self.max_active.saturating_sub(active_count);
        for _ in 0..to_activate {
            if let Some(idx) = self.pending.pop() {
                self.active_reps.push(idx);
            }
        }

        let active_indices: Vec<usize> = self.active_reps.clone();
        for &rep_idx in &active_indices {
            let rep = &mut self.reps[rep_idx];

            if rep.travel_complete {
                if rep.source_visible && rep.collapse_step < COLLAPSE_FRAMES {
                    rep.collapse_step += 1;
                }
                continue;
            }

            // Release one binary digit per frame
            if rep.next_release < rep.digits.len() {
                rep.digits[rep.next_release].active = true;
                rep.next_release += 1;
            }

            // Move active digits along path
            let path = &rep.path;
            for digit in &mut rep.digits {
                if !digit.active || digit.arrived {
                    continue;
                }
                if digit.target_idx >= path.len() {
                    digit.arrived = true;
                    continue;
                }
                let (ty, tx) = path[digit.target_idx];
                let dy = ty - digit.cy;
                let dx = tx - digit.cx;
                let dist = (dy * dy + dx * dx).sqrt();
                if dist <= 1.0 {
                    digit.cy = ty;
                    digit.cx = tx;
                    digit.target_idx += 1;
                    if digit.target_idx >= path.len() {
                        digit.arrived = true;
                    }
                } else {
                    digit.cy += dy / dist;
                    digit.cx += dx / dist;
                }
            }

            // All digits arrived → show source character with collapse
            if rep.next_release >= rep.digits.len() && rep.digits.iter().all(|d| d.arrived) {
                rep.travel_complete = true;
                rep.source_visible = true;
            }
        }

        // Transition to wipe when all reps done with travel + collapse
        if self.pending.is_empty()
            && self
                .reps
                .iter()
                .all(|r| r.travel_complete && r.collapse_step >= COLLAPSE_FRAMES)
        {
            self.phase = Phase::Wipe;
        }
    }

    fn tick_wipe(&mut self) {
        // Activate 2 diagonal groups per frame
        for _ in 0..2 {
            if self.wipe_idx < self.wipe_groups.len() {
                for &rep_idx in &self.wipe_groups[self.wipe_idx] {
                    self.reps[rep_idx].brighten_active = true;
                }
                self.wipe_idx += 1;
            }
        }

        for rep in &mut self.reps {
            if rep.brighten_active && rep.brighten_step < BRIGHTEN_FRAMES {
                rep.brighten_step += 1;
            }
        }

        if self.wipe_idx >= self.wipe_groups.len()
            && self.reps.iter().all(|r| r.brighten_step >= BRIGHTEN_FRAMES)
        {
            self.phase = Phase::Done;
        }
    }

    fn render(&self, grid: &mut Grid) {
        for row in &mut grid.cells {
            for cell in row {
                cell.visible = false;
            }
        }

        for rep in &self.reps {
            if rep.travel_complete && rep.source_visible {
                let cell = &mut grid.cells[rep.final_y][rep.final_x];
                cell.visible = true;
                cell.ch = rep.original_ch;

                if rep.brighten_active {
                    // Wipe phase: dim → final (linear)
                    let dim = rep.final_color.adjust_brightness(0.5);
                    let t = rep.brighten_step as f64 / BRIGHTEN_FRAMES as f64;
                    cell.fg = Some(Rgb::lerp(dim, rep.final_color, t).to_crossterm());
                } else {
                    // Collapse phase: white → dim (in_quad)
                    let dim = rep.final_color.adjust_brightness(0.5);
                    let t = easing::in_quad(rep.collapse_step as f64 / COLLAPSE_FRAMES as f64);
                    let white = Rgb::new(255, 255, 255);
                    cell.fg = Some(Rgb::lerp(white, dim, t).to_crossterm());
                }
            } else {
                // Render binary digits in flight (including arrived digits waiting for others)
                for digit in &rep.digits {
                    if !digit.active {
                        continue;
                    }
                    let (ry, rx) = if digit.arrived {
                        (rep.final_y as isize, rep.final_x as isize)
                    } else {
                        (digit.cy.round() as isize, digit.cx.round() as isize)
                    };
                    if ry >= 0
                        && rx >= 0
                        && (ry as usize) < self.height
                        && (rx as usize) < self.width
                    {
                        let cell = &mut grid.cells[ry as usize][rx as usize];
                        cell.visible = true;
                        cell.ch = digit.symbol;
                        cell.fg = Some(digit.color.to_crossterm());
                    }
                }
            }
        }
    }
}

/// Generate a right-angle path from outside the canvas to (target_y, target_x),
/// matching TTE's distance-aware randomized step sizes.
fn generate_path(
    rng: &mut impl Rng,
    target_y: usize,
    target_x: usize,
    width: usize,
    height: usize,
) -> Vec<(f64, f64)> {
    let mut path = Vec::new();

    // Start from random position outside canvas
    let side = rng.gen_range(0..4);
    let start = match side {
        0 => (
            rng.gen_range(0..height.max(1)) as f64,
            -(rng.gen_range(3..15) as f64),
        ),
        1 => (
            rng.gen_range(0..height.max(1)) as f64,
            (width + rng.gen_range(3..15)) as f64,
        ),
        2 => (
            -(rng.gen_range(3..15) as f64),
            rng.gen_range(0..width.max(1)) as f64,
        ),
        _ => (
            (height + rng.gen_range(3..15)) as f64,
            rng.gen_range(0..width.max(1)) as f64,
        ),
    };
    path.push(start);

    let ty = target_y as f64;
    let tx = target_x as f64;
    let mut last_orientation = if rng.gen_bool(0.5) { "col" } else { "row" };

    loop {
        let (ly, lx) = *path.last().unwrap();
        if (ly - ty).abs() < 0.5 && (lx - tx).abs() < 0.5 {
            break;
        }

        let col_dir: f64 = if lx > tx {
            -1.0
        } else if lx < tx {
            1.0
        } else {
            0.0
        };
        let row_dir: f64 = if ly > ty {
            -1.0
        } else if ly < ty {
            1.0
        } else {
            0.0
        };
        let max_col_dist = (lx - tx).abs() as usize;
        let max_row_dist = (ly - ty).abs() as usize;

        let next = if last_orientation == "col" && max_row_dist > 0 {
            let max_step = max_row_dist.min(10usize.max((width as f64 * 0.2) as usize));
            let step = rng.gen_range(1..=max_step.max(1));
            last_orientation = "row";
            (ly + step as f64 * row_dir, lx)
        } else if last_orientation == "row" && max_col_dist > 0 {
            let max_step = max_col_dist.min(4);
            let step = rng.gen_range(1..=max_step.max(1));
            last_orientation = "col";
            (ly, lx + step as f64 * col_dir)
        } else {
            (ty, tx)
        };

        path.push(next);
    }

    path.push((ty, tx));
    path
}
