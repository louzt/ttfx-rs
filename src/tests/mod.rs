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
            let cell = &grid.cells[y][x];
            let orig = &expected.cells[y][x];

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
//
// For effects with no random component we can assert specific properties
// about intermediate or final state in addition to convergence.

#[test]
fn wipe_activates_chars_in_diagonal_order() {
    // In the wipe effect characters at x+y=0 (top-left) activate before
    // characters at larger diagonals.
    //
    // The easer uses in_out_circ which is very slow to start (S-curve),
    // so we need ~10 ticks before the first diagonal is activated on a
    // 5×3 grid (7 groups, dm=2, easer_speed ≈ 0.071).
    let input = "ABCDE\nFGHIJ\nKLMNO";
    let mut grid = Grid::from_input(input);
    let mut effect = WipeEffect::new(&grid);

    // At 8 ticks the in_out_circ easer is mid-curve: groups 0–4 activated,
    // groups 5–6 still pending (all 7 groups activate around tick 11).
    for _ in 0..8 {
        effect.tick(&mut grid);
    }

    // Top-left (diagonal 0) activates around tick 4 — must be visible.
    assert!(
        grid.cells[0][0].visible,
        "wipe: top-left cell should be visible after 8 ticks"
    );
    // Bottom-right (diagonal 6) activates around tick 11 — must still be hidden.
    assert!(
        !grid.cells[2][4].visible,
        "wipe: bottom-right cell should not be visible yet after 8 ticks"
    );
}

#[test]
fn print_reveals_row_by_row() {
    // The print effect types each row at the canvas bottom and scrolls
    // already-typed rows up. After only a few ticks at least one cell on
    // the bottom row must be visible but the effect must not yet be done.
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
    let bottom_visible = grid.cells[bottom].iter().any(|c| c.visible);
    assert!(
        bottom_visible,
        "print: typing row (canvas bottom) should have at least one visible cell"
    );
}

#[test]
fn grid_chars_are_preserved_through_wipe() {
    // Explicit character-by-character check on a known string.
    let input = "ABC";
    let mut grid = Grid::from_input(input);
    let mut effect = WipeEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "WipeEffect");

    assert_eq!(grid.cells[0][0].ch, 'A');
    assert_eq!(grid.cells[0][1].ch, 'B');
    assert_eq!(grid.cells[0][2].ch, 'C');
}

#[test]
fn grid_chars_are_preserved_through_print() {
    let input = "XYZ";
    let mut grid = Grid::from_input(input);
    let mut effect = PrintEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "PrintEffect");

    assert_eq!(grid.cells[0][0].ch, 'X');
    assert_eq!(grid.cells[0][1].ch, 'Y');
    assert_eq!(grid.cells[0][2].ch, 'Z');
}

// ── binarypath-specific tests ───────────────────────────────────────────────

/// Space cells must remain spaces after binarypath completes.
/// Regression test: binary digits flying through space positions used to
/// overwrite cell.ch with '0'/'1', which persisted into the final frame.
#[test]
fn binarypath_spaces_not_corrupted() {
    let input = "A B C";
    let mut grid = Grid::from_input(input);
    let mut effect = BinaryPathEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "BinaryPathEffect");

    // Positions 1 and 3 are spaces — they must still be spaces.
    assert_eq!(
        grid.cells[0][1].ch, ' ',
        "space at (0,1) was corrupted by binary digits"
    );
    assert_eq!(
        grid.cells[0][3].ch, ' ',
        "space at (0,3) was corrupted by binary digits"
    );
    // And the actual characters must be correct.
    assert_eq!(grid.cells[0][0].ch, 'A');
    assert_eq!(grid.cells[0][2].ch, 'B');
    assert_eq!(grid.cells[0][4].ch, 'C');
}

/// Trailing padding spaces (from multi-line input where lines differ in
/// length) must not show binary digits after completion.
#[test]
fn binarypath_padding_spaces_clean() {
    let input = "Hi\nWorld!";
    let mut grid = Grid::from_input(input);
    // Grid is 6 wide (length of "World!"), row 0 is "Hi" + 4 padding spaces.
    assert_eq!(grid.width, 6);

    let mut effect = BinaryPathEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "BinaryPathEffect");

    // Padding cells on row 0 must be spaces.
    for x in 2..6 {
        assert_eq!(
            grid.cells[0][x].ch, ' ',
            "padding space at (0,{}) was corrupted",
            x
        );
    }
}

/// After binarypath completes, every cell in the grid must either be its
/// original character (for non-space positions) or a space.
/// This is a broad version of the space-corruption test.
#[test]
fn binarypath_all_cells_correct_after_completion() {
    let input = TEST_INPUT;
    let mut grid = Grid::from_input(input);
    let expected = Grid::from_input(input);
    let mut effect = BinaryPathEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "BinaryPathEffect");

    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = &grid.cells[y][x];
            let orig = &expected.cells[y][x];
            assert_eq!(
                cell.ch, orig.ch,
                "cell ({},{}) has '{}' but expected '{}' after binarypath",
                y, x, cell.ch, orig.ch
            );
        }
    }
}

/// During the travel phase, no cell outside the grid bounds should be
/// written to. This is a smoke test that the effect doesn't panic on
/// inputs with internal spacing.
#[test]
fn binarypath_sparse_input_completes() {
    let input = "  A  \n     \n  B  ";
    let mut grid = Grid::from_input(input);
    let mut effect = BinaryPathEffect::new(&grid);
    let frames = run_to_done(&mut |g| effect.tick(g), &mut grid, "BinaryPathEffect");
    assert!(frames > 0, "effect should take at least 1 frame");
    assert_eq!(grid.cells[0][2].ch, 'A');
    assert_eq!(grid.cells[2][2].ch, 'B');
}

// ── blackhole-specific tests ────────────────────────────────────────────────

/// Space cells must remain spaces after blackhole completes.
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
                grid.cells[y][x].ch, expected.cells[y][x].ch,
                "blackhole: cell ({},{}) has '{}' expected '{}'",
                y, x, grid.cells[y][x].ch, expected.cells[y][x].ch
            );
        }
    }
}

/// Blackhole should handle sparse input with mostly spaces.
#[test]
fn blackhole_sparse_input_completes() {
    let input = "  A  \n     \n  B  ";
    let mut grid = Grid::from_input(input);
    let mut effect = BlackholeEffect::new(&grid);
    let frames = run_to_done(&mut |g| effect.tick(g), &mut grid, "BlackholeEffect");
    assert!(frames > 0);
    assert_eq!(grid.cells[0][2].ch, 'A');
    assert_eq!(grid.cells[2][2].ch, 'B');
}

// ── wormhole-specific tests ─────────────────────────────────────────────────

/// Space cells must remain spaces after wormhole completes.
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
                grid.cells[y][x].ch, expected.cells[y][x].ch,
                "wormhole: cell ({},{}) has '{}' expected '{}'",
                y, x, grid.cells[y][x].ch, expected.cells[y][x].ch
            );
        }
    }
}

/// After the flash phase, all chars should be at their final positions
/// with visible=true.
#[test]
fn wormhole_all_visible_after_completion() {
    let input = TEST_INPUT;
    let mut grid = Grid::from_input(input);
    let mut effect = WormholeEffect::new(&grid);
    run_to_done(&mut |g| effect.tick(g), &mut grid, "WormholeEffect");

    for (y, x) in Grid::from_input(input).char_positions() {
        assert!(
            grid.cells[y][x].visible,
            "wormhole: cell ({},{}) not visible after completion",
            y, x
        );
    }
}
