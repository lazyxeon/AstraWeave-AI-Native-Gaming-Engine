use astraweave_core::{IVec2, Team, World};
use astraweave_render::{Camera, CameraController, Instance, Renderer};
use glam::{vec3, Vec2};
use std::{sync::Arc, time::Instant};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::PhysicalKey,
};

fn world_to_instances(world: &World, scale: f32) -> Vec<Instance> {
    let mut v = Vec::new();
    // obstacles as gray cubes (representing textured stone blocks)
    for (x, y) in world.obstacles.iter() {
        let pos = vec3(*x as f32 * scale, 0.5, *y as f32 * scale);
        v.push(Instance::from_pos_scale_color(
            pos,
            vec3(0.9, 1.0, 0.9) * 0.9,
            [0.6, 0.6, 0.7, 1.0], // Stone-like color
        ));
    }
    // entities: player (blue), comp (green), enemy (red) - representing characters with textures
    for e in world.all_of_team(0) {
        // player
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(
            vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale),
            vec3(0.7, 1.0, 0.7),
            [0.2, 0.4, 1.0, 1.0],
        ));
    }
    for e in world.all_of_team(1) {
        // companion
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(
            vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale),
            vec3(0.7, 1.0, 0.7),
            [0.2, 1.0, 0.4, 1.0],
        ));
    }
    for e in world.all_of_team(2) {
        // enemies
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(
            vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale),
            vec3(0.7, 1.0, 0.7),
            [1.0, 0.2, 0.2, 1.0],
        ));
    }
    v
}

// Simple texture validation function
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
            "assets/default_n.png"
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
    println!("🎮 AstraWeave Visual 3D Demo - Enhanced with Texture Validation");
    
    // Validate textures before starting graphics
    validate_textures()?;
    
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Veilweaver 3D - Texture-Ready Demo")
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)?,
    );

    // Build a small demo world
    let mut world = World::new();
    for y in 1..=8 {
        world.obstacles.insert((6, y));
    } // vertical wall (will show as stone-like blocks)
    let _player = world.spawn("Player", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let _comp = world.spawn("Companion", IVec2 { x: 3, y: 2 }, Team { id: 1 }, 80, 30);
    let _enemy = world.spawn("Enemy", IVec2 { x: 12, y: 2 }, Team { id: 2 }, 60, 0);

    println!("🖥️  Creating renderer...");
    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;

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

    event_loop
        .run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        renderer.resize(size.width, size.height);
                        camera.aspect = (size.width as f32 / size.height.max(1) as f32).max(0.1);
                    }
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state,
                                physical_key: PhysicalKey::Code(code),
                                ..
                            },
                        ..
                    } => {
                        let pressed = state == ElementState::Pressed;
                        controller.process_keyboard(code, pressed);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        controller.process_mouse_button(button, state == ElementState::Pressed);
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        controller.process_mouse_move(
                            &mut camera,
                            Vec2::new(position.x as f32, position.y as f32),
                        );
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    // update
                    let now = Instant::now();
                    let dt = (now - last).as_secs_f32();
                    last = now;
                    controller.update_camera(&mut camera, dt);
                    renderer.update_camera(&camera);
                    // render
                    if let Err(e) = renderer.render() {
                        eprintln!("render error: {e:?}");
                    }
                    // request next frame
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .map_err(|e| anyhow::anyhow!("Event loop error: {}", e))
}
