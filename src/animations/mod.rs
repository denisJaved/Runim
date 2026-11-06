use std::fmt::Debug;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use glam::Vec2;
use log::warn;
use parking_lot::RwLock;
use runim::prelude::{ObjectID, Scene, Shape};
use runim::IntoDuration;
use crate::animations::noop::NoOpAnimation;
use crate::shapes::RIObject;

mod color;
mod compositors;
mod transform;
pub mod easing;
mod easing_wrapper;
mod noop;

pub use color::{RGBColorTween, HSVColorTween};
pub use compositors::{SequentialCompositor, ParallelCompositor};
pub use transform::{MoveAnimation};
pub use easing_wrapper::{AnimationEasing, EasingFn};
pub(crate) use easing_wrapper::DynEasingFn;
use crate::interpolations::InterpolationBuilder;
use crate::prelude::Interpolation;

pub struct DynamicShape<'a> {
    pub(crate) state: &'a mut Box<dyn Shape>,
    pub(crate) id: ObjectID
}

impl DynamicShape<'_> {
    pub fn as_ref<T: Shape>(&self) -> Option<&T> {
        self.state.as_any().downcast_ref::<T>()
    }

    pub fn as_mut<T: Shape>(&mut self) -> Option<&mut T> {
        self.state.as_any_mut().downcast_mut::<T>()
    }
    
    pub fn id(&self) -> &ObjectID {
        &self.id
    }
}

impl Deref for DynamicShape<'_> {
    type Target = dyn Shape;

    fn deref(&self) -> &Self::Target {
        self.state.as_ref()
    }
}

impl DerefMut for DynamicShape<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state.as_mut()
    }
}

pub trait AnimationBase: Debug + Send + Sync + 'static {
    /// Returns full duration of the animation.
    fn duration(&self) -> f64;

    /// Returns one or multiple [ObjectID]s of shapes affected
    /// by this animation.
    ///
    /// [Animation::apply] will be called for each one.
    fn affected_shapes(&self) -> &AffectedShapes;
}

pub trait Animation: AnimationBase + Debug + Send + Sync + 'static {
    /// Applies animation to provided state. Provided state is always a fresh
    /// clone, so animations do not accumulate.
    ///
    /// `d` (∈ [0, 1]) is progress of this animation. Starting from `0` and finishing with `1`.
    fn apply(&self, state: &mut DynamicShape, d: f64);
}

/// Provides shortcut functions for animation composing.
///
/// This trait is automatically implemented for all `#[derive(Animation)]`.
///
/// If you wish to implement it yourself add `#[animation(custom_compositor)]` after derive call.
///
/// See docs of trait's functions for more info and examples.
pub trait AnimationCompositor: Animation {
    /// Plays another animation **after** this one.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// let scene = dispatcher.new_scene();
    /// let square = dispatcher.scene(&scene).add_rectangle().set_color(Color::RED).id();
    ///
    /// dispatcher.play(
    ///     RGBColorTween::new(square, 1.5, Color::RED, Color::BLUE)
    ///     .then(RGBColorTween::new(square, 1.5, Color::BLUE, Color::GREEN))
    ///     .then(RGBColorTween::new(square, 1.5, Color::GREEN, Color::RED))
    /// );
    /// // Square will turn blue, green, and lastly red.
    /// ```
    ///
    /// Example above will result in following timeline:
    /// ```plain
    /// Time      ↓ 0.0             ↓ 1.5             ↓ 3.0           ↓ 4.5
    /// Animation [  RED -> BLUE  ] [ BLUE -> GREEN ] [ GREEN -> RED  ]
    /// ```
    /// Meaning animations will play one after another without overlapping.
    /// 
    /// See [SequentialCompositor] for more information.
    #[must_use = "Animation must be passed to a dispatcher to be played"]
    fn then<A: Animation>(self, other: A) -> SequentialCompositor where Self: Sized {
        let mut v: Vec<Arc<dyn Animation>> = Vec::with_capacity(2);
        v.push(Arc::new(self));
        v.push(Arc::new(other));
        SequentialCompositor::new(v)
    }

