use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = BubblesEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn bubble_falls_downward() {
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    let eff = BubblesEffect::new(&g);
    for b in &eff.bubbles {
        assert!(
            b.anchor_start_y < 0.0,
            "bubble must start above canvas, got {}",
            b.anchor_start_y
        );
        assert!(
            b.anchor_end_y > b.anchor_start_y,
            "bubble anchor must fall down (rtte top-down), start {} end {}",
            b.anchor_start_y,
            b.anchor_end_y
        );
    }
}

#[test]
fn lowest_row_matches_max_final_y() {
    let g = Grid::from_input("AAAAAA\n      \nBBBBBB");
    let eff = BubblesEffect::new(&g);
    for b in &eff.bubbles {
        let actual_max = b
            .char_indices
            .iter()
            .map(|&i| eff.chars[i].final_y)
            .max()
            .unwrap() as f64;
        assert_eq!(b.lowest_row, actual_max);
    }
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = BubblesEffect::new(&g);
    let mut grid = Grid::from_input("ABC\nDEF");
    eff.tick(&mut grid);
    for (y, row) in grid.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if !cell.visible {
                assert_eq!(
                    cell.ch, eff.original_chars[y][x],
                    "invisible cell ({y},{x}) must be reset to original",
                );
            }
        }
    }
}

#[test]
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = BubblesEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..6000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("bubbles did not complete");
}
