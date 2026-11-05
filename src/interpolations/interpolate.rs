use glam::{Vec2, Vec3};
use crate::shapes::{Color, ColorHSV};

pub trait Interpolate: 'static {
    fn lerp(&self, other: &Self, t: f64) -> Self;
}

impl Interpolate for f32 {
    fn lerp(&self, other: &Self, t: f64) -> Self { self + (other - self) * t as f32 }
}
impl Interpolate for f64 {
    fn lerp(&self, other: &Self, t: f64) -> Self { self + (other - self) * t }
}
impl Interpolate for Vec2 {
    fn lerp(&self, other: &Self, t: f64) -> Self { Vec2::lerp(*self, *other, t as f32) }
}
impl Interpolate for Vec3 {
    fn lerp(&self, other: &Self, t: f64) -> Self { Vec3::lerp(*self, *other, t as f32) }
}
impl Interpolate for Color {
    fn lerp(&self, other: &Self, t: f64) -> Self { Color::interpolate(self, other, t as f32) }
}
impl Interpolate for ColorHSV {
    fn lerp(&self, other: &Self, t: f64) -> Self { ColorHSV::interpolate(self, other, t as f32) }
}