    /// Plays another animation **during** this one.
    ///
    /// # Example
    /// ```
    /// use glam::Vec2;
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// let scene = dispatcher.new_scene();
    /// let square = dispatcher.scene(&scene).add_rectangle().set_color(Color::RED).id();
    ///
    /// dispatcher.play(
    ///     RGBColorTween::new(square, 1.5, Color::RED, Color::BLUE)
    ///     .with(MoveAnimation::new(square, 2.0, Vec2::new(0.0, 0.0), Vec2::new(1.5, 0.0)))
    /// );
    /// // Square will turn red and move to right at the same time
    /// ```
    ///
    /// Example above will result in following timeline:
    /// ```plain
    /// Time      ↓ 0.0           ↓ 1.5    ↓ 2.0
    /// Animation [  RED -> BLUE  ].........
    ///           [          MOVE          ]
    /// ```
    /// Meaning animations will play simultaneously.
    ///
    /// Red to blue animation will stop earlier, because its duration is shorter.
    /// 
    /// See [ParallelCompositor] for more information.
    #[must_use = "Animation must be passed to a dispatcher to be played"]
    fn with<A: Animation>(self, other: A) -> ParallelCompositor where Self: Sized {
        let mut v: Vec<Arc<dyn Animation>> = Vec::with_capacity(2);
        v.push(Arc::new(self));
        v.push(Arc::new(other));
        ParallelCompositor::new(v)
    }

    /// Wait **before** this animation.
    ///
    /// # Example
    /// ```
    /// use glam::Vec2;
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// let scene = dispatcher.new_scene();
    /// let square = dispatcher.scene(&scene).add_rectangle().set_color(Color::RED).id();
    ///
    /// dispatcher.play(
    ///     MoveAnimation::new(square, 2.0, Vec2::new(0.0, 0.0), Vec2::new(1.5, 0.0))
    ///     .prepend(1.0)
    /// );
    /// // Square 1 second will pass, then square will move.
    /// ```
    ///
    /// Example above will result in following timeline:
    /// ```plain
    /// Time      ↓ 0.0       ↓ 1.0                ↓ 3.0
    /// Animation ............[        MOVE        ]
    /// ```
    /// Meaning animation will play after specified time gap.
    #[must_use = "Animation must be passed to a dispatcher to be played"]
    fn prepend<D: IntoDuration>(self, duration: D) -> SequentialCompositor where Self: Sized {
        NoOpAnimation::new_shapeless(duration).then(self)
    }

    /// Wait **after** this animation.
    ///
    /// # Example
    /// ```
    /// use glam::Vec2;
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// let scene = dispatcher.new_scene();
    /// let square = dispatcher.scene(&scene).add_rectangle().set_color(Color::RED).id();
    ///
    /// dispatcher.play(
    ///     MoveAnimation::new(square, 2.0, Vec2::new(0.0, 0.0), Vec2::new(1.5, 0.0))
    ///     .append(1.0)
    /// );
    /// // Square 1 second will pass, then square will move.
    /// ```
    ///
    /// Example above will result in following timeline:
    /// ```plain
    /// Time      ↓ 0.0                ↓ 2.0       ↓ 3.0
    /// Animation [        MOVE        ]............
    /// ```
    /// Meaning animation will wait after playing for specified duration.
    #[must_use = "Animation must be passed to a dispatcher to be played"]
    fn append<D: IntoDuration>(self, duration: D) -> SequentialCompositor where Self: Sized {
        self.then(NoOpAnimation::new_shapeless(duration))
    }

    /// Adds custom easing over this animation.
    /// 
    /// This function accepts boxed easing functions.
    /// For `'static` references use [AnimationCompositor::add_easing_ref].
    /// 
    /// # Example
    /// ```
    /// use glam::Vec2;
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// let scene = dispatcher.new_scene();
    /// let square = dispatcher.scene(&scene).add_rectangle().set_color(Color::RED).id();
    ///
    /// dispatcher.play(
    ///     MoveAnimation::new(square, 2.0, Vec2::new(0.0, 0.0), Vec2::new(1.5, 0.0))
    ///     .add_easing_ref(&easing::reversed)
    /// );
    /// ```
    #[must_use = "Animation must be passed to a dispatcher to be played"]
    fn add_easing(self, fnc: Box<dyn EasingFn>) -> AnimationEasing where Self: Sized {
        AnimationEasing::new(self, fnc)
    }

    #[must_use = "Animation must be passed to a dispatcher to be played"]
    fn add_easing_ref(self, fnc: &'static dyn EasingFn) -> AnimationEasing where Self: Sized {
        AnimationEasing::new_ref(self, fnc)
    }
}

/// Represents shapes affected by [Animation].
///
/// [Generic functions](https://doc.rust-lang.org/reference/items/functions.html#r-items.fn.generics)
/// may use [IntoAffectedShapes] to accept values of this enum more conveniently.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AffectedShapes {
    Single(ObjectID),
    Vector(Vec<ObjectID>)
}

