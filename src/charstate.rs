/// Per-character animation state, motion paths, and scene system.
/// This matches TTE's base_character + animation + motion modules.
use crate::gradient::Rgb;

/// A 2D coordinate (column, row) using floats for sub-cell positioning
#[derive(Clone, Copy, Debug)]
pub struct Coord {
    pub col: f64,
    pub row: f64,
}

impl Coord {
    pub fn new(col: f64, row: f64) -> Self {
        Self { col, row }
    }

    pub fn distance_to(&self, other: &Coord) -> f64 {
        let dc = self.col - other.col;
        let dr = self.row - other.row;
        (dc * dc + dr * dr).sqrt()
    }

    pub fn as_grid(&self) -> (usize, usize) {
        (self.row.round() as usize, self.col.round() as usize)
    }
}

/// A waypoint in a motion path
#[derive(Clone, Debug)]
pub struct Waypoint {
    pub target: Coord,
    pub speed: f64,
    pub easing: fn(f64) -> f64,
}

/// Motion path with waypoints
#[derive(Clone, Debug)]
pub struct MotionPath {
    pub waypoints: Vec<Waypoint>,
    pub current_wp: usize,
    pub progress: f64, // 0.0 to 1.0 within current waypoint
    pub start: Coord,
    pub complete: bool,
}

impl MotionPath {
    pub fn new(start: Coord, waypoints: Vec<Waypoint>) -> Self {
        Self {
            waypoints,
            current_wp: 0,
            progress: 0.0,
            start,
            complete: false,
        }
    }

    pub fn single(start: Coord, target: Coord, speed: f64, easing: fn(f64) -> f64) -> Self {
        Self::new(
            start,
            vec![Waypoint {
                target,
                speed,
                easing,
            }],
        )
    }

    /// Advance the path by one frame. Returns current position.
    pub fn tick(&mut self) -> Coord {
        if self.complete || self.waypoints.is_empty() {
            return self.current_target().unwrap_or(self.start);
        }

        let wp = &self.waypoints[self.current_wp];
        let origin = if self.current_wp == 0 {
            self.start
        } else {
            self.waypoints[self.current_wp - 1].target
        };

        let dist = origin.distance_to(&wp.target);
        if dist < 0.01 {
            self.progress = 1.0;
        } else {
            self.progress += wp.speed / dist;
        }

        if self.progress >= 1.0 {
            self.progress = 1.0;
            if self.current_wp + 1 < self.waypoints.len() {
                self.current_wp += 1;
                self.progress = 0.0;
            } else {
                self.complete = true;
            }
        }

        let t = (wp.easing)(self.progress.clamp(0.0, 1.0));
        Coord {
            col: origin.col + (wp.target.col - origin.col) * t,
            row: origin.row + (wp.target.row - origin.row) * t,
        }
    }

    fn current_target(&self) -> Option<Coord> {
        self.waypoints.last().map(|wp| wp.target)
    }
}

/// A single frame in a scene animation
#[derive(Clone, Debug)]
pub struct SceneFrame {
    pub symbol: char,
    pub color: Rgb,
    pub duration: usize, // how many ticks to hold this frame
}

/// A named animation scene (sequence of frames)
#[derive(Clone, Debug)]
pub struct Scene {
    pub id: &'static str,
    pub frames: Vec<SceneFrame>,
    pub current: usize,
    pub hold_counter: usize,
    pub looping: bool,
    pub complete: bool,
}

impl Scene {
    pub fn new(id: &'static str, frames: Vec<SceneFrame>, looping: bool) -> Self {
        Self {
            id,
            frames,
            current: 0,
            hold_counter: 0,
            looping,
            complete: false,
        }
    }

    /// Create a scene from a gradient applied to a sequence of symbols
    pub fn from_gradient_symbols(
        id: &'static str,
        symbols: &[char],
        gradient: &[Rgb],
        frames_per_symbol: usize,
    ) -> Self {
        let mut frames = Vec::new();
        let total = symbols.len().max(gradient.len());
        for i in 0..total {
            let sym = symbols[i.min(symbols.len() - 1)];
            let color = gradient[i.min(gradient.len() - 1)];
            frames.push(SceneFrame {
                symbol: sym,
                color,
                duration: frames_per_symbol,
            });
        }
        Self::new(id, frames, false)
    }

