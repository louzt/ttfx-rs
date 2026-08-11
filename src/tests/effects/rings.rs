use super::*;

#[test]
fn skips_space_cells() {
    // Regression: rtte previously created a CharState for every grid cell
    // including spaces, dragging blanks through the animation.
    let g = Grid::from_input("a b\nc d");
    let eff = RingsEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn no_dm_doubling_in_durations() {
    // TTE defaults: spin_duration=200, disperse_duration=200, cycles=3.
    // Regression: rtte was multiplying by dm=2.
    let g = Grid::from_input("hi");
    let eff = RingsEffect::new(&g);
    assert_eq!(eff.spin_duration, 200);
    assert_eq!(eff.disperse_duration, 200);
    assert_eq!(eff.cycles_remaining, 3);
}

#[test]
fn start_phase_lasts_100_frames() {
    // TTE: _initial_phase_time_remaining = 100; chars sit at home for 100
    // ticks before the disperse phase begins.
    let g = Grid::from_input("ABCDE");
    let mut eff = RingsEffect::new(&g);
    let mut grid = Grid::from_input("ABCDE");
    for _ in 0..100 {
        eff.tick(&mut grid);
        assert_eq!(eff.phase, Phase::Start);
    }
    eff.tick(&mut grid);
    assert_eq!(eff.phase, Phase::Disperse);
}

#[test]
fn final_gradient_uses_text_bounds() {
    // The same text should yield the same per-char colors regardless of how
    // much surrounding blank canvas there is. Regression: rtte was using
    // canvas bounds, so adding empty rows around the text shifted colors.
    let tight = Grid::from_input("AB\nCD");
    let padded = Grid::from_input("    \n AB \n CD \n    ");
    let e_tight = RingsEffect::new(&tight);
    let e_padded = RingsEffect::new(&padded);

    let lookup = |eff: &RingsEffect, ch: char| -> Rgb {
        eff.chars
            .iter()
            .find(|c| c.original_ch == ch)
            .expect("char present")
            .final_color
    };
    for ch in ['A', 'B', 'C', 'D'] {
        assert_eq!(
            lookup(&e_tight, ch),
            lookup(&e_padded, ch),
            "char {ch}: gradient must use text bounds, not canvas bounds"
        );
    }
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = RingsEffect::new(&g);
    let mut grid = Grid::from_input("ABC\nDEF");
    eff.tick(&mut grid);
    for (y, row) in grid.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if !cell.visible {
                assert_eq!(
                    cell.ch, eff.original_chars[y][x],
                    "invisible cell ({y},{x}) must be reset to original"
                );
            }
        }
    }
}

#[test]
fn rings_assigned_palette_colors_in_order() {
    // TTE: ring N is colored ring_colors[N % len(ring_colors)] — NOT random
    // per character. Regression: rtte was assigning by character.
    let g = Grid::from_input(
        "AAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAA",
    );
    let eff = RingsEffect::new(&g);
    if !eff.rings.is_empty() {
        let palette = [
            Rgb::from_hex("ab48ff"),
            Rgb::from_hex("e7b2b2"),
            Rgb::from_hex("fffebd"),
        ];
        for (i, ring) in eff.rings.iter().enumerate() {
            assert_eq!(ring.ring_color, palette[i % palette.len()]);
        }
    }
}

#[test]
fn ring_circle_doubles_x_distance() {
    // TTE's find_coords_on_circle doubles the x-distance from origin to
    // correct for terminal cell aspect ratio (cells are ~2:1 height:width).
    // Regression: rtte was drawing plain r*cos(θ), so rings rendered half
    // as wide as TTE's.
    let mut input = String::new();
    for _ in 0..30 {
        input.push_str(&"A".repeat(80));
        input.push('\n');
    }
    let g = Grid::from_input(&input);
    let eff = RingsEffect::new(&g);
    assert!(!eff.rings.is_empty());
    let ring = &eff.rings[0];
    let cy = (g.height as f64 - 1.0) / 2.0;
    let cx = (g.width as f64 - 1.0) / 2.0;
    let max_dx = ring
        .coords
        .iter()
        .map(|(_, x)| (x - cx).abs())
        .fold(0.0_f64, f64::max);
    let max_dy = ring
        .coords
        .iter()
        .map(|(y, _)| (y - cy).abs())
        .fold(0.0_f64, f64::max);
    assert!(
        (max_dx - 2.0 * max_dy).abs() < 0.5,
        "ring should be ~2× wider than tall (got dx={max_dx}, dy={max_dy})"
    );
}

#[test]
fn ring_rotation_speed_in_default_range() {
    // TTE default spin_speed = (0.25, 1.0).
    let g = Grid::from_input("AAAAAAAAAA\nAAAAAAAAAA\nAAAAAAAAAA\nAAAAAAAAAA\nAAAAAAAAAA");
    let eff = RingsEffect::new(&g);
    for ring in &eff.rings {
        assert!(
            ring.rotation_speed >= 0.25 && ring.rotation_speed <= 1.0,
            "ring rotation_speed {} out of [0.25..1.0]",
            ring.rotation_speed
        );
    }
}

#[test]
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = RingsEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..20_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("rings did not complete");
}
