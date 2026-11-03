use runim::renderer::DrawContext;
use runim::shapes::{Color, RIObject, Shape, Transform2D};
use glam::Vec3;
use runim::Shape;

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
impl Shape for Rectangle {
    fn draw(&self, ctx: &mut DrawContext) {
        ctx.offset2(self.transform.pos);
        
        ctx.vertex(Vec3::new(self.width, 0.0, 0.0), self.color);
        ctx.vertex(Vec3::new(self.width, self.height, 0.0), self.color);
        ctx.vertex(Vec3::new(0.0, self.height, 0.0), self.color);

        ctx.vertex(Vec3::new(0.0, 0.0, 0.0), self.color);
        ctx.vertex(Vec3::new(self.width, 0.0, 0.0), self.color);
        ctx.vertex(Vec3::new(0.0, self.height, 0.0), self.color);
        
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

#[derive(Shape, Debug, Clone, PartialEq)]
pub struct Circle {
    radius: f32,
    transform: Transform2D
}
impl Shape for Circle {
    fn draw(&self, ctx: &mut DrawContext) {
        todo!()
    }

    fn transform(&self) -> &Transform2D {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }
}

impl Default for Circle {
    fn default() -> Self {
        Circle {
            radius: 0.5,
            transform: Transform2D::default()
        }
    }
}

impl RIObject<Circle> {
    pub fn set_radius(&self, radius: f32) -> &RIObject<Circle> {
        {
            let mut guard = self.shape_mut();
            guard.radius = radius;
        }
        self
    }

    pub fn set_diameter(&self, diameter: f32) -> &RIObject<Circle> {
        {
            let mut guard = self.shape_mut();
            guard.radius = diameter / 2f32;
        }
        self
    }
}