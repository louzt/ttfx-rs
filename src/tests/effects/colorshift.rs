use super::*;

#[test]
fn gradient_frames_is_two() {
    let g = Grid::from_input("hi");
    let eff = ColorShiftEffect::new(&g);
    assert_eq!(
        eff.gradient_frames, 2,
        "gradient_frames must be 2 (regression of 2*dm)"
    );
}

#[test]
fn enters_transition_phase_before_done() {
    let g = Grid::from_input("hi");
    let mut eff = ColorShiftEffect::new(&g);
    let mut grid = Grid::from_input("hi");
    let mut saw_transition = false;
    for _ in 0..50_000 {
        if eff.phase == Phase::Transitioning {
            saw_transition = true;
        }
        if eff.tick(&mut grid) {
            break;
        }
    }
    assert!(
        saw_transition,
        "Transitioning phase must occur (regression of brutal final snap)"
    );
}

#[test]
fn final_colors_invariant_to_empty_padding() {
    let g_tight = Grid::from_input("AAAA\nBBBB\nCCCC");
    let g_padded = Grid::from_input("AAAA\nBBBB\nCCCC\n    \n    \n    \n    \n    ");
    let eff_tight = ColorShiftEffect::new(&g_tight);
    let eff_padded = ColorShiftEffect::new(&g_padded);
    for y in 0..3 {
        for x in 0..4 {
            assert_eq!(
                eff_tight.final_colors[y][x], eff_padded.final_colors[y][x],
                "final color at ({y},{x}) must be the same with or without empty padding rows (regression of canvas-bounds gradient)"
            );
        }
    }
}

#[test]
fn completes() {
    let g = Grid::from_input("hi");
    let mut eff = ColorShiftEffect::new(&g);
    let mut grid = Grid::from_input("hi");
    for _ in 0..50_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("colorshift did not complete");
}
