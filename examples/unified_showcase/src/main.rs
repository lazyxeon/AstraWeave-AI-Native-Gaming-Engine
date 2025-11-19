use glam::{Mat4, Quat, Vec3};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod gltf_loader;

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
    tangent: [f32; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x2,
        3 => Float32x4,
        4 => Float32x4,
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniforms {
    view_proj: [[f32; 4]; 4],
    position: [f32; 3],
    _padding: f32,
    color: [f32; 3],
    _padding2: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniforms {
    model: [[f32; 4]; 4],
}

struct Material {
    name: String,
    bind_group: wgpu::BindGroup,
}

struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    material_index: usize,
}

struct SceneObject {
    mesh_index: usize,
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    model_bind_group: wgpu::BindGroup,
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

struct ShowcaseApp {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    
    render_pipeline: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    
    material_layout: wgpu::BindGroupLayout,
    model_layout: wgpu::BindGroupLayout,
    
    depth_texture: wgpu::TextureView,
    msaa_texture: wgpu::TextureView,
    
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_pos: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,
    
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    
    materials: Vec<Material>,
    meshes: Vec<Mesh>,
    objects: Vec<SceneObject>,
    
    sky_bind_group: wgpu::BindGroup,
    sky_mesh_index: usize,
    
    mouse_pressed: bool,
}

impl ShowcaseApp {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        println!("Window size: {}x{}", size.width, size.height);
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            },
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
            
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Layouts
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let light_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Layout"),
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
        });

        let model_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Model Layout"),
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
        });

        // Pipelines
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader V2"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_v2.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_layout, &light_layout, &material_layout, &model_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let sky_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Sky Layout"),
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
        });

        let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sky Pipeline Layout"),
            bind_group_layouts: &[&camera_layout, &sky_layout],
            push_constant_ranges: &[],
        });

        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sky Pipeline"),
            layout: Some(&sky_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_skybox"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_skybox"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling for skybox to ensure visibility
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Buffers
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniforms {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                camera_pos: [0.0, 0.0, 0.0],
                _padding: 0.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[LightUniforms {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                position: [50.0, 100.0, 50.0],
                _padding: 0.0,
                color: [1.2, 1.1, 1.0], // Warm sunlight
                _padding2: 0.0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        // Skybox Texture
        let sky_img = image::open("assets/sky_equirect.png").expect("Missing sky_equirect.png").to_rgba8();
        let sky_size = wgpu::Extent3d { width: sky_img.width(), height: sky_img.height(), depth_or_array_layers: 1 };
        let sky_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Sky Texture"),
            size: sky_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &sky_texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &sky_img,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * sky_img.width()), rows_per_image: Some(sky_img.height()) },
            sky_size,
        );
        let sky_view = sky_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sky_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        
        let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sky Bind Group"),
            layout: &sky_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&sky_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sky_sampler) },
            ],
        });

        // Textures
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        }).create_view(&wgpu::TextureViewDescriptor::default());

        let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Texture"),
            size: wgpu::Extent3d { width: config.width, height: config.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        }).create_view(&wgpu::TextureViewDescriptor::default());

        let mut app = Self {
            window, surface, device, queue, config,
            render_pipeline, sky_pipeline,
            material_layout, model_layout,
            depth_texture, msaa_texture,
            camera_buffer, camera_bind_group,
            camera_pos: Vec3::new(0.0, 5.0, 15.0), // Closer to ground
            camera_yaw: 0.0, // Reset yaw
            camera_pitch: -0.1, // Look slightly down
            light_buffer, light_bind_group,
            materials: Vec::new(),
            meshes: Vec::new(),
            objects: Vec::new(),
            sky_bind_group,
            sky_mesh_index: 0,
            mouse_pressed: false,
        };

        app.init_scene();
        app
    }

    fn init_scene(&mut self) {
        println!("Initializing scene...");
        self.sky_mesh_index = self.create_sphere_mesh(500.0, 32, 32);
        println!("Sky mesh created.");

        // Materials
        let ground_mat = self.create_material_from_texture("Ground", "assets/grass.png");
        let tower_mat = self.create_material_from_color("Tower", [0.7, 0.7, 0.7, 1.0]);
        let leaves_mat = self.create_material_from_color("Leaves", [0.0, 0.8, 0.0, 1.0]);
        let wood_mat = self.create_material_from_color("Wood", [0.4, 0.2, 0.1, 1.0]);
        let rock_mat = self.create_material_from_color("Rock", [0.5, 0.5, 0.5, 1.0]);
        let tent_mat = self.create_material_from_color("Tent", [0.8, 0.2, 0.2, 1.0]); // Red tent
        println!("Materials created.");

        // Ground
        let ground_mesh = self.create_plane_mesh(100.0, ground_mat);
        self.objects.push(SceneObject {
            mesh_index: ground_mesh,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            model_bind_group: self.create_model_bind_group(Mat4::IDENTITY),
        });
        println!("Ground object added.");

        // Helper to load model
        let mut load_model = |path: &str, mat: usize, pos: Vec3, scale: f32, rot: Quat| {
             if let Ok(indices) = self.load_gltf(path, mat) {
                for idx in indices {
                    self.objects.push(SceneObject {
                        mesh_index: idx,
                        position: pos,
                        rotation: rot,
                        scale: Vec3::splat(scale),
                        model_bind_group: self.create_model_bind_group(Mat4::from_scale_rotation_translation(Vec3::splat(scale), rot, pos)),
                    });
                }
             } else {
                 println!("Failed to load {}", path);
             }
        };

        // Tower
        load_model("assets/models/tower.glb", tower_mat, Vec3::ZERO, 5.0, Quat::IDENTITY);

        // Campfire
        load_model("assets/models/campfire_logs.glb", wood_mat, Vec3::new(15.0, 0.0, 15.0), 5.0, Quat::IDENTITY);
        
        // Tent
        load_model("assets/models/tent_smallOpen.glb", tent_mat, Vec3::new(25.0, 0.0, 15.0), 5.0, Quat::from_rotation_y(1.5));

        // Rocks
        load_model("assets/models/rock_largeA.glb", rock_mat, Vec3::new(-15.0, 0.0, 15.0), 5.0, Quat::from_rotation_y(0.5));
        load_model("assets/models/rock_smallA.glb", rock_mat, Vec3::new(-12.0, 0.0, 18.0), 5.0, Quat::IDENTITY);

        // Trees
        let tree_positions = [
             (Vec3::new(30.0, 0.0, 30.0), "assets/models/tree_default.glb"),
             (Vec3::new(-30.0, 0.0, 30.0), "assets/models/tree_pineDefaultA.glb"),
             (Vec3::new(30.0, 0.0, -30.0), "assets/models/tree_oak.glb"),
             (Vec3::new(-45.0, 0.0, -15.0), "assets/models/tree_simple.glb"),
             (Vec3::new(-24.0, 0.0, -36.0), "assets/models/tree_pineRoundA.glb"),
        ];
        
        for (pos, path) in tree_positions {
            load_model(path, leaves_mat, pos, 5.0, Quat::IDENTITY);
        }

        println!("Scene initialization complete.");
    }

    fn create_model_bind_group(&self, model_mat: Mat4) -> wgpu::BindGroup {
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(&[ModelUniforms { model: model_mat.to_cols_array_2d() }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Model Bind Group"),
            layout: &self.model_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    fn create_material_from_color(&mut self, name: &str, color: [f32; 4]) -> usize {
        let size = wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        let data = [
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        ];
        
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &data,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            size,
        );
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &self.material_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
        
        self.materials.push(Material { name: name.to_string(), bind_group });
        self.materials.len() - 1
    }

    fn create_material_from_texture(&mut self, name: &str, path: &str) -> usize {
        let img = image::open(path).expect(&format!("Missing texture: {}", path)).to_rgba8();
        let size = wgpu::Extent3d { width: img.width(), height: img.height(), depth_or_array_layers: 1 };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &img,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * img.width()), rows_per_image: Some(img.height()) },
            size,
        );
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &self.material_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
        
        self.materials.push(Material { name: name.to_string(), bind_group });
        self.materials.len() - 1
    }

    fn create_sphere_mesh(&mut self, radius: f32, sectors: u32, stacks: u32) -> usize {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        for i in 0..=stacks {
            let v = i as f32 / stacks as f32;
            let phi = v * std::f32::consts::PI;
            
            for j in 0..=sectors {
                let u = j as f32 / sectors as f32;
                let theta = u * 2.0 * std::f32::consts::PI;
                
                let x = radius * phi.sin() * theta.cos();
                let y = radius * phi.cos();
                let z = radius * phi.sin() * theta.sin();
                
                vertices.push(Vertex {
                    position: [x, y, z],
                    normal: [x/radius, y/radius, z/radius],
                    uv: [u, v],
                    color: [1.0, 1.0, 1.0, 1.0],
                    tangent: [0.0; 4],
                });
            }
        }
        
        for i in 0..stacks {
            for j in 0..sectors {
                let first = (i * (sectors + 1)) + j;
                let second = first + sectors + 1;
                indices.extend_from_slice(&[first, second, first + 1, second, second + 1, first + 1]);
            }
        }
        
        self.create_mesh_from_data(&vertices, &indices, 0)
    }
    
    fn create_plane_mesh(&mut self, size: f32, mat_idx: usize) -> usize {
        let s = size / 2.0;
        let uv_scale = 50.0; // Increase tiling for better detail
        let vertices = vec![
            Vertex { position: [-s, 0.0, -s], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0, 1.0] },
            Vertex { position: [s, 0.0, -s], normal: [0.0, 1.0, 0.0], uv: [uv_scale, 0.0], color: [1.0; 4], tangent: [1.0, 0.0, 0.0, 1.0] },
            Vertex { position: [s, 0.0, s], normal: [0.0, 1.0, 0.0], uv: [uv_scale, uv_scale], color: [1.0; 4], tangent: [1.0, 0.0, 0.0, 1.0] },
            Vertex { position: [-s, 0.0, s], normal: [0.0, 1.0, 0.0], uv: [0.0, uv_scale], color: [1.0; 4], tangent: [1.0, 0.0, 0.0, 1.0] },
        ];
        // Flip indices to correct winding order (CCW for Up-facing normal)
        let indices = vec![0, 2, 1, 0, 3, 2];
        self.create_mesh_from_data(&vertices, &indices, mat_idx)
    }
    
    fn create_cube_mesh(&mut self, size: f32, mat_idx: usize) -> usize {
        let s = size / 2.0;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let mut add_face = |p: [Vec3; 4], n: Vec3| {
            let base = vertices.len() as u32;
            vertices.push(Vertex { position: p[0].to_array(), normal: n.to_array(), uv: [0.0, 1.0], color: [1.0; 4], tangent: [0.0; 4] });
            vertices.push(Vertex { position: p[1].to_array(), normal: n.to_array(), uv: [1.0, 1.0], color: [1.0; 4], tangent: [0.0; 4] });
            vertices.push(Vertex { position: p[2].to_array(), normal: n.to_array(), uv: [1.0, 0.0], color: [1.0; 4], tangent: [0.0; 4] });
            vertices.push(Vertex { position: p[3].to_array(), normal: n.to_array(), uv: [0.0, 0.0], color: [1.0; 4], tangent: [0.0; 4] });
            indices.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
        };
        
        add_face([Vec3::new(-s,-s,s), Vec3::new(s,-s,s), Vec3::new(s,s,s), Vec3::new(-s,s,s)], Vec3::Z);
        add_face([Vec3::new(s,-s,-s), Vec3::new(-s,-s,-s), Vec3::new(-s,s,-s), Vec3::new(s,s,-s)], -Vec3::Z);
        add_face([Vec3::new(s,-s,s), Vec3::new(s,-s,-s), Vec3::new(s,s,-s), Vec3::new(s,s,s)], Vec3::X);
        add_face([Vec3::new(-s,-s,-s), Vec3::new(-s,-s,s), Vec3::new(-s,s,s), Vec3::new(-s,s,-s)], -Vec3::X);
        add_face([Vec3::new(-s,s,s), Vec3::new(s,s,s), Vec3::new(s,s,-s), Vec3::new(-s,s,-s)], Vec3::Y);
        add_face([Vec3::new(-s,-s,-s), Vec3::new(s,-s,-s), Vec3::new(s,-s,s), Vec3::new(-s,-s,s)], -Vec3::Y);
        
        self.create_mesh_from_data(&vertices, &indices, mat_idx)
    }

    fn create_mesh_from_data(&mut self, vertices: &[Vertex], indices: &[u32], mat_idx: usize) -> usize {
        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        self.meshes.push(Mesh {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            material_index: mat_idx,
        });
        self.meshes.len() - 1
    }

    fn load_gltf(&mut self, path: &str, default_mat: usize) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        let loaded = gltf_loader::load_gltf(std::path::Path::new(path))?;
        let mut indices = Vec::new();
        for mesh_data in loaded {
            let vertices: Vec<Vertex> = mesh_data.vertices.iter().map(|v| Vertex {
                position: v.position,
                normal: v.normal,
                uv: v.uv,
                color: v.color,
                tangent: v.tangent,
            }).collect();
            indices.push(self.create_mesh_from_data(&vertices, &mesh_data.indices, default_mat));
        }
        Ok(indices)
    }

    fn update(&mut self) {
        let view = Mat4::look_at_rh(
            self.camera_pos,
            self.camera_pos + Quat::from_rotation_y(self.camera_yaw) * Quat::from_rotation_x(self.camera_pitch) * Vec3::NEG_Z, // Use NEG_Z
            Vec3::Y,
        );
        // println!("Camera Pos: {:?}, Yaw: {}, Pitch: {}", self.camera_pos, self.camera_yaw, self.camera_pitch);
        let proj = Mat4::perspective_rh(45.0_f32.to_radians(), self.config.width as f32 / self.config.height as f32, 0.1, 1000.0);
        
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[CameraUniforms {
            view_proj: (proj * view).to_cols_array_2d(),
            camera_pos: self.camera_pos.to_array(),
            _padding: 0.0,
        }]));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // println!("Render frame start");
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_texture,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.15, a: 1.0 }), // Dark Blue-Gray
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.sky_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.sky_bind_group, &[]);
            let sky_mesh = &self.meshes[self.sky_mesh_index];
            render_pass.set_vertex_buffer(0, sky_mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(sky_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..sky_mesh.num_indices, 0, 0..1);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);

            for obj in &self.objects {
                let mesh = &self.meshes[obj.mesh_index];
                let material = &self.materials[mesh.material_index];
                render_pass.set_bind_group(2, &material.bind_group, &[]);
                render_pass.set_bind_group(3, &obj.model_bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
    
    fn handle_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(key), state, .. }, .. } => {
                let speed = 0.5;
                if *state == ElementState::Pressed {
                    match key {
                        KeyCode::KeyW => self.camera_pos += Quat::from_rotation_y(self.camera_yaw) * Vec3::NEG_Z * speed,
                        KeyCode::KeyS => self.camera_pos += Quat::from_rotation_y(self.camera_yaw) * Vec3::Z * speed,
                        KeyCode::KeyA => self.camera_pos += Quat::from_rotation_y(self.camera_yaw) * Vec3::NEG_X * speed,
                        KeyCode::KeyD => self.camera_pos += Quat::from_rotation_y(self.camera_yaw) * Vec3::X * speed,
                        KeyCode::Space => self.camera_pos.y += speed,
                        KeyCode::ShiftLeft => self.camera_pos.y -= speed,
                        _ => {}
                    }
                }
                true
            }
            WindowEvent::MouseInput { state, button: MouseButton::Right, .. } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }
    
    fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        if self.mouse_pressed {
            let sensitivity = 0.005;
            self.camera_yaw += (delta.0 as f32) * sensitivity;
            self.camera_pitch += (delta.1 as f32) * sensitivity;
        }
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            
            self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d { width: self.config.width, height: self.config.height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            }).create_view(&wgpu::TextureViewDescriptor::default());

            self.msaa_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("MSAA Texture"),
                size: wgpu::Extent3d { width: self.config.width, height: self.config.height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            }).create_view(&wgpu::TextureViewDescriptor::default());
        }
    }
}

struct App {
    state: Option<ShowcaseApp>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attrs = Window::default_attributes().with_title("AstraWeave Showcase V2");
            let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
            let state = pollster::block_on(ShowcaseApp::new(window));
            self.state = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(state) = &mut self.state {
            if id == state.window.id() {
                if !state.handle_input(&event) {
                    match event {
                        WindowEvent::CloseRequested => event_loop.exit(),
                        WindowEvent::Resized(physical_size) => state.resize(physical_size),
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(winit::dpi::PhysicalSize::new(state.config.width, state.config.height)), // Reconfigure
                                Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if let Some(state) = &mut self.state {
            if let DeviceEvent::MouseMotion { delta } = event {
                state.handle_mouse_motion(delta);
            }
        }
    }
    
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &mut self.state {
            state.window.request_redraw();
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App { state: None };
    event_loop.run_app(&mut app).unwrap();
}
