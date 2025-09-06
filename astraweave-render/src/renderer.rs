use anyhow::Result;
use glam::{vec3, Mat4};
use std::borrow::Cow;
use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::depth::Depth;
use crate::primitives;
use crate::types::{Instance, InstanceRaw, Mesh};

const SHADER_SRC: &str = r#"
struct VSIn {
  @location(0) position: vec3<f32>,
  @location(1) normal:   vec3<f32>,
  @location(2) m0: vec4<f32>,
  @location(3) m1: vec4<f32>,
  @location(4) m2: vec4<f32>,
  @location(5) m3: vec4<f32>,
  @location(6) n0: vec3<f32>,
  @location(7) n1: vec3<f32>,
  @location(8) n2: vec3<f32>,
  @location(9) color: vec4<f32>,
};

struct VSOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) color: vec4<f32>,
};

struct Camera {
  view_proj: mat4x4<f32>,
  light_dir: vec3<f32>,
  _pad: f32,
};

@group(0) @binding(0) var<uniform> uCamera: Camera;

@vertex
fn vs(input: VSIn) -> VSOut {
  let model = mat4x4<f32>(input.m0, input.m1, input.m2, input.m3);
  let world = model * vec4<f32>(input.position, 1.0);
  var out: VSOut;
  out.pos = uCamera.view_proj * world;
  // normal matrix simplified (assuming uniform scale); for accuracy pass and use 3x3
  out.normal = normalize((model * vec4<f32>(input.normal, 0.0)).xyz);
  out.world_pos = world.xyz;
  out.color = input.color;
  return out;
}

