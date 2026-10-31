use runim::prelude::*;
use runim::renderer::debug::DebugRenderer;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let mut dispatcher = RunimDispatcher::<DebugRenderer>::new()?;
    dispatcher.begin(AnimationConfig::p360());
    
    Ok(())
}
