use crate::animations::DynamicShape;
use crate::shapes::{Circle, Color};
use glam::Vec2;

#[derive(Copy, Clone)]
pub struct Property<T: 'static> {
    pub get: for<'a> fn(&'a DynamicShape) -> &'a T,
    pub set: for<'a> fn(&'a mut DynamicShape, T) -> ()
}

macro_rules! shape_prop {
    ($name:ident: $ty:ty = $get:expr, $set:expr) => {
        pub const $name: ::runim::interpolations::Property<$ty> = ::runim::interpolations::Property {
            get: $get,
            set: $set
        };
    };
    ($name:ident for $cast:ty: $ty:ty = |$get_i:ident| $get:expr, |$set_i:ident, $set_v:ident| $set:expr) => {
        shape_prop!($name: $ty = |__private_x| {let $get_i = unwrap(__private_x.as_ref::<$cast>()); $get}, |__private_x, $set_v| {let $set_i = unwrap(__private_x.as_mut::<$cast>()); $set});
    }
}

fn unwrap<T>(opt: Option<T>) -> T {
    match opt {
        None => panic!("Unsupported interpolation"),
        Some(s) => s
    }
}

shape_prop!(POSITION: Vec2 = |x| &x.transform().pos, |x, v| x.transform_mut().pos = v);
shape_prop!(COLOR: Color = |x| unwrap(x.as_colored()).color(), |x, v| unwrap(x.as_colored_mut()).set_color(v));
shape_prop!(CIRCLE_RADIUS for Circle: f32 = |x| &x.radius, |x, v| x.radius = v);
shape_prop!(CIRCLE_ANGLES for Circle: u16 = |x| &x.angles, |x, v| x.angles = v);
