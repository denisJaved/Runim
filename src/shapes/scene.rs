use std::ops::Deref;
use std::sync::Arc;
use crate::shapes::Shape;

type ObjectID = u32;

/// Stores shapes and other elements rendered in a Runim animation.
///
/// Objects inside the scene can be accessed with arena-handle like [RIObject]
///
/// Examples:
/// ```
/// use runim::prelude::*;
///
/// let scene = Scene::new();
/// ```
pub struct Scene {
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
        }
    }
}


pub struct RIObject<T: Shape> {
    id: ObjectID,
    shape: T
}

impl<T: Shape> Deref for RIObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.shape
    }
}

impl<T: Shape> RIObject<T> {

}