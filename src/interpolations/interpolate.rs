use glam::{Vec2, Vec3};
use crate::shapes::{Color, ColorHSV};

pub trait Interpolate: 'static {
    fn lerp(&self, other: &Self, t: f64) -> Self;
}

macro_rules! impl_whole {
    ($ty:ty) => {
        impl Interpolate for $ty {
            fn lerp(&self, other: &Self, t: f64) -> Self {
                let a = *self as f64;
                let b = *other as f64;
                (a + (b - a) * t).round() as Self
            }
        }
    };
}

impl_whole!(u8);
impl_whole!(i8);
impl_whole!(u16);
impl_whole!(i16);
impl_whole!(u32);
impl_whole!(i32);
impl_whole!(u64);
impl_whole!(i64);

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
