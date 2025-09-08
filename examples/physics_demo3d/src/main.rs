use astraweave_physics::{Layers, PhysicsWorld};
use astraweave_render::{Camera, CameraController, Instance, Renderer};
use glam::{vec3, Vec2, Vec3};
use std::{sync::Arc, time::Instant};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("AstraWeave Physics Demo")
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)?,
    );

    // Renderer
    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;
    let mut camera = Camera {
        position: vec3(0.0, 8.0, 16.0),
        yaw: -3.14 / 2.0,
        pitch: -0.45,
        fovy: 60f32.to_radians(),
        aspect: 16.0 / 9.0,
        znear: 0.1,
        zfar: 500.0,
    };
    let mut cam_ctl = CameraController::new(10.0, 0.005);

    // Physics world
    let mut phys = PhysicsWorld::new(vec3(0.0, -9.81, 0.0));
    let _ground = phys.create_ground_plane(vec3(100.0, 0.0, 100.0), 1.0);

    // A climbable wall (static)
    let _wall = phys.add_static_trimesh(
        &[
            vec3(5.0, 0.0, 0.0),
            vec3(5.0, 3.0, 0.0),
            vec3(5.0, 0.0, 3.0),
            vec3(5.0, 3.0, 3.0),
            vec3(5.0, 0.0, 3.0),
            vec3(5.0, 3.0, 0.0),
        ],
        &[[0, 1, 2], [3, 2, 1]],
        Layers::CHARACTER | Layers::DEFAULT,
    );

    // Character (kinematic)
    let char_id = phys.add_character(vec3(-2.0, 1.0, 0.0), vec3(0.4, 0.9, 0.4));

    // Destructible demo crate
    let mut destruct_ids: Vec<u64> = vec![];
    destruct_ids.push(phys.add_destructible_box(
        vec3(-1.0, 1.0, 2.0),
        vec3(0.4, 0.4, 0.4),
        3.0,
        50.0,
        12.0,
    ));

    // Water pool toggle
    let mut water_on = true;
    phys.add_water_aabb(vec3(-2.0, 0.0, -2.0), vec3(2.0, 1.2, 2.0), 1000.0, 0.8);

    // Wind toggle
    let mut wind_on = false;

    // Instances
    let mut instances: Vec<Instance> = vec![];

    // Char input
    let mut move_dir = Vec3::ZERO;
    let mut climb_try = false;

    let mut last = Instant::now();

    event_loop.run(move |event, elwt| {
        match event {
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
                    let down = state == ElementState::Pressed;
                    match code {
                        KeyCode::KeyW => cam_ctl.process_keyboard(code, down),
                        KeyCode::KeyS => cam_ctl.process_keyboard(code, down),
                        KeyCode::KeyA => cam_ctl.process_keyboard(code, down),
                        KeyCode::KeyD => cam_ctl.process_keyboard(code, down),
                        KeyCode::Space => cam_ctl.process_keyboard(code, down),
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                            cam_ctl.process_keyboard(code, down)
                        }

                        // Character control (J/K/L/I)
                        KeyCode::KeyJ => {
                            if down {
                                move_dir.x = -2.5;
                            } else {
                                move_dir.x = 0.0;
                            }
                        }
                        KeyCode::KeyL => {
                            if down {
                                move_dir.x = 2.5;
                            } else {
                                move_dir.x = 0.0;
                            }
                        }
                        KeyCode::KeyI => {
                            if down {
                                move_dir.z = -2.5;
                            } else {
                                move_dir.z = 0.0;
                            }
                        }
                        KeyCode::KeyK => {
                            if down {
                                move_dir.z = 2.5;
                            } else {
                                move_dir.z = 0.0;
                            }
                        }
                        KeyCode::KeyC => {
                            climb_try = down;
                        }

                        // Wind toggle
                        KeyCode::KeyT if down => {
                            wind_on = !wind_on;
                            if wind_on {
                                phys.set_wind(vec3(1.0, 0.0, 0.2).normalize(), 8.0);
                            } else {
                                phys.set_wind(vec3(0.0, 0.0, 0.0), 0.0);
                            }
                            println!("Wind: {}", if wind_on { "ON" } else { "OFF" });
                        }

                        // Water toggle
                        KeyCode::KeyG if down => {
                            water_on = !water_on;
                            if water_on {
                                phys.add_water_aabb(
                                    vec3(-2.0, 0.0, -2.0),
                                    vec3(2.0, 1.2, 2.0),
                                    1000.0,
                                    0.8,
                                );
                            } else {
                                phys.clear_water();
                            }
                            println!("Water: {}", if water_on { "ON" } else { "OFF" });
                        }

                        // Drop dynamic box
                        KeyCode::KeyF if down => {
                            phys.add_dynamic_box(
                                vec3(0.0, 4.0, 0.0),
                                vec3(0.3, 0.3, 0.3),
                                1.0,
                                Layers::DEFAULT,
                            );
                        }

                        // Spawn ragdoll (using dynamic box as placeholder)
                        KeyCode::KeyB if down => {
                            let _rag = phys.add_dynamic_box(
                                vec3(0.0, 1.2, -1.5),
                                vec3(0.2, 0.5, 0.2),
                                70.0,
                                Layers::DEFAULT,
                            );
                            println!("Spawned ragdoll (box placeholder)");
                        }

                        // Spawn destructible
                        KeyCode::KeyN if down => {
                            let id = phys.add_destructible_box(
                                vec3(-0.5, 1.0, -1.0),
                                vec3(0.4, 0.4, 0.4),
                                3.0,
                                60.0,
                                14.0,
                            );
                            destruct_ids.push(id);
                            println!("Spawned destructible");
                        }

                        // Force-break nearest
                        KeyCode::KeyM if down => {
                            if let Some(id) = destruct_ids.pop() {
                                phys.break_destructible(id);
                                println!("Break destructible");
                            }
                        }

                        _ => {}
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
                WindowEvent::CursorMoved { position, .. } => {
                    cam_ctl.process_mouse_move(
                        &mut camera,
                        Vec2::new(position.x as f32, position.y as f32),
                    );
                }
                _ => {}
            },
            Event::AboutToWait => {
                let now = Instant::now();
                let dt = (now - last).as_secs_f32();
                last = now;

                cam_ctl.update_camera(&mut camera, dt);

                // Character movement (simplified - no vertical control)
                let desired = vec3(move_dir.x, 0.0, move_dir.z);
                phys.control_character(char_id, desired, dt, climb_try);
                phys.step();

                // Build instances to render:
                instances.clear();

                // ground plane is drawn by renderer
                // draws dynamic/kinematic cubes and capsules as cubes (for demo)
                // Note: simplified rendering since body_ids is private
                for (handle, _body) in phys.bodies.iter() {
                    if let Some(id) = phys.id_of(handle) {
                        if let Some(m) = phys.body_transform(id) {
                            let color = if phys.char_map.contains_key(&id) {
                                [0.2, 1.0, 0.4, 1.0]
                            } else {
                                [0.8, 0.8, 0.85, 1.0]
                            };
                            instances.push(astraweave_render::Instance {
                                transform: m,
                                color,
                            });
                        }
                    }
                }

                renderer.update_camera(&camera);
                renderer.update_instances(&instances);
                if let Err(e) = renderer.render() {
                    eprintln!("render error: {e:?}");
                }

                window.request_redraw();
            }
            _ => {}
        }
    })?;
    Ok(())
}
