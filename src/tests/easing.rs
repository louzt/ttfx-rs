use super::*;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn all_functions_at_zero_return_zero() {
    let fns: &[fn(f64) -> f64] = &[
        linear,
        in_sine,
        out_sine,
        in_out_sine,
        in_quad,
        out_quad,
        in_out_quad,
        in_cubic,
        out_cubic,
        in_out_cubic,
        in_quart,
        out_quart,
        in_out_quart,
        in_quint,
        out_quint,
        in_out_quint,
        in_expo,
        out_expo,
        in_out_expo,
        in_circ,
        out_circ,
        in_out_circ,
        in_back,
        out_back,
        in_out_back,
        in_elastic,
        out_elastic,
        in_bounce,
        out_bounce,
        in_out_bounce,
    ];
    for f in fns {
        assert!(approx_eq(f(0.0), 0.0), "f(0) != 0 for {:?}", f as *const _);
    }
}

#[test]
fn all_functions_at_one_return_one() {
    let fns: &[fn(f64) -> f64] = &[
        linear,
        in_sine,
        out_sine,
        in_out_sine,
        in_quad,
        out_quad,
        in_out_quad,
        in_cubic,
        out_cubic,
        in_out_cubic,
        in_quart,
        out_quart,
        in_out_quart,
        in_quint,
        out_quint,
        in_out_quint,
        in_expo,
        out_expo,
        in_out_expo,
        in_circ,
        out_circ,
        in_out_circ,
        in_elastic,
        out_elastic,
        in_bounce,
        out_bounce,
        in_out_bounce,
    ];
    for f in fns {
        let v = f(1.0);
        assert!(
            (v - 1.0).abs() < 1e-6,
            "f(1) = {} (not 1) for {:?}",
            v,
            f as *const _
        );
    }
}

#[test]
fn linear_is_identity() {
    assert!(approx_eq(linear(0.0), 0.0));
    assert!(approx_eq(linear(0.5), 0.5));
    assert!(approx_eq(linear(1.0), 1.0));
}

#[test]
fn in_out_functions_are_symmetric_at_half() {
    let symmetric: &[fn(f64) -> f64] = &[
        in_out_sine,
        in_out_quad,
        in_out_cubic,
        in_out_quart,
        in_out_quint,
        in_out_expo,
        in_out_circ,
        in_out_bounce,
    ];
    for f in symmetric {
        let v = f(0.5);
        assert!(
            (v - 0.5).abs() < 0.01,
            "in_out f(0.5) = {} for {:?}",
            v,
            f as *const _
        );
    }
}

#[test]
fn bounce_output_always_non_negative() {
    for i in 0..=100 {
        let t = i as f64 / 100.0;
        assert!(out_bounce(t) >= 0.0);
        assert!(in_bounce(t) >= 0.0);
    }
}

#[test]
fn by_name_unknown_falls_back_to_linear() {
    let f = by_name("not_a_real_easing");
    assert!(approx_eq(f(0.5), 0.5));
}

#[test]
fn by_name_known_works() {
    let f = by_name("in_quad");
    assert!(approx_eq(f(0.5), 0.25));
}
