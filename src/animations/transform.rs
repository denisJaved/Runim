use glam::Vec2;
use crate::{Animation, IntoDuration};
use crate::animations::{AffectedShapes, Animation, DynamicShape, IntoAffectedShapes};

/// Moves object from one position to another.
#[derive(Animation, Debug, Clone)]
pub struct MoveAnimation {
    shapes: AffectedShapes,
    duration: f64,
    pos1: Vec2,
    pos2: Vec2
}

impl MoveAnimation {
    pub fn new<S, D>(shapes: S, duration: D, pos1: Vec2, pos2: Vec2) -> MoveAnimation
    where S: IntoAffectedShapes, D: IntoDuration {
        MoveAnimation {
            shapes: shapes.into_affected_shape(),
            duration: duration.as_secs_f64(),
            pos1,
            pos2,
        }
    }
}

impl Animation for MoveAnimation {
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        state.transform_mut().pos = self.pos1.lerp(self.pos2, d as f32);
    }
}