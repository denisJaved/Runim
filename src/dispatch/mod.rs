mod renderer;

use std::marker::PhantomData;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

use crate::shapes::AnimationConfig;

pub use renderer::{Command, CommandCallback, Renderer, FriendlyRenderer};

/// Dispatcher facilitates interactions between [Renderer] and Runim clients.
/// 
/// [Renderer] instance is created using trait function [Renderer::new],
/// so clients do not need to provide instances of renderers.
/// 
/// # Example
/// ```ignore
/// use runim::prelude::*;
/// use runim::renderer::debug::DebugRenderer;
///
/// // DebugRenderer will be initialized using trait bounds
/// let dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
///
/// // Prepare renderer
/// dispatcher.begin(SceneConfig::p1080());
/// 
/// // ... your animation
/// ```
pub struct RunimDispatcher<R: Renderer> {
    tx: mpsc::Sender<Command>,
    rx: mpsc::Receiver<CommandCallback>,
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
            _thread: handle,
            __marker: PhantomData
        })
    }
    
    /// Prepares [Renderer] for rendering. **Must be called exactly once!**
    /// 
    /// Example:
    /// ```ignore
    /// use runim::prelude::*;
    /// use runim::renderer::debug::DebugRenderer;
    ///
    /// let dispatcher = RunimDispatcher::<DebugRenderer>::new().unwrap();
    /// 
    /// dispatcher.begin(SceneConfig::p1080());
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
            CommandCallback::Ok => {}
        }
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
