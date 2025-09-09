use astraweave_nav::{NavMesh, Triangle};
use astraweave_render::{Camera, CameraController, Instance, Renderer};
use glam::{vec3, Vec2};
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::PhysicalKey,
};

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("NavMesh Demo")
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)?,
    );
    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;
    let mut camera = Camera {
        position: vec3(0.0, 10.0, 16.0),
        yaw: -3.14 / 2.0,
        pitch: -0.5,
        fovy: 60f32.to_radians(),
        aspect: 16.0 / 9.0,
        znear: 0.1,
        zfar: 500.0,
    };
    let mut cam_ctl = CameraController::new(10.0, 0.005);

    // Make a small "heightfield" of walkable tris with a ramp:
    let tris = vec![
        tri(
            vec3(-4.0, 0.0, -4.0),
            vec3(4.0, 0.0, -4.0),
            vec3(4.0, 0.0, 4.0),
        ),
        tri(
            vec3(-4.0, 0.0, -4.0),
            vec3(4.0, 0.0, 4.0),
            vec3(-4.0, 0.0, 4.0),
        ),
        // ramp up
        tri(
            vec3(4.0, 0.0, -1.0),
            vec3(8.0, 0.8, -1.0),
            vec3(8.0, 0.8, 1.0),
        ),
        tri(
            vec3(4.0, 0.0, -1.0),
            vec3(8.0, 0.8, 1.0),
            vec3(4.0, 0.0, 1.0),
        ),
        // plateau
        tri(
            vec3(8.0, 0.8, -1.0),
            vec3(12.0, 0.8, -1.0),
            vec3(12.0, 0.8, 1.0),
        ),
        tri(
            vec3(8.0, 0.8, -1.0),
            vec3(12.0, 0.8, 1.0),
            vec3(8.0, 0.8, 1.0),
        ),
    ];
    let nav = NavMesh::bake(&tris, 0.4, 50.0); // 50° slope allowed

    let start = vec3(-3.5, 0.0, -3.5);
    let goal = vec3(11.5, 0.8, 0.0);
    let path = nav.find_path(start, goal);

    let mut instances = vec![];

    // visualize tri centers
    for t in &nav.tris {
        instances.push(Instance::from_pos_scale_color(
            t.center + vec3(0.0, 0.05, 0.0),
            vec3(0.1, 0.1, 0.1),
            [0.7, 0.7, 0.3, 1.0],
        ));
    }
    // visualize path
    for p in &path {
        instances.push(Instance::from_pos_scale_color(
            *p + vec3(0.0, 0.08, 0.0),
            vec3(0.12, 0.12, 0.12),
            [0.2, 1.0, 0.4, 1.0],
        ));
    }

    renderer.update_instances(&instances);
    renderer.update_camera(&camera);

    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::Resized(s) => {
                renderer.resize(s.width, s.height);
                camera.aspect = s.width as f32 / s.height.max(1) as f32;
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
                cam_ctl.process_keyboard(code, state == ElementState::Pressed);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Right {
                    cam_ctl
                        .process_mouse_button(MouseButton::Right, state == ElementState::Pressed);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                cam_ctl.process_mouse_move(
                    &mut camera,
                    Vec2::new(position.x as f32, position.y as f32),
                );
            }
            _ => {}
        },
        Event::AboutToWait => {
            cam_ctl.update_camera(&mut camera, 1.0 / 60.0);
            renderer.update_camera(&camera);
            if let Err(e) = renderer.render() {
                eprintln!("{e:?}");
            }
            window.request_redraw();
        }
        _ => {}
    })?;
    Ok(())
}

#[inline]
fn tri(a: glam::Vec3, b: glam::Vec3, c: glam::Vec3) -> Triangle {
    Triangle { a, b, c }
}
