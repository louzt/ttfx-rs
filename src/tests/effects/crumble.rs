use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = CrumbleEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn all_chars_start_pending() {
    let g = Grid::from_input("hi\nyo");
    let eff = CrumbleEffect::new(&g);
    assert_eq!(eff.stage, Stage::Falling);
    for ch in &eff.chars {
        assert_eq!(
            ch.phase,
            CrumblePhase::Pending,
            "regression: chars must start Pending and be activated by stage logic"
        );
    }
    assert_eq!(eff.pending.len(), eff.chars.len());
    assert_eq!(eff.unvacuumed.len(), eff.chars.len());
}

#[test]
fn vacuum_path_curves_through_center_for_off_center_chars() {
    let g =
        Grid::from_input("AAAAAAAAAA\n          \n          \n          \n          \n         X");
    let eff = CrumbleEffect::new(&g);
    let last = eff.chars.last().unwrap();
    let cx = (g.width as f64) / 2.0;
    let cy = (g.height as f64) / 2.0;
    assert_eq!(
        last.vacuum_ctrl,
        (cx, cy),
        "vacuum bezier must curve through canvas center"
    );
    assert!(
        last.vacuum_start.0 != last.vacuum_ctrl.0,
        "test setup: char column must differ from center"
    );
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = CrumbleEffect::new(&g);
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
    let mut eff = CrumbleEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..20_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("crumble did not complete");
}
