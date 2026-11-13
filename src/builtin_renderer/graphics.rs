use std::any::Any;
use std::collections::HashMap;
use std::mem;
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::time::Duration;
use anyhow::bail;
use glam::{Vec2, Vec3};
use image::DynamicImage;
use log::{info, warn};
use wgpu::util::DeviceExt;
use wgpu::wgt;
use wgpu::CurrentSurfaceTexture::{Lost, Occluded, Outdated, Timeout, Validation};
use winit::window::Window;
use runim::animations::{AffectedShapes, Animation, DynamicShape};
use runim::builtin_renderer::camera::{CameraUniform, Projection};
use runim::builtin_renderer::vertex::Vertex;
use super::texture::{Texture, TextureHandle};
use runim::dbgn;
use runim::renderer::DrawContext;
use runim::shapes::{Color, ObjectDesc, ObjectID};
use crate::builtin_renderer::live_preview::WindowTarget;
use crate::builtin_renderer::{Frame, FrameInner, OutputTarget};
use crate::builtin_renderer::file_renderer::VideoTarget;
use crate::builtin_renderer::vertex::TexVertex;
use crate::renderer::ImageHandle;
use crate::shapes::AnimationConfig;

pub struct RunimState {
    objects: Vec<ObjectDesc>,
    animation_cache: HashMap<ObjectID, AnimationState>
}

#[derive(Clone, Debug)]
pub struct AnimationState {
    animation: Arc<dyn Animation>,
    passed: f64
}

impl AnimationState {
    pub fn new(arc: Arc<dyn Animation>) -> AnimationState {
        AnimationState {
            animation: arc,
            passed: 0.0,
        }
    }
}

pub struct ResizableBuffer {
    buffer: wgpu::Buffer,
    last_size: u32,
    usage: wgt::BufferUsages
}

impl ResizableBuffer {
    pub fn write<T: bytemuck::Pod>(&mut self, device: &mut wgpu::Device, queue: &mut wgpu::Queue, values: &Vec<T>) {
        let i = values.len() as u32;
        if i == self.last_size {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(values.as_slice()));
            return;
        }
        self.buffer.destroy();
        self.buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: dbgn!("Vertex Buffer").name(),
                contents: bytemuck::cast_slice(values.as_slice()),
                usage: self.usage,
            }
        );
        self.last_size = i;
    }
}

pub struct RunimGraphics {
    // window: Arc<Window>,
    // surface: wgpu::Surface<'static>,
    pub(crate) target: OutputTarget,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    //config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    render_pipeline: wgpu::RenderPipeline,
    tex_render_pipeline: wgpu::RenderPipeline,
    depth_texture: Texture,
    vertex_buffer: ResizableBuffer,
    tex_vertex_buffer: ResizableBuffer,
    index_buffer: wgpu::Buffer,
    pub(crate) anim_config: AnimationConfig,
    state: RunimState,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    projection: Projection,
    camera_uniform: CameraUniform
}

impl RunimGraphics {
    // TODO: Remove window from RunimGraphics.
    //       This type should not care about its render surface

