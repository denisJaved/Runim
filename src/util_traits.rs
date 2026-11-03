use std::time::Duration;

/// Types that can be converted into second durations represented by floats.
///
/// Implemented for [f32], [f64], and [Duration] by default.
///
/// Example:
/// ```
/// use runim::IntoDuration;
/// use std::time::Duration;
///
/// let seconds: f64 = 15.0;
/// let seconds32: f32 = 15.0f32;
/// let duration = Duration::from_secs(15);
///
/// assert_eq!(seconds, IntoDuration::as_secs_f64(&seconds32));
/// assert_eq!(seconds, IntoDuration::as_secs_f64(&duration));
/// ```
pub trait IntoDuration {
    fn as_secs_f32(&self) -> f32;
    fn as_secs_f64(&self) -> f64;

    fn into_secs_f32(self) -> f32 where Self: Sized {
        self.as_secs_f32()
    }
    fn into_secs_f64(self) -> f64 where Self: Sized {
        self.as_secs_f64()
    }
}

impl IntoDuration for Duration {
    fn as_secs_f32(&self) -> f32 {
        self.as_secs_f32()
    }

    fn as_secs_f64(&self) -> f64 {
        self.as_secs_f64()
    }
}

impl IntoDuration for f32 {
    fn as_secs_f32(&self) -> f32 {
        debug_assert!(*self >= 0.0 && self.is_finite(), "time duration must be finite and positive");
        *self
    }

    fn as_secs_f64(&self) -> f64 {
        debug_assert!(*self >= 0.0 && self.is_finite(), "time duration must be finite and positive");
        *self as f64
    }
}

impl IntoDuration for f64 {
    fn as_secs_f32(&self) -> f32 {
        debug_assert!(*self >= 0.0 && self.is_finite(), "time duration must be finite and positive");
        *self as f32
    }

    fn as_secs_f64(&self) -> f64 {
        debug_assert!(*self >= 0.0 && self.is_finite(), "time duration must be finite and positive");
        *self
    }
}