// LaserEtch effect — faithful TTE reimplementation
//
// A laser beam traces across characters, etching them with heat that cools.
// Sparks fall from the etch point with bezier motion.

pub const NAME: &str = "laseretch";
pub const DESCRIPTION: &str = "A laser etches characters onto the terminal.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

#[derive(Clone)]
struct SceneFrame {
    symbol: char,
    color: Rgb,
    duration: usize,
}

#[derive(Clone)]
struct CharAnim {
    y: usize,
    x: usize,
    original_ch: char,
    visible: bool,
    scene: Vec<SceneFrame>,
    scene_idx: usize,
    hold_count: usize,
    scene_complete: bool,
    final_color: Rgb,
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
}

struct Spark {
    x: f64,
    y: f64,
    // Bezier control points
    start_x: f64,
    start_y: f64,
    ctrl_x: f64,
    ctrl_y: f64,
    end_x: f64,
    end_y: f64,
    t: f64,
    speed: f64,
    symbol: char,
    scene: Vec<SceneFrame>,
    scene_idx: usize,
    hold_count: usize,
    scene_complete: bool,
    active: bool,
}

impl Spark {
    fn tick(&mut self) {
        if !self.active {
            return;
        }
        // Advance bezier path
        self.t += self.speed;
        if self.t >= 1.0 {
            self.t = 1.0;
        }
        let t = easing::out_sine(self.t);
        // Quadratic bezier
        let inv = 1.0 - t;
        self.x = inv * inv * self.start_x + 2.0 * inv * t * self.ctrl_x + t * t * self.end_x;
        self.y = inv * inv * self.start_y + 2.0 * inv * t * self.ctrl_y + t * t * self.end_y;

        // Advance scene
        if !self.scene_complete {
            self.hold_count += 1;
            if self.hold_count >= self.scene[self.scene_idx].duration {
                self.hold_count = 0;
                self.scene_idx += 1;
                if self.scene_idx >= self.scene.len() {
                    self.scene_idx = self.scene.len() - 1;
                    self.scene_complete = true;
                    self.active = false;
                }
            }
        }
    }

    fn current_color(&self) -> Rgb {
        if self.scene.is_empty() {
            return Rgb::new(255, 255, 255);
        }
        self.scene[self.scene_idx].color
    }
}

pub struct LaserEtchEffect {
    chars: Vec<CharAnim>,
    etch_order: Vec<usize>, // indices into chars, consumed from front
    etch_pos: usize,
    etch_delay: usize,
    etch_delay_counter: usize,
    etch_speed: usize,
    active_char_indices: Vec<usize>,
    // Laser beam: diagonal line of characters above etch point
    laser_active: bool,
    laser_target_y: usize,
    laser_target_x: usize,
    laser_gradient: Vec<Rgb>,
    laser_frame: usize,
    // Sparks pool
    sparks: Vec<Spark>,
    spark_scene_template: Vec<SceneFrame>,
    spark_next: usize,
    width: usize,
    height: usize,
}

