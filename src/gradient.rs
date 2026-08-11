/// Color gradient system matching TTE's gradient engine.
/// Supports multi-stop linear interpolation, coordinate mapping, and direction-based gradients.
use crossterm::style::Color;

/// An RGB color for interpolation
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Self { r, g, b }
    }

    pub fn to_crossterm(self) -> Color {
        Color::Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }

    /// Adjust HSL lightness by a factor (0.0 = black, 1.0 = same, 2.0 = double).
    /// Matches TTE's `Animation.adjust_color_brightness`: converts to HSL,
    /// scales L, converts back, preserving hue and saturation.
    pub fn adjust_brightness(self, factor: f64) -> Rgb {
        let r_n = self.r as f64 / 255.0;
        let g_n = self.g as f64 / 255.0;
        let b_n = self.b as f64 / 255.0;

        let max_v = r_n.max(g_n).max(b_n);
        let min_v = r_n.min(g_n).min(b_n);
        let mut lightness = (max_v + min_v) / 2.0;

        let (hue, saturation) = if max_v == min_v {
            (0.0, 0.0)
        } else {
            let diff = max_v - min_v;
            let saturation = if lightness > 0.5 {
                diff / (2.0 - max_v - min_v)
            } else {
                diff / (max_v + min_v)
            };
            let mut h = if (max_v - r_n).abs() < f64::EPSILON {
                (g_n - b_n) / diff + (if g_n < b_n { 6.0 } else { 0.0 })
            } else if (max_v - g_n).abs() < f64::EPSILON {
                (b_n - r_n) / diff + 2.0
            } else {
                (r_n - g_n) / diff + 4.0
            };
            h /= 6.0;
            (h, saturation)
        };

        lightness = (lightness * factor).clamp(0.0, 1.0);

        let (red, green, blue) = if saturation == 0.0 {
            (lightness, lightness, lightness)
        } else {
            let q = if lightness < 0.5 {
                lightness * (1.0 + saturation)
            } else {
                lightness + saturation - lightness * saturation
            };
            let p = 2.0 * lightness - q;
            (
                hue_to_rgb(p, q, hue + 1.0 / 3.0),
                hue_to_rgb(p, q, hue),
                hue_to_rgb(p, q, hue - 1.0 / 3.0),
            )
        };

        Rgb {
            r: (red * 255.0) as u8,
            g: (green * 255.0) as u8,
            b: (blue * 255.0) as u8,
        }
    }

    /// Lerp between two colors
    pub fn lerp(a: Rgb, b: Rgb, t: f64) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        Rgb {
            r: (a.r as f64 + (b.r as f64 - a.r as f64) * t) as u8,
            g: (a.g as f64 + (b.g as f64 - a.g as f64) * t) as u8,
            b: (a.b as f64 + (b.b as f64 - a.b as f64) * t) as u8,
        }
    }
}

fn hue_to_rgb(p: f64, q: f64, h: f64) -> f64 {
    let h = if h < 0.0 {
        h + 1.0
    } else if h > 1.0 {
        h - 1.0
    } else {
        h
    };
    if h < 1.0 / 6.0 {
        p + (q - p) * 6.0 * h
    } else if h < 1.0 / 2.0 {
        q
    } else if h < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - h) * 6.0
    } else {
        p
    }
}

/// Direction for coordinate-mapped gradients
#[derive(Clone, Copy, Debug)]
pub enum GradientDirection {
    Vertical,
    Horizontal,
    Diagonal,
    Radial,
}

/// A multi-stop color gradient
#[derive(Clone, Debug)]
pub struct Gradient {
    pub stops: Vec<Rgb>,
    pub steps: usize,
    spectrum: Vec<Rgb>,
}

impl Gradient {
    /// Create a gradient with the given color stops and number of interpolation steps per segment
    pub fn new(stops: &[Rgb], steps: usize) -> Self {
        let steps = steps.max(1);
        let mut spectrum = Vec::new();

        if stops.is_empty() {
            spectrum.push(Rgb::new(255, 255, 255));
        } else if stops.len() == 1 {
            spectrum.push(stops[0]);
        } else {
            for i in 0..stops.len() - 1 {
                for s in 0..steps {
                    let t = s as f64 / steps as f64;
                    spectrum.push(Rgb::lerp(stops[i], stops[i + 1], t));
                }
            }
            spectrum.push(*stops.last().unwrap());
        }

        Self {
            stops: stops.to_vec(),
            steps,
            spectrum,
        }
    }

    /// Get the color at position t in [0.0, 1.0]
    pub fn at(&self, t: f64) -> Rgb {
        if self.spectrum.is_empty() {
            return Rgb::new(255, 255, 255);
        }
        let t = t.clamp(0.0, 1.0);
        let idx = (t * (self.spectrum.len() - 1) as f64) as usize;
        self.spectrum[idx.min(self.spectrum.len() - 1)]
    }

