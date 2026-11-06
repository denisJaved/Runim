use std::any::Any;
use glam::{Vec2, Vec3};
use runim::shapes::Color;

pub type DrawContext = Box<dyn DrawContextImpl>;

pub trait DrawContextImpl: Any {
    fn offset(&mut self, pos: Vec3);
    fn offset2(&mut self, pos: Vec2) {
        self.offset(Vec3::new(pos.x, pos.y, 0.0))
    }

    fn clear_offset(&mut self);
    
    fn vertex(&mut self, pos: Vec3, color: Color);

    fn triangle(&mut self, p1: Vec3, p2: Vec3, p3: Vec3, color: Color) {
        self.vertex(p1, color);
        self.vertex(p2, color);
        self.vertex(p3, color);
    }

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}