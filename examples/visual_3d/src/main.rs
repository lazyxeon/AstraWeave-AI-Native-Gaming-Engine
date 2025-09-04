use std::time::Instant;
use glam::{vec3, Mat4, Vec2, Vec3};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent, ElementState, KeyEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
};
use astraweave_core::{World, IVec2, Team};
use astraweave_render::{Renderer, Camera, CameraController, Instance};

fn world_to_instances(world: &World, scale: f32) -> Vec<Instance> {
    let mut v = Vec::new();
    // obstacles as gray cubes
    for (x,y) in world.obstacles.iter() {
        let pos = vec3(*x as f32 * scale, 0.5, *y as f32 * scale);
        v.push(Instance::from_pos_scale_color(pos, vec3(0.9,1.0,0.9)*0.9, [0.5,0.5,0.5,1.0]));
    }
    // entities: player (blue), comp (green), enemy (red)
    for e in world.all_of_team(0) { // player
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale), vec3(0.7,1.0,0.7), [0.2,0.4,1.0,1.0]));
    }
    for e in world.all_of_team(1) { // companion
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale), vec3(0.7,1.0,0.7), [0.2,1.0,0.4,1.0]));
    }
    for e in world.all_of_team(2) { // enemies
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale), vec3(0.7,1.0,0.7), [1.0,0.2,0.2,1.0]));
    }
    v
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = winit::window::WindowBuilder::new()
        .with_title("Veilweaver 3D")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop)?;

    // Build a small demo world
    let mut world = World::new();
    for y in 1..=8 { world.obstacles.insert((6,y)); }  // vertical wall
    let player = world.spawn("Player",   IVec2{ x:2,  y:2 }, Team{ id:0 }, 100, 0);
    let comp   = world.spawn("Companion",IVec2{ x:3,  y:2 }, Team{ id:1 }, 80,  30);
    let enemy  = world.spawn("Enemy",    IVec2{ x:12, y:2 }, Team{ id:2 }, 60,  0);

    let mut renderer = pollster::block_on(Renderer::new(&window))?;

    let mut camera = Camera {
        position: vec3(0.0, 8.0, 12.0),
        yaw: -3.14/2.0,
        pitch: -0.6,
        fovy: 60f32.to_radians(),
        aspect: 16.0/9.0,
        znear: 0.1,
        zfar: 200.0,
    };
    let mut controller = CameraController::new(10.0, 0.005);

    let grid_scale = 1.5f32;
    renderer.update_instances(&world_to_instances(&world, grid_scale));
    renderer.update_camera(&camera);

    let mut last = Instant::now();

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::Resized(size) => {
                        renderer.resize(size.width, size.height);
                        camera.aspect = (size.width as f32 / size.height.max(1) as f32).max(0.1);
                    }
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::KeyboardInput { event: KeyEvent{ state, physical_key: PhysicalKey::Code(code), .. }, .. } => {
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
    })?;
    // (never reached)
    // Ok(())
}
