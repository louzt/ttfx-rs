use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = OrbittingVolleyEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn launchers_start_at_corners() {
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let eff = OrbittingVolleyEffect::new(&g);
    let h = (g.height as f64) - 1.0;
    let w = (g.width as f64) - 1.0;
    assert_eq!(eff.launcher_positions[0], (0.0, 0.0), "top-left");
    assert_eq!(eff.launcher_positions[1], (0.0, w), "top-right");
    assert_eq!(eff.launcher_positions[2], (h, w), "bottom-right");
    assert_eq!(eff.launcher_positions[3], (h, 0.0), "bottom-left");
}

#[test]
fn launchers_orbit_proportionally() {
    // Each launcher should reach the next corner when orbit_progress = 1.
    let positions = launcher_positions(1.0, 10, 5);
    assert_eq!(positions[0], (0.0, 9.0), "top → top-right");
    assert_eq!(positions[1], (4.0, 9.0), "right → bottom-right");
    assert_eq!(positions[2], (4.0, 0.0), "bottom → bottom-left");
    assert_eq!(positions[3], (0.0, 0.0), "left → top-left");
}

#[test]
fn no_dm_doubling_on_volley_delay() {
    let g = Grid::from_input("hi");
    let eff = OrbittingVolleyEffect::new(&g);
    assert_eq!(
        eff.volley_delay, 30,
        "regression: volley_delay must match TTE default 30 (was 60 with dm=2)"
    );
}

#[test]
fn volley_size_per_launcher_matches_tte_formula() {
    // TTE: max(1, int(volley_size * num_chars / 4)) per launcher.
    let mut input = String::new();
    for _ in 0..10 {
        input.push_str("AAAAAAAAAA\n");
    }
    let g = Grid::from_input(&input);
    let eff = OrbittingVolleyEffect::new(&g);
    let num_chars = 100;
    let expected = ((num_chars as f64 * 0.03 / 4.0) as usize).max(1);
    assert_eq!(
        eff.volley_per_launcher, expected,
        "regression: rtte was firing 4× too many chars per volley"
    );
}

#[test]
fn char_speed_uses_aspect_distance() {
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let mut eff = OrbittingVolleyEffect::new(&g);
    let mut grid = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    eff.delay_count = 0;
    eff.tick(&mut grid);
    let active = eff
        .chars
        .iter()
        .find(|c| c.active)
        .expect("at least one char active after first tick");
    let dy = active.final_y as f64 - active.start_y;
    let dx = active.final_x as f64 - active.start_x;
    let expected = 1.5 / (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0);
    assert!(
        (active.speed - expected).abs() < 1e-9,
        "regression: char speed must use hypot(dx, 2*dy)"
    );
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = OrbittingVolleyEffect::new(&g);
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
    let mut eff = OrbittingVolleyEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..10_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("orbittingvolley did not complete");
}
