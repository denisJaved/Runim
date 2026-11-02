mod animation;
mod scene;
mod primitive;
mod color;

use std::any::Any;
use std::fmt::Debug;
use glam::Vec2;
use crate::renderer::DrawContext;

pub use scene::{Scene, RIObject};
pub use primitive::{Square, Circle};
pub use animation::AnimationConfig;
pub use color::{Color, ColoredShape};
pub(crate) use scene::ObjectDesc;

pub type ObjectID = u32;

pub trait ShapeBase {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_raw(&self) -> Box<dyn Shape>;
}

#[macro_export]
macro_rules! impl_shape_base {
    // TODO: Replace with a derive-marco
    ($t:path) => {
        impl crate::shapes::ShapeBase for $t {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn clone_raw(&self) -> Box<dyn crate::shapes::Shape> {
                Box::new(core::clone::Clone::clone(self))
            }
        }
    };
}

pub trait Shape: ShapeBase + Debug + Send + Sync + 'static {
    fn draw(&self, ctx: &mut DrawContext);
    fn transform(&self) -> &Transform2D;
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
