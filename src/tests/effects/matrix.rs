use super::*;

#[test]
fn rain_time_is_15_seconds_at_60fps() {
    let g = Grid::from_input("hi");
    let eff = MatrixEffect::new(&g);
    assert_eq!(
        eff.rain_time, 900,
        "regression: rain_time must be 15s × 60fps = 900 frames (was doubled by dm)"
    );
}

#[test]
fn column_speeds_are_in_tte_range() {
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let eff = MatrixEffect::new(&g);
    for col in &eff.columns {
        assert!(
            (2..=15).contains(&col.speed),
            "regression: column speed {} outside [2,15] (was multiplied by dm)",
            col.speed
        );
    }
}

#[test]
fn unresolved_cells_keep_stable_symbols_during_resolve() {
    // Drive the effect into Resolve phase, then ensure unresolved cells'
    // glyphs only swap rarely (regression of writing a fresh random char
    // every frame, causing massive flicker).
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let mut eff = MatrixEffect::new(&g);
    let mut grid = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    eff.frame = eff.rain_time + 1;
    for col in &mut eff.columns {
        col.length = eff.height + 5;
        col.active = true;
    }
    while eff.phase != Phase::Resolve {
        if eff.tick(&mut grid) {
            break;
        }
    }
    let snapshot: Vec<Vec<char>> = grid
        .cells
        .iter()
        .map(|row| row.iter().map(|c| c.ch).collect())
        .collect();
    let mut diff_total = 0;
    let mut compared = 0;
    for _ in 0..30 {
        eff.tick(&mut grid);
        for (y, row) in grid.cells.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if !eff.chars[y][x].resolved {
                    compared += 1;
                    if cell.ch != snapshot[y][x] {
                        diff_total += 1;
                    }
                }
            }
        }
    }
    let diff_ratio = diff_total as f64 / compared.max(1) as f64;
    assert!(
        diff_ratio < 0.1,
        "regression: unresolved cells flicker too much ({:.2}% changed per tick)",
        diff_ratio * 100.0
    );
}

#[test]
fn resolve_step_total_is_24() {
    // Each char's transition should span 24 frames, matching TTE's
    // 8-step gradient × final_gradient_frames=3.
    assert_eq!(RESOLVE_TOTAL, 24);
}

#[test]
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = MatrixEffect::new(&g);
    eff.rain_time = 30;
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..50_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("matrix did not complete");
}
