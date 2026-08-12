use crate::effects::*;
/// Effect convergence and invariant tests.
///
/// Each effect is tested with a small fixed input. Tests verify:
/// - The effect terminates (tick() returns true) within a frame budget.
/// - After completion all non-space characters are visible.
/// - After completion the character content matches the original input.
///
/// Effects that use rand internally are non-deterministic in animation but
/// their final state must always match the original text.
use crate::engine::Grid;

// 3-row × 13-col test grid — enough to exercise most geometric logic.
const TEST_INPUT: &str = "Hello, World!\nfrom rtte 0.1\n  42 chars  ";

// Generous frame budget: at 60 fps this is >160 s of wall-clock animation.
// Effects with physics or long hold times may need the headroom.
const MAX_FRAMES: usize = 10_000;

// Minimal 1×1 grid used for edge-case tests.
const TINY_INPUT: &str = "X";

// ── helpers ──────────────────────────────────────────────────────────────────

/// Run an effect to completion. Returns the number of frames taken, or
/// panics if the effect does not terminate within MAX_FRAMES.
fn run_to_done<F>(tick: &mut F, grid: &mut Grid, effect_name: &str) -> usize
where
    F: FnMut(&mut Grid) -> bool,
{
    for frame in 0..MAX_FRAMES {
        if tick(grid) {
            return frame + 1;
        }
    }
    panic!(
        "Effect '{}' did not complete within {} frames",
        effect_name, MAX_FRAMES
    );
}

/// After a completed effect all non-space characters must be visible and
/// their character value must match the original input.
fn assert_final_state(grid: &Grid, original_input: &str) {
    // Rebuild expected characters from the same pipeline Grid::from_input uses
    // (strip ANSI, pad rows to uniform width).
    let expected = Grid::from_input(original_input);

    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = &grid[y][x];
            let orig = &expected[y][x];

            if orig.ch != ' ' {
                assert!(
                    cell.visible,
                    "cell ({},{}) char='{}' is not visible after effect completion",
                    y, x, orig.ch
                );
                assert_eq!(
                    cell.ch, orig.ch,
                    "cell ({},{}) has wrong char after completion (got '{}', want '{}')",
                    y, x, cell.ch, orig.ch
                );
            }
        }
    }
}

/// Convenience: build grid, run effect, check invariants.
macro_rules! effect_test {
    // Standard test against TEST_INPUT
    ($fn_name:ident, $EffectType:ty) => {
        #[test]
        fn $fn_name() {
            let input = TEST_INPUT;
            let mut grid = Grid::from_input(input);
            let mut effect = <$EffectType>::new(&grid);
            run_to_done(&mut |g| effect.tick(g), &mut grid, stringify!($EffectType));
            assert_final_state(&grid, input);
        }
    };
    // Test with custom input string
    ($fn_name:ident, $EffectType:ty, $input:expr) => {
        #[test]
        fn $fn_name() {
            let input = $input;
            let mut grid = Grid::from_input(input);
            let mut effect = <$EffectType>::new(&grid);
            run_to_done(&mut |g| effect.tick(g), &mut grid, stringify!($EffectType));
            assert_final_state(&grid, input);
        }
    };
}

// ── per-effect convergence tests ─────────────────────────────────────────────

effect_test!(beams_completes, BeamsEffect);
effect_test!(binarypath_completes, BinaryPathEffect);
effect_test!(blackhole_completes, BlackholeEffect);
effect_test!(bouncyballs_completes, BouncyBallsEffect);
effect_test!(bubbles_completes, BubblesEffect);
effect_test!(burn_completes, BurnEffect);
effect_test!(colorshift_completes, ColorShiftEffect);
effect_test!(crumble_completes, CrumbleEffect);
effect_test!(decrypt_completes, DecryptEffect);
effect_test!(errorcorrect_completes, ErrorCorrectEffect);
effect_test!(expand_completes, ExpandEffect);
effect_test!(fireworks_completes, FireworksEffect);
effect_test!(highlight_completes, HighlightEffect);
effect_test!(laseretch_completes, LaserEtchEffect);
effect_test!(matrix_completes, MatrixEffect);
effect_test!(middleout_completes, MiddleOutEffect);
effect_test!(orbittingvolley_completes, OrbittingVolleyEffect);
effect_test!(overflow_completes, OverflowEffect);
effect_test!(pour_completes, PourEffect);
effect_test!(print_completes, PrintEffect);
effect_test!(rain_completes, RainEffect);
effect_test!(randomsequence_completes, RandomSequenceEffect);
effect_test!(rings_completes, RingsEffect);
effect_test!(scattered_completes, ScatteredEffect);
effect_test!(slice_completes, SliceEffect);
effect_test!(slide_completes, SlideEffect);
effect_test!(smoke_completes, SmokeEffect);
effect_test!(spotlights_completes, SpotlightsEffect);
effect_test!(spray_completes, SprayEffect);
effect_test!(swarm_completes, SwarmEffect);
effect_test!(sweep_completes, SweepEffect);
effect_test!(synthgrid_completes, SynthGridEffect);
effect_test!(thunderstorm_completes, ThunderstormEffect);
effect_test!(unstable_completes, UnstableEffect);
effect_test!(vhstape_completes, VHSTapeEffect);
effect_test!(waves_completes, WavesEffect);
effect_test!(wipe_completes, WipeEffect);
effect_test!(wormhole_completes, WormholeEffect);

