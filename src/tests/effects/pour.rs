use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = PourEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn activation_order_is_row_bottom_to_top() {
    // For "down" pour, TTE uses ROW_BOTTOM_TO_TOP — the bottom row's chars
    // are the FIRST in the pending list. Regression: rtte was column-major.
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    let eff = PourEffect::new(&g);
    let first = &eff.chars[eff.pending[0]];
    assert_eq!(
        first.final_y,
        g.height - 1,
        "first activated char must be on the bottom row"
    );
}

#[test]
fn alternating_direction_within_rows() {
    // Bottom row group is index 0 (forward = left-to-right).
    // Next row up is index 1 (reverse = right-to-left).
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    let eff = PourEffect::new(&g);
    // first 5 entries = bottom row, left to right
    let bottom_xs: Vec<usize> = eff.pending[..5]
        .iter()
        .map(|&i| eff.chars[i].final_x)
        .collect();
    assert_eq!(bottom_xs, vec![0, 1, 2, 3, 4]);
    // next 5 entries = middle row, right to left (alternating)
    let mid_xs: Vec<usize> = eff.pending[5..10]
        .iter()
        .map(|&i| eff.chars[i].final_x)
        .collect();
    assert_eq!(mid_xs, vec![4, 3, 2, 1, 0]);
}

#[test]
fn speed_uses_aspect_distance() {
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC\nDDDDD\nEEEEE");
    let eff = PourEffect::new(&g);
    for ch in &eff.chars {
        let dy = ch.final_y as f64 - ch.start_y;
        let expected_dist = (2.0 * dy).abs().max(1.0);
        let lo = 0.4 / expected_dist;
        let hi = 0.6 / expected_dist;
        assert!(
            ch.speed >= lo - 1e-9 && ch.speed <= hi + 1e-9,
            "regression: speed {} not in [{}, {}] for char at y={}",
            ch.speed,
            lo,
            hi,
            ch.final_y
        );
    }
}

#[test]
fn gap_is_one() {
    let g = Grid::from_input("hi");
    let eff = PourEffect::new(&g);
    assert_eq!(
        eff.gap, 1,
        "regression: gap must match TTE default 1 (was 2 with dm)"
    );
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = PourEffect::new(&g);
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
    let mut eff = PourEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..5_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("pour did not complete");
}
