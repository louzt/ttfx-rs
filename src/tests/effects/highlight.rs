use super::*;

#[test]
fn highlight_gradient_has_15_colors_starting_with_base() {
    let g = Grid::from_input("AAAAA");
    let eff = HighlightEffect::new(&g);
    let ch = &eff.chars[0];
    // TTE's Gradient(base, h, h, base, steps=(3, 8, 3)) yields 15 colors.
    assert_eq!(
        ch.highlight_colors.len(),
        15,
        "regression: highlight gradient must have 15 colors (3 + 8 + 3 + 1)"
    );
    assert_eq!(
        ch.highlight_colors[0], ch.base_color,
        "regression: first highlight frame must be base (rtte previously skipped it)"
    );
    assert_eq!(
        ch.highlight_colors[14], ch.base_color,
        "last highlight frame must be base (final stop)"
    );
}

#[test]
fn brightness_uses_hsl_lightness() {
    // Pure red (255, 0, 0) at brightness 1.5 in HSL keeps R=255 and lifts G,B.
    // Old RGB-scale logic would just produce (255, 0, 0) (no change since one
    // channel is already at max and others are 0).
    let red = Rgb::new(255, 0, 0);
    let brightened = red.adjust_brightness(1.5);
    assert!(
        brightened.g > 0 || brightened.b > 0,
        "regression: HSL-based brightening must lift the non-dominant channels for pure colors, got {:?}",
        brightened
    );
}

#[test]
fn frames_per_tick_is_two() {
    let g = Grid::from_input("AAA");
    let eff = HighlightEffect::new(&g);
    assert_eq!(
        eff.chars[0].frames_per_tick, 2,
        "regression: dm doubled the per-frame duration to 4"
    );
}

#[test]
fn easer_completes_in_100_steps() {
    // TTE's SequenceEaser uses total_steps=100 regardless of sequence length.
    let g = Grid::from_input("AAA");
    let eff = HighlightEffect::new(&g);
    assert!(
        (eff.easer_speed - 0.01).abs() < 1e-9,
        "regression: easer speed must be 1/100, got {}",
        eff.easer_speed
    );
}

#[test]
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = HighlightEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..2_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("highlight did not complete");
}
