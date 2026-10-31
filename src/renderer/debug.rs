use std::sync::mpsc;
use log::info;
use crate::dispatch::{Command, CommandCallback, FriendlyRenderer, Renderer};
use crate::shapes::AnimationConfig;

pub struct DebugRenderer {
    sender: mpsc::Sender<CommandCallback>
}

impl FriendlyRenderer for DebugRenderer {
    fn begin(&mut self, config: AnimationConfig) -> anyhow::Result<()> {
        info!(target: "runim::R->D", "Begin: {config:?}");
        Ok(())
    }

    fn send(&mut self, callback: CommandCallback) {
        info!(target: "runim::D->R", "Callback: {callback:?}");
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
