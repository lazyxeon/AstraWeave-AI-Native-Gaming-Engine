//! AstraWeave Unified Showcase Demo
//! 
//! This demo showcases multiple AstraWeave engine features in one interactive 3D scene:
//! - WGPU rendering: grid + instanced cubes with per-instance transforms
//! - Camera fly controls (WASD + mouse right-drag)
//! - Rapier3D physics: ground + stack of boxes + falling sphere
//! - Simple navmesh preview + path (toy, for visualization)
//! - Spatial audio demo (plays assets/sound.ogg if present)
//! - egui overlay: toggles for features and live stats
//!
//! This demo is self-contained but can be adapted to use AstraWeave engine crates.

use anyhow::Result;
use std::{borrow::Cow, f32::consts::PI, time::Instant};
use winit::{
    event::{Event, WindowEvent, DeviceEvent, MouseScrollDelta, ElementState, KeyEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
    keyboard::{PhysicalKey, KeyCode},
};
use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3, Vec2};
use rapier3d::prelude as r3;
use rodio::{Decoder, OutputStream, Sink};
use bytemuck::{Pod, Zeroable};

// ------------------------------- Renderer types -------------------------------

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuCamera {
    view_proj: [f32; 16],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InstanceRaw {
    model: [f32; 16],
    color: [f32; 4],
}

struct RenderStuff {
    surface: wgpu::Surface<'static>,
    surface_cfg: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    depth_view: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    cube_vb: wgpu::Buffer,
    cube_ib: wgpu::Buffer,
    cube_index_count: u32,
    camera_ub: wgpu::Buffer,
    camera_bg: wgpu::BindGroup,
    instance_vb: wgpu::Buffer,
    instance_count: u32,
    msaa_samples: u32,
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;
const CUBE_VERTICES: &[[f32; 3]] = &[
    // A simple unit cube centered at origin
    // front
    [-0.5, -0.5,  0.5], [ 0.5, -0.5,  0.5], [ 0.5,  0.5,  0.5], [-0.5,  0.5,  0.5],
    // back
    [-0.5, -0.5, -0.5], [-0.5,  0.5, -0.5], [ 0.5,  0.5, -0.5], [ 0.5, -0.5, -0.5],
];
const CUBE_INDICES: &[u16] = &[
    // front
    0,1,2, 0,2,3,
    // right
    1,7,6, 1,6,2,
    // back
    7,4,5, 7,5,6,
    // left
    4,0,3, 4,3,5,
    // top
    3,2,6, 3,6,5,
    // bottom
    4,7,1, 4,1,0,
];

// ------------------------------- Egui wiring -------------------------------

struct UiState {
    show_grid: bool,
    show_navmesh: bool,
    show_path: bool,
    physics_paused: bool,
    camera_speed: f32,
    resolution_scale: f32,
    fake_ao: bool,
    fake_reflections: bool,
    fps_text: String,
    info_text: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_navmesh: true,
            show_path: true,
            physics_paused: false,
            camera_speed: 8.0,
            resolution_scale: 1.0,
            fake_ao: true,
            fake_reflections: false,
            fps_text: String::new(),
            info_text: "AstraWeave Unified Showcase".to_string(),
        }
    }
}

// ------------------------------- Camera -------------------------------

struct Camera {
    pos: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    fn view_matrix(&self) -> Mat4 {
        let forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos()
        ).normalize();
        Mat4::look_to_rh(self.pos, forward, Vec3::Y)
    }

    fn handle_inputs(&mut self, dt: f32, input: &InputState, speed: f32) {
        if input.right_mouse {
            let forward = Vec3::new(
                self.yaw.cos() * self.pitch.cos(), 
                self.pitch.sin(), 
                self.yaw.sin() * self.pitch.cos()
            ).normalize();
            let right = forward.cross(Vec3::Y).normalize();
            let mut v = Vec3::ZERO;
            if input.key_w { v += forward; }
            if input.key_s { v -= forward; }
            if input.key_a { v -= right; }
            if input.key_d { v += right; }
            if input.key_space { v += Vec3::Y; }
            if input.key_ctrl { v -= Vec3::Y; }
            self.pos += v * speed * dt;
        }
    }
}

