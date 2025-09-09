use astraweave_core::{ActionStep, IVec2, PlanIntent, Team, World};
use astraweave_render::{Camera, CameraController, Renderer};
use aw_debug::{PerfHud, ChromeTraceGuard, watch_scripts, watch_reload_signal};
use std::{path::PathBuf, time::Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct App {
    world: World,
    player: u32,
    comp: u32,
    enemy: u32,
    plan: Option<PlanIntent>,
    
    // Debug toolkit integration
    hud: PerfHud,
    last_update: Instant,
    system_timers: Vec<(String, f32)>,
}

impl App {
    fn new() -> Self {
        let mut world = World::new();
        // wall
        for y in 1..=8 {
            world.obstacles.insert((6, y));
        }
        let player = world.spawn("Player", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
        let comp = world.spawn("Comp", IVec2 { x: 2, y: 3 }, Team { id: 1 }, 80, 30);
        let enemy = world.spawn("Enemy", IVec2 { x: 12, y: 2 }, Team { id: 2 }, 60, 0);
        // trivial plan just to show rendering
        let plan = Some(PlanIntent {
            plan_id: "viz".into(),
            steps: vec![
                ActionStep::MoveTo { x: 4, y: 2 },
                ActionStep::Throw {
                    item: "smoke".into(),
                    x: 7,
                    y: 2,
                },
                ActionStep::CoverFire {
                    target_id: enemy,
                    duration: 2.0,
                },
            ],
        });
        
        // Initialize debug HUD
        let mut hud = PerfHud::new();
        hud.entity_count = 3; // player, companion, enemy
        
        // Example system timers
        let system_timers = vec![
            ("physics".into(), 0.5),
            ("ai_planning".into(), 1.2),
            ("rendering".into(), 2.0),
            ("input".into(), 0.1),
        ];
        hud.systems_snapshot = system_timers.clone();
        
        // Log initial events
        hud.log_event("system", "Application started");
        hud.log_event("world", "World initialized with 3 entities");
        
        Self {
            world,
            player,
            comp,
            enemy,
            plan,
            hud,
            last_update: Instant::now(),
            system_timers,
        }
    }
    
    fn update(&mut self) {
        // Simulate system updates and track timing
        let start = Instant::now();
        
        // Physics update
        std::thread::sleep(std::time::Duration::from_millis(1));
        self.system_timers[0].1 = start.elapsed().as_secs_f32() * 1000.0;
        
        // AI planning
        std::thread::sleep(std::time::Duration::from_millis(2));
        self.system_timers[1].1 = (start.elapsed().as_secs_f32() * 1000.0) - self.system_timers[0].1;
        
        // Update HUD with latest system timings
        self.hud.systems_snapshot = self.system_timers.clone();
        
        // Occasionally log events
        if rand::random::<f32>() < 0.05 {
            let events = [
                ("ai", "Companion evaluating plan options"),
                ("physics", "Collision resolved"),
                ("world", "Entity position updated"),
            ];
            let (category, msg) = events[rand::random::<usize>() % events.len()];
            self.hud.log_event(category, msg);
        }
        
        // Update frame timing in HUD
        self.hud.frame();
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize Chrome tracing
    let _trace_guard = ChromeTraceGuard::init("astraweave_demo_trace.json");
    
    // Set up content directory watchers
    let content_dir = PathBuf::from("content");
    std::fs::create_dir_all(&content_dir).ok();
    
    let _script_watcher = watch_scripts(content_dir.join("encounters"), || {
        println!("Script changed, reloading...");
        // In a real app, you would reload your scripts here
    }).ok();
    
    let _reload_watcher = watch_reload_signal(content_dir.clone(), || {
        println!("Reload signal detected, reloading level...");
        // In a real app, you would reload your level here
    }).ok();
    
    // Create window and event loop
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("AstraWeave Debug Toolkit Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?;
    
    // Initialize renderer
    let mut renderer = pollster::block_on(Renderer::new(&window))?;
    
    // Set up camera
    let mut camera = Camera::new(
        glam::Vec3::new(0.0, 5.0, 10.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        window.inner_size().width as f32 / window.inner_size().height as f32,
    );
    let mut camera_controller = CameraController::new(0.2);
    
    // Set up egui integration
    let mut egui_platform = egui_winit::State::new(egui::ViewportId::default(), &window, None, None);
    let mut egui_renderer = egui_wgpu::Renderer::new(
        &renderer.device,
        renderer.config.format,
        None,
        1,
    );
    let mut egui_ctx = egui::Context::default();
    
    // Create our app
    let mut app = App::new();
    
    // Run the event loop
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        // Handle egui events
        let _ = egui_platform.on_event(&egui_ctx, &event);
        
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                elwt.exit();
            }
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                if size.width > 0 && size.height > 0 {
                    renderer.resize(size.width, size.height);
                    camera.aspect = size.width as f32 / size.height as f32;
                }
            }
            Event::WindowEvent { event, .. } => {
                camera_controller.process_events(&event);
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                // Update app state
                app.update();
                
                // Update camera
                camera_controller.update_camera(&mut camera);
                
                // Begin rendering
                let output = renderer.surface.get_current_texture().unwrap();
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                
                let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
                
                // Clear the screen
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Main Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    
                    // Here you would render your 3D scene
                }
                
                // Render egui
                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [window.inner_size().width, window.inner_size().height],
                    pixels_per_point: window.scale_factor() as f32,
                };
                
                egui_ctx.begin_frame(egui_platform.take_egui_input(&window));
                
                // Create our debug window
                egui::Window::new("Debug HUD")
                    .default_pos([10.0, 10.0])
                    .default_width(350.0)
                    .show(&egui_ctx, |ui| {
                        app.hud.ui(ui);
                    });
                
                let egui_output = egui_ctx.end_frame();
                let paint_jobs = egui_ctx.tessellate(egui_output.shapes, egui_output.pixels_per_point);
                
                let user_textures = egui_renderer.paint_jobs(
                    &renderer.device,
                    &renderer.queue,
                    paint_jobs,
                    &screen_descriptor,
                );
                
                // Render the egui output
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Egui Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    
                    egui_renderer.render(&mut render_pass, paint_jobs, &screen_descriptor);
                }
                
                // Submit the work
                renderer.queue.submit(std::iter::once(encoder.finish()));
                output.present();
                
                // Update system render time
                app.system_timers[2].1 = app.last_update.elapsed().as_secs_f32() * 1000.0;
                app.last_update = Instant::now();
            }
            _ => {}
        }
    })?;
    
    Ok(())
}