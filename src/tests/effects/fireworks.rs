use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = FireworksEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn shell_chars_share_launch_position() {
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let eff = FireworksEffect::new(&g);
    for shell in &eff.shells {
        let first = &eff.chars[shell.char_indices[0]];
        let lx = first.launch_start_x;
        let ly = first.launch_start_y;
        for &ci in &shell.char_indices {
            let c = &eff.chars[ci];
            assert!(
                (c.launch_start_x - lx).abs() < 1e-9,
                "regression: shell chars launch from different x positions"
            );
            assert!(
                (c.launch_start_y - ly).abs() < 1e-9,
                "regression: shell chars launch from different y positions"
            );
        }
    }
}

#[test]
fn origin_y_above_first_char() {
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    for _ in 0..30 {
        let eff = FireworksEffect::new(&g);
        for shell in &eff.shells {
            let first = &eff.chars[shell.char_indices[0]];
            assert!(
                first.origin_y as f64 <= first.final_y as f64,
                "origin_y {} must be at or above (rtte: <=) first char's final_y {}",
                first.origin_y,
                first.final_y
            );
        }
    }
}

#[test]
fn shells_are_consecutive_not_random() {
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let eff = FireworksEffect::new(&g);
    for shell in &eff.shells {
        for w in shell.char_indices.windows(2) {
            assert_eq!(
                w[1],
                w[0] + 1,
                "regression: shell members must be consecutive in the position list"
            );
        }
    }
}

#[test]
fn first_shell_launched_is_from_bottom_rows() {
    // TTE pops shells from the end, so the first to launch contains chars
    // nearest the bottom of the text — where origin_y has the widest random
    // range. Regression: rtte was popping from the front, making early
    // rockets always go to the top.
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let mut eff = FireworksEffect::new(&g);
    let last_shell_first_y = eff.chars[eff.shells.back().unwrap().char_indices[0]].final_y;
    let mut grid = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    eff.delay_counter = 0;
    eff.tick(&mut grid);
    let first_launched = eff
        .chars
        .iter()
        .find(|c| c.phase == FWPhase::Launch)
        .expect("expected at least one launching char on first tick");
    assert_eq!(
        first_launched.final_y, last_shell_first_y,
        "first shell launched should be the last in the construction order (bottom-up firing)"
    );
}

#[test]
fn launch_speed_uses_aspect_distance() {
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let mut eff = FireworksEffect::new(&g);
    let mut grid = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    eff.delay_counter = 0;
    eff.tick(&mut grid);
    for ch in &eff.chars {
        if ch.phase == FWPhase::Launch {
            let dy = ch.origin_y - ch.launch_start_y;
            let dx = ch.origin_x - ch.launch_start_x;
            let aspect = (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0);
            let expected = 0.35 / aspect;
            assert!(
                (ch.speed - expected).abs() < 1e-9,
                "regression: launch speed must use aspect-corrected distance"
            );
            return;
        }
    }
}

#[test]
fn trajectory_includes_falling_arc_below_explode_target() {
    // After the burst, TTE drives chars through a bloom waypoint that is up
    // to 7 rows visually below the bloom control point and then through a
    // bezier whose control sits at the canvas bottom — so the chars dip down
    // before settling. Regression: rtte previously went burst → straight
    // line to input, so they never fell.
    let g = Grid::from_input("AAAAAAAAAA\nBBBBBBBBBB\nCCCCCCCCCC\nDDDDDDDDDD\nEEEEEEEEEE");
    let eff = FireworksEffect::new(&g);
    let bottom = (g.height as f64) - 1.0;
    let mut any_fallen = false;
    for ch in &eff.chars {
        if ch.bloom_target_y >= ch.explode_target_y - 0.5 && ch.bloom_target_y > 0.0 {
            any_fallen = true;
        }
        assert!(ch.bloom_target_y <= bottom + 0.001);
        assert!(ch.fall_control_y >= bottom - 0.001);
    }
    assert!(
        any_fallen,
        "regression: no char has a bloom target below the burst point"
    );
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = FireworksEffect::new(&g);
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
    let mut eff = FireworksEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..20_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("fireworks did not complete");
}
