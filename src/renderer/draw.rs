use std::any::Any;
use glam::Vec3;

pub type DrawContext = Box<dyn DrawContextImpl>;

pub trait DrawContextImpl: Any {
    fn vertex(&mut self, pos: Vec3, color: Vec3);

    fn as_any_mut(&mut self) -> &mut dyn Any;
}