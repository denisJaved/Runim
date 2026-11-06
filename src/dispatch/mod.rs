mod renderer;

use runim::shapes::{AnimationConfig, ObjectID, Scene};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::sync::{Arc, mpsc};
use std::thread;
use std::thread::{sleep, JoinHandle};
use log::info;
use thiserror::Error;

pub use renderer::{Command, CommandCallback, Renderer, FriendlyRenderer};
use runim::animations::Animation;
use runim::IntoDuration;
use crate::animations::AffectedShapes;
use crate::shapes::SceneID;

/// Dispatcher facilitates interactions between [Renderer] and Runim clients.
/// 
/// [Renderer] instance is created using trait function [Renderer::new],
/// so clients do not need to provide instances of renderers.
/// 
/// # Example
/// ```
/// use runim::prelude::*;
/// use runim::renderer::debug::DebugRenderer;
///
/// // DebugRenderer will be initialized using trait bounds
/// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
///
/// // Prepare renderer
/// dispatcher.begin(AnimationConfig::p1080());
/// 
/// // ... your animation
/// ```
pub struct RunimDispatcher<R: Renderer> {
    tx: mpsc::Sender<Command>,
    rx: mpsc::Receiver<CommandCallback>,

    scenes: HashMap<SceneID, Scene>,
    active_scene: Option<SceneID>,

    _thread: JoinHandle<()>,
    __marker: PhantomData<R>
}

impl<R: Renderer> RunimDispatcher<R> {
    /// Constructs new dispatcher.
    /// 
    /// See [RunimDispatcher] for more info.
    pub fn new() -> anyhow::Result<RunimDispatcher<R>> {
        // Dispatcher -> Renderer channel
        let (tx, rx0) = mpsc::channel();

        // Renderer -> Dispatcher channel
        let (tx0, rx) = mpsc::channel();

        let mut renderer = R::new(tx0);
        
        let handle = thread::Builder::new()
            .name(String::from("runim-renderer"))
            .spawn(move || {run_renderer(&mut renderer, rx0)})?;

        Ok(RunimDispatcher::<R> {
            tx,
            rx,
            scenes: HashMap::new(),
            active_scene: None,
            _thread: handle,
            __marker: PhantomData
        })
    }

    pub fn new_scene(&mut self) -> SceneID {
        let scene = Scene::new();
        let id = scene.id;
        self.scenes.insert(scene.id, scene);
        id
    }

    pub fn add_scene(&mut self, scene: Scene) -> SceneID {
        let id = scene.id;
        self.scenes.insert(scene.id, scene);
        id
    }
    
    /// Returns **mutable** reference to a [Scene] by its id.
    /// 
    /// This function returns mutable references by default,
    /// because mutating objects in the scene is more common
    /// than reading them. For immutable references use [RunimDispatcher::scene_imm]
    /// 
    /// # Example
    /// ```
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    /// use runim::shapes::ObjectID;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    ///
    /// let mut scene: SceneID = dispatcher.new_scene();
    /// dispatcher.scene(&scene)
    ///     .add_rectangle()
    ///     .set_pos(14.0, 14.0);
    /// ```
    pub fn scene(&mut self, id: &SceneID) -> &mut Scene {
        self.scenes.get_mut(id).expect("unknown scene")
    }
    
    pub fn scene_imm(&self, id: &SceneID) -> &Scene {
        self.scenes.get(id).expect("unknown scene")
    }
    
    pub fn set_scene(&mut self, id: SceneID) -> anyhow::Result<()> {
        let scene = self.scenes.get(&id).expect("Unknown scene");
        self.send(Command::SetScene {
            objects: scene.objects.clone()
        });
        self.active_scene = Some(id);
        Ok(())
    }
    
    pub fn wait<D: IntoDuration>(&mut self, duration: D) -> anyhow::Result<()> {
        let duration = duration.as_secs_f64();
        let active = self.active_scene.ok_or(DispatchError::NoActiveScene)?;
        self.send(Command::Wait(duration));
        self.scenes.get_mut(&active).unwrap().clock += duration;
        Ok(())
    }
    
