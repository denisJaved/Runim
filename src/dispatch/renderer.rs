use std::sync::{mpsc, Arc};
use std::time::Duration;
use crate::animations::Animation;
use crate::shapes::{AnimationConfig, ObjectDesc};

pub trait Renderer: Send + 'static {
    fn new(callback: mpsc::Sender<CommandCallback>) -> Self;

    fn execute(&mut self, command: Command);
}

pub trait FriendlyRenderer: Renderer + Send + 'static {
    fn begin(&mut self, config: AnimationConfig) -> anyhow::Result<CommandCallback>;
    fn set_scene(&mut self, objects: Vec<ObjectDesc>) -> anyhow::Result<CommandCallback>;
    fn play(&mut self, animations: Vec<Arc<dyn Animation>>, duration: Duration) -> anyhow::Result<CommandCallback>;
    fn shutdown(&mut self) -> anyhow::Result<CommandCallback>;
    fn send(&mut self, callback: CommandCallback);

    fn execute(&mut self, command: Command) {
        let res = match command {
            Command::Prepare(config) => {
                self.begin(config)
            }
            Command::SetScene {
                objects
            } => {
                self.set_scene(objects)
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
            Command::Shutdown => {
                self.shutdown()
            }
        };
        self.send(CommandCallback::from_result(res));
    }
}

#[derive(Debug)]
pub enum Command {
    Prepare(AnimationConfig),
    SetScene {
        objects: Vec<ObjectDesc>
    },
    Play {
        animations: Vec<Arc<dyn Animation>>,
        duration: Duration
    },
    Wait(Duration),
    Shutdown
}

#[derive(Debug)]
pub enum CommandCallback {
    FatalError(anyhow::Error),
    Wait(Duration),
    Ok
}

impl CommandCallback {
    fn from_result(result: anyhow::Result<CommandCallback>) -> CommandCallback {
        match result {
            Ok(v) => v,
            Err(e) => CommandCallback::FatalError(e)
        }
    }
}