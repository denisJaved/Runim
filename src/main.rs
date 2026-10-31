use runim::prelude::*;
use runim::renderer::debug::DebugRenderer;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let mut dispatcher = RunimDispatcher::<DebugRenderer>::new()?;
    dispatcher.begin(AnimationConfig::p360());
    
    let scene = dispatcher.new_scene();
    
    dispatcher.scene(&scene)
        .add_square()
        .set_pos(16.0, 3.0)
        .set_size(10.0, 10.0);
    
    dispatcher.scene(&scene)
        .add_circle()
        .set_pos(-16.0, -3.0)
        .set_radius(5.0);
    
    dispatcher.set_scene(scene)?;
    
    Ok(())
}
