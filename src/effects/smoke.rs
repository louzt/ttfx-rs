// Smoke effect — faithful TTE reimplementation
// BFS flood fill with smoke symbols, then settle to final color

pub const NAME: &str = "smoke";
pub const DESCRIPTION: &str = "Smoke floods the canvas colorizing any characters it crosses.";
pub const EXTRA_EFFECT: bool = false;

use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;
use std::collections::VecDeque;

const SMOKE_SYMBOLS: [char; 5] = ['░', '▒', '▓', '▒', '░'];

#[derive(Clone, Copy, PartialEq)]
enum SmokePhase {
    Waiting,
    Smoking,
    Paint,
    Done,
}

struct SmokeChar {
    y: usize,
    x: usize,
    original_ch: char,
    final_color: Rgb,
    phase: SmokePhase,
    frame: usize,
    smoke_total: usize,
    paint_total: usize,
}

pub struct SmokeEffect {
    chars: Vec<Vec<SmokeChar>>,
    bfs_queue: VecDeque<(usize, usize)>,
    visited: Vec<Vec<bool>>,
    bfs_delay: usize,
    bfs_counter: usize,
    smoke_gradient: Gradient,
    dm: usize,
    width: usize,
    height: usize,
    started: bool,
}

impl SmokeEffect {
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
            12,
        );
        let smoke_gradient = Gradient::new(
            &[Rgb::from_hex("242424"), Rgb::from_hex("FFFFFF")],
            SMOKE_SYMBOLS.len() * 3,
        );

        let mut chars = Vec::with_capacity(height);
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let final_color =
                    final_gradient.color_at_coord(y, x, height, width, GradientDirection::Vertical);
                row.push(SmokeChar {
                    y,
                    x,
                    original_ch: grid.cells[y][x].ch,
                    final_color,
                    phase: SmokePhase::Waiting,
                    frame: 0,
                    smoke_total: SMOKE_SYMBOLS.len() * 3 * dm,
                    paint_total: 5 * dm,
                });
            }
            chars.push(row);
        }

        let visited = vec![vec![false; width]; height];

        SmokeEffect {
            chars,
            bfs_queue: VecDeque::new(),
            visited,
            bfs_delay: 1,
            bfs_counter: 0,
            smoke_gradient,
            dm,
            width,
            height,
            started: false,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        let mut rng = rand::thread_rng();

        // Start BFS from random position
        if !self.started {
            let sy = rng.gen_range(0..self.height.max(1));
            let sx = rng.gen_range(0..self.width.max(1));
            self.bfs_queue.push_back((sy, sx));
            self.visited[sy][sx] = true;
            self.chars[sy][sx].phase = SmokePhase::Smoking;
            self.started = true;
        }

        // BFS expansion
        self.bfs_counter += 1;
        if self.bfs_counter >= self.bfs_delay {
            self.bfs_counter = 0;
            let expand_count = rng.gen_range(1..=3);
            for _ in 0..expand_count {
                if let Some((y, x)) = self.bfs_queue.pop_front() {
                    // Add neighbors
                    let dirs: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                    let mut neighbors: Vec<(usize, usize)> = Vec::new();
                    for (dy, dx) in &dirs {
                        let ny = y as isize + dy;
                        let nx = x as isize + dx;
                        if ny >= 0 && nx >= 0 {
                            let (ny, nx) = (ny as usize, nx as usize);
                            if ny < self.height && nx < self.width && !self.visited[ny][nx] {
                                neighbors.push((ny, nx));
                            }
                        }
                    }
                    use rand::seq::SliceRandom;
                    neighbors.shuffle(&mut rng);
                    for (ny, nx) in neighbors {
                        if !self.visited[ny][nx] {
                            self.visited[ny][nx] = true;
                            self.chars[ny][nx].phase = SmokePhase::Smoking;
                            self.bfs_queue.push_back((ny, nx));
                        }
                    }
                }
            }
        }

        // Tick chars
        let mut all_done = self.bfs_queue.is_empty();
        for row in &mut self.chars {
            for ch in row {
                match ch.phase {
                    SmokePhase::Waiting => {
                        all_done = false;
                    }
                    SmokePhase::Smoking => {
                        ch.frame += 1;
                        if ch.frame >= ch.smoke_total {
                            ch.phase = SmokePhase::Paint;
                            ch.frame = 0;
                        }
                        all_done = false;
                    }
                    SmokePhase::Paint => {
                        ch.frame += 1;
                        if ch.frame >= ch.paint_total {
                            ch.phase = SmokePhase::Done;
                        } else {
                            all_done = false;
                        }
                    }
                    SmokePhase::Done => {}
                }
            }
        }

        // Render
        let spec_len = self.smoke_gradient.spectrum().len().max(1);
        let starting_color = Rgb::from_hex("7A7A7A");

        for row in &self.chars {
            for ch in row {
                if ch.y >= grid.height || ch.x >= grid.width {
                    continue;
                }
                let cell = &mut grid.cells[ch.y][ch.x];
                cell.visible = true;

                match ch.phase {
                    SmokePhase::Waiting => {
                        cell.ch = ch.original_ch;
                        cell.fg = Some(starting_color.to_crossterm());
                    }
                    SmokePhase::Smoking => {
                        let sym_idx = (ch.frame / (3 * self.dm).max(1)) % SMOKE_SYMBOLS.len();
                        cell.ch = SMOKE_SYMBOLS[sym_idx];
                        let color_idx =
                            (ch.frame * spec_len / ch.smoke_total.max(1)).min(spec_len - 1);
                        cell.fg = Some(self.smoke_gradient.spectrum()[color_idx].to_crossterm());
                    }
                    SmokePhase::Paint => {
                        cell.ch = ch.original_ch;
                        let t = ch.frame as f64 / ch.paint_total as f64;
                        cell.fg = Some(
                            Rgb::lerp(Rgb::new(255, 255, 255), ch.final_color, t).to_crossterm(),
                        );
                    }
                    SmokePhase::Done => {
                        cell.ch = ch.original_ch;
                        cell.fg = Some(ch.final_color.to_crossterm());
                    }
                }
            }
        }

        all_done
    }
}
