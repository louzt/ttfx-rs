// Thunderstorm effect — rain + lightning strikes + sparks, then reveal

pub const NAME: &str = "thunderstorm";
pub const DESCRIPTION: &str = "Create a thunderstorm in the terminal.";
pub const EXTRA_EFFECT: bool = false;

use crate::easing;
use crate::engine::Grid;
use crate::gradient::{Gradient, GradientDirection, Rgb};
use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    PreStorm,
    Storm,
    PostStorm,
    Done,
}

struct Raindrop {
    y: f64,
    x: f64,
    speed: f64,
    symbol: char,
    active: bool,
}

struct LightningSegment {
    y: usize,
    x: usize,
    symbol: char,
    life: usize,
}

struct Spark {
    y: f64,
    x: f64,
    target_y: f64,
    target_x: f64,
    progress: f64,
    speed: f64,
    symbol: char,
    life: usize,
    active: bool,
}

pub struct ThunderstormEffect {
    phase: Phase,
    frame: usize,
    dm: usize,
    width: usize,
    height: usize,
    rain: Vec<Raindrop>,
    lightning: Vec<LightningSegment>,
    sparks: Vec<Spark>,
    storm_time: usize,
    fade_progress: f64,
    unfade_progress: f64,
    rain_delay: usize,
    original: Vec<Vec<char>>,
    flash_frames: usize,
    final_gradient: Gradient,
    lightning_color: Rgb,
    storm_color: Rgb,
    glow_color: Rgb,
    spark_color: Rgb,
    reveal_row: usize,
}

impl ThunderstormEffect {
    pub fn new(grid: &Grid) -> Self {
        let (width, height, dm) = (grid.width, grid.height, 2usize);
        let final_gradient = Gradient::new(
            &[
                Rgb::from_hex("8A008A"),
                Rgb::from_hex("00D1FF"),
                Rgb::from_hex("FFFFFF"),
            ],
            12,
        );

        let mut original = Vec::new();
        for y in 0..height {
            let row: Vec<char> = (0..width).map(|x| grid.cells[y][x].ch).collect();
            original.push(row);
        }

        ThunderstormEffect {
            phase: Phase::PreStorm,
            frame: 0,
            dm,
            width,
            height,
            rain: Vec::new(),
            lightning: Vec::new(),
            sparks: Vec::new(),
            storm_time: 12 * 60 * dm, // 12 seconds
            fade_progress: 0.0,
            unfade_progress: 0.0,
            rain_delay: 0,
            original,
            flash_frames: 0,
            final_gradient,
            lightning_color: Rgb::from_hex("68A3E8"),
            storm_color: Rgb {
                r: 20,
                g: 20,
                b: 30,
            },
            glow_color: Rgb::from_hex("EF5411"),
            spark_color: Rgb::from_hex("ff4d00"),
            reveal_row: 0,
        }
    }

