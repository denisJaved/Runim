//! Runim provides builtin animation renderers that use [wgpu](https://crates.io/crates/wgpu).
//!
//! Those renderers require `builtin_renderer` feature flag.

mod graphics;
mod debug;
mod texture;
mod vertex;
mod live_preview;
mod file_renderer;
mod camera;

#[derive(Debug)]
enum OutputTarget {
    Window(live_preview::WindowTarget),
    Video(file_renderer::VideoTarget)
}

struct Frame {
    pub view: wgpu::TextureView,
    inner: FrameInner,
}

enum FrameInner {
    Surface(wgpu::SurfaceTexture),
    Offscreen,
}

pub use live_preview::LivePreviewRenderer;
pub use file_renderer::FileRenderer;
