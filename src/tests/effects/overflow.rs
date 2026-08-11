use super::*;

#[test]
fn speed_is_three() {
    let g = Grid::from_input("hi");
    let eff = OverflowEffect::new(&g);
    assert_eq!(
        eff.speed, 3.0,
        "regression: speed must match TTE overflow_speed default 3 (was 1.5 with dm=2)"
    );
}

#[test]
fn final_state_has_rows_in_correct_positions() {
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    let mut eff = OverflowEffect::new(&g);
    let mut grid = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    for _ in 0..5_000 {
        if eff.tick(&mut grid) {
            break;
        }
    }
    assert_eq!(eff.phase, Phase::Done);
    // After completion the final rows must be at their expected screen rows
    // (regression of the over-scroll-then-snap Settle phase).
    let last_3 = &eff.rows[eff.rows.len() - 3..];
    for (i, row) in last_3.iter().enumerate() {
        let screen_y = row.y_offset - eff.scroll_pos;
        assert!(
            (screen_y - i as f64).abs() < 0.001,
            "final row {} should be at screen_y={}, got {}",
            i,
            i,
            screen_y
        );
        assert!(row.is_final, "row at index {} must be marked final", i);
    }
}

#[test]
fn no_settle_phase_overshoot() {
    // Stop condition should be `scroll_pos = height + overflow_rows`,
    // not `scroll_pos = last_row_offset` (which over-scrolls by height-1).
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    let mut eff = OverflowEffect::new(&g);
    let mut grid = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    while eff.phase != Phase::Done {
        if eff.tick(&mut grid) {
            break;
        }
    }
    let target = (eff.height + eff.overflow_rows) as f64;
    assert!(
        (eff.scroll_pos - target).abs() < 1e-9,
        "regression: over-scrolled past target {} (got {})",
        target,
        eff.scroll_pos
    );
}

#[test]
fn overflow_color_changes_with_screen_position() {
    // Non-final rows should use a color picked by their CURRENT screen row,
    // matching TTE's per-move set_color.
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC\nDDDDD\nEEEEE");
    let mut eff = OverflowEffect::new(&g);
    let mut grid_a = Grid::from_input("AAAAA\nBBBBB\nCCCCC\nDDDDD\nEEEEE");
    let mut grid_b = Grid::from_input("AAAAA\nBBBBB\nCCCCC\nDDDDD\nEEEEE");
    eff.tick(&mut grid_a);
    let snapshot_a: Vec<Option<crossterm::style::Color>> =
        grid_a.cells.iter().map(|row| row[0].fg).collect();
    for _ in 0..5 {
        eff.tick(&mut grid_b);
    }
    let snapshot_b: Vec<Option<crossterm::style::Color>> =
        grid_b.cells.iter().map(|row| row[0].fg).collect();
    // At least one row's color must differ between the two snapshots
    // (because the same chars are at different screen positions now and
    // therefore use different gradient indices).
    let any_changed = snapshot_a
        .iter()
        .zip(snapshot_b.iter())
        .any(|(a, b)| a != b);
    assert!(
        any_changed,
        "regression: overflow row colors must shift as rows scroll"
    );
}

#[test]
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = OverflowEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..5_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("overflow did not complete");
}
