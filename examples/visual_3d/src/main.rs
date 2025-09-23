use astraweave_core::{IVec2, Team, World};
use astraweave_render::{Camera, CameraController, Instance, Renderer, SkyConfig};
use glam::{vec3, Vec2};
use std::{sync::Arc, time::Instant};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::PhysicalKey,
};
use egui::{Context as EguiContext, Slider};
use egui_winit::State as EguiWinitState;
use egui_wgpu::Renderer as EguiRenderer;

fn world_to_instances(world: &World, scale: f32) -> Vec<Instance> {
    let mut v = Vec::new();
    for (x, y) in world.obstacles.iter() {
        let pos = vec3(*x as f32 * scale, 0.5, *y as f32 * scale);
        v.push(Instance::from_pos_scale_color(pos, vec3(0.9, 1.0, 0.9) * 0.9, [0.6, 0.6, 0.7, 1.0]));
    }
    for e in world.all_of_team(0) {
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale), vec3(0.7, 1.0, 0.7), [0.2, 0.4, 1.0, 1.0]));
    }
    for e in world.all_of_team(1) {
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale), vec3(0.7, 1.0, 0.7), [0.2, 1.0, 0.4, 1.0]));
    }
    for e in world.all_of_team(2) {
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale), vec3(0.7, 1.0, 0.7), [1.0, 0.2, 0.2, 1.0]));
    }
    v
}

fn validate_textures() -> anyhow::Result<()> {
    #[cfg(feature = "textures")]
    {
        let texture_files = [
            "assets/grass.png",
            "assets/dirt.png",
            "assets/stone.png",
            "assets/grass_n.png",
            "assets/dirt_n.png",
            "assets/stone_n.png",
            "assets/default_n.png",
        ];
        astraweave_render::texture::validate_texture_assets(&texture_files)
    }
    #[cfg(not(feature = "textures"))]
    {
        println!("🎨 Texture validation skipped (textures feature not enabled)");
        println!("✅ Visual demo will use basic colored primitives");
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    println!("🎮 AstraWeave Visual 3D Demo - with UI controls");
    validate_textures()?;

    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Veilweaver 3D - UI Overlay Demo")
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)?,
    );

    // World
    let mut world = World::new();
    for y in 1..=8 {
        world.obstacles.insert((6, y));
    } // vertical wall (will show as stone-like blocks)
    let _player = world.spawn("Player", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let _comp = world.spawn("Companion", IVec2 { x: 3, y: 2 }, Team { id: 1 }, 80, 30);
    let _enemy = world.spawn("Enemy", IVec2 { x: 12, y: 2 }, Team { id: 2 }, 60, 0);

    println!("🖥️  Creating renderer...");
    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;

    // Egui setup
    let egui_ctx = EguiContext::default();
    let mut egui_state = EguiWinitState::new(egui_ctx.clone(), egui::ViewportId::default(), &*window, None, None);
    let surface_format = renderer.surface_format();
    let mut egui_rend = EguiRenderer::new(renderer.device(), surface_format, None, 1);

    let mut camera = Camera {
        position: vec3(0.0, 8.0, 12.0),
        yaw: -3.14 / 2.0,
        pitch: -0.6,
        fovy: 60f32.to_radians(),
        aspect: 16.0 / 9.0,
        znear: 0.1,
        zfar: 200.0,
    };
    let mut controller = CameraController::new(10.0, 0.005);

    let grid_scale = 1.5f32;
    renderer.update_instances(&world_to_instances(&world, grid_scale));
    renderer.update_camera(&camera);

    let mut last = Instant::now();
    
    println!("✅ Setup complete! Note: This demo uses basic colored primitives.");
    println!("   For full texture rendering, use the unified_showcase example.");
    println!("   Controls: WASD + mouse to move camera");

    // Note: glTF loading stub is available in astraweave-asset, but not wired here to avoid feature churn

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => {
                    // Pass event to egui first
                    let _ = egui_state.on_window_event(&window, &event);
                    match event {
                        WindowEvent::Resized(size) => {
                            renderer.resize(size.width, size.height);
                            camera.aspect = (size.width as f32 / size.height.max(1) as f32).max(0.1);
                        }
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::KeyboardInput { event: KeyEvent { state, physical_key: PhysicalKey::Code(code), .. }, .. } => {
                            let pressed = state == ElementState::Pressed;
                            controller.process_keyboard(code, pressed);
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            controller.process_mouse_button(button, state == ElementState::Pressed);
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            controller.process_mouse_move(&mut camera, Vec2::new(position.x as f32, position.y as f32));
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    // update
                    let now = Instant::now();
                    let dt = (now - last).as_secs_f32();
                    last = now;
                    controller.update_camera(&mut camera, dt);
                    renderer.tick_environment(dt);
                    renderer.update_camera(&camera);

                    // Build egui UI using run() API
                    let mut hour: f32 = {
                        let t = renderer.time_of_day_mut().current_time;
                        t
                    };
                    let mut sky_cfg: SkyConfig = renderer.sky_config();

                    let raw_input = egui_state.take_egui_input(&window);
                    let egui_output = egui_ctx.run(raw_input, |ctx| {
                        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("AstraWeave Visual 3D Controls");
                                if ui.button("Reset Time").clicked() {
                                    hour = 12.0;
                                }
                            });
                        });
                        egui::Window::new("Environment").show(ctx, |ui| {
                            ui.add(Slider::new(&mut hour, 0.0..=24.0).text("Hour"));
                            ui.add(Slider::new(&mut sky_cfg.cloud_coverage, 0.0..=1.0).text("Clouds"));
                            ui.add(Slider::new(&mut sky_cfg.cloud_speed, 0.0..=5.0).text("Cloud Speed"));
                        });
                    });

                    // apply any UI-driven changes
                    renderer.time_of_day_mut().current_time = hour;
                    renderer.set_sky_config(sky_cfg);

                    egui_state.handle_platform_output(&*window, egui_output.platform_output.clone());

                    let shapes = egui_ctx.tessellate(egui_output.shapes.clone(), egui_output.pixels_per_point);

                    // render 3D + UI
                    let _ = renderer.render_with(|view, enc, device, queue, (width, height)| {
                        // Upload egui textures and paint
                        for (id, delta) in &egui_output.textures_delta.set {
                            egui_rend.update_texture(device, queue, *id, delta);
                        }
                        let screen_desc = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [width, height],
                            pixels_per_point: window.scale_factor() as f32,
                        };
                        {
                            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("egui pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view,
                                    resolve_target: None,
                                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });
                            egui_rend.render(&mut rpass, &shapes, &screen_desc);
                        }
                        for id in &egui_output.textures_delta.free {
                            egui_rend.free_texture(id);
                        }
                    });

                    // request next frame
                    window.request_redraw();
                }
                _ => {}
            }
        })
        ?;
    Ok(())
}
