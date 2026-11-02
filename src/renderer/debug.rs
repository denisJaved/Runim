use std::sync::{mpsc, Arc};
use std::time::Duration;
use log::info;
use crate::animations::Animation;
use crate::dispatch::{Command, CommandCallback, FriendlyRenderer, Renderer};
use crate::shapes::{AnimationConfig, ObjectDesc};

pub struct DebugRenderer {
    sender: mpsc::Sender<CommandCallback>
}

impl FriendlyRenderer for DebugRenderer {
    fn begin(&mut self, config: AnimationConfig) -> anyhow::Result<CommandCallback> {
        info!(target: "runim::D->R", "Begin: {config:?}");
        Ok(CommandCallback::Ok)
    }

    fn set_scene(&mut self, objects: Vec<ObjectDesc>) -> anyhow::Result<CommandCallback> {
        info!(target: "runim::D->R", "SetScene: {objects:?}");
        Ok(CommandCallback::Ok)
    }

    fn play(&mut self, animations: Vec<Arc<dyn Animation>>, duration: Duration) -> anyhow::Result<CommandCallback> {
        info!(target: "runim::D->R", "Play: {animations:?} {duration:?}");
        Ok(CommandCallback::Ok)
    }

    fn shutdown(&mut self) -> anyhow::Result<CommandCallback> {
        info!(target: "runim::D->R", "Shutdown");
        Ok(CommandCallback::Ok)
    }

    fn send(&mut self, callback: CommandCallback) {
        info!(target: "runim::R->D", "Callback: {callback:?}");
        self.sender.send(callback).expect("Renderer->Dispatcher MPSC channel disconnected");
    }
}

impl Renderer for DebugRenderer {
    fn new(sender: mpsc::Sender<CommandCallback>) -> Self {
        DebugRenderer {
            sender
        }
    }

    fn execute(&mut self, command: Command) {
        FriendlyRenderer::execute(self, command);
    }
}
