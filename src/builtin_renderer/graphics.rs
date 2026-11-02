use std::any::Any;
use std::sync::Arc;
use anyhow::bail;
use glam::Vec3;
use log::{info, warn};
use wgpu::util::DeviceExt;
use winit::window::Window;
use crate::animations::{ObjectVisual, SceneState};
use crate::builtin_renderer::vertex;
use crate::builtin_renderer::vertex::Vertex;
use super::texture::Texture;
use crate::dbgn;
use crate::renderer::DrawContextImpl;
use crate::shapes::{ObjectDesc, Transform2D};

pub struct RunimState {
    objects: Vec<ObjectDesc>,
    base: SceneState
}

pub struct RunimGraphics {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: Texture,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    fps: u32,
    state: RunimState
}

impl RunimGraphics {
    // TODO: Remove window from RunimGraphics.
    //       This type should not care about its render surface
    pub async fn new(window: Arc<Window>, fps: u32) -> anyhow::Result<RunimGraphics> {
        info!("Initializing runim renderer");
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None
        });

        let surface = instance.create_surface(window.clone())?;
        let adapter = instance.
            request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: dbgn!("WGPU Device").name(),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let present_format = surface_caps
            .present_modes
            .iter()
            .find(|f| **f == wgpu::PresentMode::Fifo)
            .unwrap_or(&surface_caps.present_modes[0]);

        {
            let info = device.adapter_info();
            info!("Found applicable device {} using {surface_format:?}/{present_format:?} format", info.name);
        }

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width,
            height: size.height,
            present_mode: *present_format,
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        let depth_texture = Texture::create_depth_texture(
            &device, &config, dbgn!("Depth Texture").name()
        );
        let _texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: dbgn!("Texture Bind Group Layout").name(),
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: dbgn!("Shader").name(),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: dbgn!("Render Pipeline Layout").name(),
                bind_group_layouts: &[
                    //Some(&texture_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: dbgn!("Vertex Buffer").name(),
                contents: bytemuck::cast_slice(&[
                    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
                    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
                    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
                ]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&[0u16, 1u16, 2u16]),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: dbgn!("Render Pipeline").name(),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(vertex::Vertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        info!("Runim renderer is ready!");

        Ok(RunimGraphics {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
            render_pipeline,
            depth_texture,
            vertex_buffer,
            index_buffer,
            fps,
            state: RunimState {
                objects: vec![],
                base: SceneState::with_capacity(0),
            }
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;

            self.depth_texture = Texture::create_depth_texture(
                &self.device, &self.config, dbgn!("depth_texture").name()
            );

            //self.projection.resize(width, height);
            /*self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );*/
        }
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        use wgpu::CurrentSurfaceTexture::*;
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            Success(t) => t,
            Suboptimal(t) => t,
            Timeout | Occluded | Validation => {
                return Ok(());
            }
            Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            Lost => {
                bail!("Lost graphical device")
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: dbgn!("Render Encoder").name(),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: dbgn!("Render Pass").name(),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });

        // render_pass.set_pipeline(&self.render_pipeline);
        // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        // render_pass.draw_indexed(0..4, 0, 0..1);

        let mut ctx: Box<dyn DrawContextImpl> = Box::new(RunimDrawContext::new(Transform2D::default()));

        for obj in self.state.objects.iter() {
            let visual = match self.state.base.get(obj.id) {
                Some(v) => v,
                None => {
                    warn!("Object without visual data");
                    continue
                }
            };
            let visual_guard = visual.read();

            let x = ctx.as_any_mut().downcast_mut::<RunimDrawContext>()
                .unwrap();
            x.transform2d.pos.x = visual_guard.transform.pos.x;
            x.transform2d.pos.y = visual_guard.transform.pos.y;
            x.vertices.clear();
            drop(visual_guard);

            let guard = obj.shape.read();
            guard.draw(&mut ctx);

            let x = ctx.as_any_mut().downcast_mut::<RunimDrawContext>()
                .unwrap();
            for i in 0..x.vertices.len() {
                if i % 3 != 0 { continue; }

                let v0: &Vertex = x.vertices.get(i).unwrap();
                let v1: &Vertex = x.vertices.get(i + 1).unwrap();
                let v2: &Vertex = x.vertices.get(i + 2).unwrap();
                self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&[
                    v0.clone(), v1.clone(), v2.clone()
                ]));

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..4, 0, 0..1);
            }
        }

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }


    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn set_scene(&mut self, objects: Vec<ObjectDesc>, base: SceneState) {
        self.state.objects = objects;
        self.state.base = base;
    }
}

#[derive(Debug)]
struct RunimDrawContext {
    transform2d: Transform2D,
    vertices: Vec<Vertex>
}

impl RunimDrawContext {
    pub fn new(transform: Transform2D) -> RunimDrawContext {
        RunimDrawContext {
            transform2d: transform,
            vertices: vec![],
        }
    }
}

impl DrawContextImpl for RunimDrawContext {
    fn vertex(&mut self, mut pos: Vec3, color: Vec3) {
        pos.x += self.transform2d.pos.x;
        pos.y += self.transform2d.pos.y;
        self.vertices.push(Vertex {
            position: pos.to_array(),
            color: color.to_array()
        });
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}