use astraweave_audio::{AudioEngine, ListenerPose, MusicTrack};
use astraweave_render::{Camera, CameraController, Renderer};
use glam::{vec3, Vec2};
use std::{sync::Arc, time::Instant};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

fn main() -> anyhow::Result<()> {
    // Window & camera (we’ll drive the listener from the camera)
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Audio Spatial Demo")
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)?,
    );
    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;
    let mut camera = Camera {
        position: vec3(0.0, 2.0, 6.0),
        yaw: -1.57,
        pitch: -0.2,
        fovy: 60f32.to_radians(),
        aspect: 16.0 / 9.0,
        znear: 0.1,
        zfar: 300.0,
    };
    let mut cam_ctl = CameraController::new(12.0, 0.005);

    // Audio engine
    let mut audio = AudioEngine::new()?;
    audio.set_master_volume(1.0);

    // Try to play BGM (will error if file missing; safe to comment or replace)
    let _ = audio.play_music(
        MusicTrack {
            path: "assets/audio/bgm.ogg".into(),
            looped: true,
        },
        1.0,
    );

    let mut last = Instant::now();
    let mut emitter_id: u64 = 1;

    event_loop
        .run(move |event, elwt| {
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
                        cam_ctl.process_keyboard(code, state == ElementState::Pressed);
                        if state == ElementState::Pressed {
                            match code {
                                KeyCode::Digit1 => {
                                    // center beep
                                    let _ = audio.play_sfx_3d_beep(
                                        100,
                                        vec3(0.0, 1.0, 0.0),
                                        880.0,
                                        0.25,
                                        0.5,
                                    );
                                }
                                KeyCode::Digit2 => {
                                    // left beep
                                    let _ = audio.play_sfx_3d_beep(
                                        101,
                                        vec3(-3.0, 1.0, 0.0),
                                        660.0,
                                        0.25,
                                        0.5,
                                    );
                                }
                                KeyCode::Digit3 => {
                                    // right beep
                                    let _ = audio.play_sfx_3d_beep(
                                        102,
                                        vec3(3.0, 1.0, 0.0),
                                        440.0,
                                        0.25,
                                        0.5,
                                    );
                                }
                                KeyCode::KeyM => {
                                    // switch music (crossfade)
                                    let _ = audio.play_music(
                                        MusicTrack {
                                            path: "assets/audio/bgm_alt.ogg".into(),
                                            looped: true,
                                        },
                                        1.25,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Right {
                            cam_ctl.process_mouse_button(
                                MouseButton::Right,
                                state == ElementState::Pressed,
                            );
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => cam_ctl.process_mouse_move(
                        &mut camera,
                        Vec2::new(position.x as f32, position.y as f32),
                    ),
                    _ => {}
                },
                Event::AboutToWait => {
                    let dt = (Instant::now() - last).as_secs_f32();
                    last = Instant::now();
                    cam_ctl.update_camera(&mut camera, dt);
                    renderer.update_camera(&camera);

                    // update listener from camera (Y-up, forward from yaw/pitch)
                    let forward =
                        glam::Quat::from_euler(glam::EulerRot::YXZ, camera.yaw, camera.pitch, 0.0)
                            * vec3(0.0, 0.0, -1.0);
                    audio.update_listener(ListenerPose {
                        position: camera.position,
                        forward,
                        up: vec3(0.0, 1.0, 0.0),
                    });
                    audio.tick(dt);

                    if let Err(e) = renderer.render() {
                        eprintln!("{e:?}");
                    }
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .map_err(|e| anyhow::anyhow!("Event loop error: {}", e))
}
