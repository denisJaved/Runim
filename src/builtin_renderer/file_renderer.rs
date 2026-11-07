use crate::animations::Animation;
use crate::dispatch::{Command, CommandCallback, FriendlyRenderer, Renderer};
use crate::prelude::AnimationConfig;
use crate::shapes::ObjectDesc;
use anyhow::anyhow;
use ffmpeg_next as fg;
use ffmpeg_next::format::Pixel;
use fg::encoder::video::Encoder;
use fg::software::scaling;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;
use log::{debug, info};
use crate::builtin_renderer::graphics::RunimGraphics;
// TODO: Test entire file. FFMpeg stuff is vibecoded

pub struct FileRenderer {
    callback: mpsc::Sender<CommandCallback>,
    app: Option<FileRendererApp>
}

struct FileRendererApp {
    graphics: RunimGraphics,
    callback: mpsc::Sender<CommandCallback>,
    octx: fg::format::context::Output,
    encoder: Encoder,
    scaler: scaling::Context,
    stream_index: usize,
    frame_index: i64,
    fps: u32,
    path: PathBuf,
    rgba_frame: fg::frame::Video,
    yuv_frame: fg::frame::Video,
    encoder_input: mpsc::Receiver<Vec<u8>>
}

#[derive(Debug)]
pub struct VideoTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
    pub config: AnimationConfig,
    pub padded_row: u32,
    pub readback: wgpu::Buffer,
    pub encoder: mpsc::Sender<Vec<u8>>
}

// SAFETY: FileRendererApp is only constructed after Command::Prepare is received
//         by FileRenderer, which is only possible after FileRenderer was moved to
//         the Runim renderer thread. After that, it won't be ever moved rendering
//         moves of FileRendererApp impossible, so this Send isn't actually needed.
unsafe impl Send for FileRendererApp {}

impl FileRendererApp {
    fn encode_rgba(&mut self, width: u32, height: u32, rgba: &[u8]) -> anyhow::Result<()> {
        // copy into RGBA frame plane 0 (respect its internal stride)
        let stride = self.rgba_frame.stride(0);
        let dst = self.rgba_frame.data_mut(0);
        for y in 0..height as usize {
            let src_off = y * (width * 4) as usize;
            let dst_off = y * stride;
            dst[dst_off..dst_off + (width * 4) as usize]
                .copy_from_slice(&rgba[src_off..src_off + (width * 4) as usize]);
        }

        self.scaler.run(&self.rgba_frame, &mut self.yuv_frame)?;

        let ts = self.frame_index;
        self.frame_index += 1;
        self.yuv_frame.set_pts(Some(ts));

        self.encoder.send_frame(&self.yuv_frame)?;
        self.drain_packets()?;
        Ok(())
    }

    fn drain_packets(&mut self) -> anyhow::Result<()> {
        let dst_tb = self.octx.stream(self.stream_index).unwrap().time_base();
        let src_tb = fg::Rational::new(1, self.fps as i32);

        loop {
            let mut packet = fg::Packet::empty();
            match self.encoder.receive_packet(&mut packet) {
                Ok(()) => {
                    packet.set_stream(self.stream_index);
                    packet.rescale_ts(src_tb, dst_tb);

                    // `write_interleaved` consumes the packet; the fresh
                    // allocation next iteration avoids any reuse hazard.
                    packet.write_interleaved(&mut self.octx)?;
                    //packet.write(&mut self.octx)?;   // av_write_frame, no reorder buffer
                }
                Err(fg::Error::Eof) => break,
                Err(fg::Error::Other { errno }) if errno == fg::error::EAGAIN => break,
                Err(e) => anyhow::bail!("receive_packet failed: {e}"),
            }
        }
        Ok(())
    }
}

impl FriendlyRenderer for FileRendererApp {
    fn begin(&mut self, config: AnimationConfig) -> anyhow::Result<CommandCallback> {
        // NO OP
        Ok(CommandCallback::Ok)
    }

    fn set_scene(&mut self, objects: Vec<ObjectDesc>) -> anyhow::Result<CommandCallback> {
        self.graphics.set_scene(objects);
        Ok(CommandCallback::Ok)
    }

