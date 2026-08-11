use super::*;

#[test]
fn grid_from_single_line() {
    let g = Grid::from_input("hello");
    assert_eq!(g.width, 5);
    assert_eq!(g.height, 1);
    assert_eq!(g.cells[0][0].ch, 'h');
    assert_eq!(g.cells[0][4].ch, 'o');
}

#[test]
fn grid_from_multiline_pads_to_width() {
    let g = Grid::from_input("ab\nxyz");
    assert_eq!(g.width, 3);
    assert_eq!(g.height, 2);
    assert_eq!(g.cells[0][2].ch, ' ');
}

#[test]
fn grid_cells_start_invisible() {
    let g = Grid::from_input("hi");
    assert!(g.cells[0].iter().all(|c| !c.visible));
}

#[test]
fn grid_set_all_visible() {
    let mut g = Grid::from_input("hi");
    g.set_all_visible();
    assert!(g.all_visible());
}

#[test]
fn grid_set_all_invisible() {
    let mut g = Grid::from_input("hi");
    g.set_all_visible();
    g.set_all_invisible();
    assert!(!g.all_visible());
}

#[test]
fn grid_char_positions_skips_spaces() {
    let g = Grid::from_input("a b");
    let pos = g.char_positions();
    assert_eq!(pos.len(), 2);
    assert!(pos.contains(&(0, 0)));
    assert!(pos.contains(&(0, 2)));
}

#[test]
fn grid_strips_ansi_escapes() {
    let input = "\x1b[32mgreen\x1b[0m";
    let g = Grid::from_input(input);
    assert_eq!(g.width, 5);
    assert_eq!(g.cells[0][0].ch, 'g');
    assert_eq!(g.cells[0][4].ch, 'n');
}

#[test]
fn grid_from_empty_input_is_empty() {
    let g = Grid::from_input("");
    assert_eq!(g.height, 0);
    assert_eq!(g.width, 0);
}
