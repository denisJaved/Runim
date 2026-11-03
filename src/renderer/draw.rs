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

    fn as_any(&self) -> &dyn Any;
    
    fn as_any_mut(&mut self) -> &mut dyn Any;
}