use std::sync::{mpsc, Arc};
use std::time::Duration;
use log::info;
use crate::animations::{Animation, SceneState};
use crate::dispatch::{Command, CommandCallback, FriendlyRenderer, Renderer};
use crate::shapes::{AnimationConfig, ObjectDesc};

pub struct DebugRenderer {
    sender: mpsc::Sender<CommandCallback>
}

impl FriendlyRenderer for DebugRenderer {
    fn begin(&mut self, config: AnimationConfig) -> anyhow::Result<()> {
        info!(target: "runim::D->R", "Begin: {config:?}");
        Ok(())
    }

    fn set_scene(&mut self, objects: Vec<ObjectDesc>, base: SceneState) -> anyhow::Result<()> {
        info!(target: "runim::D->R", "SetScene: {objects:?} {base:?}");
        Ok(())
    }

    fn play(&mut self, animations: Vec<Arc<dyn Animation>>, duration: Duration) -> anyhow::Result<()> {
        info!(target: "runim::D->R", "Play: {animations:?} {duration:?}");
        Ok(())
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
