use std::fs;
use std::time::Instant;

use glam::{vec3, Vec2};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

use astraweave_audio::AudioEngine;
use astraweave_npc::{
    llm::MockLlm, load_profile_from_toml_str, EngineCommandSink, NpcManager, NpcWorldView,
};
use astraweave_physics::PhysicsWorld;
use astraweave_render::{Camera, CameraController, Instance, Renderer};

fn main() -> anyhow::Result<()> {
    // Window + renderer
    let event_loop = EventLoop::new()?;
    let window = std::sync::Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("NPC Town Demo")
            .with_inner_size(PhysicalSize::new(1280, 720))
            .build(&event_loop)?,
    );
    // Pass Arc
    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;
    let mut camera = Camera {
        position: vec3(0.0, 6.0, 14.0),
        yaw: -1.57,
        pitch: -0.35,
        fovy: 60f32.to_radians(),
        aspect: 16.0 / 9.0,
        znear: 0.1,
        zfar: 300.0,
    };
    let mut cam_ctl = CameraController::new(12.0, 0.005);

    // Physics + audio
    let mut phys = PhysicsWorld::new(vec3(0.0, -9.81, 0.0));
    phys.create_ground_plane(vec3(100.0, 0.0, 100.0), 1.0);
    let mut audio = AudioEngine::new()?;
    audio.set_master_volume(1.0);

    // NPC Manager + profiles
    let mut npcs = NpcManager::new(Box::new(MockLlm));

    let merchant_toml = fs::read_to_string("assets/npc/merchant.toml")?;
    let guard_toml = fs::read_to_string("assets/npc/guard.toml")?;
    let merchant = load_profile_from_toml_str(&merchant_toml)?;
    let guard = load_profile_from_toml_str(&guard_toml)?;

    let merchant_id = npcs.spawn_from_profile(&mut phys, merchant);
    let guard_id = npcs.spawn_from_profile(&mut phys, guard);

    // Demo input: "utterance mode"
    let mut utter_hello = true; // toggles hello / buy / danger
    let mut utter_buy = false;
    let mut utter_danger = false;

    let mut last = Instant::now();
    let mut instances: Vec<Instance> = vec![];

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
                    let down = state == ElementState::Pressed;
                    cam_ctl.process_keyboard(code, down);

                    if down {
                        match code {
                            KeyCode::KeyH => {
                                utter_hello = true;
                                utter_buy = false;
                                utter_danger = false;
                                println!("Utterance: hello");
                            }
                            KeyCode::KeyB => {
                                utter_hello = false;
                                utter_buy = true;
                                utter_danger = false;
                                println!("Utterance: buy/shop");
                            }
                            KeyCode::KeyD => {
                                utter_hello = false;
                                utter_buy = false;
                                utter_danger = true;
                                println!("Utterance: danger/help");
                            }
                            KeyCode::KeyE => {
                                // talk to nearest (merchant or guard). For demo, alternate:
                                let glue = EngineCommandSink {
                                    phys: &mut phys,
                                    audio: &mut audio,
                                };
                                let view_merchant = NpcWorldView {
                                    time_of_day: 12.0,
                                    self_pos: vec3(0.0, 1.0, 0.0),
                                    player_pos: Some(camera.position),
                                    player_dist: Some(
                                        camera.position.distance(vec3(0.0, 1.0, 0.0)),
                                    ),
                                    nearby_threat: utter_danger,
                                    location_tag: Some("market".into()),
                                };
                                let view_guard = NpcWorldView {
                                    time_of_day: 12.0,
                                    self_pos: vec3(3.0, 1.0, 0.0),
                                    player_pos: Some(camera.position),
                                    player_dist: Some(
                                        camera.position.distance(vec3(3.0, 1.0, 0.0)),
                                    ),
                                    nearby_threat: utter_danger,
                                    location_tag: Some("gate".into()),
                                };
                                let utter = if utter_hello {
                                    "hello"
                                } else if utter_buy {
                                    "buy"
                                } else {
                                    "danger"
                                };
                                let _ = npcs.handle_player_utterance(
                                    merchant_id,
                                    &view_merchant,
                                    utter,
                                );
                                let _ = npcs.handle_player_utterance(guard_id, &view_guard, utter);
                                drop(glue);
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
                WindowEvent::CursorMoved { position, .. } => {
                    cam_ctl.process_mouse_move(
                        &mut camera,
                        Vec2::new(position.x as f32, position.y as f32),
                    );
                }
                _ => {}
            },
            Event::AboutToWait => {
                let dt = (Instant::now() - last).as_secs_f32();
                last = Instant::now();
                cam_ctl.update_camera(&mut camera, dt);
                renderer.update_camera(&camera);

                // Tick NPC manager with current views (simplified)
                let views = std::iter::once((
                    merchant_id,
                    NpcWorldView {
                        time_of_day: 12.0,
                        self_pos: vec3(0.0, 1.0, 0.0),
                        player_pos: Some(camera.position),
                        player_dist: Some(camera.position.distance(vec3(0.0, 1.0, 0.0))),
                        nearby_threat: utter_danger,
                        location_tag: Some("market".into()),
                    },
                ))
                .chain(std::iter::once((
                    guard_id,
                    NpcWorldView {
                        time_of_day: 12.0,
                        self_pos: vec3(3.0, 1.0, 0.0),
                        player_pos: Some(camera.position),
                        player_dist: Some(camera.position.distance(vec3(3.0, 1.0, 0.0))),
                        nearby_threat: utter_danger,
                        location_tag: Some("gate".into()),
                    },
                )))
                .collect();

                let mut glue = EngineCommandSink {
                    phys: &mut phys,
                    audio: &mut audio,
                };
                npcs.update(dt, &mut glue, &views);

                // Render simple cubes for "town" + NPCs
                instances.clear();
                // ground is drawn by renderer; add NPC markers:
                // (if you have transforms from phys, map them here; for demo, static markers)
                instances.push(Instance::from_pos_scale_color(
                    vec3(0.0, 0.5, 0.0),
                    vec3(0.6, 1.0, 0.6),
                    [0.2, 1.0, 0.4, 1.0],
                )); // merchant
                instances.push(Instance::from_pos_scale_color(
                    vec3(3.0, 0.5, 0.0),
                    vec3(0.6, 1.0, 0.6),
                    [0.2, 0.6, 1.0, 1.0],
                )); // guard

                renderer.update_instances(&instances);
                if let Err(e) = renderer.render() {
                    eprintln!("{e:?}");
                }
                window.request_redraw();
            }
            _ => {}
        }
    })?;
    Ok(())
}