// ------------------------------- Input -------------------------------

#[derive(Default)]
struct InputState {
    key_w: bool, 
    key_a: bool, 
    key_s: bool, 
    key_d: bool,
    key_space: bool, 
    key_ctrl: bool,
    right_mouse: bool,
    mouse_delta: Vec2,
    scroll_delta: f32,
}

// ------------------------------- Physics -------------------------------

struct Physics {
    pipeline: r3::PhysicsPipeline,
    gravity: r3::Vector<Real>,
    islands: r3::IslandManager,
    broad: r3::DefaultBroadPhase,
    narrow: r3::NarrowPhase,
    bodies: r3::RigidBodySet,
    colliders: r3::ColliderSet,
    impulse_joints: r3::ImpulseJointSet,
    multibody_joints: r3::MultibodyJointSet,
    ccd: r3::CCDSolver,
    query_pipeline: r3::QueryPipeline,
    integration_params: r3::IntegrationParameters,
}

type Real = f32;

// ------------------------------- Navmesh (toy) -------------------------------
#[derive(Clone)]
struct NavTri { 
    a: Vec3, 
    b: Vec3, 
    c: Vec3 
}

struct NavMesh { 
    tris: Vec<NavTri> 
}

impl NavMesh {
    fn demo() -> Self {
        let base = -2.0;
        let t = |x: f32, z: f32| Vec3::new(x, base, z);
        Self { 
            tris: vec![
                NavTri{ a: t(-5.0, 0.0), b: t(-2.0, 3.0), c: t(1.0, 0.0) },
                NavTri{ a: t(1.0, 0.0), b: t(4.0, 3.0), c: t(7.0, 0.0) },
            ]
        }
    }

    fn path(&self, start: Vec3, goal: Vec3) -> Vec<Vec3> {
        // Toy: straight-line two segments via triangle centers
        let mut path = vec![start];
        for tri in &self.tris {
            let c = (tri.a + tri.b + tri.c) / 3.0;
            path.push(c);
        }
        path.push(goal);
        path
    }
}

// ------------------------------- Main -------------------------------

