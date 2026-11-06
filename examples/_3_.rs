use glam::Vec2;
use runim::prelude::*;
use runim::interpolate;

fn main() -> anyhow::Result<()> {
    let mut dispatch = RunimDispatcher::<LivePreviewRenderer>::new()?;

    dispatch.begin(AnimationConfig::p360());

    let scene = dispatch.new_scene();
    let shape =
        dispatch.scene(&scene)
        .add_rectangle();
    shape.set_pos(-2.0, 0.0);

    dispatch.set_scene(scene)?;
    dispatch.wait(0.3)?;

    dispatch.play(
        interpolate!(&shape, 6.0,
            property::POSITION: Vec2::new(-2.0, 0.0) => Vec2::new(2.0, 0.0); &easing::EASE_IN_OUT,
            property::COLOR: Color::WHITE => Color::RED; LINEAR,
        )
    )?;

    dispatch.wait(0.3)?;

    Ok(())
}
