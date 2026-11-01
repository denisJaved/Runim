use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use crate::shapes::{ObjectID, Transform2D};

#[derive(Debug, Clone)]
pub struct ObjectVisual {
    pub transform: Transform2D
}

impl Default for ObjectVisual {
    fn default() -> Self {
        ObjectVisual {
            transform: Transform2D::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct SceneState {
    objects: HashMap<ObjectID, ObjectVisual>
}

impl SceneState {
    pub fn new() -> SceneState {
        SceneState {
            objects: HashMap::new()
        }
    }
    
    pub fn with_capacity(capacity: usize) -> SceneState {
        SceneState {
            objects: HashMap::with_capacity(capacity)
        }
    }
    
    pub fn get(&self, id: ObjectID) -> Option<&ObjectVisual> {
        self.objects.get(&id)
    }
    
    pub fn get_mut(&mut self, id: ObjectID) -> Option<&mut ObjectVisual> {
        self.objects.get_mut(&id)
    }
    
    pub fn insert(&mut self, id: ObjectID, v: ObjectVisual) {
        self.objects.insert(id, v);
    }
}

pub trait Animation: Debug + Send + Sync + 'static {
    fn duration(&self) -> Duration;

    /// Applies animation to provided state. Provided state is always a fresh
    /// clone, so animations do not accumulate.
    ///
    /// `d` (∈ [0, 1]) is progress of this animation. Starting from `0` and finishing with `1`.
    fn apply(&self, state: &mut SceneState, d: f64);
}