    fn new_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None
        })
    }

    async fn request_device(adapter: &wgpu::Adapter) -> anyhow::Result<(wgpu::Device, wgpu::Queue)> {
        Ok(adapter.request_device(&wgpu::DeviceDescriptor {
                label: dbgn!("WGPU Device").name(),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
        }).await?)
    }

    pub async fn new_windowed(window: Arc<Window>, fps: u32) -> anyhow::Result<RunimGraphics> {
        let instance = Self::new_instance();
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance.
            request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false
            })
            .await?;

        let (device, queue) = Self::request_device(&adapter).await?;

        let size = window.inner_size();
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
        surface.configure(&device, &config);

        {
            let info = device.adapter_info();
            info!("Found applicable device {} using {surface_format:?}/{present_format:?} format for windowed graphics", info.name);
        }

        let target = OutputTarget::Window(WindowTarget {
            window,
            surface,
            config,
            //configured: false,
        });

        Self::new(adapter, device, queue, target, AnimationConfig {
            width: size.width,
            height: size.height,
            fps
        }).await
    }

    pub async fn new_video(anim_config: AnimationConfig, tx: mpsc::Sender<Vec<u8>>) -> anyhow::Result<RunimGraphics> {
        let instance = Self::new_instance();

        // TODO: Maybe move instance.request_adapter to Self::request_device?

        let adapter = instance.
            request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false
            })
            .await?;

        let (device, queue) = Self::request_device(&adapter).await?;

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: dbgn!("Video Frame").name(),
            size: wgpu::Extent3d {
                width: anim_config.width,
                height: anim_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[]
        });
        let view = texture.create_view(&Default::default());

        let unpadded = anim_config.width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row = (unpadded + align - 1) / align * align;

        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: dbgn!("Video Readback").name(),
            size: (padded_row * anim_config.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let target = OutputTarget::Video(VideoTarget {
            texture,
            view,
            format,
            config: anim_config.clone(),
            padded_row,
            readback,
            encoder: tx,
        });

        Self::new(adapter, device, queue, target, anim_config).await
    }

    async fn new(
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        target: OutputTarget,
        anim_config: AnimationConfig
    ) -> anyhow::Result<RunimGraphics> {
        info!("Initializing runim renderer");

        let projection = Projection::new(anim_config.width, anim_config.height);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_proj(&projection);

        let depth_texture = Texture::create_depth_texture(
            &device, anim_config.width, anim_config.height, dbgn!("Depth Texture").name()
        );
        let texture_bind_group_layout =
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
        let tex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: dbgn!("Textured Shader").name(),
            source: wgpu::ShaderSource::Wgsl(include_str!("tex_shader.wgsl").into()),
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
        let vertex_buffer = ResizableBuffer {
            buffer: vertex_buffer,
            last_size: 3,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        };
        let tex_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: dbgn!("Textured Vertex Buffer").name(),
                contents: bytemuck::cast_slice::<TexVertex, u8>(&[]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        let tex_vertex_buffer = ResizableBuffer {
            buffer: tex_vertex_buffer,
            last_size: 3,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        };
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: dbgn!("Index Buffer").name(),
                contents: bytemuck::cast_slice(&[0u16, 1u16, 2u16]),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: dbgn!("Camera Buffer").name(),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: dbgn!("camera_bind_group_layout").name(),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: dbgn!("camera_bind_group").name(),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: dbgn!("Render Pipeline Layout").name(),
                bind_group_layouts: &[
                    Some(&camera_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let format = match &target {
            OutputTarget::Window(w) => w.config.format,
            OutputTarget::Video(v) => v.format
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: dbgn!("Render Pipeline").name(),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
        let tex_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: dbgn!("Render Pipeline Layout").name(),
                bind_group_layouts: &[
                    Some(&camera_bind_group_layout),
                    Some(&texture_bind_group_layout)
                ],
                immediate_size: 0,
            });
        let tex_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: dbgn!("Textured Render Pipeline").name(),
            layout: Some(&tex_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &tex_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(TexVertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &tex_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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
            target,
            device,
            queue,
            is_surface_configured: false,
            render_pipeline,
            tex_render_pipeline,
            depth_texture,
            vertex_buffer,
            tex_vertex_buffer,
            index_buffer,
            anim_config,
            state: RunimState {
                objects: vec![],
                animation_cache: HashMap::new()
            },
            camera_buffer,
            camera_bind_group,
            texture_bind_group_layout,
            projection,
            camera_uniform
        })
    }

    // pub fn resize(&mut self, width: u32, height: u32) {
    //     if width > 0 && height > 0 {
    //         self.config.width = width;
    //         self.config.height = height;
    //         self.surface.configure(&self.device, &self.config);
    //         self.is_surface_configured = true;
    //
    //         self.depth_texture = Texture::create_depth_texture(
    //             &self.device, &self.config, dbgn!("depth_texture").name()
    //         );
    //
    //         //self.projection.resize(width, height);
    //         /*self.queue.write_buffer(
    //             &self.camera_buffer,
    //             0,
    //             bytemuck::cast_slice(&[self.camera_uniform]),
    //         );*/
    //     }
    // }

    pub fn render(&mut self, dt: Duration) -> anyhow::Result<()> {
        use wgpu::CurrentSurfaceTexture::*;

        let frame = match &self.target {
            OutputTarget::Window(w) => {
                w.window.request_redraw();
                // if !w.configured {
                //     return Ok(());
                // }
                match w.surface.get_current_texture() {
                    Success(tex) | Suboptimal(tex) => {
                        let view = tex.texture.create_view(&Default::default());
                        Frame { view, inner: FrameInner::Surface(tex) }
                    }
                    Timeout | Occluded | Validation => {
                        return Ok(());
                    }
                    Outdated => {
                        //w.surface.configure(&self.device, &self.config);
                        warn!("Windowed surface is outdated...");
                        return Ok(());
                    }
                    Lost => {
                        bail!("Lost graphical device")
                    }
                }
            }
            OutputTarget::Video(v) => {
                Frame {
                    view: v.view.clone(),
                    inner: FrameInner::Offscreen
                }
            }
        };

        let mut ctx: RunimDrawContext = RunimDrawContext::new(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout
        );
        let seconds = dt.as_secs_f64();

        for obj in self.state.objects.iter() {
            let mut guard = obj.shape.write();

            if let Some(anim) = self.state.animation_cache.get_mut(&obj.id) {
                let mut shape = guard.clone_raw();
                let mut dynamic = DynamicShape {
                    state: &mut shape,
                    id: obj.id,
                };
                anim.passed += seconds;
                let total = anim.animation.duration();
                anim.animation.apply(&mut dynamic, (anim.passed / total).min(1.0));
                shape.draw(&mut ctx, guard.extract_state_mut());
                continue;
            }
            guard.draw(&mut ctx, None);
        }
        let (vertices, mut tex_vertices) = ctx.unpack();

        let mut submit = false;
        let is_empty = vertices.is_empty();
        if !is_empty {
            self.vertex_buffer.write(&mut self.device, &mut self.queue, &vertices);
            submit = true;
        }
        let is_tex_empty = tex_vertices.is_empty();
        if !is_tex_empty {
            let mut ordered: Vec<TexVertex> = Vec::new();
            let mut count = 0u32;
            for (_, (begin, _, c)) in tex_vertices.iter_mut() {
                let _ = mem::replace(begin, count);
                count += c.len() as u32;
                for x in c {
                    ordered.push(x.clone()); // TODO: Get rid of the cloning somehow
                }
            }
            self.tex_vertex_buffer.write(&mut self.device, &mut self.queue, &ordered);
            submit = true;
        }
        if submit {
            self.queue.submit([]);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: dbgn!("Render Encoder").name(),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: dbgn!("Render Pass").name(),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &frame.view,
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

        if !is_empty {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
            /*render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..4, 0, 0..1);*/
            render_pass.draw(0..self.vertex_buffer.last_size, 0..1);
        }

        if !is_tex_empty {
            for (_, (begin, tex, v)) in tex_vertices {
                let len = v.len() as u32;
                const SIZE: wgpu::BufferAddress = size_of::<TexVertex>() as wgpu::BufferAddress;

                render_pass.set_pipeline(&self.tex_render_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &tex.texture.bind, &[]);
                render_pass.set_vertex_buffer(0, self.tex_vertex_buffer.buffer
                    .slice((begin as wgpu::BufferAddress * SIZE)..((begin + len) as wgpu::BufferAddress * SIZE)));
                /*render_pass.set_vertex_buffer(0, self.tex_vertex_buffer.buffer
                    .slice(..));*/
                render_pass.draw(0..len, 0..1);
            }
        }

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));

        match (&self.target, frame.inner) {
            (OutputTarget::Window(_), FrameInner::Surface(tex)) => {
                self.queue.present(tex);
            }
            (OutputTarget::Video(v), FrameInner::Offscreen) => {
                let mut enc = self.device.create_command_encoder(&Default::default());
                enc.copy_texture_to_buffer(
                    wgpu::TexelCopyTextureInfo {
                        texture: &v.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyBufferInfo {
                        buffer: &v.readback,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(v.padded_row),
                            rows_per_image: Some(self.anim_config.height),
                        },
                    },
                    wgpu::Extent3d {
                        width: self.anim_config.width,
                        height: self.anim_config.height,
                        depth_or_array_layers: 1,
                    },
                );
                self.queue.submit(Some(enc.finish()));

                // Blocking map. Pipeline this if throughput matters.
                let slice = v.readback.slice(..);
                let (tx, rx) = std::sync::mpsc::channel();
                slice.map_async(wgpu::MapMode::Read, move |r| { let _ = tx.send(r); });
                self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None })?;
                rx.recv().unwrap()?;

                let data = slice.get_mapped_range()?;
                let unpadded = (self.anim_config.width * 4) as usize;
                let mut rgba = Vec::with_capacity(unpadded * self.anim_config.height as usize);
                for row in data.chunks(v.padded_row as usize) {
                    rgba.extend_from_slice(&row[..unpadded]);
                }
                drop(data);
                v.readback.unmap();

                v.encoder.send(rgba)?;
            }
            _ => unreachable!() // Do we really need frame.inner?
        }

        Ok(())
    }

    pub fn set_scene(&mut self, objects: Vec<ObjectDesc>) {
        self.state.objects = objects;
    }
    
    pub fn set_animations(&mut self, animations: Vec<Arc<dyn Animation>>) {
        self.state.animation_cache.clear();
        for x in animations {
            match x.affected_shapes() {
                AffectedShapes::Single(s) => {
                    self.state.animation_cache.insert(*s, AnimationState::new(x));
                }
                AffectedShapes::Vector(v) => {
                    for s in v {
                        self.state.animation_cache.insert(*s, AnimationState::new(x.clone()));
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct RunimDrawContext<'a> {
    offset: Vec3,
    vertices: Vec<Vertex>,
    tex_vertices: HashMap<u64, (u32, TextureHandle, Vec<TexVertex>)>,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    texture_layout: &'a wgpu::BindGroupLayout,
}

impl RunimDrawContext<'_> {
    pub fn new<'a>(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        texture_layout: &'a wgpu::BindGroupLayout
    ) -> RunimDrawContext<'a> {
        RunimDrawContext {
            offset: Vec3::ZERO,
            vertices: Vec::new(),
            tex_vertices: HashMap::new(),
            device,
            queue,
            texture_layout
        }
    }

    pub fn unpack(self) -> (Vec<Vertex>, HashMap<u64, (u32, TextureHandle, Vec<TexVertex>)>) {
        (self.vertices, self.tex_vertices)
    }
}

impl<'a> DrawContext for RunimDrawContext<'a> {
    fn offset(&mut self, pos: Vec3) {
        self.offset = pos;
    }

    fn clear_offset(&mut self) {
        self.offset = Vec3::ZERO;
    }

    fn vertex(&mut self, mut pos: Vec3, color: Color) {
        pos.x += self.offset.x;
        pos.y += self.offset.y;
        pos.z -= 1f32;
        self.vertices.push(Vertex {
            position: pos.to_array(),
            color: color.to_array()
        });
    }

    fn tex_vertex(&mut self, mut pos: Vec3, tex_pos: Vec2, tex: &ImageHandle) {
        pos.x += self.offset.x;
        pos.y += self.offset.y;
        pos.z -= 1f32;
        let tex = tex.as_any().downcast_ref::<TextureHandle>().unwrap();
        self.tex_vertices.entry(tex.texture.id).or_insert_with(|| (0, tex.clone(), Vec::new()))
            .2.push(TexVertex {
                position: pos.to_array(),
                tex: tex_pos.to_array(),
            });
    }

    fn load_image(&mut self, image: &DynamicImage) -> anyhow::Result<ImageHandle> {
        let texture = Texture::from_image(
            self.device, self.queue, image, dbgn!("DrawContext#").name(),
            wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING
        )?;
        Ok(Box::new(texture.into_handle(self.device, self.texture_layout)))
    }
}