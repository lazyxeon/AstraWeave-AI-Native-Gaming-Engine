use glam::{Mat4, Quat, Vec3};
use std::sync::Arc;
use wgpu::util::DeviceExt;
/// AstraWeave Unified Showcase - CLEAN IMPLEMENTATION
/// Built on proper foundations with correct texture/material handling
///
/// Architecture:
/// - Each material gets its own texture binding (no atlas complexity)
/// - GLTF models use their original UVs (no remapping needed)
/// - Simple, straightforward rendering pipeline
/// - Based on how Bevy/Godot actually handle materials
use winit::{event::*, event_loop::EventLoop};

mod gltf_loader;

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================

/// Vertex format with vertex colors (for Kenney models)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],   // Vertex color (RGBA)
    tangent: [f32; 4], // Tangent (xyz) + handedness (w)
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x2, // uv
        3 => Float32x4, // color
        4 => Float32x4, // tangent
    ];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Camera uniforms
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    _padding: f32,
}

/// Per-object uniforms (model matrix for positioning each object)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniforms {
    model: [[f32; 4]; 4],
}

/// Per-object instance data
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    model_matrix: [[f32; 4]; 4],
}

/// Material - simple texture reference
struct Material {
    name: String,
    albedo_texture: wgpu::Texture,
    albedo_view: wgpu::TextureView,
    normal_texture: Option<wgpu::Texture>,
    normal_view: Option<wgpu::TextureView>,
    roughness_texture: Option<wgpu::Texture>,
    roughness_view: Option<wgpu::TextureView>,
    bind_group: wgpu::BindGroup,
}

/// Mesh - geometry with material index
struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    material_index: usize,
}

/// Scene object instance
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
    // GPU resources
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,

    // Rendering pipeline
    render_pipeline: wgpu::RenderPipeline,
    material_bind_group_layout: wgpu::BindGroupLayout,
    model_bind_group_layout: wgpu::BindGroupLayout,

    // Camera
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_position: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,

    // Scene data
    materials: Vec<Material>,
    meshes: Vec<Mesh>,
    objects: Vec<SceneObject>,

    // Shared resources
    sampler: wgpu::Sampler,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