    /// Get full spectrum
    pub fn spectrum(&self) -> &[Rgb] {
        &self.spectrum
    }

    /// Number of colors in the spectrum
    pub fn len(&self) -> usize {
        self.spectrum.len()
    }

    /// Get color by index
    pub fn get(&self, idx: usize) -> Rgb {
        self.spectrum[idx.min(self.spectrum.len() - 1)]
    }

    /// Build a coordinate-to-color mapping for the given bounds
    pub fn color_at_coord(
        &self,
        row: usize,
        col: usize,
        max_row: usize,
        max_col: usize,
        direction: GradientDirection,
    ) -> Rgb {
        let t = match direction {
            GradientDirection::Vertical => {
                if max_row == 0 {
                    0.0
                } else {
                    1.0 - row as f64 / max_row as f64
                }
            }
            GradientDirection::Horizontal => {
                if max_col == 0 {
                    0.0
                } else {
                    col as f64 / max_col as f64
                }
            }
            GradientDirection::Diagonal => {
                // TTE: `((row*2) + col) / ((max_row*2) + max_col)` in bottom-up
                // coords (row 0 = visual bottom, max_row = visual top). The
                // 2× weights the row dimension so the diagonal looks
                // approximately equal-angled despite the terminal cell aspect.
                // In rtte top-down (row 0 = visual top), flip the row to keep
                // the visual orientation: bottom-left = first stop, top-right
                // = last stop.
                let denom = 2 * max_row + max_col;
                if denom == 0 {
                    0.0
                } else {
                    let flipped_row = max_row.saturating_sub(row);
                    (2 * flipped_row + col) as f64 / denom as f64
                }
            }
            GradientDirection::Radial => {
                let cr = max_row as f64 / 2.0;
                let cc = max_col as f64 / 2.0;
                let max_dist = (cr * cr + cc * cc).sqrt();
                if max_dist == 0.0 {
                    0.0
                } else {
                    let dr = row as f64 - cr;
                    let dc = col as f64 - cc;
                    (dr * dr + dc * dc).sqrt() / max_dist
                }
            }
        };
        self.at(t)
    }
}

#[cfg(test)]
#[path = "tests/gradient.rs"]
mod tests;

/// Predefined color palettes matching TTE defaults
pub mod palettes {
    use super::Rgb;

    pub fn matrix_rain() -> Vec<Rgb> {
        vec![Rgb::from_hex("92be92"), Rgb::from_hex("185318")]
    }
    pub fn matrix_highlight() -> Rgb {
        Rgb::from_hex("dbffdb")
    }
    pub fn fire() -> Vec<Rgb> {
        vec![
            Rgb::new(255, 255, 255),
            Rgb::new(255, 255, 0),
            Rgb::new(255, 165, 0),
            Rgb::new(200, 50, 0),
            Rgb::new(40, 40, 40),
        ]
    }
    pub fn decrypt_cipher() -> Vec<Rgb> {
        vec![
            Rgb::from_hex("008000"),
            Rgb::from_hex("00cb00"),
            Rgb::from_hex("00ff00"),
        ]
    }
    pub fn decrypt_final() -> Rgb {
        Rgb::from_hex("eda000")
    }
    pub fn default_final() -> Vec<Rgb> {
        vec![
            Rgb::from_hex("8A008A"),
            Rgb::from_hex("00D1FF"),
            Rgb::from_hex("FFFFFF"),
        ]
    }
    pub fn purple_cyan_white() -> Vec<Rgb> {
        vec![
            Rgb::from_hex("8A008A"),
            Rgb::from_hex("00D1FF"),
            Rgb::from_hex("FFFFFF"),
        ]
    }
    pub fn rainbow() -> Vec<Rgb> {
        vec![
            Rgb::new(255, 0, 0),
            Rgb::new(255, 165, 0),
            Rgb::new(255, 255, 0),
            Rgb::new(0, 255, 0),
            Rgb::new(0, 0, 255),
            Rgb::new(75, 0, 130),
            Rgb::new(238, 130, 238),
        ]
    }
    pub fn star_colors() -> Vec<Rgb> {
        vec![
            Rgb::from_hex("ffcc0d"),
            Rgb::from_hex("ff7326"),
            Rgb::from_hex("ff194d"),
            Rgb::from_hex("bf2669"),
            Rgb::from_hex("702a8c"),
            Rgb::new(255, 255, 255),
        ]
    }
    pub fn lightning() -> Rgb {
        Rgb::from_hex("68A3E8")
    }
    pub fn error_red() -> Rgb {
        Rgb::from_hex("e74c3c")
    }
    pub fn correct_green() -> Rgb {
        Rgb::from_hex("45bf55")
    }
}
