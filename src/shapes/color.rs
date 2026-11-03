use glam::Vec3;
use runim::prelude::Shape;
use runim::shapes::RIObject;

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
///     .add_rectangle()
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
    /// let red = Color::new_rgb(255, 0, 0);
    /// let cyan = Color::new_rgb(39, 232, 228);
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
    /// let red = Color::new_hex(0xFF0000);
    /// let cyan = Color::new_hex(0x27E8E4);
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
    /// let red = Color::new_hex(0xFF0000);
    ///
    /// assert_eq!(white.to_array(), [1.0, 1.0, 1.0]);
    /// assert_eq!(red.to_array(), [1.0, 0.0, 0.0]);
    /// ```
    pub fn to_array(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    pub fn into_hsv(self) -> ColorHSV {
        self.into()
    }

    pub fn interpolate(a: &Color, b: &Color, t: f32) -> Color {
        Color {
            r: a.r + (b.r - a.r) * t,
            g: a.g + (b.g - a.g) * t,
            b: a.b + (b.b - a.b) * t
        }
    }

    pub fn round(self) -> Color {
        Color {
            r: (self.r * 100.0).round() / 100.0,
            g: (self.g * 100.0).round() / 100.0,
            b: (self.b * 100.0).round() / 100.0,
        }
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

/// Represents an HSV color. This type is used in Shapes.
///
/// # Example
/// ```
/// use runim::prelude::*;
/// use runim::renderer::debug::DebugRenderer;
///
/// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
/// let scene = dispatcher.new_scene();
///
/// let fun_color = ColorHSV::new(157.0, 32.0, 99.2);
///
/// dispatcher.scene(&scene)
///     .add_rectangle()
///     .set_color(fun_color); // A funny looking square!
/// ```
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ColorHSV {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32
}

impl ColorHSV {
    /// Constructs new color from [f32] HSV values. `h ∈ [0, 360], s ∈ [0, 100], v ∈ [0, 100]`.
    ///
    /// This method is [const](https://rust-lang.github.io/rfcs/0911-const-fn.html) and may
    /// be evaluated during compilation.
    pub const fn new(hue: f32, saturation: f32, value: f32) -> ColorHSV {
        ColorHSV { hue, saturation, value }
    }

    pub fn interpolate(a: &ColorHSV, b: &ColorHSV, t: f32) -> ColorHSV {
        // TODO: Fix it. it is broken
        let dh = (b.hue - a.hue + 180.0).rem_euclid(360.0) - 180.0;
        ColorHSV {
            hue: a.hue + t * dh,
            saturation: a.saturation + t * (b.saturation - a.saturation),
            value: a.value + t * (b.value - a.value),
        }
    }

    pub fn round(self) -> ColorHSV {
        ColorHSV {
            hue: self.hue.round(),
            saturation: (self.saturation * 10.0).round() / 10.0,
            value: (self.value * 10.0).round() / 10.0,
        }
    }

    pub fn into_rgb(self) -> Color {
        self.into()
    }
}

impl From<Color> for ColorHSV {
    fn from(value: Color) -> Self {
        let max = value.r.max(value.g).max(value.b);
        let min = value.r.min(value.g).min(value.b);
        let dif = max - min;

        let hue: f32 = match dif {
            0.0 => 0.0,
            _ if max == value.r => 60.0 * ((value.g - value.b) / dif % 6.0),
            _ if max == value.g => 60.0 * ((value.b - value.r) / dif + 2.0),
            _ => 60.0 * ((value.r - value.g) / dif + 4.0),
        };

        let saturation = match max {
            0.0 => 0.0,
            _ => dif / max * 100.0
        };

        ColorHSV {
            hue,
            saturation,
            value: max * 100.0,
        }
    }
}

impl Into<Color> for ColorHSV {
    fn into(self) -> Color {
        let c = (self.value / 100.0) * (self.saturation / 100.0);
        let x = c * (1.0 - ((self.hue / 60.0) % 2.0 - 1.0).abs());
        let m = (self.value / 100.0) - c;

        let (r, g, b) = match self.hue {
            0.0..60.0 => (c, x, 0.0),
            60.0..120.0 => (x, c, 0.0),
            120.0..180.0 => (0.0, c, x),
            180.0..240.0 => (0.0, x, c),
            240.0..300.0 => (x, 0.0, c),
            _ => (c, 0.0, x)
        };

        Color {
            r: r + m,
            g: g + m,
            b: b + m,
        }
    }
}

/// Base trait for all [Shape]s that have primary [Color].
///
/// It used for animating color changes and for [RIObject::set_color].
pub trait ColoredShape: Shape {
    /// Returns reference to [Shape]'s primary [Color].
    fn color(&self) -> &Color;

    /// Returns mutable reference to [Shape]'s primary [Color].
    fn color_mut(&mut self) -> &mut Color;

    /// Replaces [Shape]'s primary color with a new one.
    ///
    /// This method can be called directly on a [RIObject] using [RIObject::set_color].
    fn set_color(&mut self, color: Color);
}

impl<T: ColoredShape> RIObject<T> {
    /// Replaces linked [Shape]'s primary color.
    ///
    /// This function accepts any type that can be turned into [Color].
    /// That includes [Vec3] in RGB, [Color], [ColorHSV].
    /// 
    /// # Example
    /// ```
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// let scene = dispatcher.new_scene();
    /// 
    /// dispatcher.scene(&scene)
    ///     .add_rectangle()
    ///     .set_color(Color::RED); // <-- Turns rectangle red
    /// ```
    pub fn set_color<C: Into<Color>>(&self, color: C) -> &RIObject<T> {
        {
            let mut guard = self.shape_mut();
            guard.set_color(color.into());
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_to_hsv() {
        assert_eq!(
            Color::new_hex(0xFFFFFF).into_hsv().round(),
            ColorHSV::new(0.0, 0.0, 100.0)
        );
        assert_eq!(
            Color::new_hex(0xFF0000).into_hsv().round(),
            ColorHSV::new(0.0, 100.0, 100.0)
        );
        assert_eq!(
            Color::new_hex(0xDEFECA).into_hsv().round(),
            ColorHSV::new(97.0, 20.5, 99.6)
        );
        assert_eq!(
            Color::new_hex(0xACFDDE).into_hsv().round(),
            ColorHSV::new(157.0, 32.0, 99.2)
        );
    }

    #[test]
    fn hsv_to_rgb() {
        assert_eq!(
            Color::new_hex(0xFFFFFF).round(),
            ColorHSV::new(0.0, 0.0, 100.0).into_rgb().round()
        );
        assert_eq!(
            Color::new_hex(0xFF0000).round(),
            ColorHSV::new(0.0, 100.0, 100.0).into_rgb().round()
        );
        assert_eq!(
            Color::new_hex(0xDEFECA).round(),
            ColorHSV::new(96.9, 20.5, 99.6).into_rgb().round()
        );
        assert_eq!(
            Color::new_hex(0xACFDDE).round(),
            ColorHSV::new(157.0, 32.0, 99.2).into_rgb().round()
        );
    }
}