impl ShowcaseApp {
    async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        // Create wgpu instance and surface
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Main Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        println!("✅ GPU initialized: {}", adapter.get_info().name);

        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler (use Repeat for individual textures - this works correctly!)
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create shader (clean shader with optional normal mapping support)
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Clean Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_clean.wgsl").into()),
        });

        // Create bind group layouts
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
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

        let material_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Material Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Normal map (optional)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Roughness/metallic/ao (packed) or roughness single channel
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Model bind group layout (for per-object transforms)
        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Model Bind Group Layout"),
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

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &material_bind_group_layout,
                &model_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
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
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Disable culling temporarily to debug ground plane
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create camera buffer
        let camera_uniforms = CameraUniforms {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0, 0.0, 0.0],
            _padding: 0.0,
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        println!("✅ Render pipeline created");

        Self {
            device,
            queue,
            surface,
            surface_config,
            render_pipeline,
            material_bind_group_layout,
            model_bind_group_layout,
            camera_buffer,
            camera_bind_group,
            camera_position: Vec3::new(0.0, 10.0, 20.0),
            camera_yaw: 0.0,
            camera_pitch: -20.0_f32.to_radians(),
            materials: Vec::new(),
            meshes: Vec::new(),
            objects: Vec::new(),
            sampler,
            depth_texture,
            depth_view,
        }
    }

    // ========================================================================
    // STEP 1: TEXTURE LOADING
    // ========================================================================

    /// Load a texture from file
    fn load_texture(
        &self,
        path: &str,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), Box<dyn std::error::Error>> {
        let img = image::open(path)?.to_rgba8();
        let (width, height) = img.dimensions();

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(path),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        println!("  ✅ Loaded texture: {} ({}×{})", path, width, height);
        Ok((texture, view))
    }

    /// Create a material from a texture file
    fn create_material(
        &mut self,
        name: &str,
        texture_path: &str,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // Load albedo
        let (texture, view) = self.load_texture(texture_path)?;

        // Try to load normal map and roughness/mra maps using common suffixes
        let p = std::path::Path::new(texture_path);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("texture");
        let dir = p.parent().unwrap_or(std::path::Path::new(""));
        let normal_path = dir.join(format!("{}_n.png", stem));
        let roughness_path = dir.join(format!("{}_mra.png", stem));

        let (normal_texture, normal_view) = if normal_path.exists() {
            let (t, v) = self.load_texture(normal_path.to_str().unwrap())?;
            (Some(t), Some(v))
        } else {
            (None, None)
        };

        let (roughness_texture, roughness_view) = if roughness_path.exists() {
            let (t, v) = self.load_texture(roughness_path.to_str().unwrap())?;
            (Some(t), Some(v))
        } else {
            (None, None)
        };

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Bind Group", name)),
            layout: &self.material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                // Normal map (binding 2) if present, otherwise fallback to albedo view
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        normal_view.as_ref().unwrap_or(&view),
                    ),
                },
                // Roughness/MRA map (binding 3) if present, otherwise fallback to albedo view
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        roughness_view.as_ref().unwrap_or(&view),
                    ),
                },
            ],
        });

        self.materials.push(Material {
            name: name.to_string(),
            albedo_texture: texture,
            albedo_view: view,
            normal_texture,
            normal_view,
            roughness_texture,
            roughness_view,
            bind_group,
        });

        println!("  📦 Created material: {}", name);
        Ok(self.materials.len() - 1)
    }

    /// Create a fallback colored texture
    fn create_fallback_texture(&self, color: [u8; 4]) -> (wgpu::Texture, wgpu::TextureView) {
        // Create a simple 4x4 solid color texture
        let img_data = vec![color; 16]; // 4x4 pixels
        let flat_data: Vec<u8> = img_data.into_iter().flat_map(|c| c).collect();

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Fallback Texture"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &flat_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4), // 4 pixels × 4 bytes
                rows_per_image: Some(4),    // 4 rows
            },
            wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Create material with fallback
    fn create_material_with_fallback(
        &mut self,
        name: &str,
        texture_path: &str,
        fallback_color: [u8; 4],
    ) -> usize {
        match self.create_material(name, texture_path) {
            Ok(index) => index,
            Err(e) => {
                println!("  ⚠️  Failed to load {}: {}", texture_path, e);
                println!("  🎨 Using fallback color for {}", name);

                let (texture, view) = self.create_fallback_texture(fallback_color);

                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("{} Fallback Bind Group", name)),
                    layout: &self.material_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                    ],
                });

                self.materials.push(Material {
                    name: name.to_string(),
                    albedo_texture: texture,
                    albedo_view: view,
                    normal_texture: None,
                    normal_view: None,
                    roughness_texture: None,
                    roughness_view: None,
                    bind_group,
                });

                self.materials.len() - 1
            }
        }
    }

    // ========================================================================
    // STEP 2: GLTF MESH LOADING
    // ========================================================================

    /// Load a GLTF mesh
    fn load_gltf_mesh(
        &mut self,
        path: &str,
        material_index: usize,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // Try to canonicalize path for better error messages
        let path_buf = std::path::Path::new(path);
        log::info!("Loading GLTF: {}", path);
        let loaded = match gltf_loader::load_gltf(path_buf) {
            Ok(l) => l,
            Err(e) => {
                // Try fallback: path relative to workspace 'assets' folder
                let fallback = std::path::Path::new("assets")
                    .join(path.strip_prefix("assets/").unwrap_or(path));
                log::warn!(
                    "Primary gltf import failed for '{}', trying fallback path: {}",
                    path,
                    fallback.display()
                );
                let res = gltf_loader::load_gltf(&fallback);
                if res.is_err() {
                    // Print detailed diagnostics before returning the original error
                    // As a last resort, try to load from workspace 'assets' folder
                    let canonical = std::fs::canonicalize(path)
                        .unwrap_or_else(|_| std::path::PathBuf::from(path));
                    log::warn!(
                        "Canonical import failed, trying fallback path: {}",
                        fallback.display()
                    );
                    let res2 = gltf_loader::load_gltf(&fallback);
                    if res2.is_err() {
                        log::error!(
                            "Detailed GLTF load failure for '{}': {}\nCanonical: {}\nFallback: {}",
                            path,
                            e,
                            canonical.display(),
                            fallback.display()
                        );
                    }
                }
                res? // This will return the error or the loaded mesh
            }
        };

        println!(
            "  ✅ Loaded GLTF: {} ({} vertices, {} triangles)",
            path,
            loaded.vertices.len(),
            loaded.indices.len() / 3
        );

        // Convert to our Vertex format (preserving vertex colors!)
        let vertices: Vec<Vertex> = loaded
            .vertices
            .iter()
            .map(|v| Vertex {
                position: v.position,
                normal: v.normal,
                uv: v.uv,
                color: v.color, // Preserve vertex colors from GLTF!
                tangent: v.tangent,
            })
            .collect();

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Vertices", path)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Indices", path)),
                contents: bytemuck::cast_slice(&loaded.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.meshes.push(Mesh {
            vertex_buffer,
            index_buffer,
            num_indices: loaded.indices.len() as u32,
            material_index,
        });

        Ok(self.meshes.len() - 1)
    }

    /// Load mesh with fallback to procedural cube
    fn load_gltf_mesh_with_fallback(&mut self, path: &str, material_index: usize) -> usize {
        match self.load_gltf_mesh(path, material_index) {
            Ok(index) => index,
            Err(e) => {
                println!("  ⚠️  Failed to load {}: {}", path, e);
                println!("  📦 Using fallback cube geometry");
                self.create_cube_mesh(material_index)
            }
        }
    }

    /// Helper: Create a large ground plane for terrain
    fn create_ground_plane(&mut self, size: f32, material_index: usize) -> usize {
        let half_size = size / 2.0;
        let uv_repeat = size / 10.0; // Moderate texture repeat

        let green = [0.3, 0.6, 0.3, 1.0]; // Green color for grass

        let vertices = vec![
            // Ground plane (y=0, facing up with green vertex color)
            // Vertices in CCW order when viewed from above
            Vertex {
                position: [-half_size, 0.0, -half_size],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color: green,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [half_size, 0.0, -half_size],
                normal: [0.0, 1.0, 0.0],
                uv: [uv_repeat, 0.0],
                color: green,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [half_size, 0.0, half_size],
                normal: [0.0, 1.0, 0.0],
                uv: [uv_repeat, uv_repeat],
                color: green,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-half_size, 0.0, half_size],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, uv_repeat],
                color: green,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
        ];

        // CCW winding for front-face (visible from above)
        let indices = vec![
            0, 1, 2, // First triangle (CCW when viewed from +Y)
            2, 3, 0, // Second triangle (CCW when viewed from +Y)
        ];

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Ground Plane Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Ground Plane Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let mesh = Mesh {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            material_index,
        };

        self.meshes.push(mesh);
        self.meshes.len() - 1
    }

    /// Create a simple cube mesh as fallback
    fn create_cube_mesh(&mut self, material_index: usize) -> usize {
        let s = 1.0;
        let white = [1.0, 1.0, 1.0, 1.0]; // White vertex color

        let vertices = vec![
            // Front face
            Vertex {
                position: [-s, -s, s],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 1.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, -s, s],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 1.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, s, s],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 0.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-s, s, s],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            // Back face
            Vertex {
                position: [s, -s, -s],
                normal: [0.0, 0.0, -1.0],
                uv: [0.0, 1.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-s, -s, -s],
                normal: [0.0, 0.0, -1.0],
                uv: [1.0, 1.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-s, s, -s],
                normal: [0.0, 0.0, -1.0],
                uv: [1.0, 0.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, s, -s],
                normal: [0.0, 0.0, -1.0],
                uv: [0.0, 0.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            // Top face
            Vertex {
                position: [-s, s, s],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 1.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, s, s],
                normal: [0.0, 1.0, 0.0],
                uv: [1.0, 1.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, s, -s],
                normal: [0.0, 1.0, 0.0],
                uv: [1.0, 0.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-s, s, -s],
                normal: [0.0, 1.0, 0.0],
                uv: [0.0, 0.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            // Bottom face
            Vertex {
                position: [-s, -s, -s],
                normal: [0.0, -1.0, 0.0],
                uv: [0.0, 1.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, -s, -s],
                normal: [0.0, -1.0, 0.0],
                uv: [1.0, 1.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, -s, s],
                normal: [0.0, -1.0, 0.0],
                uv: [1.0, 0.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-s, -s, s],
                normal: [0.0, -1.0, 0.0],
                uv: [0.0, 0.0],
                color: white,
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            // Right face
            Vertex {
                position: [s, -s, s],
                normal: [1.0, 0.0, 0.0],
                uv: [0.0, 1.0],
                color: white,
                tangent: [0.0, 0.0, 1.0, 1.0],
            },
            Vertex {
                position: [s, -s, -s],
                normal: [1.0, 0.0, 0.0],
                uv: [1.0, 1.0],
                color: white,
                tangent: [0.0, 0.0, 1.0, 1.0],
            },
            Vertex {
                position: [s, s, -s],
                normal: [1.0, 0.0, 0.0],
                uv: [1.0, 0.0],
                color: white,
                tangent: [0.0, 0.0, 1.0, 1.0],
            },
            Vertex {
                position: [s, s, s],
                normal: [1.0, 0.0, 0.0],
                uv: [0.0, 0.0],
                color: white,
                tangent: [0.0, 0.0, 1.0, 1.0],
            },
            // Left face
            Vertex {
                position: [-s, -s, -s],
                normal: [-1.0, 0.0, 0.0],
                uv: [0.0, 1.0],
                color: white,
                tangent: [0.0, 0.0, -1.0, 1.0],
            },
            Vertex {
                position: [-s, -s, s],
                normal: [-1.0, 0.0, 0.0],
                uv: [1.0, 1.0],
                color: white,
                tangent: [0.0, 0.0, -1.0, 1.0],
            },
            Vertex {
                position: [-s, s, s],
                normal: [-1.0, 0.0, 0.0],
                uv: [1.0, 0.0],
                color: white,
                tangent: [0.0, 0.0, -1.0, 1.0],
            },
            Vertex {
                position: [-s, s, -s],
                normal: [-1.0, 0.0, 0.0],
                uv: [0.0, 0.0],
                color: white,
                tangent: [0.0, 0.0, -1.0, 1.0],
            },
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, // Front
            4, 5, 6, 4, 6, 7, // Back
            8, 9, 10, 8, 10, 11, // Top
            12, 13, 14, 12, 14, 15, // Bottom
            16, 17, 18, 16, 18, 19, // Right
            20, 21, 22, 20, 22, 23, // Left
        ];

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube Vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube Indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.meshes.push(Mesh {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            material_index,
        });

        self.meshes.len() - 1
    }

    // ========================================================================
    // STEP 3: SCENE CREATION
    // ========================================================================

    /// Helper: Create a model bind group for an object's transform
    fn create_model_bind_group(
        &self,
        position: Vec3,
        rotation: Quat,
        scale: Vec3,
    ) -> wgpu::BindGroup {
        // Build model matrix
        let model_matrix = Mat4::from_scale_rotation_translation(scale, rotation, position);

        let model_uniforms = ModelUniforms {
            model: model_matrix.to_cols_array_2d(),
        };

        let model_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Buffer"),
                contents: bytemuck::cast_slice(&[model_uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Model Bind Group"),
            layout: &self.model_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_buffer.as_entire_binding(),
            }],
        })
    }

    fn load_scene(&mut self) {
        println!("📦 Loading scene...");
        println!();
        println!("=== MATERIALS ===");

        // Use high-resolution PBR textures from assets/ folder (2048-4096px)
        let brown_mat = self.create_material_with_fallback(
            "Tree Bark (PBR)",
            "assets/tree_bark.png", // 4096×4096 high-res bark texture
            [139, 90, 43, 255],     // Brown fallback
        );

        let green_mat = self.create_material_with_fallback(
            "Grass (PBR)",
            "assets/grass.png", // 2048×2048 high-res grass texture
            [76, 153, 51, 255], // Green fallback
        );

        let gray_mat = self.create_material_with_fallback(
            "Stone (PBR)",
            "assets/stone.png",   // 2048×2048 high-res stone texture
            [128, 128, 128, 255], // Gray fallback
        );

        let skin_mat = self.create_material_with_fallback(
            "Rock Slate (PBR)",
            "assets/rock_slate.png", // 4096×4096 high-res rock texture
            [218, 165, 132, 255],    // Fallback
        );

        println!();
        println!("=== MESHES ===");

        // Create ground plane (HUGE for visibility testing)
        println!("  🌍 Creating terrain...");
        let ground_mesh = self.create_ground_plane(500.0, green_mat);

        // Load multiple tree variants (brown material for trunks)
        let tree_default =
            self.load_gltf_mesh_with_fallback("assets/models/tree_default.glb", brown_mat);
        let tree_oak = self.load_gltf_mesh_with_fallback("assets/models/tree_oak.glb", brown_mat);
        let tree_pine =
            self.load_gltf_mesh_with_fallback("assets/models/tree_pineDefaultA.glb", brown_mat);
        let tree_meshes = vec![tree_default, tree_oak, tree_pine];

        // Load multiple rock variants (gray material for stones)
        let rock_large_a =
            self.load_gltf_mesh_with_fallback("assets/models/rock_largeA.glb", gray_mat);
        let rock_large_b =
            self.load_gltf_mesh_with_fallback("assets/models/rock_largeB.glb", gray_mat);
        let rock_small_a =
            self.load_gltf_mesh_with_fallback("assets/models/rock_smallA.glb", gray_mat);
        let rock_meshes = vec![rock_large_a, rock_large_b, rock_small_a];

        // Load Kenney character/NPC models (skin tone material)
        println!("  🧍 Loading characters...");
        let char_a = self.load_gltf_mesh_with_fallback("assets/models/character-a.glb", skin_mat);
        let char_b = self.load_gltf_mesh_with_fallback("assets/models/character-b.glb", skin_mat);
        let char_c = self.load_gltf_mesh_with_fallback("assets/models/character-c.glb", skin_mat);
        let char_d = self.load_gltf_mesh_with_fallback("assets/models/character-d.glb", skin_mat);
        let char_e = self.load_gltf_mesh_with_fallback("assets/models/character-e.glb", skin_mat);
        let character_meshes = vec![char_a, char_b, char_c, char_d, char_e];

        let building_mesh =
            self.load_gltf_mesh_with_fallback("assets/models/rock_largeA.glb", gray_mat);

        println!();
        println!("=== SCENE OBJECTS ===");

        // Add ground plane (LOWERED MORE for visibility)
        println!("  🌍 Placing terrain...");
        self.objects.push(SceneObject {
            mesh_index: ground_mesh,
            position: Vec3::new(0.0, -5.0, 0.0), // Much lower for testing
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            model_bind_group: self.create_model_bind_group(
                Vec3::new(0.0, -5.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
            ),
        });

        // Create scene objects - trees in a grid with variety
        println!("  🌲 Placing trees...");
        let mut tree_idx = 0;
        for x in -3..=3 {
            for z in -3..=3 {
                if (x + z) % 2 == 0 {
                    // Checkerboard pattern
                    let position = Vec3::new(x as f32 * 8.0, 0.0, z as f32 * 8.0);
                    let rotation = Quat::from_rotation_y((x * z) as f32 * 0.5);
                    let scale = Vec3::splat(1.5);

                    // Cycle through tree variants
                    let mesh_index = tree_meshes[tree_idx % tree_meshes.len()];
                    tree_idx += 1;

                    self.objects.push(SceneObject {
                        mesh_index,
                        position,
                        rotation,
                        scale,
                        model_bind_group: self.create_model_bind_group(position, rotation, scale),
                    });
                }
            }
        }

        // Rocks scattered around with variety
        println!("  🪨 Placing rocks...");
        for i in 0..15 {
            let angle = (i as f32 / 15.0) * std::f32::consts::TAU;
            let radius = 20.0 + (i as f32 * 2.0);
            let position = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
            let rotation = Quat::from_rotation_y(angle);
            let scale = Vec3::splat(1.0 + (i as f32 * 0.1));

            // Cycle through rock variants
            let mesh_index = rock_meshes[i % rock_meshes.len()];

            self.objects.push(SceneObject {
                mesh_index,
                position,
                rotation,
                scale,
                model_bind_group: self.create_model_bind_group(position, rotation, scale),
            });
        }

        // Characters/NPCs scattered around the scene
        println!("  🧍 Placing characters...");
        for i in 0..10 {
            let angle = (i as f32 / 10.0) * std::f32::consts::TAU;
            let radius = 12.0 + (i as f32 * 1.5);
            let position = Vec3::new(
                angle.cos() * radius,
                0.0, // Standing on ground
                angle.sin() * radius,
            );
            let rotation = Quat::from_rotation_y(angle + std::f32::consts::PI); // Face center
            let scale = Vec3::splat(1.0);

            // Cycle through character variants
            let mesh_index = character_meshes[i % character_meshes.len()];

            self.objects.push(SceneObject {
                mesh_index,
                position,
                rotation,
                scale,
                model_bind_group: self.create_model_bind_group(position, rotation, scale),
            });
        }

        // Buildings in corners
        println!("  🏛️  Placing buildings...");
        let building_positions = [
            Vec3::new(-30.0, 0.0, -30.0),
            Vec3::new(30.0, 0.0, -30.0),
            Vec3::new(-30.0, 0.0, 30.0),
            Vec3::new(30.0, 0.0, 30.0),
        ];

        for pos in building_positions {
            let rotation = Quat::IDENTITY;
            let scale = Vec3::new(2.0, 3.0, 2.0);

            self.objects.push(SceneObject {
                mesh_index: building_mesh,
                position: pos,
                rotation,
                scale,
                model_bind_group: self.create_model_bind_group(pos, rotation, scale),
            });
        }

        println!();
        println!("✅ Scene loaded successfully!");
        println!("  📊 Statistics:");
        println!("     Materials: {}", self.materials.len());
        println!("     Meshes: {}", self.meshes.len());
        println!("     Objects: {}", self.objects.len());
        println!();
    }

    // ========================================================================
    // STEP 4 & 5: UPDATE AND RENDER
    // ========================================================================

    fn update(&mut self, dt: f32) {
        // Rotate camera around origin
        self.camera_yaw += dt * 0.2;

        let camera_distance = 50.0;
        self.camera_position = Vec3::new(
            self.camera_yaw.cos() * camera_distance,
            15.0,
            self.camera_yaw.sin() * camera_distance,
        );

        // Update camera uniforms
        let view = Mat4::look_at_rh(self.camera_position, Vec3::ZERO, Vec3::Y);

        let proj = Mat4::perspective_rh(
            60.0_f32.to_radians(),
            self.surface_config.width as f32 / self.surface_config.height as f32,
            0.1,
            1000.0,
        );

        let camera_uniforms = CameraUniforms {
            view_proj: (proj * view).to_cols_array_2d(),
            camera_pos: self.camera_position.to_array(),
            _padding: 0.0,
        };

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniforms]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.53,
                            g: 0.81,
                            b: 0.92,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // Render all objects
            for object in &self.objects {
                let mesh = &self.meshes[object.mesh_index];
                let material = &self.materials[mesh.material_index];

                // Set material bind group
                render_pass.set_bind_group(1, &material.bind_group, &[]);

                // Set model bind group (per-object transform)
                render_pass.set_bind_group(2, &object.model_bind_group, &[]);

                // Set mesh buffers
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                // Draw
                render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);

            // Recreate depth texture
            self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture"),
                size: wgpu::Extent3d {
                    width: new_size.width,
                    height: new_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.depth_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
        }
    }
}

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

