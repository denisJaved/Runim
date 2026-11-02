use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use log::warn;
use parking_lot::RwLock;
use crate::prelude::{ObjectID, Scene, Shape};

#[repr(transparent)]
pub struct DynamicShape<'a> {
    state: &'a mut Box<dyn Shape>
}

impl DynamicShape<'_> {
    pub fn as_ref<T: Shape>(&self) -> Option<&T> {
        self.state.as_any().downcast_ref::<T>()
    }

    pub fn as_mut<T: Shape>(&mut self) -> Option<&mut T> {
        self.state.as_any_mut().downcast_mut::<T>()
    }
}

pub enum AffectedShapes {
    Single(ObjectID),
    Vector(Vec<ObjectID>)
}

impl AffectedShapes {
    pub(crate) fn apply_all(&self, scene: &mut Scene, animation: &Arc<dyn Animation>, d: f64) {
        match self {
            AffectedShapes::Single(s) => {
                Self::animate(scene.get_raw(*s), animation, d);
            }
            AffectedShapes::Vector(s) => {
                for x in s {
                    Self::animate(scene.get_raw(*x), animation, d);
                }
            }
        }
    }

    pub(crate) fn animate(shape: Option<Arc<RwLock<Box<dyn Shape>>>>, animation: &Arc<dyn Animation>, d: f64) {
        match shape {
            Some(s) => {
                let mut guard = s.write();
                let mut untyped = DynamicShape {
                    state: &mut guard,
                };
                animation.apply(&mut untyped, d);
                drop(guard);
            }
            None => {
                warn!("Tried to animate unknown shape. Use debugger for details")
            }
        }
    }
}

pub trait Animation: Debug + Send + Sync + 'static {
    fn duration(&self) -> Duration;

    /// Returns one or multiple [ObjectID]s of shapes affected
    /// by this animation.
    ///
    /// [Animation::apply] will be called for each one.
    fn affected_shapes(&self) -> &AffectedShapes;

    /// Applies animation to provided state. Provided state is always a fresh
    /// clone, so animations do not accumulate.
    ///
    /// `d` (∈ [0, 1]) is progress of this animation. Starting from `0` and finishing with `1`.
    fn apply(&self, state: &mut DynamicShape, d: f64);
}