// ── edge-case tests (tiny / single-char input) ────────────────────────────────

effect_test!(beams_tiny, BeamsEffect, TINY_INPUT);
effect_test!(binarypath_tiny, BinaryPathEffect, TINY_INPUT);
effect_test!(blackhole_tiny, BlackholeEffect, TINY_INPUT);
effect_test!(bouncyballs_tiny, BouncyBallsEffect, TINY_INPUT);
effect_test!(bubbles_tiny, BubblesEffect, TINY_INPUT);
effect_test!(burn_tiny, BurnEffect, TINY_INPUT);
effect_test!(colorshift_tiny, ColorShiftEffect, TINY_INPUT);
effect_test!(crumble_tiny, CrumbleEffect, TINY_INPUT);
effect_test!(decrypt_tiny, DecryptEffect, TINY_INPUT);
effect_test!(errorcorrect_tiny, ErrorCorrectEffect, TINY_INPUT);
effect_test!(expand_tiny, ExpandEffect, TINY_INPUT);
effect_test!(fireworks_tiny, FireworksEffect, TINY_INPUT);
effect_test!(highlight_tiny, HighlightEffect, TINY_INPUT);
effect_test!(laseretch_tiny, LaserEtchEffect, TINY_INPUT);
effect_test!(matrix_tiny, MatrixEffect, TINY_INPUT);
effect_test!(middleout_tiny, MiddleOutEffect, TINY_INPUT);
effect_test!(orbittingvolley_tiny, OrbittingVolleyEffect, TINY_INPUT);
effect_test!(overflow_tiny, OverflowEffect, TINY_INPUT);
effect_test!(pour_tiny, PourEffect, TINY_INPUT);
effect_test!(print_tiny, PrintEffect, TINY_INPUT);
effect_test!(rain_tiny, RainEffect, TINY_INPUT);
effect_test!(randomsequence_tiny, RandomSequenceEffect, TINY_INPUT);
effect_test!(rings_tiny, RingsEffect, TINY_INPUT);
effect_test!(scattered_tiny, ScatteredEffect, TINY_INPUT);
effect_test!(slice_tiny, SliceEffect, TINY_INPUT);
effect_test!(slide_tiny, SlideEffect, TINY_INPUT);
effect_test!(smoke_tiny, SmokeEffect, TINY_INPUT);
effect_test!(spotlights_tiny, SpotlightsEffect, TINY_INPUT);
effect_test!(spray_tiny, SprayEffect, TINY_INPUT);
effect_test!(swarm_tiny, SwarmEffect, TINY_INPUT);
effect_test!(sweep_tiny, SweepEffect, TINY_INPUT);
effect_test!(synthgrid_tiny, SynthGridEffect, TINY_INPUT);
effect_test!(thunderstorm_tiny, ThunderstormEffect, TINY_INPUT);
effect_test!(unstable_tiny, UnstableEffect, TINY_INPUT);
effect_test!(vhstape_tiny, VHSTapeEffect, TINY_INPUT);
effect_test!(waves_tiny, WavesEffect, TINY_INPUT);
effect_test!(wipe_tiny, WipeEffect, TINY_INPUT);
effect_test!(wormhole_tiny, WormholeEffect, TINY_INPUT);

// ── deterministic behaviour tests ────────────────────────────────────────────

#[test]
fn wipe_activates_chars_in_diagonal_order() {
    let input = "ABCDE\nFGHIJ\nKLMNO";
    let mut grid = Grid::from_input(input);
    let mut effect = WipeEffect::new(&grid);

    for _ in 0..8 {
        effect.tick(&mut grid);
    }

    assert!(
        grid[0][0].visible,
        "wipe: top-left cell should be visible after 8 ticks"
    );
    assert!(
        !grid[2][4].visible,
        "wipe: bottom-right cell should not be visible yet after 8 ticks"
    );
}

#[test]
fn print_reveals_row_by_row() {
    let input = "ABCDE\nFGHIJ\nKLMNO";
    let mut grid = Grid::from_input(input);
    let mut effect = PrintEffect::new(&grid);

    let mut done_at: Option<usize> = None;
    for i in 0..5 {
        if effect.tick(&mut grid) {
            done_at = Some(i);
            break;
        }
    }
    assert!(
        done_at.is_none(),
        "print should still be running after a handful of ticks (done at {done_at:?})"
    );
    let bottom = grid.height - 1;
    let bottom_visible = grid[bottom].iter().any(|c| c.visible);
    assert!(
        bottom_visible,
        "print: typing row (canvas bottom) should have at least one visible cell"
    );
}

