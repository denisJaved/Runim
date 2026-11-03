mod animation;
mod scene;
mod primitive;
mod color;

use std::any::Any;
use std::fmt::Debug;
use glam::Vec2;
use runim::renderer::DrawContext;

pub use scene::{Scene, RIObject};
pub use primitive::{Rectangle, Circle};
pub use animation::AnimationConfig;
pub use color::{Color, ColorHSV, ColoredShape};
pub(crate) use scene::ObjectDesc;

pub type ObjectID = u32;

/// Base trait for all [Shape]s.
/// 
/// This trait is not intended to be implemented manually.
/// Use Shape derive macro instead.
pub trait ShapeBase: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Returns this shape as [ColoredShape] reference if it is one. [None], otherwise.
    fn as_colored(&self) -> Option<&dyn ColoredShape>;
    /// Returns this shape as mutable [ColoredShape] reference if it is one. [None], otherwise.
    fn as_colored_mut(&mut self) -> Option<&mut dyn ColoredShape>;
    /// Returns untyped clone of this [Shape].
    fn clone_raw(&self) -> Box<dyn Shape>;
}

pub trait Shape: ShapeBase + Debug + Send + Sync + 'static {
    /// Renders this shape in provided [DrawContext].
    /// 
    /// Check [DrawContext]'s docs for more info.
    fn draw(&self, ctx: &mut DrawContext);
    
    /// Returns reference to this shape's [Transform2D].
    /// 
    /// # Example
    /// ```
    /// use glam::Vec2;
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// let scene = dispatcher.new_scene();
    ///
    /// let rect = dispatcher.scene(&scene).add_rectangle();
    /// rect.set_pos(13.0, 16.0);
    ///
    /// let guard = rect.shape();
    /// let x = guard.transform(); // <-- here
    /// assert_eq!(x.pos, Vec2::new(13.0, 16.0));
    /// ```
    fn transform(&self) -> &Transform2D;

    /// Returns mutable reference to this shape's [Transform2D].
    /// 
    /// # Example
    /// ```
    /// use glam::Vec2;
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// let scene = dispatcher.new_scene();
    ///
    /// let rect = dispatcher.scene(&scene)
    ///     .add_rectangle();
    ///
    /// {
    ///     let mut guard = rect.shape_mut();
    ///     guard.transform_mut().pos.x = 25.0; // <-- Here
    /// }
    /// 
    /// let guard = rect.shape();
    /// assert_eq!(guard.transform().pos.x, 25.0);
    /// ```
    fn transform_mut(&mut self) -> &mut Transform2D;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transform2D {
    pub pos: Vec2,
    pub rot: f32
}

impl Default for Transform2D {
    fn default() -> Self {
        Transform2D {
            pos: Vec2::ZERO,
            rot: 0f32
        }
    }
}