fn main() -> Result<()> {
    println!("🌟 AstraWeave Unified Showcase Demo");
    println!("📋 Features: 3D Rendering | Physics | Navigation | Audio | UI");
    println!("🎮 Controls: Right-click + move mouse to look, WASD to move, P to pause physics, T to teleport sphere");
    println!("⚡ Starting demo...");
    println!();

    // Window & event loop
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("AstraWeave — Unified Showcase")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?;

    // WGPU setup
    let mut r = pollster::block_on(setup_renderer(&window))?;

    // Physics world
    let mut physics = build_physics_world();

    // Instances: ground props (just visualize some cubes)
    let mut instances = build_show_instances();
    let instance_vb = r.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        label: Some("instances"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
    r.instance_vb = instance_vb;
    r.instance_count = instances.len() as u32;

    // Audio (optional)
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    if let Ok(file) = std::fs::File::open("assets/sound.ogg") {
        if let Ok(source) = Decoder::new(std::io::BufReader::new(file)) {
            sink.append(source);
            sink.set_volume(0.3);
            sink.play();
        }
    }

    // Navmesh demo (unused for now)
    let _nav = NavMesh::demo();
    let mut _nav_goal = Vec3::new(7.0, -2.0, 0.0);

    // Camera
    let mut camera = Camera { 
        pos: Vec3::new(0.0, 2.5, 6.0), 
        yaw: -PI/2.0, 
        pitch: -0.1 
    };
    let mut input = InputState::default();
    let mut ui = UiState::default();

    // Stats
    let mut last = Instant::now();
    let mut fps_acc = 0.0f32;
    let mut fps_cnt = 0u32;

    // Event loop
    event_loop.run(move |event, elwt| {
        match event {
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                if input.right_mouse {
                    input.mouse_delta += Vec2::new(delta.0 as f32, delta.1 as f32);
                }
            }
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::KeyboardInput { event: KeyEvent { physical_key, state, .. }, .. } => {
                        let pressed = state == ElementState::Pressed;
                        match physical_key {
                            PhysicalKey::Code(KeyCode::KeyW) => input.key_w = pressed,
                            PhysicalKey::Code(KeyCode::KeyA) => input.key_a = pressed,
                            PhysicalKey::Code(KeyCode::KeyS) => input.key_s = pressed,
                            PhysicalKey::Code(KeyCode::KeyD) => input.key_d = pressed,
                            PhysicalKey::Code(KeyCode::Space) => input.key_space = pressed,
                            PhysicalKey::Code(KeyCode::ControlLeft) | 
                            PhysicalKey::Code(KeyCode::ControlRight) => input.key_ctrl = pressed,
                            PhysicalKey::Code(KeyCode::Escape) => {
                                if pressed { elwt.exit(); }
                            }
                            PhysicalKey::Code(KeyCode::KeyP) => {
                                if pressed { ui.physics_paused = !ui.physics_paused; }
                            }
                            PhysicalKey::Code(KeyCode::KeyT) => {
                                if pressed { 
                                    teleport_sphere_to(&mut physics, camera.pos + Vec3::new(0.0, 1.0, 0.0));
                                }
                            }
                            _ => {}
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == winit::event::MouseButton::Right {
                            input.right_mouse = state == ElementState::Pressed;
                            // reset accumulated delta when grabbing
                            input.mouse_delta = Vec2::ZERO;
                        }
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        if let MouseScrollDelta::LineDelta(_, y) = delta { 
                            input.scroll_delta = y; 
                        }
                    }
                    WindowEvent::Resized(size) => {
                        r.surface_cfg.width = size.width.max(1);
                        r.surface_cfg.height = size.height.max(1);
                        r.surface.configure(&r.device, &r.surface_cfg);
                        r.depth_view = create_depth(&r.device, r.surface_cfg.width, r.surface_cfg.height, r.msaa_samples);
                    }
                    WindowEvent::RedrawRequested => {
                        let now = Instant::now();
                        let dt = now - last;
                        last = now;

                        // FPS smooth
                        let fps = 1.0 / dt.as_secs_f32().max(1e-5);
                        fps_acc += fps; 
                        fps_cnt += 1;
                        if fps_cnt >= 30 {
                            ui.fps_text = format!("{:.1} fps", fps_acc / fps_cnt as f32);
                            fps_acc = 0.0; 
                            fps_cnt = 0;
                        }

                        // Camera update (mouse look + mov)
                        if input.right_mouse {
                            camera.yaw -= input.mouse_delta.x * 0.0025;
                            camera.pitch -= input.mouse_delta.y * 0.0020;
                            camera.pitch = camera.pitch.clamp(-1.4, 1.4);
                        }
                        input.mouse_delta = Vec2::ZERO;
                        
                        if input.scroll_delta.abs() > 0.1 {
                            ui.camera_speed = (ui.camera_speed + input.scroll_delta).clamp(1.0, 50.0);
                            input.scroll_delta = 0.0;
                        }
                        camera.handle_inputs(dt.as_secs_f32(), &input, ui.camera_speed);

                        // Physics
                        if !ui.physics_paused {
                            physics_step(&mut physics);
                            // keep sphere audible by clamping height
                            let sb = physics.bodies.iter().find(|(_, b)| b.user_data == 2).map(|(_, b)| b.position().translation.vector);
                            if let Some(p) = sb {
                                let _nav_goal = Vec3::new(p.x as f32, -2.0, p.z as f32);
                            }
                        }

                        // Build instance buffer (sync sim → render)
                        sync_instances_from_physics(&physics, &mut instances);
                        r.queue.write_buffer(&r.instance_vb, 0, bytemuck::cast_slice(&instances));

                        // Print status occasionally
                        if fps_cnt == 30 {
                            println!("🌟 AstraWeave Showcase - {} | Camera: ({:.1}, {:.1}, {:.1}) | Physics: {}", 
                                ui.fps_text, camera.pos.x, camera.pos.y, camera.pos.z,
                                if ui.physics_paused { "PAUSED" } else { "RUNNING" });
                        }

                        // Update camera uniform (with resolution scale hack for demo)
                        let width = (r.surface_cfg.width as f32 * ui.resolution_scale).max(1.0);
                        let height = (r.surface_cfg.height as f32 * ui.resolution_scale).max(1.0);
                        let proj = Mat4::perspective_rh(60f32.to_radians(), width/height, 0.01, 5000.0);
                        let view = camera.view_matrix();
                        let cam = GpuCamera { view_proj: (proj * view).to_cols_array() };
                        r.queue.write_buffer(&r.camera_ub, 0, bytemuck::bytes_of(&cam));

                        // ---- Render frame ----
                        let frame = match r.surface.get_current_texture() {
                            Ok(f) => f,
                            Err(_) => { 
                                r.surface.configure(&r.device, &r.surface_cfg); 
                                r.surface.get_current_texture().unwrap() 
                            }
                        };
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let mut encoder = r.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ 
                            label: Some("main-encoder") 
                        });

                        {
                            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                                label: Some("main-pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment{
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations{ 
                                        load: wgpu::LoadOp::Clear(wgpu::Color{r: 0.05, g: 0.07, b: 0.09, a: 1.0}), 
                                        store: wgpu::StoreOp::Store 
                                    },
                                })],
                                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment{
                                    view: &r.depth_view,
                                    depth_ops: Some(wgpu::Operations{ 
                                        load: wgpu::LoadOp::Clear(1.0), 
                                        store: wgpu::StoreOp::Store 
                                    }),
                                    stencil_ops: None,
                                }),
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                            rp.set_pipeline(&r.pipeline);
                            rp.set_bind_group(0, &r.camera_bg, &[]);
                            // instances: cubes (stack + grid markers)
                            rp.set_vertex_buffer(0, r.cube_vb.slice(..));
                            rp.set_vertex_buffer(1, r.instance_vb.slice(..));
                            rp.set_index_buffer(r.cube_ib.slice(..), wgpu::IndexFormat::Uint16);
                            rp.draw_indexed(0..r.cube_index_count, 0, 0..r.instance_count);
                            // Optionally draw grid/navmesh with line list? For brevity omitted;
                            // (You can add a line pipeline to overlay primitives.)
                        }
                        
                        r.queue.submit(Some(encoder.finish()));
                        frame.present();
                    }
                    _ => {}
                }
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}

