use super::*;

#[test]
fn pairs_only_swap_non_space_cells() {
    let mut input = String::new();
    for _ in 0..10 {
        input.push_str("AAAAAAAAAA\n");
    }
    let g = Grid::from_input(&input);
    for _ in 0..50 {
        let eff = ErrorCorrectEffect::new(&g);
        for (s1, s2) in &eff.swaps {
            assert_ne!(
                s1.original_ch, ' ',
                "regression: swap pool must skip space cells"
            );
            assert_ne!(s2.original_ch, ' ');
        }
    }
}

#[test]
fn pair_count_matches_tte_formula() {
    let mut input = String::new();
    for _ in 0..10 {
        input.push_str("AAAAAAAAAA\n");
    }
    let g = Grid::from_input(&input);
    let eff = ErrorCorrectEffect::new(&g);
    let num_text_chars = 100;
    let expected = (num_text_chars as f64 * 0.1) as usize;
    assert_eq!(
        eff.swaps.len(),
        expected,
        "regression: pair count was halved by stray /2"
    );
}

#[test]
fn move_speed_proportional_to_distance() {
    let mut input = String::new();
    for _ in 0..10 {
        input.push_str("AAAAAAAAAA\n");
    }
    let g = Grid::from_input(&input);
    let eff = ErrorCorrectEffect::new(&g);
    for (s1, s2) in &eff.swaps {
        let dx = s1.orig_x as f64 - s1.wrong_x as f64;
        let dy = s1.orig_y as f64 - s1.wrong_y as f64;
        let aspect = (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0);
        let expected = 0.9 / aspect;
        assert!(
            (s1.move_speed - expected).abs() < 1e-9,
            "regression: hard-coded 0.9-per-frame speed (ignoring distance + aspect)"
        );
        assert!((s2.move_speed - expected).abs() < 1e-9);
    }
}

#[test]
fn swap_delay_is_six() {
    let g = Grid::from_input("hi\nyo");
    let eff = ErrorCorrectEffect::new(&g);
    assert_eq!(eff.swap_delay, 6, "regression: dm doubled the delay");
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABCDEFGHIJ\nABCDEFGHIJ\nABCDEFGHIJ");
    let mut eff = ErrorCorrectEffect::new(&g);
    let mut grid = Grid::from_input("ABCDEFGHIJ\nABCDEFGHIJ\nABCDEFGHIJ");
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
    let g = Grid::from_input("ABCDEFGHIJ\nABCDEFGHIJ\nABCDEFGHIJ");
    let mut eff = ErrorCorrectEffect::new(&g);
    let mut grid = Grid::from_input("ABCDEFGHIJ\nABCDEFGHIJ\nABCDEFGHIJ");
    for _ in 0..10_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("errorcorrect did not complete");
}
