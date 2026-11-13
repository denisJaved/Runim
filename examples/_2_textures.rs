use glam::Vec2;
use runim::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut dispatcher = RunimDispatcher::<LivePreviewRenderer>::new()?;
    dispatcher.begin(AnimationConfig::p360());

    let scene = dispatcher.new_scene();

    // Prefer loading images from files instead of including it in real code
    let s1 = dispatcher.scene(&scene)
        .add_img(image::load_from_memory(include_bytes!("fox.png"))?)
        .set_pos(0.0, -1.0)
        .set_size(1.0, 1.0)
        .id();
    let s2 = dispatcher.scene(&scene)
        .add_img(image::load_from_memory(include_bytes!("colorful_square.png"))?)
        .set_pos(0.0, 0.0)
        .set_size(1.0, 1.0)
        .id();
    let s3 = dispatcher.scene(&scene)
        .add_img(image::load_from_memory(include_bytes!("happy.png"))?)
        .set_pos(0.0, 1.0)
        .set_size(1.0, 1.0)
        .id();

    dispatcher.set_scene(scene)?;

    dispatcher.wait(0.2)?;
    dispatcher.play((
        MoveAnimation::new(s1, 2.0, Vec2::new(-2.0, -1.0), Vec2::new(2.0, -1.0))
            .add_easing_ref(&easing::EASE_IN_OUT),
        MoveAnimation::new(s2, 2.0, Vec2::new(2.0, 0.0), Vec2::new(-2.0, 0.0))
            .add_easing_ref(&easing::EASE_IN_OUT),
        MoveAnimation::new(s3, 2.0, Vec2::new(-2.0, 1.0), Vec2::new(2.0, 1.0))
            .add_easing_ref(&easing::EASE_IN_OUT)
    ))?;
    dispatcher.wait(0.3)?;

    Ok(())
}