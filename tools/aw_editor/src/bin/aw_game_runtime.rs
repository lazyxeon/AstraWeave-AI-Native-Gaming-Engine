//! AstraWeave Game Runtime
//!
//! This is the standalone game runtime binary that runs games created
//! with the AstraWeave Editor. It loads the game project configuration,
//! initializes the engine systems, and runs the game loop.
//!
//! # Usage
//!
//! ```bash
//! # Run from project directory (contains game.toml)
//! aw_game_runtime
//!
//! # Or specify project path
//! aw_game_runtime --project /path/to/game
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use aw_editor_lib::game_project::GameProject;

/// AstraWeave Game Runtime - Runs games built with AstraWeave Engine
#[derive(Parser, Debug)]
#[command(name = "aw_game_runtime")]
#[command(about = "Standalone runtime for AstraWeave games")]
#[command(version)]
struct Args {
    /// Path to game project directory (contains game.toml)
    #[arg(short, long)]
    project: Option<PathBuf>,

    /// Enable development mode (debug overlay, hot reload)
    #[arg(short, long)]
    dev: bool,

    /// Override window width
    #[arg(long)]
    width: Option<u32>,

    /// Override window height
    #[arg(long)]
    height: Option<u32>,

    /// Run in fullscreen mode
    #[arg(short, long)]
    fullscreen: bool,

    /// Skip splash screen
    #[arg(long)]
    skip_splash: bool,
}

/// Game application state
struct GameApp {
    project: GameProject,
    project_path: PathBuf,
    args: Args,
    window: Option<Arc<Window>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    frame_count: u64,
    start_time: std::time::Instant,
}

impl GameApp {
    fn new(project: GameProject, project_path: PathBuf, args: Args) -> Self {
        Self {
            project,
            project_path,
            args,
            window: None,
            device: None,
            queue: None,
            surface: None,
            surface_config: None,
            frame_count: 0,
            start_time: std::time::Instant::now(),
        }
    }

    fn init_gpu(&mut self, window: Arc<Window>) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Week 8: Improved error handling with user-friendly messages
        let surface = match instance.create_surface(window.clone()) {
            Ok(s) => s,
            Err(e) => {
                log::error!("❌ Failed to create graphics surface: {}", e);
                log::error!("   This may indicate a display driver issue.");
                log::error!("   Please ensure your graphics drivers are up to date.");
                panic!("Graphics initialization failed: {}", e);
            }
        };

        let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })) {
            Ok(a) => a,
            Err(e) => {
                log::error!("❌ No compatible graphics adapter found: {}", e);
                log::error!("   AstraWeave requires a GPU with Vulkan, DirectX 12, or Metal support.");
                log::error!("   Please check:");
                log::error!("   - Graphics drivers are installed and up to date");
                log::error!("   - Your GPU supports modern graphics APIs");
                panic!("No compatible GPU found: {}", e);
            }
        };

        let (device, queue) = match pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("game_device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        })) {
            Ok((d, q)) => (d, q),
            Err(e) => {
                log::error!("❌ Failed to create graphics device: {}", e);
                log::error!("   GPU: {}", adapter.get_info().name);
                log::error!("   This may indicate insufficient GPU resources.");
                panic!("GPU device creation failed: {}", e);
            }
        };

        log::info!("🖥️ GPU initialized: {}", adapter.get_info().name);

        // Configure surface
        let size = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        self.window = Some(window);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
    }

    fn render(&mut self) {
        let Some(surface) = &self.surface else { return };
        let Some(device) = &self.device else { return };
        let Some(queue) = &self.queue else { return };

        self.frame_count += 1;

        // Get next frame
        let frame = match surface.get_current_texture() {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed to acquire next frame: {}", e);
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_encoder"),
        });

        // Clear screen to dark color
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        // Print FPS every 60 frames
        if self.frame_count % 60 == 0 {
            let elapsed = self.start_time.elapsed().as_secs_f64();
            let fps = self.frame_count as f64 / elapsed;
            log::debug!("Frame {}: {:.1} FPS", self.frame_count, fps);
        }
    }
}

impl ApplicationHandler for GameApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_width = self.args.width.unwrap_or(1280);
        let window_height = self.args.height.unwrap_or(720);

        let window_attrs = WindowAttributes::default()
            .with_title(&self.project.project.name)
            .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height))
            .with_resizable(true);

        // Week 8: Improved error handling for window creation
        let window = match event_loop.create_window(window_attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                log::error!("❌ Failed to create game window: {}", e);
                log::error!("   Requested size: {}x{}", window_width, window_height);
                log::error!("   This may indicate a display configuration issue.");
                panic!("Window creation failed: {}", e);
            }
        };

        log::info!("🪟 Created window: {}x{}", window_width, window_height);

        self.init_gpu(window);

        // Load entry scene
        let project_root = self.project.project_root(&self.project_path);
        let scene_path = project_root.join(&self.project.build.entry_scene);
        log::info!("🎬 Loading entry scene: {}", scene_path.display());
        log::info!("🎮 Entering game loop...");
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("👋 Game closing...");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    if let (Some(device), Some(surface), Some(config)) =
                        (&self.device, &self.surface, &mut self.surface_config)
                    {
                        config.width = new_size.width;
                        config.height = new_size.height;
                        surface.configure(device, config);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    let args = Args::parse();

    log::info!("🎮 AstraWeave Game Runtime starting...");

    // Find or load game project
    let project_path = if let Some(path) = &args.project {
        path.join("game.toml")
    } else {
        GameProject::find_project_file()
            .context("Could not find game.toml in current directory or parent directories")?
    };

    log::info!("📂 Loading project from: {}", project_path.display());

    let project = GameProject::load(&project_path).context("Failed to load game project")?;

    // Validate project
    project
        .validate()
        .map_err(|errors| anyhow::anyhow!("Project validation failed: {:?}", errors))?;

    log::info!(
        "✅ Loaded project: {} v{}",
        project.project.name,
        project.project.version
    );

    // Create event loop and run
    let event_loop = EventLoop::new()?;
    let mut app = GameApp::new(project, project_path, args);

    event_loop.run_app(&mut app)?;

    Ok(())
}
