/// Specifies size and frame rate for Runim rendering.
///
/// # Example
/// ```ignore
/// use runim::shapes::AnimationConfig;
///
/// let config = AnimationConfig {
///     width: 640,
///     height: 360,
///     fps: 60
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32
}

impl AnimationConfig {
    /// [AnimationConfig] preset: `640x360px * 30 fps`.
    ///
    /// This preset is recommended for previewing the animation.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let config = AnimationConfig::p360();
    ///
    /// assert_eq!(config.width, 640);
    /// assert_eq!(config.height, 360);
    /// assert_eq!(config.fps, 30);
    /// ```
    pub fn p360() -> AnimationConfig {
        AnimationConfig {
            width: 640,
            height: 360,
            fps: 30,
        }
    }

    /// [AnimationConfig] preset: `1920x1080px * 60 fps`.
    ///
    /// This preset is recommended for medium-quality renders.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let config = AnimationConfig::p1080();
    ///
    /// assert_eq!(config.width, 1920);
    /// assert_eq!(config.height, 1080);
    /// assert_eq!(config.fps, 60);
    /// ```
    pub fn p1080() -> AnimationConfig {
        AnimationConfig {
            width: 1920,
            height: 1080,
            fps: 60,
        }
    }

    /// [AnimationConfig] preset: `3840x2160px * 144 fps`.
    ///
    /// This preset is recommended for high-quality renders.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let config = AnimationConfig::p2160();
    ///
    /// assert_eq!(config.width, 3840);
    /// assert_eq!(config.height, 2160);
    /// assert_eq!(config.fps, 144);
    /// ```
    pub fn p2160() -> AnimationConfig {
        AnimationConfig {
            width: 3840,
            height: 2160,
            fps: 144,
        }
    }

    /// Updates size of this configuration. For builder-like usage.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let config = AnimationConfig::p1080()
    ///     .with_size(123, 456);
    ///
    /// assert_eq!(config.width, 123);
    /// assert_eq!(config.height, 456);
    /// ```
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Updates frame rate of this configuration. For builder-like usage.
    ///
    /// # Example
    /// ```
    /// use runim::prelude::*;
    ///
    /// let config = AnimationConfig::p1080()
    ///     .with_fps(10);
    ///
    /// assert_eq!(config.fps, 10);
    /// ```
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }
}
