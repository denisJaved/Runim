//! # Runim - Rust Animation Renderer
//!
//! Runim is a library providing tools for creating and rendering
//! math animations using Rust programming language inspired
//! by [Manim](https://www.manim.community).
//!
//! # Installation
//! Run the following Cargo command to install Runim's latest stable version:
//! ```sh
//! cargo add runim
//! ```
//! Or add the following lines to your `Cargo.toml`:
//! ```yaml
//! [dependencies]
//! runim = "0.1.0"
//! ```

pub mod dispatch;
pub mod renderer;
pub mod shapes;
pub mod animations;
mod util_traits;
#[cfg(feature = "builtin_renderer")]
pub mod builtin_renderer;

extern crate self as runim;

pub use runim_derive::{Shape, Animation};
pub use util_traits::IntoDuration;

/// The Runim Prelude.
///
/// The purpose of this module is to alleviate imports of pragmatic Runim types and traits
/// by adding a glob import in code of Runim API clients:
/// ```
/// use runim::prelude::*;
/// ```
///
/// If importing objects with commonly used names like `Scene` causes name collisions,
/// [runim::prelude::prefixed] might be used.
///
/// This option requires `prefixed_prelude` feature flag to be enabled.
pub mod prelude {
    pub use runim::dispatch::RunimDispatcher;
    pub use runim::shapes::{AnimationConfig, Scene, RIObject, Shape, Circle, Rectangle, ObjectID, Color, ColorHSV};
    pub use runim::animations::{RGBColorTween, HSVColorTween, AnimationCompositor, MoveAnimation};
    
    #[cfg(feature = "builtin_renderer")]
    pub use runim::builtin_renderer::LivePreviewRenderer;

    /// Alternative version of the [runim::prelude] with all re-exports having `RI` prefix.
    ///
    /// This module allows using Runim prelide without claiming commonly used names like `Scene`.
    #[cfg(feature = "prefixed_prelude")]
    pub mod prefixed {
        pub use super::RunimDispatcher as RIDispatcher;
        pub use super::AnimationConfig as RIAnimationConfig;
        pub use super::Scene as RIScene;
        pub use super::RIObject;
        pub use super::Shape as RIShape;
        pub use super::Circle as RICircle;
        pub use super::Rectangle as RISquare;
        pub use super::ObjectID as RIObjID;
        pub use super::Color as RIColor;
        pub use super::ColorHSV as RIColorHSV;

        pub use super::RGBColorTween as RIRGBColorTween;
        pub use super::HSVColorTween as RIHSVColorTween;
        pub use super::AnimationCompositor as RIAnimationCompositor;
        pub use super::MoveAnimation as RIMoveAnimation;

        #[cfg(feature = "builtin_renderer")]
        pub use super::LivePreviewRenderer as RILivePreviewRenderer;
    }
}