impl AffectedShapes {
    pub(crate) fn apply_all(&self, scene: &mut Scene, animation: &Arc<dyn Animation>, d: f64) {
        match self {
            AffectedShapes::Single(s) => {
                Self::animate(scene.get_raw(*s), *s, animation, d);
            }
            AffectedShapes::Vector(s) => {
                for x in s {
                    Self::animate(scene.get_raw(*x), *x, animation, d);
                }
            }
        }
    }

    fn animate(shape: Option<Arc<RwLock<Box<dyn Shape>>>>, id: ObjectID, animation: &Arc<dyn Animation>, d: f64) {
        match shape {
            Some(s) => {
                let mut guard = s.write();
                let mut untyped = DynamicShape {
                    state: &mut guard,
                    id
                };
                animation.apply(&mut untyped, d);
                drop(guard);
            }
            None => {
                warn!("Tried to animate unknown shape. Use debugger for details")
            }
        }
    }

    pub fn contains(&self, id: ObjectID) -> bool {
        match self {
            AffectedShapes::Single(s) => *s == id,
            AffectedShapes::Vector(s) => s.contains(&id)
        }
    }

    pub(crate) fn append(&mut self, other: AffectedShapes) {
        if let AffectedShapes::Single(s) = &self {
            let _ = mem::replace(self, AffectedShapes::Vector(vec![*s]));
            return;
        }
        if let AffectedShapes::Vector(vec) = self {
            match other {
                AffectedShapes::Single(value) => {
                    vec.push(value);
                }
                AffectedShapes::Vector(mut values) => {
                    vec.append(&mut values);
                }
            }
            return;
        }
        unreachable!() // Normal match can't be used here due to borrow checker's restrictions
    }

    pub(crate) fn dedup(&mut self) {
        if let AffectedShapes::Vector(vec) = self {
            vec.dedup();
        }
    }
}

/// Types that can be converted into [AffectedShapes].
///
/// Intended to be used in [generic functions](https://doc.rust-lang.org/reference/items/functions.html#r-items.fn.generics).
///
/// This trait is implemented for following types by default:
/// - [ObjectID]
/// - ([ObjectID], [ObjectID])
/// - ([ObjectID], [ObjectID], [ObjectID])
/// - [Vec]<[ObjectID]>
/// - [RIObject]
pub trait IntoAffectedShapes {
    fn into_affected_shape(self) -> AffectedShapes;
}

impl IntoAffectedShapes for ObjectID {
    fn into_affected_shape(self) -> AffectedShapes {
        AffectedShapes::Single(self)
    }
}

impl IntoAffectedShapes for (ObjectID, ObjectID) {
    fn into_affected_shape(self) -> AffectedShapes {
        AffectedShapes::Vector(vec![self.0, self.1])
    }
}

impl IntoAffectedShapes for (ObjectID, ObjectID, ObjectID) {
    fn into_affected_shape(self) -> AffectedShapes {
        AffectedShapes::Vector(vec![self.0, self.1, self.2])
    }
}

impl IntoAffectedShapes for Vec<ObjectID> {
    fn into_affected_shape(self) -> AffectedShapes {
        AffectedShapes::Vector(self)
    }
}

impl<T: Shape> IntoAffectedShapes for &RIObject<T> {
    fn into_affected_shape(self) -> AffectedShapes {
        AffectedShapes::Single(self.id())
    }
}

/// Provides shortcut functions to create common animations
pub trait AnimationHelper: IntoAffectedShapes + Sized {
    #[inline]
    #[must_use = "Animation must be passed to a dispatcher to be played"]
    fn interpolate<D: IntoDuration>(self, duration: D) -> InterpolationBuilder {
        Interpolation::builder(self, duration)
    }

    #[inline]
    #[must_use = "Animation must be passed to a dispatcher to be played"]
    fn move_from_to<D: IntoDuration, P: Into<Vec2>>(self, duration: D, from: P, to: P) -> MoveAnimation {
        MoveAnimation::new(self, duration, from.into(), to.into())
    }
}

impl<T: IntoAffectedShapes + Sized> AnimationHelper for T {}

impl<T: Shape> RIObject<T> {
    #[inline]
    #[must_use = "Animation must be passed to a dispatcher to be played"]
    pub fn move_to<D: IntoDuration, P: Into<Vec2>>(&self, duration: D, to: P) -> MoveAnimation {
        let from = self.shape().transform().pos;
        MoveAnimation::new(self, duration, from, to.into())
    }
}
