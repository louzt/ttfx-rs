// Print effect — typewriter that types each row at the canvas bottom and
// scrolls already-typed rows up by one each time a new row begins. The print
// head stays on the bottom row, moving L→R while typing and performing a
// carriage return (eased horizontal motion) between rows.

pub const NAME: &str = "print";
pub const DESCRIPTION: &str = "Lines are printed one at a time following a print head. Print head performs line feed, carriage return.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};

#[derive(Clone)]
struct SceneFrame {
    symbol: char,
    color: Rgb,
    duration: usize,
}

#[derive(Clone)]
struct PrintChar {
    final_x: usize,
    original_ch: char,
    visible: bool,
    scene: Vec<SceneFrame>,
    scene_idx: usize,
    hold_count: usize,
    scene_complete: bool,
    final_color: Rgb,
}

impl PrintChar {
    fn tick(&mut self) {
        if self.scene_complete || self.scene.is_empty() {
            return;
        }
        self.hold_count += 1;
        if self.hold_count >= self.scene[self.scene_idx].duration {
            self.hold_count = 0;
            self.scene_idx += 1;
            if self.scene_idx >= self.scene.len() {
                self.scene_idx = self.scene.len() - 1;
                self.scene_complete = true;
            }
        }
    }

    fn current_symbol(&self) -> char {
        if self.scene.is_empty() {
            return self.original_ch;
        }
        if self.scene_complete {
            // Last frame is `(original_ch, final_color)` — use it directly so
            // chars that finished their typed animation (or were short-
            // circuited, e.g. spaces) don't keep rendering frame 0's '█'.
            return self.scene.last().unwrap().symbol;
        }
        self.scene[self.scene_idx].symbol
    }

    fn current_color(&self) -> Rgb {
        if self.scene.is_empty() {
            return self.final_color;
        }
        if self.scene_complete {
            return self.scene.last().unwrap().color;
        }
        self.scene[self.scene_idx].color
    }
}

#[derive(PartialEq, Debug)]
enum Phase {
    Typing,
    CarriageReturn,
    Complete,
}

pub struct PrintEffect {
    chars: Vec<Vec<PrintChar>>, // [input_row][col]
    cur_y: Vec<isize>,          // current screen row for each input row (-1 = unstarted)
    pending_rows: Vec<usize>,   // input rows to process (top-to-bottom)
    processed_rows: Vec<usize>, // already-typed rows
    typing_row: Option<usize>,
    col_pos: usize,
    print_speed: usize,
    phase: Phase,
    head_visible: bool,
    head_x: f64,
    cr_start_x: f64,
    cr_target_x: f64,
    cr_progress: f64,
    cr_speed_per_unit: f64,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn last_non_space_col(chars: &[PrintChar]) -> usize {
    chars
        .iter()
        .rposition(|c| c.original_ch != ' ')
        .unwrap_or(chars.len().saturating_sub(1))
}

impl PrintEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("02b8bd"),
                Rgb::from_hex("c1f0e3"),
                Rgb::from_hex("00ffa0"),
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

        let block_syms = ['█', '▓', '▒', '░'];
        let typing_head_color = Rgb::new(255, 255, 255);

        let mut chars: Vec<Vec<PrintChar>> = Vec::with_capacity(height);
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let original_ch = grid.cells[y][x].ch;
                let ry = y.saturating_sub(text_top);
                let rx = x.saturating_sub(text_left);
                let final_color = final_gradient.color_at_coord(
                    ry,
                    rx,
                    text_h,
                    text_w,
                    GradientDirection::Diagonal,
                );

                // Typed animation: █ → ▓ → ▒ → ░ → original_ch with gradient
                // typing_head_color → final_color over 5 steps × 3 frames
                // (matches TTE's `Gradient(typing_head_color, final, steps=5)`).
                let mut scene = Vec::with_capacity(5);
                for (i, &sym) in block_syms.iter().enumerate() {
                    let t = i as f64 / 4.0;
                    let color = Rgb::lerp(typing_head_color, final_color, t);
                    scene.push(SceneFrame {
                        symbol: sym,
                        color,
                        duration: 3,
                    });
                }
                scene.push(SceneFrame {
                    symbol: original_ch,
                    color: final_color,
                    duration: 3,
                });

