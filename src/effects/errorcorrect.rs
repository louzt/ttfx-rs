// ErrorCorrect effect — swap pairs of chars; each pair animates error → block-wipe → move → block-wipe → fade

pub const NAME: &str = "errorcorrect";
pub const DESCRIPTION: &str =
    "Some characters start in the wrong position and are corrected in sequence.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::seq::SliceRandom;

#[derive(Clone, Copy, PartialEq, Debug)]
enum PairPhase {
    Waiting,
    Error,
    BlockWipeIn,
    Moving,
    BlockWipeOut,
    FinalFade,
    Done,
}

struct SwapChar {
    orig_y: usize,
    orig_x: usize,
    wrong_y: usize,
    wrong_x: usize,
    cur_y: f64,
    cur_x: f64,
    original_ch: char,
    phase: PairPhase,
    frame_count: usize,
    scene_idx: usize,
    move_progress: f64,
    move_speed: f64,
    final_color: Rgb,
}

const BLOCK_WIPE_IN: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const BLOCK_WIPE_OUT: [char; 7] = ['▇', '▆', '▅', '▄', '▃', '▂', '▁'];

pub struct ErrorCorrectEffect {
    swaps: Vec<(SwapChar, SwapChar)>,
    non_swapped: Vec<(usize, usize, char, Rgb)>,
    swap_delay: usize,
    delay_counter: usize,
    activated_up_to: usize,
    error_color: Rgb,
    correct_color: Rgb,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

impl ErrorCorrectEffect {
    pub fn new(grid: &Grid) -> Self {
        let width = grid.width;
        let height = grid.height;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("FFFFFF"),
            ],
            12,
        );
        let error_color = Rgb::from_hex("e74c3c");
        let correct_color = Rgb::from_hex("45bf55");

        let original_chars: Vec<Vec<char>> = grid
            .cells
            .iter()
            .map(|row| row.iter().map(|c| c.ch).collect())
            .collect();

