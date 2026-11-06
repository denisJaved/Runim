//! Adapters module provide implementations of [Renderer]

use std::sync::{mpsc, Arc};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use log::warn;
use pollster::block_on;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use runim::animations::Animation;
use runim::builtin_renderer::graphics::RunimGraphics;
use runim::dispatch::{Command, CommandCallback, FriendlyRenderer, Renderer};
use runim::prelude::AnimationConfig;
use runim::shapes::ObjectDesc;

/// [Renderer] that display output live in a window.
pub struct LivePreviewRenderer {
    proxy: EventLoopProxy<Command>,
    _handle: Option<JoinHandle<()>>
}

impl LivePreviewRenderer {
    fn new(callback: Sender<CommandCallback>) -> anyhow::Result<LivePreviewRenderer> {
        let (tx, rx) = mpsc::channel();
        let thread = thread::Builder::new()
            .name(String::from("runim-live-preview"))
            .spawn(move || {
                // TODO: Handle Result properly for all below
                let mut event_loop =
                    EventLoop::<Command>::with_user_event();

                #[cfg(target_os = "linux")]
                {
                    use winit::platform::wayland::EventLoopBuilderExtWayland;
                    event_loop.with_any_thread(true);
                }
                #[cfg(target_os = "linux")]
                {
                    use winit::platform::x11::EventLoopBuilderExtX11;
                    event_loop.with_any_thread(true);
                }
                #[cfg(target_os = "windows")]
                {
                    use winit::platform::windows::EventLoopBuilderExtWindows;
                    event_loop.with_any_thread(true);
                }
                #[cfg(target_os = "macos")]
                compile_error!("Runim builtin renderer does not support MacOS!");

                let event_loop = event_loop.build().unwrap();
                tx.send(event_loop.create_proxy()).unwrap();
                drop(tx);

                let mut app = LiveRendererApp {
                    window: None,
                    graphics: None,
                    last_render: Instant::now(),
                    callback
                };
                event_loop.run_app(&mut app).unwrap();
            })?;

        Ok(LivePreviewRenderer {
            proxy: rx.recv()?,
            _handle: Some(thread),
        })
    }
}

impl Renderer for LivePreviewRenderer {
    fn new(callback: Sender<CommandCallback>) -> Self {
        Self::new(callback).unwrap()
        // TODO: Proper error handling
    }

    fn execute(&mut self, command: Command) {
        self.proxy.send_event(command).unwrap()
        // TODO: Proper error handling
    }
}

struct LiveRendererApp {
    window: Option<Arc<Window>>,
    graphics: Option<RunimGraphics>,
    last_render: Instant,
    callback: Sender<CommandCallback>
}

impl Renderer for LiveRendererApp {
    fn new(_callback: Sender<CommandCallback>) -> Self {
        unreachable!() // LiveRendererApp created internally in LivePreviewRenderer
    }

    fn execute(&mut self, command: Command) {
        FriendlyRenderer::execute(self, command)
    }
}

impl FriendlyRenderer for LiveRendererApp {
    fn begin(&mut self, config: AnimationConfig) -> anyhow::Result<CommandCallback> {
        let window = self.window.clone().unwrap();
        let mut graphics = block_on(RunimGraphics::new(
            window,
            config.fps
        ))?;
        graphics.resize(config.width, config.height);
        self.graphics = Some(graphics);
        Ok(CommandCallback::Ok)
    }

    fn set_scene(&mut self, objects: Vec<ObjectDesc>) -> anyhow::Result<CommandCallback> {
        self.graphics.as_mut().unwrap().set_scene(objects);
        Ok(CommandCallback::Ok)
    }

    fn play(&mut self, animations: Vec<Arc<dyn Animation>>, duration: f64) -> anyhow::Result<CommandCallback> {
        //sleep(duration);
        self.graphics.as_mut().unwrap().set_animations(animations);
        Ok(CommandCallback::Wait(Duration::from_secs_f64(duration)))
    }

    fn shutdown(&mut self) -> anyhow::Result<CommandCallback> {
        self.window = None;
        self.graphics = None;
        Ok(CommandCallback::Ok)
    }

    fn send(&mut self, callback: CommandCallback) {
        self.callback.send(callback).unwrap();
    }
}

impl ApplicationHandler<Command> for LiveRendererApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // NO OP. Window is created during FriendlyRenderer::begin
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, command: Command) {
        match &command {
            Command::Prepare(config) => {
                let mut attr = Window::default_attributes();
                attr.title = String::from("Runium live preview");
                attr.resizable = false;
                attr.inner_size = Some(Size::Physical(PhysicalSize {
                    width: config.width,
                    height: config.height
                }));

                let window = event_loop.create_window(attr).unwrap();  // TODO: Handle properly
                self.window = Some(Arc::new(window));
            }
            Command::Shutdown => {
                event_loop.exit();
            }
            _ => {}
        }
        FriendlyRenderer::execute(self, command)
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        use WindowEvent::*;
        let state = match &mut self.graphics {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            RedrawRequested => {
                let now = Instant::now();
                let dt = now - self.last_render;
                self.last_render = now;
                state.render(dt).unwrap(); // TODO: Proper error handling
            }
            Resized(_) => {
                warn!("Runim live renderer window can't be resized");
            }
            CloseRequested => {
                // TODO
            }
            _ => {}
        }
    }
}