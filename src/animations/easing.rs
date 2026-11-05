use crate::animations::EasingFn;

pub fn linear(v: f64) -> f64 {
    v
}

pub fn reversed(v: f64) -> f64 {
    1.0 - v
}

#[cfg(feature = "cubic_bezier_presets")]
mod presets {
    use super::CubicBezier;

    pub const EASE: CubicBezier = CubicBezier::new(0.25, 0.1, 0.25, 1.0);
    pub const EASE_IN: CubicBezier = CubicBezier::new(0.42, 0.0, 1.0, 1.0);
    pub const EASE_OUT: CubicBezier = CubicBezier::new(0.0, 0.0, 0.58, 1.0);
    pub const EASE_IN_OUT: CubicBezier = CubicBezier::new(0.42, 0.0, 0.58, 1.0);
    // TODO: More presets
}

#[cfg(feature = "cubic_bezier_presets")]
pub use presets::*;

/// Creates new owned easing function that uses a cubic [Bézier curve](https://developer.mozilla.org/en-US/docs/Glossary/Bezier_curve).
///
/// This functions works like [CSS cuber-bezier() function](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/easing-function/cubic-bezier).
/// Check out its docs for more information.
pub fn cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64) -> Box<dyn EasingFn> {
    Box::new(CubicBezier::new(x1, y1, x2, y2))
}

pub struct CubicBezier {
    // TODO: Test! This is vibecoded

    // polynomial coefficients for x(t) = ax*t^3 + bx*t^2 + cx*t
    ax: f64,
    bx: f64,
    cx: f64,
    // polynomial coefficients for y(t) = ay*t^3 + by*t^2 + cy*t
    ay: f64,
    by: f64,
    cy: f64,
}

impl CubicBezier {
    // TODO: Test! This is vibecoded

    pub const fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> CubicBezier {
        debug_assert!(0.0 <= x1 && x1 <= 1.0 && x1.is_finite(), "x1 must be contained by [0, 1]");
        debug_assert!(0.0 <= x2 && x2 <= 1.0 && x2.is_finite(), "x2 must be contained by [0, 1]");

        let cx = 3.0 * x1;
        let bx = 3.0 * (x2 - x1) - cx;
        let ax = 1.0 - cx - bx;

        let cy = 3.0 * y1;
        let by = 3.0 * (y2 - y1) - cy;
        let ay = 1.0 - cy - by;

        CubicBezier {
            ax, bx, cx, ay, by, cy
        }
    }

    #[inline]
    fn sample_x(&self, t: f64) -> f64 {
        ((self.ax * t + self.bx) * t + self.cx) * t
    }

    #[inline]
    fn sample_y(&self, t: f64) -> f64 {
        ((self.ay * t + self.by) * t + self.cy) * t
    }

    #[inline]
    fn sample_dx(&self, t: f64) -> f64 {
        (3.0 * self.ax * t + 2.0 * self.bx) * t + self.cx
    }

    /// Solve x(t) = target for t in [0, 1].
    fn solve_t(&self, target: f64) -> f64 {
        const EPS: f64 = 1e-6;

        // Newton-Raphson
        let mut t = target;
        for _ in 0..8 {
            let err = self.sample_x(t) - target;
            if err.abs() < EPS {
                return t;
            }
            let d = self.sample_dx(t);
            if d.abs() < EPS {
                break;
            }
            t -= err / d;
            t = t.clamp(0.0, 1.0);
        }

        // Bisection fallback (x is monotonic since x1, x2 ∈ [0,1])
        let mut lo = 0.0_f64;
        let mut hi = 1.0_f64;
        t = target;
        while hi - lo > 1e-7 {
            let x = self.sample_x(t);
            if (x - target).abs() < EPS {
                return t;
            }
            if target > x {
                lo = t;
            } else {
                hi = t;
            }
            t = 0.5 * (hi + lo);
        }
        t
    }
}

impl EasingFn for CubicBezier {
    fn apply(&self, v: f64) -> f64 {
        match v {
            ..=0.0 => 0.0,
            1.0.. => 1.0,
            _ => self.sample_y(self.solve_t(v))
        }
    }
}