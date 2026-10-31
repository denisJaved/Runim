use std::sync::mpsc;
use crate::shapes::AnimationConfig;

pub trait Renderer: Send + 'static {
    fn new(callback: mpsc::Sender<CommandCallback>) -> Self;

    fn execute(&mut self, command: Command);
}

pub trait FriendlyRenderer: Renderer + Send + 'static {
    fn begin(&mut self, config: AnimationConfig) -> anyhow::Result<()>;
    fn send(&mut self, callback: CommandCallback);

    fn execute(&mut self, command: Command) {
        let res = match command {
            Command::Prepare(config) => {
                self.begin(config)
            }
        };
        self.send(CommandCallback::from_result(res));
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Prepare(AnimationConfig)
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