                row.push(PrintChar {
                    final_x: x,
                    original_ch,
                    visible: false,
                    scene,
                    scene_idx: 0,
                    hold_count: 0,
                    scene_complete: false,
                    final_color,
                });
            }
            chars.push(row);
        }

        // Pending rows in top-to-bottom input order (TTE: ROW_TOP_TO_BOTTOM).
        let pending_rows: Vec<usize> = (0..height).collect();
        let cur_y = vec![-1isize; height];

        PrintEffect {
            chars,
            cur_y,
            pending_rows,
            processed_rows: Vec::new(),
            typing_row: None,
            col_pos: 0,
            print_speed: 2,
            phase: Phase::Typing,
            head_visible: false,
            head_x: 0.0,
            cr_start_x: 0.0,
            cr_target_x: 0.0,
            cr_progress: 0.0,
            // TTE print_head_return_speed = 1.5 along same row (purely
            // horizontal). Distance has no row component, so progress per
            // frame = 1.5 / |Δcol|.
            cr_speed_per_unit: 1.5,
            width,
            height,
            original_chars,
        }
    }

    fn start_next_row(&mut self) {
        let typing_y = self.height as isize - 1;
        let next_row = self.pending_rows.remove(0);
        // TTE: Row.__init__ sets untyped_chars = [col 0..=right_extent]. The
        // later left_extent filter in __next__ only trims leading FILL chars,
        // which rtte's Grid::from_input never produces (input chars come
        // first, padding only at the end). So input spaces at the start of a
        // row are real input — they get typed like any other char.
        self.cur_y[next_row] = typing_y;
        self.typing_row = Some(next_row);
        self.col_pos = 0;
        self.head_x = 0.0;
        // TTE hides the typing head as soon as CR finishes — during typing
        // the "head" visual is the typed_animation's first frame ('█') on
        // each char being typed.
        self.head_visible = false;
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        match self.phase {
            Phase::Typing => {
                if self.typing_row.is_none() {
                    if self.pending_rows.is_empty() {
                        self.phase = Phase::Complete;
                    } else if self.processed_rows.is_empty() {
                        // Very first row — type immediately, no CR.
                        self.start_next_row();
                        self.head_visible = false;
                    } else {
                        // Should have been transitioned via CarriageReturn.
                        self.phase = Phase::CarriageReturn;
                    }
                }
                if let Some(row) = self.typing_row {
                    let row_chars = &mut self.chars[row];
                    // TTE Row.__init__ trims to right_extent = max non-fill
                    // col. With no fill in rtte's Grid that's the same as
                    // last non-space col. Type cols 0..=last_col, including
                    // any leading or interior spaces — every input char gets
                    // its full typed_animation just like in TTE.
                    let last_col = last_non_space_col(row_chars);
                    let mut typed = 0;
                    while typed < self.print_speed && self.col_pos <= last_col {
                        let ca = &mut row_chars[self.col_pos];
                        ca.visible = true;
                        ca.scene_idx = 0;
                        ca.hold_count = 0;
                        ca.scene_complete = false;
                        self.head_x = self.col_pos as f64;
                        self.col_pos += 1;
                        typed += 1;
                    }
                    if self.col_pos > last_col {
                        // Row fully typed.
                        self.processed_rows.push(row);
                        self.typing_row = None;
                        if self.pending_rows.is_empty() {
                            self.phase = Phase::Complete;
                            self.head_visible = false;
                        } else {
                            // Line feed: scroll all typed rows up by 1 BEFORE
                            // the carriage return so the head moves across an
                            // empty bottom row (TTE order: row.move_up()
                            // first, then activate the CR path).
                            for &r in &self.processed_rows {
                                self.cur_y[r] -= 1;
                            }
                            // CR target = col 0. TTE's left_extent (start of
                            // untyped_chars after filtering) is 0 unless the
                            // canvas pads the row with leading FILL chars,
                            // which rtte's Grid never does.
                            self.cr_start_x = self.head_x;
                            self.cr_target_x = 0.0;
                            self.cr_progress = 0.0;
                            self.phase = Phase::CarriageReturn;
                            self.head_visible = true;
                        }
                    }
                }
            }
            Phase::CarriageReturn => {
                let dx = (self.cr_target_x - self.cr_start_x).abs().max(1.0);
                let speed = self.cr_speed_per_unit / dx;
                self.cr_progress = (self.cr_progress + speed).min(1.0);
                let eased = easing::in_out_quad(self.cr_progress);
                self.head_x = self.cr_start_x + (self.cr_target_x - self.cr_start_x) * eased;
                if self.cr_progress >= 1.0 {
                    self.start_next_row();
                    self.phase = Phase::Typing;
                }
            }
            Phase::Complete => {
                let any_active = self
                    .chars
                    .iter()
                    .any(|row| row.iter().any(|c| c.visible && !c.scene_complete));
                if !any_active {
                    for (input_y, row) in self.chars.iter().enumerate() {
                        if input_y < grid.height {
                            for ca in row {
                                if ca.final_x < grid.width {
                                    let cell = &mut grid.cells[input_y][ca.final_x];
                                    cell.visible = true;
                                    cell.ch = ca.original_ch;
                                    cell.fg = Some(ca.final_color.to_crossterm());
                                }
                            }
                        }
                    }
                    return true;
                }
            }
        }

        // Advance per-char animations.
        for row in &mut self.chars {
            for ca in row {
                if ca.visible && !ca.scene_complete {
                    ca.tick();
                }
            }
        }

        // Render: clear, then draw each char at its current screen row.
        for (y, row) in grid.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                cell.visible = false;
                cell.ch = self.original_chars[y][x];
                cell.fg = None;
            }
        }
        for input_y in 0..self.height {
            let screen_y = self.cur_y[input_y];
            if screen_y < 0 || screen_y >= self.height as isize {
                continue;
            }
            let screen_y = screen_y as usize;
            for ca in &self.chars[input_y] {
                if !ca.visible {
                    continue;
                }
                if ca.final_x >= self.width {
                    continue;
                }
                let cell = &mut grid.cells[screen_y][ca.final_x];
                cell.visible = true;
                cell.ch = ca.current_symbol();
                cell.fg = Some(ca.current_color().to_crossterm());
            }
        }

        // Render print head at canvas bottom.
        if self.head_visible && self.phase != Phase::Complete {
            let hy = self.height.saturating_sub(1);
            let hx = self.head_x.round() as isize;
            if hx >= 0 && (hx as usize) < self.width && hy < self.height {
                let cell = &mut grid.cells[hy][hx as usize];
                cell.visible = true;
                cell.ch = '█';
                cell.fg = Some(Rgb::new(255, 255, 255).to_crossterm());
            }
        }

        false
    }
}

#[cfg(test)]
#[path = "../tests/effects/print.rs"]
mod tests;
