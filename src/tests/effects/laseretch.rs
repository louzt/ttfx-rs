use super::*;

#[test]
fn etch_delay_is_one() {
    let g = Grid::from_input("hi");
    let eff = LaserEtchEffect::new(&g);
    assert_eq!(
        eff.etch_delay, 1,
        "regression: etch_delay must match TTE default of 1"
    );
}

#[test]
fn cool_gradient_blends_into_final_color() {
    // TTE's cool_gradient is `Gradient(yellow, orange, final, steps=8)` —
    // 17 colors. Plus the leading '^' frame = 18 scene frames.
    let g = Grid::from_input("AAAAA\nBBBBB");
    let eff = LaserEtchEffect::new(&g);
    let ch = &eff.chars[0];
    assert_eq!(
        ch.scene.len(),
        18,
        "scene must have ^ + 17 cool/blend frames"
    );
    let last = ch.scene.last().unwrap();
    assert_eq!(
        last.color, ch.final_color,
        "regression: last cool frame must be the final color (cool_gradient appended it as a stop)"
    );
}

#[test]
fn frame_durations_match_tte() {
    let g = Grid::from_input("X");
    let eff = LaserEtchEffect::new(&g);
    for f in &eff.chars[0].scene {
        assert_eq!(
            f.duration, 3,
            "regression: per-frame duration must be 3 (was 6 from a wrong fps assumption)"
        );
    }
}

#[test]
fn laser_gradient_loops_seamlessly() {
    // Mirrored spectrum so cycle has no seam.
    let g = Grid::from_input("X");
    let eff = LaserEtchEffect::new(&g);
    let last = eff.laser_gradient.last().unwrap();
    let first = eff.laser_gradient.first().unwrap();
    let dr = first.r as i32 - last.r as i32;
    let dg = first.g as i32 - last.g as i32;
    let db = first.b as i32 - last.b as i32;
    let seam_jump = ((dr * dr + dg * dg + db * db) as f64).sqrt();
    assert!(
        seam_jump < 80.0,
        "regression: laser gradient must wrap smoothly, got seam jump {}",
        seam_jump
    );
}

#[test]
fn spark_speed_scales_with_distance() {
    let g = Grid::from_input("XXXXXXXXXX\nXXXXXXXXXX\nXXXXXXXXXX\nXXXXXXXXXX\nXXXXXXXXXX");
    let mut eff = LaserEtchEffect::new(&g);
    eff.emit_spark(0, 5);
    let s = eff.sparks.iter().find(|s| s.active).expect("spark active");
    let dy = s.end_y - s.start_y;
    let dx = s.end_x - s.start_x;
    let expected = 0.3 / (dx * dx + (2.0 * dy).powi(2)).sqrt().max(1.0);
    assert!(
        (s.speed - expected).abs() < 1e-9,
        "regression: spark speed must be 0.3/aspect_dist (was hard-coded 0.015)"
    );
}

#[test]
fn completes() {
    let g = Grid::from_input("hi\nyo");
    let mut eff = LaserEtchEffect::new(&g);
    let mut grid = Grid::from_input("hi\nyo");
    for _ in 0..20_000 {
        if eff.tick(&mut grid) {
            return;
        }
    }
    panic!("laseretch did not complete");
}
