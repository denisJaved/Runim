pub mod debug;
mod draw;

pub use draw::DrawContext;

/*pub struct LivePreviewRenderer {
    sender: mpsc::Sender<CommandCallback>
}

impl FriendlyRenderer for LivePreviewRenderer {
    fn begin(&mut self, config: SceneConfig) -> anyhow::Result<()> {
        info!("Live preview is starting with {config:?}");
        Ok(())
    }

    fn send(&mut self, callback: CommandCallback) {
        self.sender.send(callback).expect("Renderer->Dispatcher MPSC channel disconnected");
    }
}

impl Renderer for LivePreviewRenderer {
    fn new(sender: mpsc::Sender<CommandCallback>) -> Self {
        LivePreviewRenderer {
            sender
        }
    }

    fn execute(&mut self, command: Command) {
        FriendlyRenderer::execute(self, command);
    }
}
*/