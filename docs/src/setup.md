## Installation
Before using Runim it needs to be installed.
Run following command in your [Cargo](https://doc.rust-lang.org/cargo/) project:
```shell
cargo add runim
```
Or add following lines to your `Cargo.toml`:
```toml
[dependencies]
runim = "<version of Runim to use>"
```
<p>
Latest version is <img alt="Crates.io Version" src="https://img.shields.io/crates/v/runim?style=flat-square&label=%20">.
</p>

## Creating Dispatcher and Renderer
Renderer is an object that will actually draw our animation.
There are two built-in option:
- [`LivePreviewRenderer`](https://docs.rs/runim/latest/runim/builtin_renderer/struct.LivePreviewRenderer.html) - 
Renders animation live to a window.
- [`FileRenderer`](https://docs.rs/runim/latest/runim/builtin_renderer/struct.FileRenderer.html) -
Renders animation to a video file using ffmpeg.

> [!NOTE]
> You don't need to install FFMpeg for `FileRenderer` to work,
> because Runim is linked to it using [ffmpeg-next](https://crates.io/crates/ffmpeg-next).

Renderers should not be instantiated manually, because Runim
creates them automatically. You just need to provide type
Renderer uses:
```rust
use runim::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut dispatch = RunimDispatcher::<LivePreviewRenderer>::new()?;
    
    Ok(())
}
```

`Runimdispatch` we created is our channel to talk with the renderer.
When we want to draw something, its functions need to be called.

If we run this code, nothing will happen. That's because Renderers only
start working after we configured them. Let's do that:

```rust
# use runim::prelude::*;

# fn main() -> anyhow::Result<()> {
#     let mut dispatch = RunimDispatcher::<LivePreviewRenderer>::new()?;
    dispatch.begin(AnimationConfig {
        width: 640,
        height: 360,
        fps: 30,
    });
#    Ok(())
# }
```

Updated code will make a small window show up for a moment.
That's our animation! The only thing left is just to draw something.

## Drawing
Everything that can be drawn is called [Shapes](https://docs.rs/runim/latest/runim/shapes/trait.Shape.html),
but they don't just exist globally. Every shape should be stored in a [Scene](https://docs.rs/runim/latest/runim/shapes/struct.Scene.html).
Let's create one with circle in it:
```rust
# use runim::prelude::*;

# fn main() -> anyhow::Result<()> {
#     let mut dispatch = RunimDispatcher::<LivePreviewRenderer>::new()?;
#     dispatch.begin(AnimationConfig {
#         width: 640,
#         height: 360,
#         fps: 30,
#     });
    let scene = dispatch.new_scene();

    let circle = dispatch.scene(&scene)
        .add::<Circle>();
#    Ok(())
# }
```

[`Scene::add`](https://docs.rs/runim/latest/runim/shapes/struct.Scene.html#method.add) function returns a [RIObject](https://docs.rs/runim/latest/runim/shapes/struct.RIObject.html) -
our handle to created shape. It can be used to configure and animate our circle,
but before we do that let's try drawing it:
```rust
# use runim::prelude::*;

# fn main() -> anyhow::Result<()> {
#     let mut dispatch = RunimDispatcher::<LivePreviewRenderer>::new()?;
#     dispatch.begin(AnimationConfig {
#         width: 640,
#         height: 360,
#         fps: 30,
#     });
#    let scene = dispatch.new_scene();

#    let circle = dispatch.scene(&scene)
#        .add::<Circle>();
#    
    dispatch.set_scene(scene)?;
    dispatch.wait(3.0)?;
#    Ok(())
# }
```
First, we will tell renderer to draw our scene. Then, we'll wait for 3 seconds.
This means that animation will render for 3 seconds without any animations.

Try running this code! You will see a white circle for three seconds before window will close.

# Simple Animations
Rendering static images is boring. For our video to be interesting, it needs animations.

Animations are changes to shapes that happen gradually during our animation.
Like circle turning red in 3 seconds. Time to make it happen.
```rust
# use runim::prelude::*;

# fn main() -> anyhow::Result<()> {
#     let mut dispatch = RunimDispatcher::<LivePreviewRenderer>::new()?;
#     dispatch.begin(AnimationConfig {
#         width: 640,
#         height: 360,
#         fps: 30,
#     });
#    let scene = dispatch.new_scene();

#    let circle = dispatch.scene(&scene)
#        .add::<Circle>();
#    
#     dispatch.set_scene(scene)?;
#     dispatch.wait(0.3)?;
# 
    dispatch.play(
        RGBColorTween::new(&circle, 3.0, Color::WHITE, Color::RED)
    )?;
#     
#    dispatch.wait(0.3)?;
#    Ok(())
# }
```
If you run this code, you will see circle slowly turning red.