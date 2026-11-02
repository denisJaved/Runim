use glam::Vec3;
use crate::prelude::Shape;
use crate::shapes::RIObject;

/// Represents an RGB color. This type is used in Shapes.
///
/// # Example
/// ```
/// use runim::prelude::*;
/// use runim::renderer::debug::DebugRenderer;
///
/// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
/// let scene = dispatcher.new_scene();
///
/// let red = Color::new_hex(0xFF0000);
///
/// dispatcher.scene(&scene)
///     .add_square()
///     .set_color(red); // A red square!
/// ```
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl Color {
    pub const WHITE: Color = Color::new_hex(0xFFFFFF);
    pub const BLACK: Color = Color::new_hex(0x000000);
    pub const RED: Color = Color::new_hex(0xFF0000);
    pub const GREEN: Color = Color::new_hex(0x00FF00);
    pub const BLUE: Color = Color::new_hex(0x0000FF);

    /// Constructs new color from [f32] RGB values. `r ∈ [0, 1], g ∈ [0, 1], b ∈ [0, 1]`.
    ///
    /// This method is [const](https://rust-lang.github.io/rfcs/0911-const-fn.html) and may
    /// be evaluated during compilation.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let white = Color::new(1.0, 1.0, 1.0);
    /// let red = Color::new(1.0, 0.0, 0.0);
    /// let cyan = Color::new(0.1529, 0.9098, 0.8941);
    ///
    /// assert_eq!(white, Color::WHITE);
    /// assert_eq!(red, Color::RED);
    /// // There is no constant for cyan
    /// ```
    pub const fn new(r: f32, g: f32, b: f32) -> Color {
        debug_assert!(r >= 0.0 && r <= 1.0 && r.is_finite());
        debug_assert!(g >= 0.0 && g <= 1.0 && g.is_finite());
        debug_assert!(b >= 0.0 && b <= 1.0 && b.is_finite());

        Color { r, g, b }
    }

    /// Constructs new color from [u8] RGB values. `r ∈ [0, 255], g ∈ [0, 255], b ∈ [0, 255]`.
    ///
    /// This method is [const](https://rust-lang.github.io/rfcs/0911-const-fn.html) and may
    /// be evaluated during compilation.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let white = Color::new_rgb(255, 255, 255);
    /// let red = Color::new(255, 0, 0);
    /// let cyan = Color::new(39, 232, 228);
    ///
    /// assert_eq!(white, Color::WHITE);
    /// assert_eq!(red, Color::RED);
    /// // There is no constant for cyan
    /// ```
    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r as f32 / 255f32,
            g: g as f32 / 255f32,
            b: b as f32 / 255f32
        }
    }

    /// Constructs new color from [u32] value. `0x00RRGGBB`.
    ///
    /// This method is [const](https://rust-lang.github.io/rfcs/0911-const-fn.html) and may
    /// be evaluated during compilation.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let white = Color::new_hex(0xFFFFFF);
    /// let red = Color::new(0xFF0000);
    /// let cyan = Color::new(0x27E8E4);
    ///
    /// assert_eq!(white, Color::WHITE);
    /// assert_eq!(red, Color::RED);
    /// // There is no constant for cyan
    /// ```
    pub const fn new_hex(value: u32) -> Color {
        Self::new_rgb(
            ((value >> 16) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8
        )
    }


    /// Converts color to [f32] array `[r, g, b]`, where `r ∈ [0, 1], g ∈ [0, 1], b ∈ [0, 1]`.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let white = Color::new_hex(0xFFFFFF);
    /// let red = Color::new(0xFF0000);
    ///
    /// assert_eq!(white.to_array(), [1.0, 1.0, 1.0]);
    /// assert_eq!(red.to_array(), [1.0, 0.0, 0.0]);
    /// ```
    pub fn to_array(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl Into<Vec3> for Color {
    fn into(self) -> Vec3 {
        Vec3 {
            x: self.r,
            y: self.g,
            z: self.b,
        }
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Self::new(r, g, b)
    }
}

/// Base trait for all [Shape]s that have primary [Color].
///
/// It used for animating color changes and for [RIObject::set_color].
pub trait ColoredShape: Shape {
    fn color(&self) -> &Color;
    fn color_mut(&mut self) -> &mut Color;
    fn set_color(&mut self, color: Color);
}

impl<T: ColoredShape> RIObject<T> {
    pub fn set_color(&self, color: Color) -> &RIObject<T> {
        {
            let mut guard = self.shape_mut();
            guard.set_color(color);
        }
        self
    }
}

#[macro_export]
macro_rules! impl_shape_color {
    // TODO: Replace with a derive-marco
    ($t:path) => {
        impl crate::shapes::ColoredShape for $t {
            fn color(&self) -> &crate::shapes::Color {
                &self.color
            }

            fn color_mut(&mut self) -> &mut crate::shapes::Color {
                &mut self.color
            }

            fn set_color(&mut self, color: crate::shapes::Color) {
                self.color = color;
            }
        }
    };
}