impl LaserEtchEffect {
    pub fn new(grid: &Grid) -> Self {
        let mut rng = rand::thread_rng();
        let width = grid.width;
        let height = grid.height;

        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("ffffff"),
            ],
            8,
        );
        // Looping laser gradient: forward + reverse spectrum so the cycle has
        // no seam (matches TTE's Gradient(loop=True)).
        let laser_grad = Gradient::new(&[Rgb::from_hex("ffffff"), Rgb::from_hex("376cff")], 6);
        let mut laser_spectrum = laser_grad.spectrum().to_vec();
        {
            let mut rev = laser_spectrum.clone();
            rev.reverse();
            rev.pop();
            laser_spectrum.extend(rev);
        }
        let spark_grad = Gradient::new(
            &[
                Rgb::from_hex("ffffff"),
                Rgb::from_hex("ffe680"),
                Rgb::from_hex("ff7b00"),
                Rgb::from_hex("1a0900"),
            ],
            6,
        );
        let spark_spectrum = spark_grad.spectrum().to_vec();

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

        let mut chars = Vec::with_capacity(height * width);
        for y in 0..height {
            for x in 0..width {
                let original_ch = grid.cells[y][x].ch;
                let ry = y.saturating_sub(text_top);
                let rx = x.saturating_sub(text_left);
                let final_color = final_gradient.color_at_coord(
                    ry,
                    rx,
                    text_h,
                    text_w,
                    GradientDirection::Vertical,
                );

                // Per-char cooling gradient: yellow → orange → final_color, 8 steps.
                // TTE's `Gradient(*cool_stops, final_color, steps=8)` produces 17
                // colors (2 segments × 8 + 1 final) and replaces what rtte
                // previously implemented as a separate "transition" phase.
                let cool_stops = [
                    Rgb::from_hex("ffe680"),
                    Rgb::from_hex("ff7b00"),
                    final_color,
                ];
                let cool_grad = Gradient::new(&cool_stops, 8);
                let cool_spectrum: Vec<Rgb> = cool_grad.spectrum().to_vec();

                let mut scene = Vec::with_capacity(1 + cool_spectrum.len());
                // Etch indicator "^"
                scene.push(SceneFrame {
                    symbol: '^',
                    color: Rgb::from_hex("ffe680"),
                    duration: 3,
                });
                for c in &cool_spectrum {
                    scene.push(SceneFrame {
                        symbol: original_ch,
                        color: *c,
                        duration: 3,
                    });
                }

                chars.push(CharAnim {
                    y,
                    x,
                    original_ch,
                    visible: false,
                    scene,
                    scene_idx: 0,
                    hold_count: 0,
                    scene_complete: false,
                    final_color,
                });
            }
        }

        // Build etch order using recursive backtracker maze pattern
        let etch_order = Self::build_etch_order(width, height, &mut rng);

        // Spark scene template: TTE's spark_cooling_frames default = 7.
        let spark_scene: Vec<SceneFrame> = spark_spectrum
            .iter()
            .map(|c| SceneFrame {
                symbol: '.',
                color: *c,
                duration: 7,
            })
            .collect();

        // Create spark pool
        let spark_count = 200.min(width * height);
        let sparks: Vec<Spark> = (0..spark_count)
            .map(|_| {
                let sym = match rng.gen_range(0..3) {
                    0 => '.',
                    1 => ',',
                    _ => '*',
                };
                let mut scene = spark_scene.clone();
                for f in &mut scene {
                    f.symbol = sym;
                }
                Spark {
                    x: 0.0,
                    y: 0.0,
                    start_x: 0.0,
                    start_y: 0.0,
                    ctrl_x: 0.0,
                    ctrl_y: 0.0,
                    end_x: 0.0,
                    end_y: 0.0,
                    t: 0.0,
                    speed: 0.015,
                    symbol: sym,
                    scene,
                    scene_idx: 0,
                    hold_count: 0,
                    scene_complete: false,
                    active: false,
                }
            })
            .collect();

        LaserEtchEffect {
            chars,
            etch_order,
            etch_pos: 0,
            etch_delay: 1,
            etch_delay_counter: 0,
            etch_speed: 1,
            active_char_indices: Vec::new(),
            laser_active: false,
            laser_target_y: 0,
            laser_target_x: 0,
            laser_gradient: laser_spectrum,
            laser_frame: 0,
            sparks,
            spark_scene_template: spark_scene,
            spark_next: 0,
            width,
            height,
        }
    }

    fn build_etch_order(width: usize, height: usize, rng: &mut impl Rng) -> Vec<usize> {
        // Recursive backtracker spanning tree on a grid
        // Visits every cell exactly once in a maze-like path
        let total = width * height;
        if total == 0 {
            return Vec::new();
        }

        let mut visited = vec![false; total];
        let mut order = Vec::with_capacity(total);
        let mut stack: Vec<usize> = Vec::new();

        // Start from random position
        let start = rng.gen_range(0..total);
        visited[start] = true;
        order.push(start);
        stack.push(start);

        while let Some(&current) = stack.last() {
            let cy = current / width;
            let cx = current % width;

            // Collect unvisited neighbors
            let mut neighbors = Vec::new();
            if cy > 0 {
                let n = (cy - 1) * width + cx;
                if !visited[n] {
                    neighbors.push(n);
                }
            }
            if cy + 1 < height {
                let n = (cy + 1) * width + cx;
                if !visited[n] {
                    neighbors.push(n);
                }
            }
            if cx > 0 {
                let n = cy * width + (cx - 1);
                if !visited[n] {
                    neighbors.push(n);
                }
            }
            if cx + 1 < width {
                let n = cy * width + (cx + 1);
                if !visited[n] {
                    neighbors.push(n);
                }
            }

            if neighbors.is_empty() {
                stack.pop();
            } else {
                let next = neighbors[rng.gen_range(0..neighbors.len())];
                visited[next] = true;
                order.push(next);
                stack.push(next);
            }
        }

        order
    }

    fn emit_spark(&mut self, target_y: usize, target_x: usize) {
        let mut rng = rand::thread_rng();
        let spark_len = self.sparks.len();
        let idx = self.spark_next;
        self.spark_next = (idx + 1) % spark_len;
        let spark = &mut self.sparks[idx];

        spark.start_x = target_x as f64;
        spark.start_y = target_y as f64;
        // Fall target: random x offset, bottom of canvas.
        let fall_x =
            (target_x as f64 + rng.gen_range(-20.0..=20.0)).clamp(0.0, self.width as f64 - 1.0);
        let fall_y = self.height as f64 - 1.0;
        spark.end_x = fall_x;
        spark.end_y = fall_y;
        // Bezier control: column of fall target, row offset from etch point.
        // TTE uses `position.row + random.randint(-10, 20)` in bottom-up coords,
        // which puts the control mostly *above* the etch point (creating a
        // fountain arc). In rtte's top-down coords, "above" means smaller y,
        // so the equivalent range is [-20, 10].
        spark.ctrl_x = fall_x;
        spark.ctrl_y = target_y as f64 + rng.gen_range(-20.0..=10.0);
        spark.t = 0.0;
        // Path-based speed (TTE uses speed=0.3 along the bezier with
        // double_row_diff aspect correction).
        let dy = spark.end_y - spark.start_y;
        let dx = spark.end_x - spark.start_x;
        let aspect_dist = (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0);
        spark.speed = 0.3 / aspect_dist;
        spark.x = target_x as f64;
        spark.y = target_y as f64;
        spark.scene_idx = 0;
        spark.hold_count = 0;
        spark.scene_complete = false;
        spark.active = true;
        // Randomize symbol
        let sym = match rng.gen_range(0..3) {
            0 => '.',
            1 => ',',
            _ => '*',
        };
        spark.symbol = sym;
        for f in &mut spark.scene {
            f.symbol = sym;
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        // Etch characters
        if self.etch_pos < self.etch_order.len() {
            if self.etch_delay_counter == 0 {
                for _ in 0..self.etch_speed {
                    // Skip all spaces instantly (TTE skips spaces in a tight loop)
                    while self.etch_pos < self.etch_order.len() {
                        let peek = self.etch_order[self.etch_pos];
                        if peek < self.chars.len() && self.chars[peek].original_ch == ' ' {
                            self.chars[peek].visible = true;
                            self.chars[peek].scene_complete = true;
                            self.etch_pos += 1;
                        } else {
                            break;
                        }
                    }

                    if self.etch_pos >= self.etch_order.len() {
                        break;
                    }
                    let idx = self.etch_order[self.etch_pos];
                    self.etch_pos += 1;

                    if idx < self.chars.len() {
                        self.chars[idx].visible = true;
                        self.chars[idx].scene_idx = 0;
                        self.chars[idx].hold_count = 0;
                        self.chars[idx].scene_complete = false;
                        self.active_char_indices.push(idx);

                        let ty = self.chars[idx].y;
                        let tx = self.chars[idx].x;
                        self.laser_target_y = ty;
                        self.laser_target_x = tx;
                        self.laser_active = true;

                        self.emit_spark(ty, tx);
                    }
                }
                self.etch_delay_counter = self.etch_delay;
            } else {
                self.etch_delay_counter -= 1;
            }
        } else {
            self.laser_active = false;
        }

        // Tick all active character animations
        let chars = &mut self.chars;
        for &idx in &self.active_char_indices {
            chars[idx].tick();
        }
        self.active_char_indices
            .retain(|&idx| !self.chars[idx].scene_complete);

        // Tick sparks
        for spark in &mut self.sparks {
            spark.tick();
        }

        // Advance laser gradient animation
        self.laser_frame += 1;

        // Check completion
        let all_etched = self.etch_pos >= self.etch_order.len();
        let all_cooled = self.active_char_indices.is_empty();
        let all_sparks_done = self.sparks.iter().all(|s| !s.active);

        if all_etched && all_cooled && all_sparks_done {
            // Final state
            for ca in &self.chars {
                if ca.y < grid.height && ca.x < grid.width {
                    let cell = &mut grid.cells[ca.y][ca.x];
                    cell.visible = true;
                    cell.ch = ca.original_ch;
                    cell.fg = Some(ca.final_color.to_crossterm());
                }
            }
            return true;
        }

        // Render characters to grid
        for ca in &self.chars {
            if ca.y < grid.height && ca.x < grid.width {
                let cell = &mut grid.cells[ca.y][ca.x];
                cell.visible = ca.visible;
                if ca.visible {
                    if ca.scene_complete {
                        cell.ch = ca.original_ch;
                        cell.fg = Some(ca.final_color.to_crossterm());
                    } else {
                        cell.ch = ca.current_symbol();
                        cell.fg = Some(ca.current_color().to_crossterm());
                    }
                }
            }
        }

        // Render laser beam (diagonal line from target upward-right). TTE's
        // beam has `canvas.top + 1` chars so it always extends from the etch
        // point through the top of the canvas regardless of canvas size.
        if self.laser_active {
            let laser_len = self.height + 1;
            for i in 0..laser_len {
                let by = self.laser_target_y as isize - i as isize;
                let bx = self.laser_target_x as isize + i as isize;
                if by >= 0 && by < grid.height as isize && bx >= 0 && bx < grid.width as isize {
                    let cell = &mut grid.cells[by as usize][bx as usize];
                    cell.visible = true;
                    // Looping laser gradient (mirrored for seamless cycle).
                    let gi = (self.laser_frame / 3 + i) % self.laser_gradient.len();
                    cell.fg = Some(self.laser_gradient[gi].to_crossterm());
                    if i == 0 {
                        cell.ch = '*';
                    } else {
                        cell.ch = '/';
                    }
                }
            }
        }

        // Render sparks (overlay on grid)
        for spark in &self.sparks {
            if !spark.active {
                continue;
            }
            let sy = spark.y.round() as isize;
            let sx = spark.x.round() as isize;
            if sy >= 0 && sy < grid.height as isize && sx >= 0 && sx < grid.width as isize {
                let cell = &mut grid.cells[sy as usize][sx as usize];
                cell.visible = true;
                cell.ch = spark.symbol;
                cell.fg = Some(spark.current_color().to_crossterm());
            }
        }

        false
    }
}

#[cfg(test)]
#[path = "../tests/effects/laseretch.rs"]
mod tests;
