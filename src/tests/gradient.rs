use super::*;

#[test]
fn rgb_from_hex() {
    let c = Rgb::from_hex("ff8000");
    assert_eq!(c.r, 255);
    assert_eq!(c.g, 128);
    assert_eq!(c.b, 0);
}

#[test]
fn rgb_from_hex_with_hash() {
    let c = Rgb::from_hex("#00ff00");
    assert_eq!(c.r, 0);
    assert_eq!(c.g, 255);
    assert_eq!(c.b, 0);
}

#[test]
fn rgb_lerp_midpoint() {
    let a = Rgb::new(0, 0, 0);
    let b = Rgb::new(200, 100, 50);
    let mid = Rgb::lerp(a, b, 0.5);
    assert_eq!(mid.r, 100);
    assert_eq!(mid.g, 50);
    assert_eq!(mid.b, 25);
}

#[test]
fn rgb_lerp_clamped() {
    let a = Rgb::new(10, 10, 10);
    let b = Rgb::new(20, 20, 20);
    assert_eq!(Rgb::lerp(a, b, -1.0), a);
    assert_eq!(Rgb::lerp(a, b, 2.0), b);
}

#[test]
fn rgb_adjust_brightness() {
    let c = Rgb::new(100, 100, 100);
    let doubled = c.adjust_brightness(2.0);
    assert_eq!(doubled.r, 200);
    let zeroed = c.adjust_brightness(0.0);
    assert_eq!(zeroed.r, 0);
    let capped = c.adjust_brightness(10.0);
    assert_eq!(capped.r, 255);
}

#[test]
fn gradient_at_endpoints() {
    let g = Gradient::new(&[Rgb::new(0, 0, 0), Rgb::new(255, 255, 255)], 10);
    let start = g.at(0.0);
    let end = g.at(1.0);
    assert_eq!(start.r, 0);
    assert_eq!(end.r, 255);
}

#[test]
fn gradient_single_stop() {
    let g = Gradient::new(&[Rgb::new(42, 42, 42)], 5);
    assert_eq!(g.len(), 1);
    assert_eq!(g.at(0.5).r, 42);
}

#[test]
fn gradient_color_at_coord_vertical() {
    let g = Gradient::new(&[Rgb::new(0, 0, 0), Rgb::new(255, 0, 0)], 10);
    let top = g.color_at_coord(0, 0, 4, 4, GradientDirection::Vertical);
    let bot = g.color_at_coord(3, 0, 4, 4, GradientDirection::Vertical);
    assert!(top.r > bot.r, "top should be brighter in vertical gradient");
}

#[test]
fn gradient_color_at_coord_horizontal() {
    let g = Gradient::new(&[Rgb::new(0, 0, 0), Rgb::new(255, 0, 0)], 10);
    let left = g.color_at_coord(0, 0, 4, 4, GradientDirection::Horizontal);
    let right = g.color_at_coord(0, 3, 4, 4, GradientDirection::Horizontal);
    assert!(
        right.r > left.r,
        "right should be brighter in horizontal gradient"
    );
}

#[test]
fn gradient_color_at_coord_diagonal_runs_bottom_left_to_top_right() {
    // Regression: TTE's diagonal goes visual bottom-left (first stop) to
    // visual top-right (last stop). In rtte top-down coords that's
    // (max_row, 0) → (0, max_col). rtte previously used top-left → bottom-
    // right, which inverted the gradient.
    let g = Gradient::new(&[Rgb::new(0, 0, 0), Rgb::new(255, 0, 0)], 10);
    let bottom_left = g.color_at_coord(4, 0, 4, 4, GradientDirection::Diagonal);
    let top_right = g.color_at_coord(0, 4, 4, 4, GradientDirection::Diagonal);
    assert_eq!(bottom_left.r, 0, "bottom-left must map to first stop");
    assert_eq!(top_right.r, 255, "top-right must map to last stop");
}
