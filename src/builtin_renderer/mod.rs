//! Runim provides builtin animation renderers that use [wgpu](https://crates.io/crates/wgpu).
//!
//! Those renderers require `builtin_renderer` feature flag.

mod graphics;
mod debug;
mod texture;
mod vertex;
mod adapters;

pub use adapters::LivePreviewRenderer;
