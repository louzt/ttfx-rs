use super::*;

#[test]
fn skips_space_cells() {
    let g = Grid::from_input("a b\nc d");
    let eff = DecryptEffect::new(&g);
    assert_eq!(eff.chars.len(), 4);
    for ch in &eff.chars {
        assert_ne!(ch.original_ch, ' ');
    }
}

#[test]
fn encrypted_pool_includes_box_drawing_and_misc() {
    let pool = build_encrypted_symbols();
    assert!(
        pool.contains(&'─'),
        "missing box-drawing ─ (regression of small pool)"
    );
    assert!(pool.contains(&'╋'), "missing box-drawing ╋");
    assert!(pool.len() > 400, "pool too small ({} symbols)", pool.len());
}

#[test]
fn all_chars_synchronize_into_decrypting_phase() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = DecryptEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    let mut saw_decrypting_with_all_in_fast = false;
    for _ in 0..2_000 {
        if eff.tick(&mut grid) {
            break;
        }
        if eff.phase == EffectPhase::Decrypting {
            let in_fast = eff
                .chars
                .iter()
                .filter(|c| c.phase == CharPhase::FastDecrypt)
                .count();
            if in_fast == eff.chars.len() {
                saw_decrypting_with_all_in_fast = true;
                break;
            }
        }
    }
    assert!(
        saw_decrypting_with_all_in_fast,
        "regression: all chars must enter FastDecrypt simultaneously when typing phase ends"
    );
}

#[test]
fn cells_reset_to_original_each_tick() {
    let g = Grid::from_input("ABC\nDEF");
    let mut eff = DecryptEffect::new(&g);
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
    let mut eff = DecryptEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..20_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("decrypt did not complete");
}
