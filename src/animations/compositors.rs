//! Provides simple animation compositors
//!
//! Animation compositors are special [Animation]s that execute and control other animations.
//!
//! Most of builtin compositors can be used without directly accessing their types.
//! Check out [AnimationCompositor] trait for more info.

use crate::animations::{AffectedShapes, Animation, AnimationCompositor, DynamicShape};
use crate::dispatch::IntoAnimations;
use runim_derive::Animation;
use std::sync::Arc;
use crate::animations::noop::NoOpAnimation;
use crate::IntoDuration;

/// Executes given animations sequentially.
///
/// This can be used to pack multiple animations in one object
/// or to execute them in order.
///
/// # Example
/// ```
/// use runim::prelude::*;
/// use runim::animations::SequentialCompositor;
/// use runim::renderer::debug::DebugRenderer;
///
/// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
/// let scene = dispatcher.new_scene();
/// let square = dispatcher.scene(&scene).add_rectangle().set_color(Color::RED).id();
///
/// // Example 1
/// dispatcher.play(
///     RGBColorTween::new(square, 1.0, Color::RED, Color::BLUE),
///     RGBColorTween::new(square, 1.0, Color::BLUE, Color::GREEN)
/// );
///
/// // Example 2
/// dispatcher.play(SequentialCompositor::new((
///     RGBColorTween::new(square, 1.0, Color::RED, Color::BLUE),
///     RGBColorTween::new(square, 1.0, Color::BLUE, Color::GREEN)
/// )));
/// ```
///
/// In code above first example will result in following timeline:
/// ```plain
/// Time      ↓ 0.0           ↓ 1.0
/// Animation [  RED -> BLUE  ]
///           [ BLUE -> GREEN ]
/// ```
/// Both animations played at the same time. Moreover, using multiple
/// animations that affect same objects without a compositor is UB.
///
/// Second example solves this by producing following timeline:
/// ```plain
/// Time      ↓ 0.0           ↓ 1.0            ↓ 2.0
/// Animation [  RED -> BLUE  ][ BLUE -> GREEN ]
/// ```
/// Now animations don't overlap and square smoothly changes from red to green passing green.
///
/// ## AnimationCompositor trait
/// `SequentialCompositor::new` takes up a lot of space and does not look clean.
///
/// This can be solved using [AnimationCompositor::then] function:
/// ```ignore
/// dispatcher.play(
///     RGBColorTween::new(square, 1.0, Color::RED, Color::BLUE)
///     .then(RGBColorTween::new(square, 1.0, Color::BLUE, Color::GREEN))
/// );
/// ```
/// More than one animation can be chained like this:
/// ```ignore
/// dispatcher.play(
///     RGBColorTween::new(square, 1.0, Color::RED, Color::BLUE)
///     .then(RGBColorTween::new(square, 1.0, Color::BLUE, Color::GREEN))
///     .then(RGBColorTween::new(square, 1.0, Color::GREEN, Color::RED))
/// );
/// ```
/// ```plain
/// Time      ↓ 0.0             ↓ 1.0             ↓ 2.0           ↓ 3.0
/// Animation [  RED -> BLUE  ] [ BLUE -> GREEN ] [ GREEN -> RED  ]
/// ```
///
/// # Limitations
/// Duration of a sequential compositor can't be changed manually.
/// It is always equal to sum of all animations in the compositor.
///
/// [SequentialCompositor] marks all shapes affected that are affected by
/// animations inside, although affected shapes of composed animations are respected.
#[derive(Animation, Debug, Clone)]
#[animation(custom_compositor)]
pub struct SequentialCompositor {
    shapes: AffectedShapes,
    duration: f64,
    parts: Vec<(f64, Arc<dyn Animation>)>
}

impl SequentialCompositor {
    /// Creates new sequential compositor with given animations.
    ///
    /// Check type docs for examples.
    pub fn new<A>(animations: A) -> SequentialCompositor
    where A: IntoAnimations {
        let mut offset: f64 = 0.0;
        let mut shapes = AffectedShapes::Vector(vec![]);

        let parts: Vec<(f64, Arc<dyn Animation>)> = animations.into_animations()
            .into_iter()
            .map(|r| {
                // Add all affects shapes from all animations
                shapes.append(r.affected_shapes().clone());

                let o = offset;
                offset += r.duration();
                (o, r)
            })
            .collect();

        shapes.dedup();

        SequentialCompositor {
            shapes,
            duration: offset,
            parts,
        }
    }

    /// Adds animation to begging of this sequence.
    pub fn prepend<A: Animation>(&mut self, animation: A) {
        let secs = animation.duration().as_secs_f64();
        self.duration += secs;

        for x in self.parts.iter_mut() {
            x.0 += secs;
        }

        self.parts.insert(0, (0.0, Arc::new(animation)));
    }

    /// Adds animation to end of this sequence.
    pub fn append<A: Animation>(&mut self, animation: A) {
        let secs = animation.duration().as_secs_f64();
        self.parts.push((self.duration, Arc::new(animation)));
        self.duration += secs;
    }
}

impl Animation for SequentialCompositor {
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        let x = d * self.duration;
        for i in self.parts.iter() {
            if i.0 > x { continue }
            if !i.1.affected_shapes().contains(state.id) { return; }

            let d0 = (x - i.0) / i.1.duration();
            i.1.apply(state, d0);
        }
    }
}

impl AnimationCompositor for SequentialCompositor {
    fn then<A: Animation>(mut self, other: A) -> SequentialCompositor
    where Self: Sized
    {
        self.duration += other.duration();
        self.shapes.append(other.affected_shapes().clone());
        self.shapes.dedup();

        let last = self.parts.last().unwrap();
        let offset = last.0 + last.1.duration();

        self.parts.push((offset, Arc::new(other)));
        self
    }

