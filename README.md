# Runim - Rust Animation Renderer
Runim (***Ru***st A***nim***ation Library) is a library providing tools for creating and rendering
animations using Rust programming language inspired by [Manim](https://www.manim.community).

# Installation
Run the following Cargo command to install Runim's latest stable version:
```sh
cargo add runim
```
Or add following text to `Cargo.toml`:
```toml
[dependencies]
runim = "0.0.0-alpha1"
```

# Basic Usage
```rust
use runim::prelude::*;
use runim::interpolate;

fn main() -> anyhow::Result<()> {
    // Create Renderer and channel to communicate with it
    let mut dispatch = RunimDispatcher::<LivePreviewRenderer>::new()?;
    
    // Prepare the renderer
    dispatch.begin(AnimationConfig {
        width: 640,
        height: 360,
        fps: 30,
    });
    
    // Create new scene to store Shapes
    let scene = dispatch.new_scene();

    // Create one
    let shape = dispatch.scene(&scene)
        .add_rectangle();
    shape.set_pos(-2.0, 0.0);

    // Tell Renderer to use the scene
    dispatch.set_scene(scene)?;

    // Move it and make in turn red
    dispatch.play(
        interpolate!(&shape, 6.0,
            property::POSITION: Vec2::new(-2.0, 0.0) => Vec2::new(2.0, 0.0); &easing::EASE_IN_OUT,
            property::COLOR: Color::WHITE => Color::RED; LINEAR,
        )
    )?;
    
    Ok(())
}
```

Read [docs.rs](https://docs.rs/runim/) for details.