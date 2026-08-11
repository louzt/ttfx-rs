// SynthGrid effect — grid expand, dissolve characters, grid collapse

pub const NAME: &str = "synthgrid";
pub const DESCRIPTION: &str =
    "Create a grid which fills with characters dissolving into the final text.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    GridExpand,
    AddChars,
    GridCollapse,
    Done,
}

struct GridLine {
    is_row: bool,
    pos: usize,    // row or column index
    extent: usize, // how far the line has extended
    max_extent: usize,
    color: Rgb,
}

struct SynthChar {
    y: usize,
    x: usize,
    original_ch: char,
    final_color: Rgb,
    dissolve_frames: Vec<(char, Rgb)>,
    dissolve_idx: usize,
    active: bool,
    done: bool,
}

pub struct SynthGridEffect {
    grid_lines: Vec<GridLine>,
    chars: Vec<SynthChar>,
    phase: Phase,
    frame: usize,
    dm: usize,
    width: usize,
    height: usize,
    max_active_blocks: usize,
    pending: Vec<usize>,
    expand_speed: usize,
    collapse_idx: usize,
    grid_gradient: Gradient,
}

impl SynthGridEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height, dm) = (grid.width, grid.height, 2usize);
        let text_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("FFFFFF"),
            ],
            12,
        );
        let grid_gradient = Gradient::new(&[Rgb::from_hex("CC00CC"), Rgb::from_hex("ffffff")], 12);

        let mut rng = rand::thread_rng();
        let dissolve_symbols = ['░', '▒', '▓'];

        // Calculate grid gaps
        let row_gap = (height as f64 * 0.2).max(2.0) as usize;
        let col_gap = (width as f64 * 0.2).max(3.0) as usize;

        // Create grid lines
        let mut grid_lines = Vec::new();
        // Horizontal lines (top, bottom, internal)
        let mut y = 0;
        while y < height {
            let gc = grid_gradient.color_at_coord(y, 0, height, width, GradientDirection::Diagonal);
            grid_lines.push(GridLine {
                is_row: true,
                pos: y,
                extent: 0,
                max_extent: width,
                color: gc,
            });
            y += row_gap;
        }
        // Vertical lines
        let mut x = 0;
        while x < width {
            let gc = grid_gradient.color_at_coord(0, x, height, width, GradientDirection::Diagonal);
            grid_lines.push(GridLine {
                is_row: false,
                pos: x,
                extent: 0,
                max_extent: height,
                color: gc,
            });
            x += col_gap;
        }

        // Create chars with dissolve animations
        let mut chars = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let fc =
                    text_gradient.color_at_coord(y, x, height, width, GradientDirection::Vertical);
                let num_dissolve = rng.gen_range(15..=30);
                let mut dissolve_frames = Vec::new();
                for i in 0..num_dissolve {
                    let sym = dissolve_symbols[rng.gen_range(0..3)];
                    let t = i as f64 / num_dissolve as f64;
                    let gc = grid_gradient.color_at_coord(
                        y,
                        x,
                        height,
                        width,
                        GradientDirection::Diagonal,
                    );
                    let color = Rgb::lerp(gc, fc, t);
                    dissolve_frames.push((sym, color));
                }
                dissolve_frames.push((grid.cells[y][x].ch, fc));

                chars.push(SynthChar {
                    y,
                    x,
                    original_ch: grid.cells[y][x].ch,
                    final_color: fc,
                    dissolve_frames,
                    dissolve_idx: 0,
                    active: false,
                    done: false,
                });
            }
        }

        let total = chars.len();
        let max_active = (total as f64 * 0.1).max(1.0) as usize;
        let mut pending: Vec<usize> = (0..total).collect();
        pending.shuffle(&mut rng);

        SynthGridEffect {
            grid_lines,
            chars,
            phase: Phase::GridExpand,
            frame: 0,
            dm,
            width,
            height,
            max_active_blocks: max_active,
            pending,
            expand_speed: 3,
            collapse_idx: 0,
            grid_gradient,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        self.frame += 1;
        let _dm = self.dm;

        match self.phase {
            Phase::GridExpand => {
                let mut all_done = true;
                for gl in &mut self.grid_lines {
                    if gl.extent < gl.max_extent {
                        gl.extent = (gl.extent + self.expand_speed).min(gl.max_extent);
                        all_done = false;
                    }
                }
                if all_done {
                    self.phase = Phase::AddChars;
                }
            }
            Phase::AddChars => {
                // Activate new chars
                let active_count = self.chars.iter().filter(|c| c.active && !c.done).count();
                let to_activate = self.max_active_blocks.saturating_sub(active_count);
                for _ in 0..to_activate {
                    if let Some(idx) = self.pending.pop() {
                        self.chars[idx].active = true;
                    }
                }

                // Advance dissolve animations
                for ch in &mut self.chars {
                    if ch.active && !ch.done {
                        ch.dissolve_idx += 1;
                        if ch.dissolve_idx >= ch.dissolve_frames.len() {
                            ch.done = true;
                            ch.dissolve_idx = ch.dissolve_frames.len() - 1;
                        }
                    }
                }

                if self.pending.is_empty() && self.chars.iter().all(|c| c.done) {
                    self.phase = Phase::GridCollapse;
                    // Reverse grid line order for collapse
                    self.grid_lines.reverse();
                    self.collapse_idx = 0;
                }
            }
            Phase::GridCollapse => {
                let mut all_collapsed = true;
                for gl in &mut self.grid_lines {
                    if gl.extent > 0 {
                        gl.extent = gl.extent.saturating_sub(self.expand_speed);
                        all_collapsed = false;
                    }
                }
                if all_collapsed {
                    self.phase = Phase::Done;
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

        // Draw grid lines
        for gl in &self.grid_lines {
            if gl.extent == 0 {
                continue;
            }
            if gl.is_row {
                for x in 0..gl.extent.min(self.width) {
                    let cell = &mut grid.cells[gl.pos.min(self.height - 1)][x];
                    cell.visible = true;
                    cell.ch = '─';
                    cell.fg = Some(gl.color.to_crossterm());
                }
            } else {
                for y in 0..gl.extent.min(self.height) {
                    let cell = &mut grid.cells[y][gl.pos.min(self.width - 1)];
                    cell.visible = true;
                    cell.ch = '│';
                    cell.fg = Some(gl.color.to_crossterm());
                }
            }
        }

        // Draw chars (on top of grid)
        for ch in &self.chars {
            if !ch.active {
                continue;
            }
            let (sym, color) =
                ch.dissolve_frames[ch.dissolve_idx.min(ch.dissolve_frames.len() - 1)];
            let cell = &mut grid.cells[ch.y][ch.x];
            cell.visible = true;
            cell.ch = sym;
            cell.fg = Some(color.to_crossterm());
        }

        false
    }
}
