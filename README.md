# ttfx-rs (Terminal Text Effects in Rust)

> High-Performance, Zero-Dependency Rust Port of TerminalTextEffects (37+ Effects)

ttfx-rs is a high-speed, 100% Rust implementation of the popular terminal text animation engine. Built for 2ms cold-start execution, 120 FPS rendering, and zero runtime dependencies.

---

## Included Animation Effects (37 total)

* **Sci-Fi & Cyberpunk**: matrix, binarypath, decrypt, errorcorrect, laseretch, synthgrid, vhstape, unstable
* **Nature & Elements**: fireworks, thunderstorm, rain, smoke, waves, burn, bubbles
* **Motion & Geometry**: blackhole, wormhole, spotlights, bouncyballs, orbittingvolley, rings, swarm, spray, sweep, crumble
* **Wipes & Reveals**: wipe, slide, slice, expand, middleout, pour, print, randomsequence, scattered, colorshift, highlight, overflow

---

## Quick Usage

```bash
# Pipe any ASCII art, text, or banner into ttfx-rs:
echo "4NV1L RUNTIME" | cargo run --release -- matrix

# Or test with custom banner:
cat my_logo.txt | cargo run --release -- fireworks
```

---

## License

MIT OR Apache-2.0 — Copyright (c) 2026 Louzt