    fn prepend<D: IntoDuration>(mut self, duration: D) -> SequentialCompositor
    where Self: Sized,
    {
        (&mut self).prepend(NoOpAnimation::new_shapeless(duration));
        self
    }

    fn append<D: IntoDuration>(mut self, duration: D) -> SequentialCompositor
    where Self: Sized,
    {
        (&mut self).append(NoOpAnimation::new_shapeless(duration));
        self
    }
}

/// Executes given animations simultaneously.
///
/// This can be used to mix animations that affect same shapes
/// or to execute them at the same time.
///
/// # Example
/// ```
/// use glam::Vec2;
/// use runim::prelude::*;
/// use runim::animations::ParallelCompositor;
/// use runim::renderer::debug::DebugRenderer;
///
/// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
/// let scene = dispatcher.new_scene();
/// let square = dispatcher.scene(&scene).add_rectangle().set_color(Color::RED).id();
///
/// // Example 1
/// dispatcher.play(
///     RGBColorTween::new(square, 1.0, Color::RED, Color::BLUE),
///     MoveAnimation::new(square, 1.0, Vec2::ZERO, Vec2::new(1.0, 0.0))
/// );
///
/// // Example 2
/// dispatcher.play(ParallelCompositor::new((
///     RGBColorTween::new(square, 1.0, Color::RED, Color::BLUE),
///     MoveAnimation::new(square, 1.0, Vec2::ZERO, Vec2::new(1.0, 0.0))
/// )));
///
/// // Example 3
/// dispatcher.play(ParallelCompositor::new((
///     RGBColorTween::new(square, 2.0, Color::RED, Color::BLUE),
///     MoveAnimation::new(square, 1.0, Vec2::ZERO, Vec2::new(1.0, 0.0))
/// )));
///
/// // Example 4
/// dispatcher.play(ParallelCompositor::new((
///     RGBColorTween::new(square, 2.0, Color::RED, Color::BLUE),
///     MoveAnimation::new(square, 1.0, Vec2::ZERO, Vec2::new(1.0, 0.0)).prepend(1.0)
/// )));
/// ```
///
///
/// ## Example 1, 2
/// First and second examples will produce following timeline:
/// ```plain
/// Time      ↓ 0.0           ↓ 1.0
/// Animation [  RED -> BLUE  ]
///           [     MOVE      ]
/// ```
/// This is, because [runim::prelude::RunimDispatcher::play] executes animations simultaneously too,
/// but first example is wrong.
///
/// Play function defines executing multiple animations that affect same object as UB.
/// On other hand, parallel compositors sums changes of all its animations, so it is allowed to be used like this.
///
/// ## Example 3, 4
/// Example 3 executes two animations with different duration. In this case timeline look like this:
/// ```plain
/// Time      ↓ 0.0           ↓ 1.0           ↓ 2.0
/// Animation [          RED -> BLUE          ]
///           [     MOVE      ]................
/// ```
/// Parallel compositor aligns animations to start.
/// This causes `[MOVE]` animation to fully finish halfway through matching it duration.
///
/// Fourth example uses [AnimationCompositor::prepend] to move `[MOVE]` one second forming following timeline:
/// ```plain
/// Time      ↓ 0.0           ↓ 1.0           ↓ 2.0
/// Animation [          RED -> BLUE          ]
///           ................[     MOVE      ]
/// ```
///
/// ## AnimationCompositor trait
/// `ParallelCompositor::new` takes up a lot of space and doesn't look good.
///
/// This can be solved using [AnimationCompositor::with] function. Let's see example 3 with it:
/// ```ignore
/// // Example 3
/// dispatcher.play(
///     RGBColorTween::new(square, 2.0, Color::RED, Color::BLUE)
///     .with(MoveAnimation::new(square, 1.0, Vec2::ZERO, Vec2::new(1.0, 0.0)))
/// );
/// ```
///
/// # Limitations
/// Duration of [ParallelCompositor] can't be changed manually. It always matches duration
/// of the **longest** animation in the compositor.
///
/// [ParallelCompositor] marks all shapes affected that are affected by
/// animations inside, although affected shapes of composed animations are respected.
#[derive(Animation, Debug, Clone)]
#[animation(custom_compositor)]
pub struct ParallelCompositor {
    shapes: AffectedShapes,
    duration: f64,
    parts: Vec<Arc<dyn Animation>>
}

impl ParallelCompositor {
    pub fn new<A>(animations: A) -> ParallelCompositor
    where A: IntoAnimations {
        let parts = animations.into_animations();

        let mut duration: f64 = 0.0;
        let mut shapes = AffectedShapes::Vector(vec![]);
        for x in parts.iter() {
            if x.duration() > duration {
                duration = x.duration();
            }
            shapes.append(x.affected_shapes().clone());
        }

        shapes.dedup();

        ParallelCompositor {
            shapes,
            duration,
            parts,
        }
    }
}

impl Animation for ParallelCompositor {
    fn apply(&self, state: &mut DynamicShape, d: f64) {
        let x = d * self.duration;
        for i in self.parts.iter() {
            if !i.affected_shapes().contains(state.id) { continue }

            let d0 = (x / i.duration()).min(1.0);
            i.apply(state, d0);
        }
    }
}

impl AnimationCompositor for ParallelCompositor {
    fn with<A: Animation>(mut self, other: A) -> ParallelCompositor
    where Self: Sized
    {
        if other.duration() > self.duration {
            self.duration = other.duration();
        }
        self.parts.push(Arc::new(other));
        self
    }
}