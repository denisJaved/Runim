use glam::Vec2;
use crate::animations::DynamicShape;
use crate::shapes::{Color, Rectangle};

#[derive(Copy, Clone)]
pub struct Property<T: 'static> {
    pub get: for<'a> fn(&'a DynamicShape) -> &'a T,
    pub get_mut: for<'a> fn(&'a mut DynamicShape) -> &'a mut T
}

macro_rules! shape_prop {
    ($name:ident: $ty:ty = $get:expr, $set:expr) => {
        pub const $name: runim::interpolations::Property<$ty> = runim::interpolations::Property {
            get: $get,
            get_mut: $set
        };
    };
}

fn unwrap<T>(opt: Option<T>) -> T {
    match opt {
        None => panic!("Unsupported interpolation"),
        Some(s) => s
    }
}

shape_prop!(POSITION: Vec2 = |x| &x.transform().pos, |x| &mut x.transform_mut().pos);
shape_prop!(COLOR: Color = |x| unwrap(x.as_colored()).color(), |x| unwrap(x.as_colored_mut()).color_mut());
