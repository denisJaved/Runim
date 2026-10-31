use crate::impl_shape_base;
use crate::renderer::DrawContext;
use crate::shapes::{RIObject, Shape};

#[derive(Debug, Clone)]
pub struct Square {
    width: f32,
    height: f32
}
impl_shape_base!(Square);
impl Shape for Square {
    fn draw(&self, ctx: &mut DrawContext) {
        todo!()
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
    pub fn set_size(&mut self, width: f32, height: f32) -> &mut RIObject<Square> {
        {
            let mut guard = self.shape_mut();
            guard.width = width;
            guard.height = height;
        }
        self
    }
}

#[derive(Debug, Clone)]
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
    pub fn set_radius(&mut self, radius: f32) -> &mut RIObject<Circle> {
        {
            let mut guard = self.shape_mut();
            guard.radius = radius;
        }
        self
    }

    pub fn set_diameter(&mut self, diameter: f32) -> &mut RIObject<Circle> {
        {
            let mut guard = self.shape_mut();
            guard.radius = diameter / 2f32;
        }
        self
    }
}