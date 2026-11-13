use crate::AsAny;
use glam::{vec2, Vec2, Vec3, vec3};
use image::DynamicImage;
use runim::shapes::Color;
use std::fmt::Debug;
pub type ImageHandle = Box<dyn ImageHandleImpl>;

pub trait DrawContext {
    fn offset(&mut self, pos: Vec3);
    fn offset2(&mut self, pos: Vec2) {
        self.offset(Vec3::new(pos.x, pos.y, 0.0))
    }

    fn clear_offset(&mut self);
    
    fn vertex(&mut self, pos: Vec3, color: Color);
    fn tex_vertex(&mut self, pos: Vec3, tex_pos: Vec2, tex: &ImageHandle);

    fn triangle(&mut self, p1: Vec3, p2: Vec3, p3: Vec3, color: Color) {
        self.vertex(p1, color);
        self.vertex(p2, color);
        self.vertex(p3, color);
    }

    fn tex_quad(&mut self, width: f32, height: f32, tex: &ImageHandle) {
        self.tex_vertex(Vec3::ZERO, vec2(0.0, 1.0), tex);
        self.tex_vertex(vec3(
            width, 0.0, 0.0
        ), vec2(1.0, 1.0), tex);
        self.tex_vertex(vec3(
            0.0, height, 0.0
        ), vec2(0.0, 0.0), tex);

        self.tex_vertex(vec3(
            width, 0.0, 0.0
        ), vec2(1.0, 1.0), tex);
        self.tex_vertex(vec3(
            width, height, 0.0
        ), vec2(1.0, 0.0), tex);
        self.tex_vertex(vec3(
            0.0, height, 0.0
        ), vec2(0.0, 0.0), tex);

        // self.vertex(p1, Color::RED);
        // self.vertex(vec3(
        //     p1.x + width, p1.y, p1.z
        // ), Color::RED);
        // self.vertex(vec3(
        //     p1.x, p1.y + height, p1.z
        // ), Color::RED);
        //
        // self.vertex(vec3(
        //     p1.x + width, p1.y, p1.z
        // ), Color::RED);
        // self.vertex(vec3(
        //     p1.x + width, p1.y + height, p1.z
        // ), Color::RED);
        // self.vertex(vec3(
        //     p1.x, p1.y + height, p1.z
        // ), Color::RED);
    }

    fn load_image(&mut self, image: &DynamicImage) -> anyhow::Result<ImageHandle>;
}
pub trait ImageHandleImpl: AsAny + Debug + Send + Sync {}