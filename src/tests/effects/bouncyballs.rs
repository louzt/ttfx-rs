use super::*;

fn run_to_completion(eff: &mut BouncyBallsEffect, grid: &mut Grid, cap: usize) -> usize {
    for i in 0..cap {
        if eff.tick(grid) {
            return i + 1;
        }
    }
    panic!("bouncyballs did not complete within {cap} ticks");
}

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = BouncyBallsEffect::new(&g);
    assert_eq!(eff.chars.len(), 4, "expected 4 non-space chars (a,b,c,d)");
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn start_y_within_half_screen_above() {
    let g = Grid::from_input("aaaaa\nbbbbb\nccccc\nddddd\neeeee");
    let height = g.height as f64;
    for _ in 0..50 {
        let eff = BouncyBallsEffect::new(&g);
        for ch in &eff.chars {
            assert!(
                ch.start_y <= 0.0 && ch.start_y > -0.5 * height,
                "start_y {} outside [-0.5*height, 0] (regression of 1.0..1.5 range)",
                ch.start_y
            );
        }
    }
}

#[test]
fn completes_and_renders_final_state() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = BouncyBallsEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    run_to_completion(&mut eff, &mut grid, 4000);
    for (y, row) in grid.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let original = match (y, x) {
                (0, 0) => 'h',
                (0, 1) => 'i',
                (1, 0) => 'y',
                (1, 1) => 'o',
                _ => ' ',
            };
            assert_eq!(cell.ch, original, "final cell ({y},{x}) wrong");
        }
    }
}
