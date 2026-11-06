use glam::Vec2;
use runim::interpolate;
use runim::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut dispatcher = RunimDispatcher::<LivePreviewRenderer>::new()?;
    dispatcher.begin(AnimationConfig::p360());

    let scene = dispatcher.new_scene();

    let square =
        dispatcher.scene(&scene)
        .add_rectangle();
    square.set_pos(-2.0, 0.0);

    dispatcher.set_scene(scene)?;

    dispatcher.wait(0.2)?;

    dispatcher.play(
        interpolate!(&square, 3.0,
            property::POSITION: Vec2::new(-2.0, 0.0) => Vec2::new(2.0, 0.0); &easing::EASE_IN_OUT,
            property::COLOR: Color::WHITE => Color::RED; LINEAR
        )
    )?;

    dispatcher.wait(0.3)?;

    Ok(())
}