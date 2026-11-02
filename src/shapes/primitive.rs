use glam::Vec3;
use crate::impl_shape_base;
use crate::renderer::DrawContext;
use crate::shapes::{RIObject, Shape};

#[derive(Debug, Clone, PartialEq)]
pub struct Square {
    width: f32,
    height: f32
}
impl_shape_base!(Square);
impl Shape for Square {
    fn draw(&self, ctx: &mut DrawContext) {
        ctx.vertex(Vec3::new(self.width, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        ctx.vertex(Vec3::new(self.width, self.height, 0.0), Vec3::new(0.0, 1.0, 0.0));
        ctx.vertex(Vec3::new(0.0, self.height, 0.0), Vec3::new(0.0, 1.0, 0.0));

        ctx.vertex(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        ctx.vertex(Vec3::new(self.width, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        ctx.vertex(Vec3::new(0.0, self.height, 0.0), Vec3::new(1.0, 0.0, 0.0));
    }
}

impl Default for Square {
    fn default() -> Self {
        Square {
            width: 1f32,
            height: 1f32
        }
    }
}

impl RIObject<Square> {
    pub fn set_size(&self, width: f32, height: f32) -> &RIObject<Square> {
        {
            let mut guard = self.shape_mut();
            guard.width = width;
            guard.height = height;
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    radius: f32
}
impl_shape_base!(Circle);
impl Shape for Circle {
    fn draw(&self, ctx: &mut DrawContext) {
        todo!()
    }
}

impl Default for Circle {
    fn default() -> Self {
        Circle {
            radius: 0.5
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