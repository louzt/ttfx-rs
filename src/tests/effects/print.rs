use super::*;

#[test]
fn first_row_starts_at_canvas_bottom() {
    // Typing always begins at canvas bottom, regardless of which input row
    // is being processed (TTE behavior: rows enter at row 1 and scroll up).
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    let mut eff = PrintEffect::new(&g);
    let mut grid = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    eff.tick(&mut grid);
    let typing_row = eff.typing_row.expect("first row should be active");
    assert_eq!(typing_row, 0, "must process input row 0 first");
    assert_eq!(
        eff.cur_y[0],
        (g.height as isize) - 1,
        "first typed row must be at canvas bottom"
    );
}

#[test]
fn rows_scroll_up_as_new_rows_begin() {
    let g = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    let mut eff = PrintEffect::new(&g);
    let mut grid = Grid::from_input("AAAAA\nBBBBB\nCCCCC");
    for _ in 0..2_000 {
        if eff.tick(&mut grid) {
            break;
        }
    }
    // After completion each input row should be at its input position.
    assert_eq!(eff.cur_y[0], 0, "input row 0 must end at canvas top");
    assert_eq!(eff.cur_y[1], 1);
    assert_eq!(eff.cur_y[2], 2);
}

#[test]
fn does_not_skip_leading_input_spaces_after_carriage_return() {
    // Each row's typing starts at col 0 and types every input character
    // (including leading spaces) — TTE does the same, only skipping FILL
    // chars added by canvas padding (rtte's Grid never produces fill).
    let g = Grid::from_input("AAAAA\n  CCC\nDDDDD");
    let mut eff = PrintEffect::new(&g);
    let mut grid = Grid::from_input("AAAAA\n  CCC\nDDDDD");
    for _ in 0..5_000 {
        if eff.tick(&mut grid) {
            break;
        }
        if eff.typing_row == Some(1) {
            assert_eq!(
                eff.col_pos, 0,
                "regression: rtte was skipping leading input spaces; \
                 row 1's typing should start at col 0, got col_pos={}",
                eff.col_pos
            );
            return;
        }
    }
    panic!("never reached typing of row 1");
}

#[test]
fn print_speed_is_two() {
    let g = Grid::from_input("hi");
    let eff = PrintEffect::new(&g);
    assert_eq!(eff.print_speed, 2);
}

#[test]
fn cr_speed_per_unit_is_1_5() {
    let g = Grid::from_input("hi");
    let eff = PrintEffect::new(&g);
    assert!(
        (eff.cr_speed_per_unit - 1.5).abs() < 1e-9,
        "regression: print_head_return_speed must match TTE default 1.5"
    );
}

#[test]
fn space_cells_settle_to_space_after_animation() {
    // TTE plays the full typed animation (█→▓→▒→░→input_symbol) for every
    // char including spaces, so the cell briefly shows blocks then settles
    // on the input symbol. The previous bug was that spaces never progressed
    // — they were stuck at scene[0] = '█' forever.
    let g = Grid::from_input("A B");
    let mut eff = PrintEffect::new(&g);
    let mut grid = Grid::from_input("A B");
    // Run long enough for col 1 (the space) to finish its animation:
    // typed at tick 1, animation = 5 frames × 3 ticks = 15 ticks.
    for _ in 0..40 {
        eff.tick(&mut grid);
    }
    let bottom = grid.height - 1;
    let space_pos = grid.cells.iter().enumerate().find_map(|(y, row)| {
        if row[1].ch == ' ' && row[1].visible {
            Some(y)
        } else {
            None
        }
    });
    assert!(
        space_pos.is_some(),
        "regression: typed space cell never settled to ' '; \
         current grid row {} col 1 = '{}'",
        bottom,
        grid.cells[bottom][1].ch
    );
}

#[test]
fn line_feed_happens_before_carriage_return() {
    // TTE pushes the just-typed row up BEFORE starting the CR motion so the
    // head moves across an empty bottom row. Regression: rtte was scrolling
    // up after CR finished, making the head appear to retrace the just-typed
    // line backwards.
    let g = Grid::from_input("AB\nCD");
    let mut eff = PrintEffect::new(&g);
    let mut grid = Grid::from_input("AB\nCD");
    let bottom = (g.height as isize) - 1;
    while eff.phase != Phase::CarriageReturn {
        if eff.tick(&mut grid) {
            panic!("never reached carriage return");
        }
    }
    assert_eq!(
        eff.cur_y[0],
        bottom - 1,
        "row 0 must be one above canvas bottom while CR is happening"
    );
}

#[test]
fn typing_head_hidden_during_typing() {
    // TTE only shows the typing head during carriage return; during
    // typing each char's first animation frame ('█') IS the head visual,
    // so a separate head would over-paint just-typed cells.
    let g = Grid::from_input("ABCDE");
    let mut eff = PrintEffect::new(&g);
    let mut grid = Grid::from_input("ABCDE");
    eff.tick(&mut grid);
    assert!(
        !eff.head_visible,
        "regression: head must be hidden during typing"
    );
}

#[test]
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = PrintEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..5_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("print did not complete");
}