@fragment
fn fs(input: VSOut) -> @location(0) vec4<f32> {
  let n = normalize(input.normal);
  let l = normalize(-uCamera.light_dir);
  let ndl = max(dot(n, l), 0.0);
  let ambient = 0.2;
  let diff = ndl * 0.8;
  let lit = (ambient + diff);
  let shaded = vec4<f32>(input.color.rgb * lit, input.color.a);
  return shaded;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUBO {
    view_proj: [[f32; 4]; 4],
    light_dir_pad: [f32; 4],
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth: Depth,

    #[allow(dead_code)]
    shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,

    camera_ubo: CameraUBO,
    camera_buf: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    mesh_cube: Mesh,
    mesh_plane: Mesh,

    instances: Vec<Instance>,
    instance_buf: wgpu::Buffer,

    #[allow(dead_code)]
    overlay: crate::overlay::OverlayFx,
    pub overlay_params: crate::overlay::OverlayParams,
    pub weather: crate::effects::WeatherFx,
}

impl Renderer {
    pub async fn new(window: std::sync::Arc<winit::window::Window>) -> Result<Self> {
        // WGPU init
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // Surface config
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Depth
        let depth = crate::depth::Depth::create(&device, &config);

        // Shaders / pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("basic shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER_SRC)),
        });

        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera ubo"),
            size: std::mem::size_of::<CameraUBO>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bg"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline layout"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs",
                buffers: &[
                    crate::types::Vertex::layout(),
                    crate::types::InstanceRaw::layout(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth.format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let overlay = crate::overlay::OverlayFx::new(&device, format);

        let overlay_params = crate::overlay::OverlayParams {
            fade: 0.0,
            letterbox: 0.0,
            _pad: [0.0; 2],
        };

        // Meshes
        let (cube_v, cube_i) = primitives::cube();
        let mesh_cube = Mesh {
            vertex_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("cube v"),
                contents: bytemuck::cast_slice(&cube_v),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            index_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("cube i"),
                contents: bytemuck::cast_slice(&cube_i),
                usage: wgpu::BufferUsages::INDEX,
            }),
            index_count: cube_i.len() as u32,
        };

        let (plane_v, plane_i) = primitives::plane();
        let mesh_plane = Mesh {
            vertex_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("plane v"),
                contents: bytemuck::cast_slice(&plane_v),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            index_buf: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("plane i"),
                contents: bytemuck::cast_slice(&plane_i),
                usage: wgpu::BufferUsages::INDEX,
            }),
            index_count: plane_i.len() as u32,
        };

        // Instance buffer (start empty; caller will fill)
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buf"),
            size: 1024 * 1024, // 1MB to start (grow if needed)
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // after instance_buf creation
        let weather = crate::effects::WeatherFx::new(&device, 800);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            depth,
            shader,
            pipeline,
            camera_ubo: CameraUBO {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                light_dir_pad: [0.5, 1.0, 0.8, 0.0],
            },
            camera_buf,
            camera_bind_group,
            mesh_cube,
            mesh_plane,
            instances: Vec::new(),
            instance_buf,
            overlay,
            overlay_params,
            weather,
        })
    }

    pub fn resize(&mut self, new_w: u32, new_h: u32) {
        if new_w == 0 || new_h == 0 {
            return;
        }
        self.config.width = new_w;
        self.config.height = new_h;
        self.surface.configure(&self.device, &self.config);
        self.depth = crate::depth::Depth::create(&self.device, &self.config);
    }

    pub fn update_camera(&mut self, camera: &Camera) {
        self.camera_ubo.view_proj = camera.vp().to_cols_array_2d();
        self.queue
            .write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&self.camera_ubo));
    }

    pub fn update_instances(&mut self, instances: &[Instance]) {
        self.instances.clear();
        self.instances.extend_from_slice(instances);
        let raws: Vec<InstanceRaw> = self.instances.iter().map(|i| i.raw()).collect();
        let size = (raws.len() * std::mem::size_of::<InstanceRaw>()) as u64;

        if size > self.instance_buf.size() {
            self.instance_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("instance buf (resized)"),
                size: size.next_power_of_two(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        self.queue
            .write_buffer(&self.instance_buf, 0, bytemuck::cast_slice(&raws));
    }

    pub fn set_weather(&mut self, kind: crate::effects::WeatherKind) {
        self.weather.set_kind(kind);
    }

    pub fn tick_weather(&mut self, dt: f32) {
        self.weather.update(&self.queue, dt);
    }

    pub fn render(&mut self) -> Result<()> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        // Create plane buffer before render pass
        let plane_scale = glam::Mat4::from_scale(vec3(50.0, 1.0, 50.0));
        let plane_inst = Instance {
            transform: plane_scale,
            color: [0.1, 0.12, 0.14, 1.0],
        }
        .raw();
        let plane_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("plane inst"),
                contents: bytemuck::bytes_of(&plane_inst),
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.07,
                            b: 0.10,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.camera_bind_group, &[]);

            // Ground plane (scaled)
            rp.set_vertex_buffer(0, self.mesh_plane.vertex_buf.slice(..));
            rp.set_index_buffer(
                self.mesh_plane.index_buf.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rp.set_vertex_buffer(1, plane_buf.slice(..));
            rp.draw_indexed(0..self.mesh_plane.index_count, 0, 0..1);

            // Cubes (instances)
            rp.set_vertex_buffer(0, self.mesh_cube.vertex_buf.slice(..));
            rp.set_index_buffer(
                self.mesh_cube.index_buf.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rp.set_vertex_buffer(1, self.instance_buf.slice(..));
            let inst_count = self.instances.len() as u32;
            if inst_count > 0 {
                rp.draw_indexed(0..self.mesh_cube.index_count, 0, 0..inst_count);
            }
        }

        self.queue.submit(Some(enc.finish()));
        frame.present();
        Ok(())
    }

    pub fn surface_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn render_with<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(
            &wgpu::TextureView,
            &mut wgpu::CommandEncoder,
            &wgpu::Device,
            &wgpu::Queue,
            (u32, u32),
        ),
    {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        f(
            &view,
            &mut enc,
            &self.device,
            &self.queue,
            self.surface_size(),
        );

        self.queue.submit(std::iter::once(enc.finish()));
        frame.present();
        Ok(())
    }
}
