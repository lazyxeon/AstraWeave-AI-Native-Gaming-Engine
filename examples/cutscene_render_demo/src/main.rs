use astraweave_gameplay::cutscene::*;
use astraweave_render::{Camera, CameraController, Renderer};
use glam::{vec3, Vec2};
use std::time::Instant;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = winit::window::WindowBuilder::new()
        .with_title("Cutscene Demo")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop)?;
    let mut renderer = pollster::block_on(Renderer::new(&window))?;
    let mut camera = Camera {
        position: vec3(-3.0, 5.0, 10.0),
        yaw: -1.57,
        pitch: -0.4,
        fovy: 60f32.to_radians(),
        aspect: 16.0 / 9.0,
        znear: 0.1,
        zfar: 500.0,
    };
    let mut ctl = CameraController::new(10.0, 0.005);

    let tl = Timeline {
        cues: vec![
            Cue::Title {
                text: "Veilweaver".into(),
                time: 1.5,
            },
            Cue::Wait { time: 0.5 },
            Cue::CameraTo {
                pos: vec3(0.0, 6.0, 12.0),
                yaw: -1.57,
                pitch: -0.35,
                time: 2.0,
            },
            Cue::CameraTo {
                pos: vec3(2.0, 4.0, 8.0),
                yaw: -1.40,
                pitch: -0.45,
                time: 2.0,
            },
        ],
    };
    let mut cs = CutsceneState::new();
    let mut t = 0.0f32;
    let mut last = Instant::now();
    renderer.set_letterbox(0.12);

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(s) => {
                    renderer.resize(s.width, s.height);
                    camera.aspect = s.width as f32 / s.height.max(1) as f32;
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state,
                            physical_key: PhysicalKey::Code(code),
                            ..
                        },
                    ..
                } => {
                    ctl.process_keyboard(code, state == ElementState::Pressed);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Right {
                        ctl.process_mouse_button(
                            MouseButton::Right,
                            state == ElementState::Pressed,
                        );
                    }
                }
                WindowEvent::CursorMoved { position, .. } => ctl.process_mouse_move(
                    &mut camera,
                    Vec2::new(position.x as f32, position.y as f32),
                ),
                _ => {}
            },
            Event::AboutToWait => {
                let dt = (Instant::now() - last).as_secs_f32();
                last = Instant::now();
                t += dt;
                // run cutscene: override camera but still allow slight input
                let (cam, _title, done) = cs.tick(dt, &tl);
                if let Some((pos, yaw, pitch)) = cam {
                    camera.position = pos;
                    camera.yaw = yaw;
                    camera.pitch = pitch;
                } else {
                    ctl.update_camera(&mut camera, dt);
                }
                // simple fade in first 1.0 sec
                let fade = (1.0 - (t / 1.0)).clamp(0.0, 1.0);
                renderer.set_fade(fade);
                renderer.update_camera(&camera);
                if let Err(e) = renderer.render() {
                    eprintln!("{e:?}");
                }
                window.request_redraw();
                if done && t > 4.5 {
                    renderer.set_letterbox(0.0);
                }
            }
            _ => {}
        }
    })?;
    // Ok(())
}