    pub fn tick(&mut self, grid: &mut Grid) -> bool {
        self.frame += 1;
        let dm = self.dm;
        let mut rng = rand::thread_rng();

        match self.phase {
            Phase::PreStorm => {
                self.fade_progress += 0.005 / dm as f64;
                if self.fade_progress >= 1.0 {
                    self.fade_progress = 1.0;
                    self.phase = Phase::Storm;
                    self.frame = 0;
                }
                // Render: fade text to storm color
                for y in 0..self.height {
                    for x in 0..self.width {
                        let cell = &mut grid.cells[y][x];
                        cell.visible = true;
                        cell.ch = self.original[y][x];
                        let fc = self.final_gradient.color_at_coord(
                            y,
                            x,
                            self.height,
                            self.width,
                            GradientDirection::Vertical,
                        );
                        let color = Rgb::lerp(fc, self.storm_color, self.fade_progress);
                        cell.fg = Some(color.to_crossterm());
                    }
                }
                false
            }
            Phase::Storm => {
                // Spawn rain
                self.rain_delay = self.rain_delay.saturating_sub(1);
                if self.rain_delay == 0 {
                    let drops = rng.gen_range(1..=6);
                    let raindrop_symbols = ['\\', '.', ','];
                    for _ in 0..drops {
                        self.rain.push(Raindrop {
                            y: 0.0,
                            x: rng.gen_range(0..self.width) as f64,
                            speed: rng.gen_range(0.5..1.5),
                            symbol: raindrop_symbols[rng.gen_range(0..3)],
                            active: true,
                        });
                    }
                    self.rain_delay = rng.gen_range(1..=7) * dm;
                }

                // Move rain
                for drop in &mut self.rain {
                    if !drop.active {
                        continue;
                    }
                    drop.y += drop.speed;
                    drop.x -= 0.3; // slight wind
                    if drop.y >= self.height as f64 {
                        drop.active = false;
                    }
                }
                self.rain.retain(|d| d.active);

                // Lightning strikes
                if rng.r#gen::<f64>() < 0.008 {
                    let start_x = rng.gen_range(0..self.width);
                    let strike_symbols = ['|', '/', '\\'];
                    let mut cx = start_x;
                    let mut cy = 0;
                    while cy < self.height {
                        let sym = strike_symbols[rng.gen_range(0..3)];
                        self.lightning.push(LightningSegment {
                            y: cy,
                            x: cx,
                            symbol: sym,
                            life: 6 * dm,
                        });
                        cy += 1;
                        match sym {
                            '/' => {
                                cx = cx.saturating_sub(1);
                            }
                            '\\' if cx + 1 < self.width => {
                                cx += 1;
                            }
                            _ => {}
                        }
                        // Branch chance
                        if rng.r#gen::<f64>() < 0.05 {
                            let mut bx = cx;
                            let count = rng.gen_range(2..6);
                            for offset in 0..count {
                                let by = cy + offset;
                                if by >= self.height {
                                    break;
                                }
                                let bsym = strike_symbols[rng.gen_range(0..3)];
                                self.lightning.push(LightningSegment {
                                    y: by,
                                    x: bx,
                                    symbol: bsym,
                                    life: 4 * dm,
                                });
                                match bsym {
                                    '/' => {
                                        bx = bx.saturating_sub(1);
                                    }
                                    '\\' if bx + 1 < self.width => {
                                        bx += 1;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    self.flash_frames = 3 * dm;
                    // Sparks at bottom
                    let spark_symbols = ['*', '.', '\''];
                    for _ in 0..rng.gen_range(6..=10) {
                        let angle: f64 = rng.gen_range(-1.5..1.5);
                        let dist = rng.gen_range(4.0..20.0);
                        self.sparks.push(Spark {
                            y: (self.height - 1) as f64,
                            x: cx as f64,
                            target_y: (self.height - 1) as f64 - dist * angle.sin().abs(),
                            target_x: cx as f64 + dist * angle.cos(),
                            progress: 0.0,
                            speed: rng.gen_range(0.1..0.25) / dm as f64,
                            symbol: spark_symbols[rng.gen_range(0..3)],
                            life: 30 * dm,
                            active: true,
                        });
                    }
                }

                // Update lightning
                for seg in &mut self.lightning {
                    seg.life = seg.life.saturating_sub(1);
                }
                self.lightning.retain(|s| s.life > 0);
                self.flash_frames = self.flash_frames.saturating_sub(1);

                // Update sparks
                for spark in &mut self.sparks {
                    if !spark.active {
                        continue;
                    }
                    spark.progress += spark.speed;
                    if spark.progress >= 1.0 {
                        spark.progress = 1.0;
                    }
                    spark.life = spark.life.saturating_sub(1);
                    if spark.life == 0 {
                        spark.active = false;
                    }
                }
                self.sparks.retain(|s| s.active);

                if self.frame >= self.storm_time {
                    self.phase = Phase::PostStorm;
                    self.unfade_progress = 0.0;
                }

                // Render storm
                let bg = if self.flash_frames > 0 {
                    Rgb {
                        r: 60,
                        g: 60,
                        b: 80,
                    }
                } else {
                    self.storm_color
                };
                for y in 0..self.height {
                    for x in 0..self.width {
                        let cell = &mut grid.cells[y][x];
                        cell.visible = true;
                        cell.ch = self.original[y][x];
                        cell.fg = Some(bg.to_crossterm());
                    }
                }

                // Draw rain
                for drop in &self.rain {
                    let ry = drop.y as usize;
                    let rx = drop.x as usize;
                    if ry < self.height && rx < self.width {
                        let cell = &mut grid.cells[ry][rx];
                        cell.ch = drop.symbol;
                        cell.fg = Some(
                            Rgb {
                                r: 100,
                                g: 120,
                                b: 180,
                            }
                            .to_crossterm(),
                        );
                    }
                }

                // Draw lightning
                for seg in &self.lightning {
                    if seg.y < self.height && seg.x < self.width {
                        let cell = &mut grid.cells[seg.y][seg.x];
                        cell.ch = seg.symbol;
                        let bright = (seg.life as f64 / (6.0 * dm as f64)).min(1.0);
                        let lc = self.lightning_color.adjust_brightness(bright);
                        cell.fg = Some(lc.to_crossterm());
                    }
                }

                // Draw sparks
                for spark in &self.sparks {
                    let eased = easing::out_quint(spark.progress);
                    let sy = spark.y + (spark.target_y - spark.y) * eased;
                    let sx = spark.x + (spark.target_x - spark.x) * eased;
                    let ry = sy.round() as usize;
                    let rx = sx.round() as usize;
                    if ry < self.height && rx < self.width {
                        let cell = &mut grid.cells[ry][rx];
                        cell.ch = spark.symbol;
                        let t = spark.life as f64 / (30.0 * dm as f64);
                        let sc = Rgb::lerp(self.storm_color, self.spark_color, t);
                        cell.fg = Some(sc.to_crossterm());
                    }
                }
                false
            }
            Phase::PostStorm => {
                self.unfade_progress += 0.005 / dm as f64;
                if self.unfade_progress >= 1.0 {
                    self.unfade_progress = 1.0;
                    self.phase = Phase::Done;
                }
                for y in 0..self.height {
                    for x in 0..self.width {
                        let cell = &mut grid.cells[y][x];
                        cell.visible = true;
                        cell.ch = self.original[y][x];
                        let fc = self.final_gradient.color_at_coord(
                            y,
                            x,
                            self.height,
                            self.width,
                            GradientDirection::Vertical,
                        );
                        let color = Rgb::lerp(self.storm_color, fc, self.unfade_progress);
                        cell.fg = Some(color.to_crossterm());
                    }
                }
                false
            }
            Phase::Done => true,
        }
    }
}
