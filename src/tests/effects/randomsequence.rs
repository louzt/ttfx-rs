use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = RandomSequenceEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn gradient_starts_at_starting_color() {
    // TTE's Gradient(start, final, steps=7) yields 8 colors with the FIRST
    // being the starting_color (#000000). Regression: rtte was using
    // t = (i+1)/7, skipping t=0 entirely so chars never showed pure black.
    let g = Grid::from_input("X");
    let eff = RandomSequenceEffect::new(&g);
    let first = eff.chars[0].gradient_colors[0];
    assert_eq!(
        first,
        Rgb::new(0, 0, 0),
        "first frame must be the starting_color (black)"
    );
    assert_eq!(eff.chars[0].gradient_colors.len(), 8, "must have 8 colors");
    assert_eq!(
        *eff.chars[0].gradient_colors.last().unwrap(),
        eff.chars[0].final_color,
        "last frame must be the final color"
    );
}

#[test]
fn frames_per_step_is_eight() {
    // TTE final_gradient_frames default = 8.
    let g = Grid::from_input("hi");
    let eff = RandomSequenceEffect::new(&g);
    assert_eq!(
        eff.chars[0].frames_per_step, 8,
        "regression: dm=2 doubled this to 16"
    );
}

#[test]
fn chars_per_tick_uses_non_space_count() {
    // TTE: max(int(0.007 * len(input_characters)), 1). Should round DOWN.
    let mut input = String::new();
    for _ in 0..10 {
        input.push_str("AAAAAAAAAA\n");
    }
    let g = Grid::from_input(&input);
    let eff = RandomSequenceEffect::new(&g);
    let expected = ((0.007 * 100.0) as usize).max(1);
    assert_eq!(eff.chars_per_tick, expected);
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = RandomSequenceEffect::new(&g);
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
    let mut eff = RandomSequenceEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..5_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("randomsequence did not complete");
}