    pub fn play<A: IntoAnimations>(&mut self, anims: A) -> anyhow::Result<()> {
        let anims = anims.into_animations();

        #[cfg(debug_assertions)]
        {
            // Check that no animations affect same shape.
            // Animation conflicts are UB.
            assert!(anims.len() > 0);
            let mut shapes = HashSet::<ObjectID>::new();
            for x in anims.iter() {
                match x.affected_shapes() {
                    AffectedShapes::Single(id) => {
                        assert!(
                            !shapes.contains(id),
                            "multiple animations refer to {id}. This is UB; use animation compositor instead"
                        );
                        shapes.insert(*id);
                    }
                    AffectedShapes::Vector(vec) => {
                        for id in vec {
                            assert!(
                                !shapes.contains(id),
                                "multiple animations refer to {id}. This is UB; use animation compositor instead"
                            );
                            shapes.insert(*id);
                        }
                    }
                }
            }
        }

        let duration = anims.iter().map(|a| a.duration()).fold(0.0, f64::max);
        
        let active = self.active_scene.ok_or(DispatchError::NoActiveScene)?;
        self.send(Command::Play {
            animations: anims.clone(),
            duration
        });
        let scene = self.scenes.get_mut(&active).unwrap();
        for anim in &anims {
            anim.affected_shapes().apply_all(scene, anim, 1.0);
        }
        scene.clock += duration;
        Ok(())
    }
    
    /// Prepares [Renderer] for rendering. **Must be called exactly once!**
    /// 
    /// Example:
    /// ```
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let mut dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// 
    /// dispatcher.begin(AnimationConfig::p1080());
    /// ```
    pub fn begin(&mut self, config: AnimationConfig) {
        self.send(Command::Prepare(config));
    }

    #[inline(always)]
    fn send(&mut self, command: Command) {
        self.tx.send(command).expect("Dispatcher->Renderer MPSC channel disconnected");
        match self.rx.recv().expect("Renderer->Dispatcher MPSC channel disconnected") {
            CommandCallback::FatalError(e) => {
                panic!("{e}");
            }
            CommandCallback::Wait(d) => {
                sleep(d)
            }
            CommandCallback::Ok => {}
        }
    }
}

impl<R: Renderer> Drop for RunimDispatcher<R> {
    fn drop(&mut self) {
        info!("Destroying renderer, because dispatcher was dropped");
        self.send(Command::Shutdown);
    }
}

fn run_renderer<R: Renderer>(
    renderer: &mut R,
    rx: mpsc::Receiver<Command>
) {
    while let Ok(cmd) = rx.recv() {
        renderer.execute(cmd)
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DispatchError {
    NoActiveScene,
    UnknownScene
}

impl Display for DispatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

pub trait IntoAnimations {
    fn into_animations(self) -> Vec<Arc<dyn Animation>>;
}

impl IntoAnimations for Arc<dyn Animation> {
    fn into_animations(self) -> Vec<Arc<dyn Animation>> {
        vec![self]
    }
}

impl<A: Animation> IntoAnimations for A {
    fn into_animations(self) -> Vec<Arc<dyn Animation>> {
        vec![Arc::new(self)]
    }
}

impl<A: Animation, B: Animation> IntoAnimations for (A, B) {
    fn into_animations(self) -> Vec<Arc<dyn Animation>> {
        vec![Arc::new(self.0), Arc::new(self.1)]
    }
}

impl<A: Animation, B: Animation, C: Animation> IntoAnimations for (A, B, C) {
    fn into_animations(self) -> Vec<Arc<dyn Animation>> {
        vec![Arc::new(self.0), Arc::new(self.1), Arc::new(self.2)]
    }
}

impl IntoAnimations for Vec<Arc<dyn Animation>> {
    fn into_animations(self) -> Vec<Arc<dyn Animation>> {
        self
    }
}