    fn play(&mut self, animations: Vec<Arc<dyn Animation>>, duration: f64) -> anyhow::Result<CommandCallback> {
        self.graphics.set_animations(animations);

        let frames = (duration * self.fps as f64).round() as u64;
        let dt = Duration::from_secs_f64(1.0 / self.fps as f64);
        for i in 0..frames {
            debug!("Rendering frame {i}/{frames}");
            self.graphics.render(dt)?;

            let rgba = self.encoder_input.recv()?;
            self.encode_rgba(
                self.graphics.anim_config.width,
                self.graphics.anim_config.height,
                rgba.as_slice()
            )?;
        }

        Ok(CommandCallback::Ok)
    }

    fn shutdown(&mut self) -> anyhow::Result<CommandCallback> {
        self.encoder.send_eof()?;
        self.drain_packets()?;
        self.octx.write_trailer()?;

        Ok(CommandCallback::Ignore)
    }

    fn send(&mut self, callback: CommandCallback) {
        self.callback.send(callback).unwrap()
    }
}

impl Renderer for FileRenderer {
    fn new(callback: mpsc::Sender<CommandCallback>) -> Self {
        FileRenderer {
            callback,
            app: None
        }
    }

    fn execute(&mut self, command: Command) {
        match &command {
            Command::Prepare(config) => {
                match self.prepare(config) {
                    Ok(CommandCallback::FatalError(e)) | Err(e) => {
                        println!("{:?}", e);
                        self.callback.send(CommandCallback::FatalError(e)).unwrap();
                        return;
                    }
                    _ => {}
                }
            }
            Command::Shutdown => {
                self.app.as_mut().unwrap().shutdown().unwrap();
                self.app = None;
                self.callback.send(CommandCallback::Ok).unwrap();
                return;
            }
            _ => {}
        }
        FriendlyRenderer::execute(self.app.as_mut().unwrap(), command)
    }
}
impl FileRenderer {
    fn prepare(&mut self, config: &AnimationConfig) -> anyhow::Result<CommandCallback> {
        fg::init()?;
        let path = PathBuf::from("output.mkv");
        let mut octx = fg::format::output(&path)?;

        let codec = fg::codec::encoder::find(fg::codec::Id::H264)
            .ok_or_else(|| anyhow!("H.264 Encoder is not available"))?;
        let mut encoder = fg::codec::context::Context::new_with_codec(codec)
            .encoder()
            .video()?;

        encoder.set_width(config.width);
        encoder.set_height(config.height);
        encoder.set_format(Pixel::YUV420P);
        encoder.set_time_base(fg::Rational::new(1, config.fps as i32));
        encoder.set_frame_rate(Some(fg::Rational::new(config.fps as i32, 1)));
        encoder.set_bit_rate(8_000_000);
        encoder.set_gop(config.fps * 2); // keyframe every ~2s

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        // mp4/mov/mkv want SPS/PPS in extradata (global header);
        // mpegts & co. repeat in-band and don't need it.
        let global_header = !matches!(ext, "ts" | "m2ts" | "mts" | "mpeg" | "mpg");

        let mut opts = fg::Dictionary::new();
        if global_header {
            opts.set("flags", "+global_header");
        }
        let encoder = encoder.open_as_with(codec, opts)?;
        let stream_index = {
            let mut stream = octx.add_stream(codec)?;
            stream.set_parameters(&encoder);                // copies extradata → stream->codecpar
            stream.set_time_base(fg::Rational::new(1, config.fps as i32));
            stream.index()
        };

        octx.write_header()?;

        let scaler = scaling::Context::get(
            Pixel::RGBA, config.width, config.height,
            Pixel::YUV420P, config.width, config.height,
            scaling::Flags::BILINEAR
        )?;

        let rgba_frame = fg::frame::Video::new(Pixel::RGBA, config.width, config.height);
        let yuv_frame = fg::frame::Video::new(Pixel::YUV420P, config.width, config.height);

        let (tx, rx) = mpsc::channel();
        let graphics = pollster::block_on(RunimGraphics::new_video(config.clone(), tx))?;

        self.app = Some(FileRendererApp {
            graphics, callback: self.callback.clone(),
            octx, encoder, scaler, stream_index, frame_index: 0,
            fps: config.fps, path: path.to_owned(), rgba_frame, yuv_frame,
            encoder_input: rx,
        });

        Ok(CommandCallback::Ok)
    }
}
impl Renderer for FileRendererApp {
    fn new(callback: mpsc::Sender<CommandCallback>) -> Self {
        unreachable!()
    }

    fn execute(&mut self, command: Command) {
        FriendlyRenderer::execute(self, command)
    }
}