    /// Create a scene that transitions a single symbol through a color gradient
    pub fn color_transition(
        id: &'static str,
        symbol: char,
        from: Rgb,
        to: Rgb,
        steps: usize,
        frames_per_step: usize,
    ) -> Self {
        let mut frames = Vec::new();
        for i in 0..=steps {
            let t = if steps == 0 {
                1.0
            } else {
                i as f64 / steps as f64
            };
            frames.push(SceneFrame {
                symbol,
                color: Rgb::lerp(from, to, t),
                duration: frames_per_step,
            });
        }
        Self::new(id, frames, false)
    }

    /// Tick the scene forward. Returns current frame.
    pub fn tick(&mut self) -> Option<&SceneFrame> {
        if self.complete || self.frames.is_empty() {
            return self.frames.last();
        }

        let result = &self.frames[self.current];
        self.hold_counter += 1;

        if self.hold_counter >= result.duration {
            self.hold_counter = 0;
            self.current += 1;
            if self.current >= self.frames.len() {
                if self.looping {
                    self.current = 0;
                } else {
                    self.current = self.frames.len() - 1;
                    self.complete = true;
                }
            }
        }

        Some(&self.frames[self.current.min(self.frames.len() - 1)])
    }

    pub fn current_frame(&self) -> Option<&SceneFrame> {
        if self.frames.is_empty() {
            None
        } else {
            Some(&self.frames[self.current.min(self.frames.len() - 1)])
        }
    }

    pub fn reset(&mut self) {
        self.current = 0;
        self.hold_counter = 0;
        self.complete = false;
    }
}

/// Per-character animation state
#[derive(Clone)]
pub struct CharState {
    /// Input (final) position on the grid
    pub input_coord: Coord,
    /// Current rendered position
    pub current_coord: Coord,
    /// Original character symbol
    pub input_symbol: char,
    /// Currently displayed symbol
    pub symbol: char,
    /// Current foreground color
    pub color: Option<Rgb>,
    /// Is the character visible?
    pub visible: bool,
    /// Active motion path
    pub motion: Option<MotionPath>,
    /// Ordered list of scenes; first non-complete plays
    pub scenes: Vec<Scene>,
    /// Index of currently active scene
    pub active_scene: usize,
    /// Has this character been activated (started its animation)?
    pub activated: bool,
    /// Arbitrary phase tag for effect-specific multi-phase logic
    pub phase: u8,
    /// Layer for render ordering (higher = on top)
    pub layer: u8,
    /// Tick counter since activation
    pub tick_count: usize,
}

impl CharState {
    pub fn new(input_symbol: char, row: usize, col: usize) -> Self {
        Self {
            input_coord: Coord::new(col as f64, row as f64),
            current_coord: Coord::new(col as f64, row as f64),
            input_symbol,
            symbol: input_symbol,
            color: None,
            visible: false,
            motion: None,
            scenes: Vec::new(),
            active_scene: 0,
            activated: false,
            phase: 0,
            layer: 0,
            tick_count: 0,
        }
    }

    /// Tick motion and animation
    pub fn tick(&mut self) {
        if !self.activated {
            return;
        }
        self.tick_count += 1;

        // Advance motion path
        if let Some(ref mut path) = self.motion {
            self.current_coord = path.tick();
        }

        // Advance active scene
        if self.active_scene < self.scenes.len() {
            let scene = &mut self.scenes[self.active_scene];
            if let Some(frame) = scene.tick() {
                self.symbol = frame.symbol;
                self.color = Some(frame.color);
            }
            if scene.complete && self.active_scene + 1 < self.scenes.len() {
                self.active_scene += 1;
            }
        }
    }

    /// Activate this character (make it start animating)
    pub fn activate(&mut self) {
        self.activated = true;
        self.visible = true;
    }

    /// Set appearance directly (bypassing scenes)
    pub fn set_appearance(&mut self, symbol: char, color: Rgb) {
        self.symbol = symbol;
        self.color = Some(color);
    }

    /// Is the motion path complete?
    pub fn motion_complete(&self) -> bool {
        self.motion.as_ref().is_none_or(|p| p.complete)
    }

    /// Is the last scene complete?
    pub fn animation_complete(&self) -> bool {
        if self.scenes.is_empty() {
            return true;
        }
        self.scenes.last().is_none_or(|s| s.complete)
    }

    /// Grid position for rendering
    pub fn grid_pos(&self) -> (usize, usize) {
        self.current_coord.as_grid()
    }

    /// Add a scene
    pub fn add_scene(&mut self, scene: Scene) {
        self.scenes.push(scene);
    }

    /// Activate a scene by id
    pub fn activate_scene(&mut self, id: &str) {
        for (i, scene) in self.scenes.iter().enumerate() {
            if scene.id == id {
                self.active_scene = i;
                self.scenes[i].reset();
                return;
            }
        }
    }
}
