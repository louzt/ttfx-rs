use super::*;
use std::collections::HashSet;

#[test]
fn link_order_visits_disconnected_letters() {
    let g = Grid::from_input("X X X");
    let eff = BurnEffect::new(&g);
    let visited: HashSet<(usize, usize)> = eff.char_link_order.iter().copied().collect();
    for (y, row) in g.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if cell.ch != ' ' {
                assert!(
                    visited.contains(&(y, x)),
                    "non-space cell ({y},{x}='{}') missing from spanning tree",
                    cell.ch
                );
            }
        }
    }
}

#[test]
fn space_cells_never_burn() {
    let g = Grid::from_input("A A");
    let mut eff = BurnEffect::new(&g);
    let mut grid = Grid::from_input("A A");
    for _ in 0..2000 {
        if eff.tick(&mut grid) {
            break;
        }
        for row in &eff.chars {
            for ch in row {
                if ch.original_ch == ' ' {
                    assert_eq!(
                        ch.phase,
                        BurnPhase::Waiting,
                        "space cell ({},{}) entered burn phase",
                        ch.y,
                        ch.x
                    );
                }
            }
        }
    }
}

#[test]
fn final_color_uses_text_bounds() {
    let mut tall = String::new();
    tall.push_str("AAAA\n");
    for _ in 0..5 {
        tall.push_str("    \n");
    }
    let g = Grid::from_input(&tall);
    let eff = BurnEffect::new(&g);
    let row0_color = eff.chars[0][0].final_color;
    let blue = Rgb::from_hex("00c3ff");
    let yellow = Rgb::from_hex("ffff1c");
    let dist_blue = (row0_color.r as i32 - blue.r as i32).pow(2)
        + (row0_color.g as i32 - blue.g as i32).pow(2)
        + (row0_color.b as i32 - blue.b as i32).pow(2);
    let dist_yellow = (row0_color.r as i32 - yellow.r as i32).pow(2)
        + (row0_color.g as i32 - yellow.g as i32).pow(2)
        + (row0_color.b as i32 - yellow.b as i32).pow(2);
    assert!(
        dist_yellow < dist_blue,
        "the only text row (top) should be near yellow stop, got {:?}",
        row0_color
    );
}

#[test]
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = BurnEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..4000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("burn did not complete");
}
