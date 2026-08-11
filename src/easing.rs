/// Easing functions matching TTE's easing library.
/// All functions take t in [0.0, 1.0] and return a value in [0.0, 1.0].
use std::f64::consts::PI;

pub fn linear(t: f64) -> f64 {
    t
}

pub fn in_sine(t: f64) -> f64 {
    1.0 - ((t * PI) / 2.0).cos()
}

pub fn out_sine(t: f64) -> f64 {
    ((t * PI) / 2.0).sin()
}

pub fn in_out_sine(t: f64) -> f64 {
    -(((PI * t).cos() - 1.0) / 2.0)
}

pub fn in_quad(t: f64) -> f64 {
    t * t
}

pub fn out_quad(t: f64) -> f64 {
    1.0 - (1.0 - t) * (1.0 - t)
}

pub fn in_out_quad(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

pub fn in_cubic(t: f64) -> f64 {
    t * t * t
}

pub fn out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

pub fn in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

pub fn in_quart(t: f64) -> f64 {
    t * t * t * t
}

pub fn out_quart(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(4)
}

pub fn in_out_quart(t: f64) -> f64 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
    }
}

pub fn in_quint(t: f64) -> f64 {
    t * t * t * t * t
}

pub fn out_quint(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(5)
}

pub fn in_out_quint(t: f64) -> f64 {
    if t < 0.5 {
        16.0 * t.powi(5)
    } else {
        1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
    }
}

pub fn in_expo(t: f64) -> f64 {
    if t == 0.0 {
        0.0
    } else {
        (2.0_f64).powf(10.0 * t - 10.0)
    }
}

pub fn out_expo(t: f64) -> f64 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - (2.0_f64).powf(-10.0 * t)
    }
}

pub fn in_out_expo(t: f64) -> f64 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        (2.0_f64).powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - (2.0_f64).powf(-20.0 * t + 10.0)) / 2.0
    }
}

pub fn in_circ(t: f64) -> f64 {
    1.0 - (1.0 - t * t).sqrt()
}

pub fn out_circ(t: f64) -> f64 {
    (1.0 - (t - 1.0).powi(2)).sqrt()
}

pub fn in_out_circ(t: f64) -> f64 {
    if t < 0.5 {
        (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
    } else {
        ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
    }
}

pub fn in_back(t: f64) -> f64 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    c3 * t * t * t - c1 * t * t
}

pub fn out_back(t: f64) -> f64 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

pub fn in_out_back(t: f64) -> f64 {
    let c1 = 1.70158;
    let c2 = c1 * 1.525;
    if t < 0.5 {
        ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
    } else {
        ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0) / 2.0
    }
}

pub fn in_elastic(t: f64) -> f64 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c4 = (2.0 * PI) / 3.0;
        -(2.0_f64).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * c4).sin()
    }
}

pub fn out_elastic(t: f64) -> f64 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c4 = (2.0 * PI) / 3.0;
        (2.0_f64).powf(-10.0 * t) * ((10.0 * t - 0.75) * c4).sin() + 1.0
    }
}

pub fn in_bounce(t: f64) -> f64 {
    1.0 - out_bounce(1.0 - t)
}

pub fn out_bounce(t: f64) -> f64 {
    let n1 = 7.5625;
    let d1 = 2.75;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

pub fn in_out_bounce(t: f64) -> f64 {
    if t < 0.5 {
        (1.0 - out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

/// Get an easing function by name (used for config)
pub fn by_name(name: &str) -> fn(f64) -> f64 {
    match name {
        "linear" => linear,
        "in_sine" => in_sine,
        "out_sine" => out_sine,
        "in_out_sine" => in_out_sine,
        "in_quad" => in_quad,
        "out_quad" => out_quad,
        "in_out_quad" => in_out_quad,
        "in_cubic" => in_cubic,
        "out_cubic" => out_cubic,
        "in_out_cubic" => in_out_cubic,
        "in_quart" => in_quart,
        "out_quart" => out_quart,
        "in_out_quart" => in_out_quart,
        "in_quint" => in_quint,
        "out_quint" => out_quint,
        "in_out_quint" => in_out_quint,
        "in_expo" => in_expo,
        "out_expo" => out_expo,
        "in_out_expo" => in_out_expo,
        "in_circ" => in_circ,
        "out_circ" => out_circ,
        "in_out_circ" => in_out_circ,
        "in_back" => in_back,
        "out_back" => out_back,
        "in_out_back" => in_out_back,
        "in_elastic" => in_elastic,
        "out_elastic" => out_elastic,
        "in_bounce" => in_bounce,
        "out_bounce" => out_bounce,
        "in_out_bounce" => in_out_bounce,
        _ => linear,
    }
}

#[cfg(test)]
#[path = "tests/easing.rs"]
mod tests;