// ---------------- renderer setup ----------------

async fn setup_renderer(window: &winit::window::Window) -> Result<RenderStuff> {
    let size = window.inner_size();
    let instance = wgpu::Instance::default();
    let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window)?) }?;
    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }
    ).await.unwrap();
    
    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None
    ).await?;
    
    let caps = surface.get_capabilities(&adapter);
    let format = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
    let cfg = wgpu::SurfaceConfiguration{
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &cfg);
    let msaa_samples = 1;

    // depth
    let depth_view = create_depth(&device, cfg.width, cfg.height, msaa_samples);

    // camera UBO + layout
    let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
        label: Some("camera-bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry{
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer{ 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: None 
                },
                count: None
            }
        ]
    });
    let camera_ub = device.create_buffer(&wgpu::BufferDescriptor{
        label: Some("camera-ub"),
        size: std::mem::size_of::<GpuCamera>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let camera_bg = device.create_bind_group(&wgpu::BindGroupDescriptor{
        label: Some("camera-bg"),
        layout: &camera_bgl,
        entries: &[wgpu::BindGroupEntry{ 
            binding: 0, 
            resource: camera_ub.as_entire_binding() 
        }]
    });

    // pipeline
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
        label: Some("shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
        label: Some("pipeline-layout"),
        bind_group_layouts: &[&camera_bgl],
        push_constant_ranges: &[],
    });
    
    let vb_layout_cube = wgpu::VertexBufferLayout{
        array_stride: std::mem::size_of::<[f32; 3]>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3],
    };
    let vb_layout_instance = wgpu::VertexBufferLayout{
        array_stride: std::mem::size_of::<InstanceRaw>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute{ format: wgpu::VertexFormat::Float32x4, offset: 0, shader_location: 1 },
            wgpu::VertexAttribute{ format: wgpu::VertexFormat::Float32x4, offset: 16, shader_location: 2 },
            wgpu::VertexAttribute{ format: wgpu::VertexFormat::Float32x4, offset: 32, shader_location: 3 },
            wgpu::VertexAttribute{ format: wgpu::VertexFormat::Float32x4, offset: 48, shader_location: 4 },
            wgpu::VertexAttribute{ format: wgpu::VertexFormat::Float32x4, offset: 64, shader_location: 5 },
        ],
    };
    
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
        label: Some("pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState{
            module: &shader, 
            entry_point: "vs_main",
            buffers: &[vb_layout_cube, vb_layout_instance],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState{
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: Some(wgpu::Face::Back),
            front_face: wgpu::FrontFace::Ccw,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState{
            format: DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState { 
            count: msaa_samples, 
            ..Default::default() 
        },
        fragment: Some(wgpu::FragmentState{
            module: &shader, 
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState{
                format, 
                blend: Some(wgpu::BlendState::ALPHA_BLENDING), 
                write_mask: wgpu::ColorWrites::ALL
            })],
            compilation_options: Default::default(),
        }),
        multiview: None,
    });

    // cube buffers
    let cube_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        label: Some("cube-vb"),
        contents: bytemuck::cast_slice(CUBE_VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let cube_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
        label: Some("cube-ib"),
        contents: bytemuck::cast_slice(CUBE_INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    // instance vb placeholder
    let instance_vb = device.create_buffer(&wgpu::BufferDescriptor{
        label: Some("instances"),
        size: (std::mem::size_of::<InstanceRaw>() * 1) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // egui wiring (removed for simplicity)

    Ok(RenderStuff{
        surface, 
        surface_cfg: cfg, 
        device, 
        queue, 
        depth_view,
        pipeline, 
        cube_vb, 
        cube_ib, 
        cube_index_count: CUBE_INDICES.len() as u32,
        camera_ub, 
        camera_bg,
        instance_vb, 
        instance_count: 0, 
        msaa_samples
    })
}

fn create_depth(device: &wgpu::Device, w: u32, h: u32, samples: u32) -> wgpu::TextureView {
    let tex = device.create_texture(&wgpu::TextureDescriptor{
        label: Some("depth"),
        size: wgpu::Extent3d{ 
            width: w.max(1), 
            height: h.max(1), 
            depth_or_array_layers: 1 
        },
        mip_level_count: 1, 
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    tex.create_view(&wgpu::TextureViewDescriptor::default())
}

const SHADER: &str = r#"
struct Camera { view_proj: mat4x4<f32> };
@group(0) @binding(0) var<uniform> u_camera: Camera;

struct VsIn {
  @location(0) pos: vec3<f32>,
  // instance mat4 columns
  @location(1) m0: vec4<f32>,
  @location(2) m1: vec4<f32>,
  @location(3) m2: vec4<f32>,
  @location(4) m3: vec4<f32>,
  @location(5) color: vec4<f32>,
};

struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
  let model = mat4x4<f32>(in.m0, in.m1, in.m2, in.m3);
  var out: VsOut;
  out.pos = u_camera.view_proj * model * vec4<f32>(in.pos, 1.0);
  out.color = in.color;
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  return in.color;
}
"#;

// ---------------- physics build/step & instance sync ----------------

fn build_physics_world() -> Physics {
    let mut bodies = r3::RigidBodySet::new();
    let mut colliders = r3::ColliderSet::new();
    let gravity = r3::Vector::new(0.0, -9.81, 0.0);

    // Ground
    let ground = r3::RigidBodyBuilder::fixed().translation(r3::Vector::new(0.0, -2.0, 0.0)).build();
    let g_handle = bodies.insert(ground);
    let g_col = r3::ColliderBuilder::cuboid(100.0, 0.5, 100.0).build();
    colliders.insert_with_parent(g_col, g_handle, &mut bodies);

    // Stack of boxes
    for y in 0..5 {
        for x in 0..5 {
            let rb = r3::RigidBodyBuilder::dynamic()
                .translation(r3::Vector::new(-2.5 + x as f32 * 0.7, -1.0 + y as f32 * 0.7, 0.0))
                .user_data(1).build();
            let h = bodies.insert(rb);
            let c = r3::ColliderBuilder::cuboid(0.3, 0.3, 0.3).restitution(0.2).friction(0.8).build();
            colliders.insert_with_parent(c, h, &mut bodies);
        }
    }

    // Sphere (main "hero prop")
    let s_rb = r3::RigidBodyBuilder::dynamic()
        .translation(r3::Vector::new(1.8, 1.0, 0.0))
        .ccd_enabled(true)
        .user_data(2).build();
    let s_handle = bodies.insert(s_rb);
    let s_col = r3::ColliderBuilder::ball(0.35).restitution(0.5).friction(0.3).build();
    colliders.insert_with_parent(s_col, s_handle, &mut bodies);

    Physics {
        pipeline: r3::PhysicsPipeline::new(),
        gravity,
        islands: r3::IslandManager::new(),
        broad: r3::DefaultBroadPhase::new(),
        narrow: r3::NarrowPhase::new(),
        bodies, 
        colliders, 
        impulse_joints: r3::ImpulseJointSet::new(),
        multibody_joints: r3::MultibodyJointSet::new(),
        ccd: r3::CCDSolver::new(),
        query_pipeline: r3::QueryPipeline::new(),
        integration_params: r3::IntegrationParameters {
            dt: 1.0 / 60.0, 
            ..Default::default()
        },
    }
}

fn physics_step(p: &mut Physics) {
    let hooks = ();
    let events = ();
    p.pipeline.step(
        &p.gravity, &p.integration_params,
        &mut p.islands, &mut p.broad, &mut p.narrow, &mut p.bodies,
        &mut p.colliders, &mut p.impulse_joints, &mut p.multibody_joints, &mut p.ccd, 
        Some(&mut p.query_pipeline), &hooks, &events
    );
}

fn teleport_sphere_to(p: &mut Physics, pos: Vec3) {
    let target = p.bodies.iter_mut().find(|(_, b)| b.user_data == 2);
    if let Some((_, body)) = target {
        body.set_translation(r3::Vector::new(pos.x, pos.y, pos.z), true);
        body.set_linvel(r3::Vector::new(0.0, 0.0, 0.0), true);
        body.set_angvel(r3::Vector::new(0.0, 0.0, 0.0), true);
    }
}

fn sync_instances_from_physics(p: &Physics, out: &mut Vec<InstanceRaw>) {
    // First 25 are boxes, then last is sphere; we map them into instances with colors
    let mut i = 0usize;
    for (_, body) in p.bodies.iter() {
        if body.is_fixed() { continue }
        let xf = body.position();
        let iso = xf.to_homogeneous(); // nalgebra isometry
        let m = Mat4::from_cols_array_2d(&iso.fixed_view::<4, 4>(0, 0).into());
        let color = if body.user_data == 2 { 
            [0.1, 0.8, 0.9, 1.0] // Cyan sphere
        } else { 
            [0.9, 0.6, 0.2, 1.0] // Orange boxes
        };
        if i < out.len() {
            out[i].model = m.to_cols_array();
            out[i].color = color;
            i += 1;
        }
    }
}

fn build_show_instances() -> Vec<InstanceRaw> {
    // Preallocate for 26 dynamic bodies (25 boxes + 1 sphere)
    let mut v = vec![InstanceRaw{ model: [0.0; 16], color: [1.0; 4] }; 26];
    // initial placement (will be overwritten next frame)
    for (i, inst) in v.iter_mut().enumerate() {
        let m = Mat4::from_translation(Vec3::new((i as f32) * 0.1, 0.0, 0.0));
        inst.model = m.to_cols_array();
        inst.color = if i == 25 {
            [0.1, 0.8, 0.9, 1.0] // Cyan sphere
        } else {
            [0.9, 0.6, 0.2, 1.0] // Orange boxes
        };
    }
    v
}