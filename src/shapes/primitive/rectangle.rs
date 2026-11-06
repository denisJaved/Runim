use crate::prelude::{Color, RIObject, Shape};
use crate::renderer::DrawContext;
use crate::Shape;
use crate::shapes::Transform2D;

/// A rectangular [Shape] described by [f32] width and height.
///
/// # Example
/// ```
/// use runim::prelude::*;
/// use runim::renderer::debug::DebugRenderer;
///
/// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
/// let scene = dispatcher.new_scene();
///
/// dispatcher.scene(&scene)
///     .add_rectangle()        // Create new rectangle
///     .set_pos(3.0, 4.0)      // Move it
///     .set_size(1.25, 3.0)    // Resize it
///     .set_color(Color::RED); // Recolor it
/// ```
///
/// See [RIObject] for [RIObject::set_pos], [RIObject::set_size], and [RIObject::set_color] functions.
#[derive(Shape, Debug, Clone, PartialEq)]
#[shape(colored)]
pub struct Rectangle {
    width: f32,
    height: f32,
    transform: Transform2D,
    color: Color
}
impl Rectangle {
    pub fn width(&self) -> &f32 { &self.width }
    pub fn width_mut(&mut self) -> &mut f32 { &mut self.width }
    pub fn height(&self) -> &f32 { &self.height }
    pub fn height_mut(&mut self) -> &mut f32 { &mut self.height }
}
impl Shape for Rectangle {
    fn draw(&self, ctx: &mut DrawContext) {
        use glam::vec3;
        ctx.offset(vec3(
            self.transform.pos.x - self.width / 2.0,
            self.transform.pos.y - self.height / 2.0,
            0.0
        ));

        ctx.triangle(
            vec3(self.width, 0.0, 0.0),
            vec3(self.width, self.height, 0.0),
            vec3(0.0, self.height, 0.0),
            self.color
        );

        ctx.triangle(
            vec3(0.0, 0.0, 0.0),
            vec3(self.width, 0.0, 0.0),
            vec3(0.0, self.height, 0.0),
            self.color
        );

        ctx.clear_offset();
    }

    fn transform(&self) -> &Transform2D {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Rectangle {
            width: 1f32,
            height: 1f32,
            transform: Transform2D::default(),
            color: Color::WHITE,
        }
    }
}

impl RIObject<Rectangle> {
    /// Replaces linked [Rectangle]'s width and height.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// let scene = dispatcher.new_scene();
    ///
    /// dispatcher.scene(&scene)
    ///     .add_rectangle()
    ///     .set_size(14.0, 34.0); // <-- Resizes the rectangle
    /// ```
    pub fn set_size(&self, width: f32, height: f32) -> &RIObject<Rectangle> {
        {
            let mut guard = self.shape_mut();
            guard.width = width;
            guard.height = height;
        }
        self
    }
}