use runim::animations::IntoAffectedShapes;
use runim::animations::{AffectedShapes, Animation, DynamicShape};
use runim::prelude::Color;
use runim::shapes::ColorHSV;
use runim::{Animation, IntoDuration};

/// Interpolates between primary colors of [ColoredShape]s.
/// 
/// Interpolation is done directly on RGB channels of the colors.
/// This is fast, but may look ugly. Use [HSVColorTween] if this one doesn't look right.
/// 
/// # Example
/// ```
/// use runim::prelude::*;
/// use runim::renderer::debug::DebugRenderer;
///
/// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
/// let scene = dispatcher.new_scene();
/// 
/// let square = dispatcher.scene(&scene)
///     .add_rectangle()
///     .id();
/// 
/// // Make square transition between red and blue in 3 seconds
/// dispatcher.play(RGBColorTween::new(
///     square, 3.0, Color::RED, Color::BLUE
/// ));
/// ```
#[derive(Animation, Debug, Clone)]
pub struct RGBColorTween {
    shapes: AffectedShapes,
    duration: f64,
    a: Color,
    b: Color
}
impl RGBColorTween {
    /// Creates new [RGBColorTween], see type docs for more info.
    pub fn new<S, D, C>(shapes: S, duration: D, a: C, b: C) -> RGBColorTween
    where S: IntoAffectedShapes, D: IntoDuration, C: Into<Color> {
        RGBColorTween {
            shapes: shapes.into_affected_shape(),
            duration: duration.as_secs_f64(),
            a: a.into(),
            b: b.into(),
        }
    }
}
impl Animation for RGBColorTween {
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        if let Some(shape) = state.as_colored_mut() {
            let color = Color::interpolate(&self.a, &self.b, d as f32);
            shape.set_color(color);
        }
    }
}


/// Interpolates between primary colors of [ColoredShape]s.
///
/// Unlike in [RGBColorTween] interpolation happens in HSV color space.
/// It is less performant, but looks nicer.
///
/// # Example
/// ```
/// use runim::prelude::*;
/// use runim::renderer::debug::DebugRenderer;
///
/// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
/// let scene = dispatcher.new_scene();
///
/// let square = dispatcher.scene(&scene)
///     .add_rectangle()
///     .id();
///
/// // Make square transition between red and blue in 3 seconds
/// dispatcher.play(HSVColorTween::new(
///     square, 3.0, Color::RED, Color::BLUE
/// ));
/// ```
#[derive(Animation, Debug, Clone)]
pub struct HSVColorTween {
    shapes: AffectedShapes,
    duration: f64,
    a: ColorHSV,
    b: ColorHSV
}
impl HSVColorTween {
    pub fn new<S, D, C>(shapes: S, duration: D, a: C, b: C) -> HSVColorTween
    where S: IntoAffectedShapes, D: IntoDuration, C: Into<ColorHSV> {
        HSVColorTween {
            shapes: shapes.into_affected_shape(),
            duration: duration.as_secs_f64(),
            a: a.into(),
            b: b.into(),
        }
    }
}
impl Animation for HSVColorTween {
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        if let Some(shape) = state.as_colored_mut() {
            let color = ColorHSV::interpolate(&self.a, &self.b, d as f32);
            shape.set_color(color.into_rgb());
        }
    }
}