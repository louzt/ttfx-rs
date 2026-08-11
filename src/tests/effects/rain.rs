use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = RainEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn first_group_is_bottom_row() {
    // TTE pops `min(group_by_row.keys())` = canvas.bottom = visual bottom.
    // In rtte top-down that's the row with the largest y.
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    let eff = RainEffect::new(&g);
    let bottom = g.height - 1;
    let first_idx = eff.pending[0];
    assert_eq!(
        eff.chars[first_idx].final_y, bottom,
        "regression: rain must start with the visual-bottom row"
    );
}

#[test]
fn speed_uses_aspect_distance() {
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC\nDDDDD\nEEEEE");
    let eff = RainEffect::new(&g);
    for ch in &eff.chars {
        let dy = ch.final_y as f64 - ch.start_y;
        let expected = (2.0 * dy).abs().max(1.0);
        let lo = 0.33 / expected;
        let hi = 0.57 / expected;
        assert!(
            ch.speed >= lo - 1e-9 && ch.speed <= hi + 1e-9,
            "regression: speed {} out of [{}..{}] for char at y={}",
            ch.speed,
            lo,
            hi,
            ch.final_y
        );
    }
}

#[test]
fn no_dm_doubling_in_fade() {
    // TTE fade = Gradient(raindrop, final, steps=7) → 8 colors × 3 frames = 24.
    let g = Grid::from_input("hi");
    let eff = RainEffect::new(&g);
    assert_eq!(eff.chars[0].fade_total, 24);
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = RainEffect::new(&g);
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
    let mut eff = RainEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..5_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("rain did not complete");
}
