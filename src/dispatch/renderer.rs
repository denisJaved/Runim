use std::sync::{mpsc, Arc};
use std::time::Duration;
use crate::animations::{Animation, SceneState};
use crate::shapes::{AnimationConfig, ObjectDesc};

pub trait Renderer: Send + 'static {
    fn new(callback: mpsc::Sender<CommandCallback>) -> Self;

    fn execute(&mut self, command: Command);
}

pub trait FriendlyRenderer: Renderer + Send + 'static {
    fn begin(&mut self, config: AnimationConfig) -> anyhow::Result<()>;
    fn set_scene(&mut self, objects: Vec<ObjectDesc>, base: SceneState) -> anyhow::Result<()>;
    fn play(&mut self, animations: Vec<Arc<dyn Animation>>, duration: Duration) -> anyhow::Result<()>;
    fn send(&mut self, callback: CommandCallback);

    fn execute(&mut self, command: Command) {
        let res = match command {
            Command::Prepare(config) => {
                self.begin(config)
            }
            Command::SetScene {
                objects,
                base
            } => {
                self.set_scene(objects, base)
            }
            Command::Play {
                animations,
                duration
            } => {
                self.play(animations, duration)
            }
            Command::Wait(duration ) => {
                self.play(vec![], duration)
            }
        };
        self.send(CommandCallback::from_result(res));
    }
}

#[derive(Debug)]
pub enum Command {
    Prepare(AnimationConfig),
    SetScene {
        objects: Vec<ObjectDesc>,
        base: SceneState,
    },
    Play {
        animations: Vec<Arc<dyn Animation>>,
        duration: Duration
    },
    Wait(Duration)
}

#[derive(Debug)]
pub enum CommandCallback {
    FatalError(anyhow::Error),
    Ok
}

impl CommandCallback {
    fn from_result(result: anyhow::Result<()>) -> CommandCallback {
        match result {
            Ok(_) => CommandCallback::Ok,
            Err(e) => CommandCallback::FatalError(e)
        }
    }
}