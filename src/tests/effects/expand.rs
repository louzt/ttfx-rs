use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = ExpandEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn speed_uses_aspect_corrected_distance() {
    let g = Grid::from_input(
        "..........\n..........\n..........\n..........\n..........\n..........\n..........\n..........\n..........\n..........",
    );
    let eff = ExpandEffect::new(&g);
    for ch in &eff.chars {
        let dy = ch.final_y as f64 - eff.center_y;
        let dx = ch.final_x as f64 - eff.center_x;
        let expected_dist = (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0);
        let expected_speed = 0.35 / expected_dist;
        assert!(
            (ch.speed - expected_speed).abs() < 1e-9,
            "regression: speed must use hypot(dx, 2*dy), not Euclidean"
        );
    }
}

#[test]
fn no_dm_doubling() {
    let g = Grid::from_input("X");
    let eff = ExpandEffect::new(&g);
    // expected with dm=1: 0.35 / sqrt(1.25) ≈ 0.313
    // regression with dm=2: ≈ 0.157
    assert!(
        eff.chars[0].speed > 0.25,
        "speed {} too low (regression of dm=2 halving the speed)",
        eff.chars[0].speed
    );
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = ExpandEffect::new(&g);
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
    let mut eff = ExpandEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..2_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("expand did not complete");
}
