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
/// use runim::renderer::debug::DebugRenderer;
///
/// // Standalone
/// let mut scene0 = Scene::new();
///
/// // Attached to a dispatcher
/// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
/// let scene1: ObjectID = dispatcher.new_scene();
///
/// // Move scene to a dispatcher
/// let scene2: ObjectID = dispatcher.add_scene(scene0);
/// ```
pub struct Scene {
    pub(crate) id: ObjectID,
    pub(crate) objects: Vec<ObjectDesc>,
    pub(crate) base: SceneState,
    pub(crate) next_id: ObjectID,
    pub(crate) clock: f64
}

impl Scene {
    /// Creates new standalone scene.
    ///
    /// Standalone scenes can't be used for rendering
    /// without moving them into dispatchers.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let mut scene = Scene::new();
    /// ```
    pub fn new() -> Scene {
        Scene {
            id: SCENE_ID.fetch_add(1, Ordering::Release),
            objects: Vec::new(),
            base: SceneState::new(),
            next_id: 0,
            clock: 0f64,
        }
    }

    /// Creates new standalone scene with specified capacity.
    /// Use this to prevent additional allocations if possible.
    ///
    /// Standalone scenes can't be used for rendering without moving them into dispatchers.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let mut scene = Scene::with_capacity(3);
    /// scene.add_square(); // no resizes
    /// scene.add_square(); // no resizes
    /// scene.add_square(); // no resizes
    ///
    /// scene.add_square(); // causes a resize, because capacity run out
    /// ```
    pub fn with_capacity(capacity: usize) -> Scene {
        Scene {
            id: SCENE_ID.fetch_add(1, Ordering::Release),
            objects: Vec::with_capacity(capacity),
            base: SceneState::with_capacity(capacity),
            next_id: 0,
            clock: 0f64,
        }
    }

    /// Adds existing [Shape] to this scene. It will be synced with
    /// renderer automatically.
    pub(crate) fn add_existing<S: Shape>(&mut self, shape: S) -> RIObject<S> {
        let id = self.next_id;
        self.next_id += 1;

        let shape: SharedShape = Arc::new(RwLock::new(Box::new(shape)));
        let visual: SharedVisual = Arc::new(RwLock::new(ObjectVisual::default()));

        self.objects.push(ObjectDesc {
            id,
            shape: Arc::clone(&shape),
            visual: Arc::clone(&visual)
        });
        self.base.insert(id, Arc::clone(&visual));

        RIObject {
            id,
            shape,
            visual,
            __marker: PhantomData
        }
    }

    /// Creates new instance of <`S`: [Shape] + [Default]> shape using [Default::default]
    /// and adds it to the scene. It will be synced with renderer
    /// automatically.
    ///
    /// Returned [RIObject] is a handle to the actual [Shape], so it may be dropped
    /// without removing the shape. New handle can be acquired with [Scene::get].
    ///
    /// If type annotations are not convenient, following shortcut functions may be used:
    /// - [Scene::add_square] for [Square]
    /// - [Scene::add_circle] for [Circle]
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let mut scene = Scene::new();
    ///
    /// let square: RIObject<Square> = scene.add();
    /// let circle: RIObject<Circle> = scene.add();
    /// ```
    pub fn add<S: Shape + Default>(&mut self) -> RIObject<S> {
        self.add_existing(S::default())
    }

    /// Shortcut to [Scene::add]::<[Square]>()
    #[inline(always)]
    pub fn add_square(&mut self) -> RIObject<Square> {
        self.add()
    }

    /// Shortcut to [Scene::add]::<[Circle]>()
    #[inline(always)]
    pub fn add_circle(&mut self) -> RIObject<Circle> {
        self.add()
    }

    /// Create new handle to object stored in scene by its id.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let mut scene = Scene::new();
    ///
    /// let square_id: ObjectID;
    /// {
    ///     let square = scene.add_square();
    ///     square_id = square.id();
    ///     drop(square); // Handle is destroyed!
    /// }
    /// let square = scene.get::<Square>(square_id).unwrap(); // Get a new one
    /// square.set_pos(10.0, 10.0);
    /// ```
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

    /// Removes [Shape] from scene by its id.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let mut scene = Scene::new();
    ///
    /// // Add a square to the scene and store its id
    /// let square = scene.add_square();
    /// let square_id = square.id();
    ///
    /// assert_ne!(scene.get::<Square>(square_id), None); // It exists!
    /// scene.remove(square_id); // Remove it
    /// assert_eq!(scene.get::<Square>(square_id), None); // Now it doesn't
    /// ```
    pub fn remove(&mut self, id: ObjectID) -> bool {
        let before = self.objects.len();
        self.objects.retain(|i| i.id != id);
        self.objects.len() < before
    }
}

impl Default for Scene {
    // C-COMMON-TRAITS
    fn default() -> Self {
        Scene::new()
    }
}

#[derive(Debug, Clone)]
pub struct ObjectDesc {
    pub id: ObjectID,
    pub shape: SharedShape,
    pub visual: SharedVisual
}

/// Handle to [Shape] in a [Scene]. RIObjects never require mutability,
/// because linked objects are [RwLock]ed.
///
/// Note that dropping the handle, will not destroy the shape!
///
/// [Shape] implementations should add their own functions
/// to this type, so they can be called without [RIObject::shape] calls.
/// You can do that using extension traits (see [Rust RFC-0445](https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html)).
///
/// # Example
/// ```
/// use runim::prelude::*;
///
/// let mut scene = Scene::new();
///
/// let square: RIObject<Square> = scene.add_square();
/// square.set_pos(10.0, 10.0)
///     .set_size(20.0, 20.0);
/// ```
#[derive(Clone, Debug)]
pub struct RIObject<T: Shape> {
    id: ObjectID,
    shape: SharedShape,
    visual: SharedVisual,
    __marker: PhantomData<T>
}

impl<T: Shape> RIObject<T> {
    pub fn id(&self) -> ObjectID {
        self.id
    }

    /// Sets position of linked [Shape] to provided values.
    /// Functions like this may be used builder-like.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let mut scene = Scene::new();
    /// scene.add_square()
    ///     .set_pos(10.0, 10.0);
    /// ```
    pub fn set_pos(&self, x: f32, y: f32) -> &RIObject<T> {
        self.visual.write().transform.pos = Vec2::new(x, y);
        self
    }

    /// Returns immutable [RwLock] guard to linked shape.
    pub fn shape(&self) -> MappedRwLockReadGuard<'_, T> {
        RwLockReadGuard::map(self.shape.read(), |boxed| {
            (**boxed).as_any().downcast_ref::<T>().expect("RIObject type mismatch")
        })
    }

    /// Returns mutable [RwLock] guard to linked shape.
    pub fn shape_mut(&self) -> MappedRwLockWriteGuard<'_, T> {
        RwLockWriteGuard::map(self.shape.write(), |boxed| {
            (**boxed).as_any_mut().downcast_mut::<T>().expect("RIObject type mismatch")
        })
    }
}

impl<T: Shape> PartialEq<Self> for RIObject<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T: Shape> Eq for RIObject<T> {}
