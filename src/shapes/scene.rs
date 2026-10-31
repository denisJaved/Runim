use std::marker::PhantomData;
use crate::animations::{ObjectVisual, SceneState};
use crate::shapes::{Circle, ObjectID, Shape, Square};
use glam::Vec2;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

static SCENE_ID: AtomicU32 = AtomicU32::new(0);
type SharedShape  = Arc<RwLock<Box<dyn Shape>>>;
type SharedVisual = Arc<RwLock<ObjectVisual>>;

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
    pub(crate) id: ObjectID,
    pub(crate) objects: Vec<ObjectDesc>,
    pub(crate) base: SceneState,
    pub(crate) next_id: ObjectID,
    pub(crate) clock: f64
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            id: SCENE_ID.fetch_add(1, Ordering::Release),
            objects: Vec::new(),
            base: SceneState::new(),
            next_id: 0,
            clock: 0f64,
        }
    }

    pub fn with_capacity(capacity: usize) -> Scene {
        Scene {
            id: SCENE_ID.fetch_add(1, Ordering::Release),
            objects: Vec::with_capacity(capacity),
            base: SceneState::with_capacity(capacity),
            next_id: 0,
            clock: 0f64,
        }
    }

    pub fn add<S: Shape + Clone + Default>(&mut self) -> RIObject<S> {
        let id = self.next_id;
        self.next_id += 1;

        let shape: SharedShape = Arc::new(RwLock::new(Box::new(S::default())));
        let visual: SharedVisual = Arc::new(RwLock::new(ObjectVisual::default()));

        self.objects.push(ObjectDesc {
            id,
            shape: Arc::clone(&shape),
            visual: Arc::clone(&visual)
        });

        RIObject {
            id,
            shape,
            visual,
            __marker: PhantomData
        }
    }

    pub fn add_square(&mut self) -> RIObject<Square> {
        self.add()
    }

    pub fn add_circle(&mut self) -> RIObject<Circle> {
        self.add()
    }

    pub fn get<S: Shape + Clone + Default>(&self, id: ObjectID) -> Option<RIObject<S>> {
        let desc = self.objects.iter().find(|i| i.id == id)?;
        // TODO: Verify that RIObject is correct type before returning
        //       desc.shape.read().as_any().downcast_ref::<S>()?;
        Some(RIObject::<S> {
            id,
            shape: Arc::clone(&desc.shape),
            visual: Arc::clone(&desc.visual),
            __marker: PhantomData
        })
    }

    pub fn remove(&mut self, id: ObjectID) -> bool {
        let before = self.objects.len();
        self.objects.retain(|i| i.id == id);
        self.objects.len() < before
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ObjectDesc {
    pub id: ObjectID,
    pub shape: SharedShape,
    pub visual: SharedVisual
}


pub struct RIObject<T: Shape> {
    id: ObjectID,
    shape: SharedShape,
    visual: SharedVisual,
    __marker: PhantomData<T>
}

impl<T: Shape> RIObject<T> {
    pub(crate) fn id(&self) -> ObjectID {
        self.id
    }

    pub fn set_pos(&mut self, x: f32, y: f32) -> &mut RIObject< T> {
        self.visual.write().transform.pos = Vec2::new(x, y);
        self
    }

    pub fn shape(&self) -> MappedRwLockReadGuard<'_, T> {
        RwLockReadGuard::map(self.shape.read(), |boxed| {
            (**boxed).as_any().downcast_ref::<T>().expect("RIObject type mismatch")
        })
    }

    pub fn shape_mut(&self) -> MappedRwLockWriteGuard<'_, T> {
        RwLockWriteGuard::map(self.shape.write(), |boxed| {
            (**boxed).as_any_mut().downcast_mut::<T>().expect("RIObject type mismatch")
        })
    }
}