fn main() {
    env_logger::init();

    println!("🚀 AstraWeave Unified Showcase - Clean Implementation");
    println!("   Built on solid foundations with proper texture handling");
    println!();

    let event_loop = EventLoop::new().unwrap();
    let window_attributes = winit::window::Window::default_attributes()
        .with_title("AstraWeave Unified Showcase - Clean Implementation")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

    let mut app = pollster::block_on(ShowcaseApp::new(window.clone()));
    app.load_scene();

    let mut last_time = std::time::Instant::now();
    let mut frame_count = 0u32;
    let mut fps_timer = 0.0f32;

    let _ = event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        println!();
                        println!("👋 Goodbye!");
                        elwt.exit();
                    }
                    WindowEvent::Resized(size) => {
                        app.resize(size);
                    }
                    WindowEvent::RedrawRequested => {
                        let now = std::time::Instant::now();
                        let dt = (now - last_time).as_secs_f32();
                        last_time = now;

                        // FPS counter
                        frame_count += 1;
                        fps_timer += dt;
                        if fps_timer >= 1.0 {
                            println!(
                                "📊 FPS: {} ({:.2} ms/frame)",
                                frame_count,
                                1000.0 / frame_count as f32
                            );
                            frame_count = 0;
                            fps_timer = 0.0;
                        }

                        app.update(dt);
                        match app.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => {
                                println!("⚠️  Surface lost, recreating...");
                                app.resize(window.inner_size());
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                eprintln!("❌ Out of memory!");
                                elwt.exit();
                            }
                            Err(e) => eprintln!("⚠️  Render error: {:?}", e),
                        }

                        // Request next frame
                        window.request_redraw();
                    }
                    WindowEvent::KeyboardInput {
                        event: key_event, ..
                    } => {
                        if key_event.state == ElementState::Pressed {
                            if let Some(key) = key_event.logical_key.to_text() {
                                match key {
                                    "escape" => elwt.exit(),
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    // Request redraw on next iteration
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
