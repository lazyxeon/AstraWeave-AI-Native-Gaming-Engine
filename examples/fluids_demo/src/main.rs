use astraweave_fluids::FluidSystem;
use astraweave_physics::PhysicsWorld;
use glam::{Mat4, Vec3};
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

mod fluid_renderer;
mod ocean_renderer;
mod scenarios;
mod skybox_renderer;

use scenarios::{LaboratoryScenario, OceanScenario, ScenarioManager};

use fluid_renderer::FluidRenderer;
use skybox_renderer::SkyboxRenderer;

struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy.to_radians(), self.aspect, self.znear, self.zfar);
        proj * view
    }
}

// RenderMode is now handled by ScenarioManager

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: std::sync::Arc<winit::window::Window>,

    physics_world: PhysicsWorld,
    fluid_system: FluidSystem,
    fluid_renderer: FluidRenderer,
    scenario_manager: ScenarioManager,
    skybox_renderer: Option<SkyboxRenderer>,

    camera: Camera,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,

    last_update: Instant,
}

impl State {
    fn toggle_render_mode(&mut self) {
        self.scenario_manager.next();
        if let Some(scenario) = self.scenario_manager.current() {
            log::info!("Switching to scenario: {}", scenario.name());
            scenario.init(&self.device, &self.queue, &mut self.fluid_system);
        }
    }

    async fn new(window: std::sync::Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
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

        // Initialize physics world with gravity
        let physics_world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

        // Initialize fluid system with 20000 particles
        let particle_count = 20000;
        let mut fluid_system = FluidSystem::new(&device, particle_count);

        // The fluid system parameters are public fields, so we can set them directly
        fluid_system.smoothing_radius = 0.5;
        fluid_system.target_density = 1.0;
        fluid_system.pressure_multiplier = 100.0;
        fluid_system.viscosity = 0.01;
        fluid_system.gravity = -9.81;
        fluid_system.cell_size = 1.2;
        fluid_system.grid_width = 64;
        fluid_system.grid_height = 64;
        fluid_system.grid_depth = 64;

        log::info!("Initialized fluid system with {} particles", particle_count);

        // Initialize scenarios
        let mut scenario_manager = ScenarioManager::new();
        scenario_manager.add_scenario(Box::new(LaboratoryScenario::new()));
        scenario_manager.add_scenario(Box::new(OceanScenario::new(
            &device,
            &queue,
            surface_format,
        )));

        if let Some(scenario) = scenario_manager.current() {
            scenario.init(&device, &queue, &mut fluid_system);
        }

        // Initialize skybox renderer
        let skybox_path = "assets/hdri/polyhaven/kloppenheim_02_puresky_2k.hdr";
        let skybox_renderer = if std::path::Path::new(skybox_path).exists() {
            Some(SkyboxRenderer::new(
                &device,
                &queue,
                surface_format,
                skybox_path,
            ))
        } else {
            None
        };

        // Initialize fluid renderer
        let max_particles = particle_count;
        let fluid_renderer = FluidRenderer::init(&device, surface_format, max_particles);

        // Setup camera
        let camera = Camera {
            eye: Vec3::new(8.0, 6.0, 8.0),
            target: Vec3::new(3.2, 3.2, 3.2),
            up: Vec3::Y,
            aspect: size.width as f32 / size.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            physics_world,
            fluid_system,
            fluid_renderer,
            scenario_manager,
            skybox_renderer,
            camera,
            depth_texture,
            depth_view,
            last_update: Instant::now(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

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

            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_update).as_secs_f32();
        self.last_update = now;

        // Cap dt to avoid instability
        let dt = dt.min(0.016);

        // Create encoder for fluid simulation
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Fluid Update Encoder"),
            });

        // Update fluid simulation
        self.fluid_system.step(&mut encoder, &self.queue, dt);

        // Update current scenario
        if let Some(scenario) = self.scenario_manager.current() {
            scenario.update(
                dt,
                &mut self.fluid_system,
                &mut self.physics_world,
                self.camera.eye,
            );
        }

        // Submit encoder
        self.queue.submit(std::iter::once(encoder.finish()));

        log::debug!(
            "Frame time: {:.3}ms, Particles: {}",
            dt * 1000.0,
            self.fluid_system.particle_count
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

        // Clear pass
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
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
        }

        // Render based on mode
        let view_proj = self.camera.build_view_projection_matrix();

        // Render skybox
        /*
        if let Some(ref skybox) = self.skybox_renderer {
            skybox.render(
                &mut encoder,
                &view,
                &self.depth_view,
                &self.queue,
                view_proj,
                self.camera.eye,
            );
        }
        */

        // Render current scenario
        if let Some(scenario) = self.scenario_manager.current() {
            let skybox_view = if let Some(ref skybox) = self.skybox_renderer {
                skybox.get_skybox_view()
            } else {
                &self.depth_view // DUMMY
            };

            scenario.render(
                &mut encoder,
                &view,
                &self.depth_view,
                &self.device,
                &self.queue,
                &self.fluid_system,
                view_proj,
                skybox_view,
            );
        }

        // Render fluid particles (using current renderer for now)
        let particle_buffer = self.fluid_system.get_particle_buffer();
        if let Some(ref skybox) = self.skybox_renderer {
            self.fluid_renderer.render(
                &mut encoder,
                &view,
                &self.depth_view,
                &self.device,
                &self.queue,
                particle_buffer,
                self.fluid_system.particle_count,
                view_proj,
                skybox.get_skybox_view(),
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window = std::sync::Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes().with_title("AstraWeave Fluids Demo"),
                    )
                    .unwrap(),
            );
            self.state = Some(pollster::block_on(State::new(window)));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = &mut self.state {
            if window_id == state.window.id() {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => event_loop.exit(),
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Space),
                                ..
                            },
                        ..
                    } => {
                        state.toggle_render_mode();
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
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
