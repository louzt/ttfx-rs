use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = MiddleOutEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn speeds_use_aspect_corrected_distance() {
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let eff = MiddleOutEffect::new(&g);
    for ch in &eff.chars {
        let d2_y = ch.final_y as f64 - ch.mid_y;
        let d2_x = ch.final_x as f64 - ch.mid_x;
        let expected_d2 = (d2_x * d2_x + (2.0 * d2_y).powi(2)).sqrt().max(1.0);
        let expected_speed_full = 0.6 / expected_d2;
        assert!(
            (ch.speed_full - expected_speed_full).abs() < 1e-9,
            "regression: speed_full must use hypot(dx, 2*dy), not Euclidean"
        );
    }
}

#[test]
fn no_dm_doubling() {
    // For a single char at canvas center, the per-frame speed
    // should not be halved by a dm=2 multiplier.
    let g = Grid::from_input("X\nY\nZ\nW\nM");
    let eff = MiddleOutEffect::new(&g);
    // Pick a char furthest from center vertically to get a stable measurement.
    let max_speed = eff
        .chars
        .iter()
        .map(|c| c.speed_full)
        .fold(0.0_f64, f64::max);
    assert!(
        max_speed > 0.05,
        "regression: max per-frame speed too small ({}); dm=2 likely halved it",
        max_speed
    );
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = MiddleOutEffect::new(&g);
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
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = MiddleOutEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..5_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("middleout did not complete");
}
