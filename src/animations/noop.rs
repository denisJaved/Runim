use runim_derive::Animation;
use runim::animations::{AffectedShapes, Animation, DynamicShape, IntoAffectedShapes};
use runim::IntoDuration;

/// No-op animation.
/// 
/// This animation simply does not do anything.
/// It may be used as placeholder to wait between animations
#[derive(Animation, Debug, Clone)]
pub struct NoOpAnimation {
    shapes: AffectedShapes,
    duration: f64,
}

impl NoOpAnimation {
    pub fn new<S, D>(shapes: S, duration: D) -> NoOpAnimation
    where S: IntoAffectedShapes, D: IntoDuration {
        NoOpAnimation {
            shapes: shapes.into_affected_shape(),
            duration: duration.as_secs_f64()
        }
    }
    
    pub fn new_shapeless<D>(duration: D) -> NoOpAnimation
    where D: IntoDuration {
        NoOpAnimation {
            shapes: AffectedShapes::Vector(vec![]),
            duration: duration.as_secs_f64()
        }
    }
}

impl Animation for NoOpAnimation {
    fn apply(&self, _state: &mut DynamicShape, _d: f64) {
        // NO OP
    }
}