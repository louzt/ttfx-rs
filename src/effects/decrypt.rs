// Decrypt effect — faithful TTE reimplementation
//
// Two phases:
// 1. Typing: chars are revealed left-to-right with a brief block-char rollover
//    then settle on a random ciphertext glyph.
// 2. Decrypting: when typing finishes, ALL chars synchronously start
//    fast_decrypt → slow_decrypt → discovered.

pub const NAME: &str = "decrypt";
pub const DESCRIPTION: &str = "Display a movie style decryption effect.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

#[derive(Clone)]
struct SceneFrame {
    symbol: char,
    color: Rgb,
    duration: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum CharPhase {
    Pending,
    Typing,
    Waiting,
    FastDecrypt,
    SlowDecrypt,
    Discovered,
    Done,
}

struct CharAnim {
    y: usize,
    x: usize,
    original_ch: char,
    phase: CharPhase,
    scene: Vec<SceneFrame>,
    scene_idx: usize,
    hold_count: usize,
    scene_complete: bool,
    final_color: Rgb,
    fast_decrypt_scene: Vec<SceneFrame>,
    slow_decrypt_scene: Vec<SceneFrame>,
    discovered_scene: Vec<SceneFrame>,
}

impl CharAnim {
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
        self.scene[self.scene_idx].symbol
    }

    fn current_color(&self) -> Rgb {
        if self.scene.is_empty() {
            return self.final_color;
        }
        self.scene[self.scene_idx].color
    }

    fn activate_fast_decrypt(&mut self) {
        self.phase = CharPhase::FastDecrypt;
        self.scene = self.fast_decrypt_scene.clone();
        self.scene_idx = 0;
        self.hold_count = 0;
        self.scene_complete = false;
    }

    fn maybe_advance(&mut self) {
        if !self.scene_complete {
            return;
        }
        match self.phase {
            CharPhase::FastDecrypt => {
                self.phase = CharPhase::SlowDecrypt;
                self.scene = self.slow_decrypt_scene.clone();
                self.scene_idx = 0;
                self.hold_count = 0;
                self.scene_complete = false;
            }
            CharPhase::SlowDecrypt => {
                self.phase = CharPhase::Discovered;
                self.scene = self.discovered_scene.clone();
                self.scene_idx = 0;
                self.hold_count = 0;
                self.scene_complete = false;
            }
            CharPhase::Discovered => {
                self.phase = CharPhase::Done;
            }
            CharPhase::Typing => {
                self.phase = CharPhase::Waiting;
            }
            _ => {}
        }
    }
}

#[derive(PartialEq, Debug)]
enum EffectPhase {
    Typing,
    Decrypting,
    Complete,
}

pub struct DecryptEffect {
    chars: Vec<CharAnim>,
    typing_order: Vec<usize>,
    typing_pos: usize,
    typing_speed: usize,
    phase: EffectPhase,
    width: usize,
    height: usize,
    original_chars: Vec<Vec<char>>,
}

fn build_encrypted_symbols() -> Vec<char> {
    let mut out = Vec::with_capacity(523);
    for n in 33..=126u32 {
        if let Some(c) = char::from_u32(n) {
            out.push(c);
        }
    }
    for n in 9608..=9631u32 {
        if let Some(c) = char::from_u32(n) {
            out.push(c);
        }
    }
    for n in 9472..=9598u32 {
        if let Some(c) = char::from_u32(n) {
            out.push(c);
        }
    }
    for n in 174..=451u32 {
        if let Some(c) = char::from_u32(n) {
            out.push(c);
        }
    }
    out
}

