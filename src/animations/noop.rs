use runim_derive::Animation;
use runim::animations::{AffectedShapes, Animation, DynamicShape, IntoAffectedShapes};
use runim::IntoDuration;

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
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        // NO OP
    }
}