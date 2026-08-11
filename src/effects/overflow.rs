// Overflow effect — text scrolls/overflows then settles into correct order.
// Rows have stacked y-offsets below the canvas; advancing scroll_pos brings
// them up through the canvas. The final rows are appended in order, so when
// the last cycle of overflow is gone the final rows are already in place.

pub const NAME: &str = "overflow";
pub const DESCRIPTION: &str = "Input text overflows and scrolls the terminal in a random order until eventually appearing ordered.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    Overflow,
    Done,
}

struct OverflowRow {
    chars: Vec<(char, Rgb)>,
    y_offset: f64,
    is_final: bool,
}

pub struct OverflowEffect {
    rows: Vec<OverflowRow>,
    phase: Phase,
    speed: f64,
    width: usize,
    height: usize,
    scroll_pos: f64,
    overflow_rows: usize,
    overflow_spectrum: Vec<Rgb>,
}

impl OverflowEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;

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
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("FFFFFF"),
            ],
            12,
        );
        let overflow_gradient = Gradient::new(
            &[
                Rgb::from_hex("f2ebc0"),
                Rgb::from_hex("8dbfb3"),
                Rgb::from_hex("f2ebc0"),
            ],
            12,
        );

        let mut rng = rand::thread_rng();
        let mut original: Vec<Vec<char>> = Vec::with_capacity(height);
        for y in 0..height {
            let row: Vec<char> = (0..width).map(|x| grid.cells[y][x].ch).collect();
            original.push(row);
        }

        let cycles = rng.gen_range(2..=4);
        let mut rows: Vec<OverflowRow> = Vec::new();

        for _ in 0..cycles {
            let mut indices: Vec<usize> = (0..height).collect();
            indices.shuffle(&mut rng);
            for &idx in &indices {
                let row_chars: Vec<(char, Rgb)> = (0..width)
                    .map(|x| (original[idx][x], Rgb::new(0, 0, 0)))
                    .collect();
                rows.push(OverflowRow {
                    chars: row_chars,
                    y_offset: (height + rows.len()) as f64,
                    is_final: false,
                });
            }
        }
        let overflow_rows = rows.len();
        for (y, orig_row) in original.iter().enumerate() {
            let row_chars: Vec<(char, Rgb)> = (0..width)
                .map(|x| {
                    let ry = y.saturating_sub(text_top);
                    let rx = x.saturating_sub(text_left);
                    let color = final_gradient.color_at_coord(
                        ry,
                        rx,
                        text_h,
                        text_w,
                        GradientDirection::Vertical,
                    );
                    (orig_row[x], color)
                })
                .collect();
            rows.push(OverflowRow {
                chars: row_chars,
                y_offset: (height + rows.len()) as f64,
                is_final: true,
            });
        }

        OverflowEffect {
            rows,
            phase: Phase::Overflow,
            // overflow_speed default = 3 rows/frame (TTE: 1..=overflow_speed
            // per active tick, no delay between).
            speed: 3.0,
            width,
            height,
            scroll_pos: 0.0,
            overflow_rows,
            overflow_spectrum: overflow_gradient.spectrum().to_vec(),
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        if self.phase == Phase::Done {
            return true;
        }

        // Advance scroll until the first final row reaches the canvas top
        // (screen_y = 0) — at that exact point all final rows occupy their
        // correct positions [0, height-1] in order.
        let target_scroll = (self.height + self.overflow_rows) as f64;
        if self.scroll_pos < target_scroll {
            self.scroll_pos = (self.scroll_pos + self.speed).min(target_scroll);
        }
        if self.scroll_pos >= target_scroll {
            self.phase = Phase::Done;
        }

        for row in &mut grid.cells {
            for cell in row {
                cell.visible = false;
            }
        }

        for overflow_row in &self.rows {
            let screen_y = overflow_row.y_offset - self.scroll_pos;
            let ry = screen_y.round() as isize;
            if ry < 0 || ry >= self.height as isize {
                continue;
            }
            let ry = ry as usize;
            // Non-final rows: color from overflow gradient indexed by current
            // screen row, so the color shifts as the row scrolls (matches
            // TTE's `row.set_color` per move based on current_coord.row).
            let position_color = if !overflow_row.is_final {
                let n = self.overflow_spectrum.len().max(1);
                let idx = ry.min(n - 1);
                Some(self.overflow_spectrum[idx])
            } else {
                None
            };
            for (x, &(ch, color)) in overflow_row.chars.iter().enumerate() {
                if x >= self.width {
                    break;
                }
                let cell = &mut grid.cells[ry][x];
                cell.visible = true;
                cell.ch = ch;
                let display_color = position_color.unwrap_or(color);
                cell.fg = Some(display_color.to_crossterm());
            }
        }

        self.phase == Phase::Done
    }
}

#[cfg(test)]
#[path = "../tests/effects/overflow.rs"]
mod tests;