#[test]
fn grid_chars_are_preserved_through_wipe() {
    let input = "ABC";
    let mut grid = Grid::from_input(input);
    let mut effect = WipeEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "WipeEffect");

    assert_eq!(grid[0][0].ch, 'A');
    assert_eq!(grid[0][1].ch, 'B');
    assert_eq!(grid[0][2].ch, 'C');
}

#[test]
fn grid_chars_are_preserved_through_print() {
    let input = "XYZ";
    let mut grid = Grid::from_input(input);
    let mut effect = PrintEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "PrintEffect");

    assert_eq!(grid[0][0].ch, 'X');
    assert_eq!(grid[0][1].ch, 'Y');
    assert_eq!(grid[0][2].ch, 'Z');
}

// ── binarypath-specific tests ───────────────────────────────────────────────

#[test]
fn binarypath_spaces_not_corrupted() {
    let input = "A B C";
    let mut grid = Grid::from_input(input);
    let mut effect = BinaryPathEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "BinaryPathEffect");

    assert_eq!(
        grid[0][1].ch, ' ',
        "space at (0,1) was corrupted by binary digits"
    );
    assert_eq!(
        grid[0][3].ch, ' ',
        "space at (0,3) was corrupted by binary digits"
    );
    assert_eq!(grid[0][0].ch, 'A');
    assert_eq!(grid[0][2].ch, 'B');
    assert_eq!(grid[0][4].ch, 'C');
}

#[test]
fn binarypath_padding_spaces_clean() {
    let input = "Hi\nWorld!";
    let mut grid = Grid::from_input(input);
    assert_eq!(grid.width, 6);

    let mut effect = BinaryPathEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "BinaryPathEffect");

    for x in 2..6 {
        assert_eq!(
            grid[0][x].ch, ' ',
            "padding space at (0,{}) was corrupted",
            x
        );
    }
}

#[test]
fn binarypath_all_cells_correct_after_completion() {
    let input = TEST_INPUT;
    let mut grid = Grid::from_input(input);
    let expected = Grid::from_input(input);
    let mut effect = BinaryPathEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "BinaryPathEffect");

    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = &grid[y][x];
            let orig = &expected[y][x];
            assert_eq!(
                cell.ch, orig.ch,
                "cell ({},{}) has '{}' but expected '{}' after binarypath",
                y, x, cell.ch, orig.ch
            );
        }
    }
}

#[test]
fn binarypath_sparse_input_completes() {
    let input = "  A  \n     \n  B  ";
    let mut grid = Grid::from_input(input);
    let mut effect = BinaryPathEffect::new(&grid);
    let frames = run_to_done(&mut |g| effect.tick(g), &mut grid, "BinaryPathEffect");
    assert!(frames > 0, "effect should take at least 1 frame");
    assert_eq!(grid[0][2].ch, 'A');
    assert_eq!(grid[2][2].ch, 'B');
}

// ── blackhole-specific tests ────────────────────────────────────────────────

#[test]
fn blackhole_spaces_not_corrupted() {
    let input = "A B C";
    let mut grid = Grid::from_input(input);
    let expected = Grid::from_input(input);
    let mut effect = BlackholeEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "BlackholeEffect");

    for y in 0..grid.height {
        for x in 0..grid.width {
            assert_eq!(
                grid[y][x].ch, expected[y][x].ch,
                "blackhole: cell ({},{}) has '{}' expected '{}'",
                y, x, grid[y][x].ch, expected[y][x].ch
            );
        }
    }
}

#[test]
fn blackhole_sparse_input_completes() {
    let input = "  A  \n     \n  B  ";
    let mut grid = Grid::from_input(input);
    let mut effect = BlackholeEffect::new(&grid);
    let frames = run_to_done(&mut |g| effect.tick(g), &mut grid, "BlackholeEffect");
    assert!(frames > 0);
    assert_eq!(grid[0][2].ch, 'A');
    assert_eq!(grid[2][2].ch, 'B');
}

// ── wormhole-specific tests ─────────────────────────────────────────────────

#[test]
fn wormhole_spaces_not_corrupted() {
    let input = "A B C";
    let mut grid = Grid::from_input(input);
    let expected = Grid::from_input(input);
    let mut effect = WormholeEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "WormholeEffect");

    for y in 0..grid.height {
        for x in 0..grid.width {
            assert_eq!(
                grid[y][x].ch, expected[y][x].ch,
                "wormhole: cell ({},{}) has '{}' expected '{}'",
                y, x, grid[y][x].ch, expected[y][x].ch
            );
        }
    }
}

#[test]
fn wormhole_all_visible_after_completion() {
    let input = TEST_INPUT;
    let mut grid = Grid::from_input(input);
    let mut effect = WormholeEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "WormholeEffect");

    for (y, x) in Grid::from_input(input).char_positions() {
        assert!(
            grid[y][x].visible,
            "wormhole: cell ({},{}) not visible after completion",
            y, x
        );
    }
}