        let mut text_top = usize::MAX;
        let mut text_bottom = 0usize;
        let mut text_left = usize::MAX;
        let mut text_right = 0usize;
        let mut text_positions: Vec<(usize, usize)> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                if grid.cells[y][x].ch != ' ' {
                    text_positions.push((y, x));
                    text_top = text_top.min(y);
                    text_bottom = text_bottom.max(y);
                    text_left = text_left.min(x);
                    text_right = text_right.max(x);
                }
            }
        }
        let text_h = text_bottom.saturating_sub(text_top).max(1);
        let text_w = text_right.saturating_sub(text_left).max(1);

        let final_color_at = |y: usize, x: usize| -> Rgb {
            let ry = y.saturating_sub(text_top);
            let rx = x.saturating_sub(text_left);
            final_gradient.color_at_coord(ry, rx, text_h, text_w, GradientDirection::Vertical)
        };

        let num_chars = text_positions.len();
        let num_pairs = (num_chars as f64 * 0.1) as usize;

        let mut rng = rand::thread_rng();
        let mut shuffled = text_positions.clone();
        shuffled.shuffle(&mut rng);

        let mut swapped_set = std::collections::HashSet::new();
        let mut swaps: Vec<(SwapChar, SwapChar)> = Vec::new();
        let mut i = 0;
        while swaps.len() < num_pairs && i + 1 < shuffled.len() {
            let (y1, x1) = shuffled[i];
            let (y2, x2) = shuffled[i + 1];
            i += 2;
            swapped_set.insert((y1, x1));
            swapped_set.insert((y2, x2));

            let dx = x2 as f64 - x1 as f64;
            let dy = y2 as f64 - y1 as f64;
            let aspect_dist = (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0);
            let move_speed = 0.9 / aspect_dist;

            let s1 = SwapChar {
                orig_y: y1,
                orig_x: x1,
                wrong_y: y2,
                wrong_x: x2,
                cur_y: y2 as f64,
                cur_x: x2 as f64,
                original_ch: grid.cells[y1][x1].ch,
                phase: PairPhase::Waiting,
                frame_count: 0,
                scene_idx: 0,
                move_progress: 0.0,
                move_speed,
                final_color: final_color_at(y1, x1),
            };
            let s2 = SwapChar {
                orig_y: y2,
                orig_x: x2,
                wrong_y: y1,
                wrong_x: x1,
                cur_y: y1 as f64,
                cur_x: x1 as f64,
                original_ch: grid.cells[y2][x2].ch,
                phase: PairPhase::Waiting,
                frame_count: 0,
                scene_idx: 0,
                move_progress: 0.0,
                move_speed,
                final_color: final_color_at(y2, x2),
            };
            swaps.push((s1, s2));
        }

        let mut non_swapped = Vec::new();
        for &(y, x) in &text_positions {
            if !swapped_set.contains(&(y, x)) {
                non_swapped.push((y, x, grid.cells[y][x].ch, final_color_at(y, x)));
            }
        }

        ErrorCorrectEffect {
            swaps,
            non_swapped,
            swap_delay: 6,
            delay_counter: 0,
            activated_up_to: 0,
            error_color,
            correct_color,
            width,
            height,
            original_chars,
        }
    }

    fn tick_swap_char(sc: &mut SwapChar) {
        sc.frame_count += 1;
        match sc.phase {
            PairPhase::Waiting | PairPhase::Done => {}
            PairPhase::Error => {
                if sc.frame_count >= 3 {
                    sc.frame_count = 0;
                    sc.scene_idx += 1;
                    if sc.scene_idx >= 20 {
                        sc.phase = PairPhase::BlockWipeIn;
                        sc.scene_idx = 0;
                    }
                }
            }
            PairPhase::BlockWipeIn => {
                if sc.frame_count >= 3 {
                    sc.frame_count = 0;
                    sc.scene_idx += 1;
                    if sc.scene_idx >= BLOCK_WIPE_IN.len() {
                        sc.phase = PairPhase::Moving;
                        sc.scene_idx = 0;
                        sc.move_progress = 0.0;
                    }
                }
            }
            PairPhase::Moving => {
                sc.move_progress = (sc.move_progress + sc.move_speed).min(1.0);
                sc.cur_y =
                    sc.wrong_y as f64 + (sc.orig_y as f64 - sc.wrong_y as f64) * sc.move_progress;
                sc.cur_x =
                    sc.wrong_x as f64 + (sc.orig_x as f64 - sc.wrong_x as f64) * sc.move_progress;
                if sc.move_progress >= 1.0 {
                    sc.phase = PairPhase::BlockWipeOut;
                    sc.frame_count = 0;
                    sc.scene_idx = 0;
                }
            }
            PairPhase::BlockWipeOut => {
                if sc.frame_count >= 3 {
                    sc.frame_count = 0;
                    sc.scene_idx += 1;
                    if sc.scene_idx >= BLOCK_WIPE_OUT.len() {
                        sc.phase = PairPhase::FinalFade;
                        sc.scene_idx = 0;
                    }
                }
            }
            PairPhase::FinalFade => {
                if sc.frame_count >= 3 {
                    sc.frame_count = 0;
                    sc.scene_idx += 1;
                    if sc.scene_idx >= 10 {
                        sc.phase = PairPhase::Done;
                    }
                }
            }
        }
    }

    fn render_swap_char(sc: &SwapChar, grid: &mut Grid, error_color: Rgb, correct_color: Rgb) {
        let (ry, rx) = match sc.phase {
            PairPhase::Waiting | PairPhase::Error | PairPhase::BlockWipeIn => {
                (sc.wrong_y, sc.wrong_x)
            }
            PairPhase::Moving => (sc.cur_y.round() as usize, sc.cur_x.round() as usize),
            _ => (sc.orig_y, sc.orig_x),
        };

        if ry >= grid.height || rx >= grid.width {
            return;
        }
        let cell = &mut grid.cells[ry][rx];
        cell.visible = true;

        match sc.phase {
            PairPhase::Waiting => {
                cell.ch = sc.original_ch;
                cell.fg = Some(error_color.to_crossterm());
            }
            PairPhase::Error => {
                if sc.scene_idx % 2 == 0 {
                    cell.ch = '▓';
                    cell.fg = Some(error_color.to_crossterm());
                } else {
                    cell.ch = sc.original_ch;
                    cell.fg = Some(Rgb::new(255, 255, 255).to_crossterm());
                }
            }
            PairPhase::BlockWipeIn => {
                let idx = sc.scene_idx.min(BLOCK_WIPE_IN.len() - 1);
                cell.ch = BLOCK_WIPE_IN[idx];
                cell.fg = Some(error_color.to_crossterm());
            }
            PairPhase::Moving => {
                cell.ch = '█';
                let t = sc.move_progress;
                cell.fg = Some(Rgb::lerp(error_color, correct_color, t).to_crossterm());
            }
            PairPhase::BlockWipeOut => {
                let idx = sc.scene_idx.min(BLOCK_WIPE_OUT.len() - 1);
                cell.ch = BLOCK_WIPE_OUT[idx];
                cell.fg = Some(correct_color.to_crossterm());
            }
            PairPhase::FinalFade => {
                cell.ch = sc.original_ch;
                let t = sc.scene_idx as f64 / 10.0;
                cell.fg = Some(Rgb::lerp(correct_color, sc.final_color, t).to_crossterm());
            }
            PairPhase::Done => {
                cell.ch = sc.original_ch;
                cell.fg = Some(sc.final_color.to_crossterm());
            }
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        if self.activated_up_to < self.swaps.len() {
            if self.delay_counter == 0 {
                let (s1, s2) = &mut self.swaps[self.activated_up_to];
                s1.phase = PairPhase::Error;
                s1.frame_count = 0;
                s1.scene_idx = 0;
                s2.phase = PairPhase::Error;
                s2.frame_count = 0;
                s2.scene_idx = 0;
                self.activated_up_to += 1;
                self.delay_counter = self.swap_delay;
            } else {
                self.delay_counter -= 1;
            }
        }

        for (s1, s2) in &mut self.swaps {
            Self::tick_swap_char(s1);
            Self::tick_swap_char(s2);
        }

        for (y, row) in grid.cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                cell.visible = false;
                cell.ch = self.original_chars[y][x];
                cell.fg = None;
            }
        }

        for &(y, x, ch, fc) in &self.non_swapped {
            if y < grid.height && x < grid.width {
                let cell = &mut grid.cells[y][x];
                cell.visible = true;
                cell.ch = ch;
                cell.fg = Some(fc.to_crossterm());
            }
        }

        for (s1, s2) in &self.swaps {
            Self::render_swap_char(s1, grid, self.error_color, self.correct_color);
            Self::render_swap_char(s2, grid, self.error_color, self.correct_color);
        }

        self.activated_up_to >= self.swaps.len()
            && self
                .swaps
                .iter()
                .all(|(s1, s2)| s1.phase == PairPhase::Done && s2.phase == PairPhase::Done)
    }
}

#[cfg(test)]
#[path = "../tests/effects/errorcorrect.rs"]
mod tests;