impl DecryptEffect {
    pub fn new(grid: &Grid) -> Self {
        let mut rng = rand::thread_rng();
        let width = grid.width;
        let height = grid.height;

        let cipher_colors = [
            Rgb::from_hex("008000"),
            Rgb::from_hex("00cb00"),
            Rgb::from_hex("00ff00"),
        ];

        let final_gradient = Gradient::new(&[Rgb::from_hex("eda000")], 12);

        let encrypted_symbols = build_encrypted_symbols();
        let block_chars = ['▉', '▓', '▒', '░'];

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

        let mut chars: Vec<CharAnim> = Vec::new();
        let mut typing_order: Vec<usize> = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let original_ch = grid.cells[y][x].ch;
                if original_ch == ' ' {
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
                let cipher_color = cipher_colors[rng.gen_range(0..cipher_colors.len())];

                let mut typing_scene = Vec::new();
                for &blk in &block_chars {
                    typing_scene.push(SceneFrame {
                        symbol: blk,
                        color: cipher_colors[rng.gen_range(0..cipher_colors.len())],
                        duration: 2,
                    });
                }
                typing_scene.push(SceneFrame {
                    symbol: encrypted_symbols[rng.gen_range(0..encrypted_symbols.len())],
                    color: cipher_color,
                    duration: 1,
                });

                let fast_decrypt_scene: Vec<SceneFrame> = (0..80)
                    .map(|_| SceneFrame {
                        symbol: encrypted_symbols[rng.gen_range(0..encrypted_symbols.len())],
                        color: cipher_color,
                        duration: 2,
                    })
                    .collect();

                let slow_iters = rng.gen_range(1..=15);
                let slow_decrypt_scene: Vec<SceneFrame> = (0..slow_iters)
                    .map(|_| {
                        let dur = if rng.gen_range(0..=100) <= 30 {
                            rng.gen_range(35..60)
                        } else {
                            rng.gen_range(3..6)
                        };
                        SceneFrame {
                            symbol: encrypted_symbols[rng.gen_range(0..encrypted_symbols.len())],
                            color: cipher_color,
                            duration: dur,
                        }
                    })
                    .collect();

                let discover_steps = 10;
                let discovered_scene: Vec<SceneFrame> = (0..discover_steps)
                    .map(|i| {
                        let t = (i + 1) as f64 / discover_steps as f64;
                        let color = Rgb::lerp(Rgb::new(255, 255, 255), final_color, t);
                        SceneFrame {
                            symbol: original_ch,
                            color,
                            duration: 5,
                        }
                    })
                    .collect();

                let idx = chars.len();
                typing_order.push(idx);
                chars.push(CharAnim {
                    y,
                    x,
                    original_ch,
                    phase: CharPhase::Pending,
                    scene: typing_scene,
                    scene_idx: 0,
                    hold_count: 0,
                    scene_complete: false,
                    final_color,
                    fast_decrypt_scene,
                    slow_decrypt_scene,
                    discovered_scene,
                });
            }
        }

        DecryptEffect {
            chars,
            typing_order,
            typing_pos: 0,
            typing_speed: 2,
            phase: EffectPhase::Typing,
            width,
            height,
            original_chars,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let mut rng = rand::thread_rng();

        match self.phase {
            EffectPhase::Typing => {
                if self.typing_pos < self.typing_order.len() && rng.gen_range(0..=100) <= 75 {
                    for _ in 0..self.typing_speed {
                        if self.typing_pos >= self.typing_order.len() {
                            break;
                        }
                        let idx = self.typing_order[self.typing_pos];
                        self.typing_pos += 1;
                        let ca = &mut self.chars[idx];
                        ca.phase = CharPhase::Typing;
                        ca.scene_idx = 0;
                        ca.hold_count = 0;
                        ca.scene_complete = false;
                    }
                }
                let typing_done = self.typing_pos >= self.typing_order.len()
                    && self
                        .chars
                        .iter()
                        .all(|c| c.phase != CharPhase::Typing || c.scene_complete);
                if typing_done {
                    for c in &mut self.chars {
                        if c.phase != CharPhase::Pending {
                            c.activate_fast_decrypt();
                        }
                    }
                    self.phase = EffectPhase::Decrypting;
                }
            }
            EffectPhase::Decrypting => {
                let all_done = self
                    .chars
                    .iter()
                    .all(|c| c.phase == CharPhase::Done || c.phase == CharPhase::Pending);
                if all_done {
                    self.phase = EffectPhase::Complete;
                }
            }
            EffectPhase::Complete => {}
        }

        for c in &mut self.chars {
            match c.phase {
                CharPhase::Pending | CharPhase::Waiting | CharPhase::Done => {}
                _ => {
                    c.tick();
                    c.maybe_advance();
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

        for c in &self.chars {
            if c.y >= grid.height || c.x >= grid.width {
                continue;
            }
            let cell = &mut grid.cells[c.y][c.x];
            match c.phase {
                CharPhase::Pending => {}
                CharPhase::Done => {
                    cell.visible = true;
                    cell.ch = c.original_ch;
                    cell.fg = Some(c.final_color.to_crossterm());
                }
                _ => {
                    cell.visible = true;
                    cell.ch = c.current_symbol();
                    cell.fg = Some(c.current_color().to_crossterm());
                }
            }
        }

        self.phase == EffectPhase::Complete
    }
}

#[cfg(test)]
#[path = "../tests/effects/decrypt.rs"]
mod tests;
