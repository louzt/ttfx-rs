# ttfx-rs (Terminal Text Effects in Rust)

> High-Performance, Zero-Dependency Rust Port of TerminalTextEffects (38 Effects)

`ttfx-rs` is a high-speed, 100% Rust implementation of the popular terminal text animation engine. Built for 2ms cold-start execution, 120 FPS rendering, zero runtime dependencies, and dual-mode execution (CLI binary + Rust library crate).

* **Live Interactive Web Showcase**: [https://louzt.github.io/ttfx-rs/](https://louzt.github.io/ttfx-rs/)
* **GitHub Repository**: [https://github.com/louzt/ttfx-rs](https://github.com/louzt/ttfx-rs)

---

## Key Features

* **38 Native ANSI Animation Effects**: Full physics-driven particle animations (Matrix, Fireworks, Burn, Blackhole, Laser Etch, VHS Tape, Decrypt, Synthgrid, etc.).
* **Dual-Target Crate Architecture**: Functions as both a standalone CLI binary (`ttfx`) and an embeddable Rust library crate (`lib.rs`).
* **Ultra-Fast & Lightweight**: 3.3 MB static binary, 2ms startup time, 120 FPS rendering engine.
* **100% Zero Runtime Dependencies**: Pure Rust implementation with cross-platform terminal ANSI support via `crossterm`.

---

## Included Animation Effects (38 total)

* **Sci-Fi & Cyberpunk**: matrix, binarypath, decrypt, errorcorrect, laseretch, synthgrid, vhstape, unstable
* **Nature & Elements**: fireworks, thunderstorm, rain, smoke, waves, burn, bubbles
* **Motion & Geometry**: blackhole, wormhole, spotlights, bouncyballs, orbittingvolley, rings, swarm, spray, sweep, crumble
* **Wipes & Reveals**: wipe, slide, slice, expand, middleout, pour, print, randomsequence, scattered, colorshift, highlight, overflow

---

## CLI Installation & Usage

### 1. Installation

Install globally on your system using Cargo:

```bash
cargo install --git https://github.com/louzt/ttfx-rs
```

### 2. Basic Pipeline Usage

Pipe any text, ASCII art, or banner into `ttfx`:

```bash
# Apply Matrix effect to custom text:
echo "LOUZT" | ttfx matrix

# Apply Burn effect to a banner file:
cat my_logo.txt | ttfx burn

# Apply a random effect:
echo "4NV1L RUNTIME" | ttfx --random-effect
```

---

## Rust Library Integration

You can import `ttfx-rs` directly into any Rust application, TUI tool, or CLI framework.

### 1. Add Dependency to `Cargo.toml`

```toml
[dependencies]
ttfx-rs = { git = "https://github.com/louzt/ttfx-rs.git" }
```

### 2. Example Rust Code

```rust
use ttfx_rs::{ALL_EFFECTS, Grid, run_animation};

fn main() {
    let input_text = "LOUZT WORKSPACE MANAGER";
    let mut grid = Grid::from_input(input_text);

    // Find desired effect by name
    let info = ALL_EFFECTS
        .iter()
        .find(|e| e.name == "laseretch")
        .expect("Effect not found");

    // Instantiate effect
    let mut effect = (info.create)(&grid);

    // Run animation in terminal at 60 FPS
    run_animation(&mut grid, 60, |g, _frame| effect.tick(g));
}
```

---

## License

MIT OR Apache-2.0 — Copyright (c) 2026 Louzt
