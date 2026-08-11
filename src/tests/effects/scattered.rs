use super::*;

#[test]
fn no_dm_doubling_in_hold() {
    // TTE _initial_hold_frames = 25. Regression: rtte was multiplying by dm=2.
    let g = Grid::from_input("hi");
    let eff = ScatteredEffect::new(&g);
    assert_eq!(eff.hold_frames, 25);
}

#[test]
fn aspect_corrected_distance() {
    // TTE uses double_row_diff=True for path-length: hypot(dx, 2*dy).
    // Regression: rtte was using plain hypot(dx, dy), so vertical motions
    // were too fast.
    let g = Grid::from_input("AB\nCD");
    let eff = ScatteredEffect::new(&g);
    let movement_speed = 0.5_f64;
    for cm in &eff.chars {
        let dy = cm.final_y as f64 - cm.start_y;
        let dx = cm.final_x as f64 - cm.start_x;
        let expected_dist = (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0);
        let expected_speed = movement_speed / expected_dist;
        assert!(
            (cm.speed - expected_speed).abs() < 1e-9,
            "speed {} doesn't match aspect-corrected expected {}",
            cm.speed,
            expected_speed
        );
    }
}

#[test]
fn gradient_starts_at_spectrum_first_stop() {
    // TTE: Gradient(spectrum[0], final, steps=10) → 11 colors, FIRST is the
    // start color (#ff9048). Regression: rtte was indexing t=(i+1)/10, so
    // the first frame was already 10% along the gradient.
    let g = Grid::from_input("X");
    let eff = ScatteredEffect::new(&g);
    let first = eff.chars[0].color_steps[0];
    assert_eq!(first, Rgb::from_hex("ff9048"));
    assert_eq!(eff.chars[0].color_steps.len(), 11);
    assert_eq!(
        *eff.chars[0].color_steps.last().unwrap(),
        eff.chars[0].final_color,
    );
}

#[test]
fn final_gradient_uses_text_bounds() {
    // Padding the input with blank rows shouldn't shift per-char colors.
    let tight = Grid::from_input("AB\nCD");
    let padded = Grid::from_input("    \n AB \n CD \n    ");
    let lookup = |eff: &ScatteredEffect, ch: char| -> Rgb {
        eff.chars
            .iter()
            .find(|c| c.original_ch == ch)
            .expect("char present")
            .final_color
    };
    let e_tight = ScatteredEffect::new(&tight);
    let e_padded = ScatteredEffect::new(&padded);
    for ch in ['A', 'B', 'C', 'D'] {
        assert_eq!(lookup(&e_tight, ch), lookup(&e_padded, ch), "char {ch}");
    }
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = ScatteredEffect::new(&g);
    let mut grid = Grid::from_input("ABC\nDEF");
    eff.tick(&mut grid);
    for (y, row) in grid.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if !cell.visible {
                assert_eq!(cell.ch, eff.original_chars[y][x]);
            }
        }
    }
}

#[test]
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = ScatteredEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..5_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("scattered did not complete");
}
