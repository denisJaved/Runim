use std::any::Any;
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use glam::Vec3;
use log::info;
use crate::animations::{AffectedShapes, Animation, DynamicShape};
use crate::dispatch::{Command, CommandCallback, FriendlyRenderer, Renderer};
use crate::prelude::{AnimationConfig, Color};
use crate::renderer::DrawContextImpl;
use crate::shapes::{ObjectDesc, ObjectID};

/// Renderer used for testing.
/// 
/// It does not actually render anything, but
/// it runs animations and no-op draws.
pub struct SilentTestRenderer {
    callback: mpsc::Sender<CommandCallback>,
    state: Option<LocalState>
}

struct LocalState {
    config: AnimationConfig,
    objects: Vec<ObjectDesc>
}

impl SilentTestRenderer {
    fn state_mut(&mut self) -> &mut LocalState {
        match self.state.as_mut() {
            None => panic!("begin was not called"),
            Some(s) => s
        }
    }
    fn state(&self) -> &LocalState {
        match self.state.as_ref() {
            None => panic!("begin was not called"),
            Some(s) => s
        }
    }
}

impl FriendlyRenderer for SilentTestRenderer {
    fn begin(&mut self, config: AnimationConfig) -> anyhow::Result<CommandCallback> {
        assert!(self.state.is_none(), "begin called twice");
        
        self.state = Some(LocalState {
            config,
            objects: vec![]
        });
        
        Ok(CommandCallback::Ok)
    }

    fn set_scene(&mut self, objects: Vec<ObjectDesc>) -> anyhow::Result<CommandCallback> {
        self.state_mut().objects = objects;
        
        Ok(CommandCallback::Ok)
    }

    fn play(&mut self, animations: Vec<Arc<dyn Animation>>, duration: f64) -> anyhow::Result<CommandCallback> {
        let mut cache: HashMap<ObjectID, Arc<dyn Animation>> = HashMap::with_capacity(animations.len());

        for x in animations {
            match x.affected_shapes() {
                AffectedShapes::Single(s) => {
                    assert!(!cache.contains_key(s), "UB is not allowed in tests!");
                    cache.insert(*s, x);
                }
                AffectedShapes::Vector(vec) => {
                    for s in vec {
                        assert!(!cache.contains_key(s), "UB is not allowed in tests!");
                        cache.insert(*s, x.clone());
                    }
                }
            }
        }

        let fps = self.state().config.fps;
        let frames: u32 = (duration * fps as f64) as u32;
        let z = 1.0 / fps as f64;
        
        info!("Rendering {frames} frames for {duration} seconds");
        for i in 0..frames {
            let mut ctx: Box<dyn DrawContextImpl> = Box::new(TestContext());
            let t: f64 = (i as f64) * z;
            
            for x in self.state().objects.iter() {
                if let Some(anim) = cache.get(&x.id) {
                    let mut clone = x.shape.read().clone_raw();
                    let mut untyped = DynamicShape {
                        state: &mut clone,
                        id: x.id,
                    };
                    
                    let d: f64 = t / anim.duration();
                    anim.apply(&mut untyped, d);
                    clone.draw(&mut ctx);
                    continue
                }
                x.shape.read().draw(&mut ctx);
            }
        }
        
        Ok(CommandCallback::Ok)
    }

    fn shutdown(&mut self) -> anyhow::Result<CommandCallback> {
        self.state_mut();
        self.state = None;
        Ok(CommandCallback::Ok)
    }

    fn send(&mut self, callback: CommandCallback) {
        self.callback.send(callback).unwrap();
    }
}

impl Renderer for SilentTestRenderer {
    fn new(callback: mpsc::Sender<CommandCallback>) -> Self {
        SilentTestRenderer {
            callback,
            state: None
        }
    }

    fn execute(&mut self, command: Command) {
        FriendlyRenderer::execute(self, command)
    }
}

struct TestContext();

impl DrawContextImpl for TestContext {
    fn offset(&mut self, pos: Vec3) {
        
    }

    fn clear_offset(&mut self) {
        
    }

    fn vertex(&mut self, pos: Vec3, color: Color) {
        
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}