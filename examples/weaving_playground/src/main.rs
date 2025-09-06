use astraweave_core::{IVec2, Team, World};
use astraweave_gameplay::biome::generate_island_room;
use astraweave_gameplay::*;
use astraweave_nav::NavMesh;
use astraweave_physics::PhysicsWorld;
use astraweave_render::{Camera, CameraController, Instance, Renderer};
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
        .with_title("Weaving Playground")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop)?;

    // 3D renderer
    let mut renderer = pollster::block_on(Renderer::new(&window))?;
    let mut camera = Camera {
        position: vec3(-4.0, 7.0, 12.0),
        yaw: -3.14 / 2.1,
        pitch: -0.55,
        fovy: 60f32.to_radians(),
        aspect: 16.0 / 9.0,
        znear: 0.1,
        zfar: 400.0,
    };
    let mut cam_ctl = CameraController::new(10.0, 0.005);

    // Core world & physics
    let mut w = World::new();
    let mut phys = PhysicsWorld::new(vec3(0.0, -9.81, 0.0));

    // Spawn some tokens to visualize
    let player = w.spawn("Player", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let comp = w.spawn("Comp", IVec2 { x: 3, y: 2 }, Team { id: 1 }, 80, 30);
    let enemy = w.spawn("Enemy", IVec2 { x: 10, y: 2 }, Team { id: 2 }, 60, 0);

    // Simple island triangles (for nav + visual anchors)
    let tris = generate_island_room();
    let _nav = NavMesh::bake(&tris, 0.5, 55.0);

    // Weave budget
    let mut budget = WeaveBudget {
        terrain_edits: 3,
        weather_ops: 2,
    };

    let mut instances: Vec<Instance> = vec![];

    let mut last = Instant::now();
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
                        let mut log = |s: String| println!("{}", s);
                        match code {
                            KeyCode::Digit1 => {
                                let op = WeaveOp {
                                    kind: WeaveOpKind::ReinforcePath,
                                    a: vec3(2.0, 0.0, 2.0),
                                    b: None,
                                    budget_cost: 1,
                                };
                                if let Ok(cons) = apply_weave_op(
                                    &mut w,
                                    &mut phys,
                                    &tris,
                                    &mut budget,
                                    &op,
                                    &mut log,
                                ) {
                                    println!(
                                        "Consequence: drop x{}, faction {}",
                                        cons.drop_multiplier, cons.faction_disposition
                                    );
                                }
                            }
                            KeyCode::Digit2 => {
                                let op = WeaveOp {
                                    kind: WeaveOpKind::CollapseBridge,
                                    a: vec3(1.0, 0.0, -1.0),
                                    b: Some(vec3(6.0, 0.0, -1.0)),
                                    budget_cost: 1,
                                };
                                let _ = apply_weave_op(
                                    &mut w,
                                    &mut phys,
                                    &tris,
                                    &mut budget,
                                    &op,
                                    &mut log,
                                );
                            }
                            KeyCode::Digit3 => {
                                let op = WeaveOp {
                                    kind: WeaveOpKind::RedirectWind,
                                    a: vec3(0.0, 0.0, 0.0),
                                    b: Some(vec3(1.0, 0.0, 0.2)),
                                    budget_cost: 1,
                                };
                                let _ = apply_weave_op(
                                    &mut w,
                                    &mut phys,
                                    &tris,
                                    &mut budget,
                                    &op,
                                    &mut log,
                                );
                            }
                            KeyCode::Digit4 => {
                                let op = WeaveOp {
                                    kind: WeaveOpKind::LowerWater,
                                    a: vec3(0.0, 0.0, 0.0),
                                    b: None,
                                    budget_cost: 1,
                                };
                                let _ = apply_weave_op(
                                    &mut w,
                                    &mut phys,
                                    &tris,
                                    &mut budget,
                                    &op,
                                    &mut log,
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
                last += std::time::Duration::from_secs_f32(dt);
                cam_ctl.update_camera(&mut camera, dt);
                phys.step(dt);

                // Rebuild instances (simple viz)
                instances.clear();
                for (x, y) in w.obstacles.iter() {
                    instances.push(Instance::from_pos_scale_color(
                        glam::vec3(*x as f32, 0.5, *y as f32),
                        glam::vec3(0.9, 1.0, 0.9),
                        [0.45, 0.45, 0.45, 1.0],
                    ));
                }
                for e in w.all_of_team(0) {
                    let p = w.pos_of(e).unwrap();
                    instances.push(Instance::from_pos_scale_color(
                        glam::vec3(p.x as f32, 0.5, p.y as f32),
                        glam::vec3(0.7, 1.0, 0.7),
                        [0.2, 0.4, 1.0, 1.0],
                    ));
                }
                for e in w.all_of_team(1) {
                    let p = w.pos_of(e).unwrap();
                    instances.push(Instance::from_pos_scale_color(
                        glam::vec3(p.x as f32, 0.5, p.y as f32),
                        glam::vec3(0.7, 1.0, 0.7),
                        [0.2, 1.0, 0.4, 1.0],
                    ));
                }
                for e in w.all_of_team(2) {
                    let p = w.pos_of(e).unwrap();
                    instances.push(Instance::from_pos_scale_color(
                        glam::vec3(p.x as f32, 0.5, p.y as f32),
                        glam::vec3(0.7, 1.0, 0.7),
                        [1.0, 0.2, 0.2, 1.0],
                    ));
                }
                renderer.update_instances(&instances);
                renderer.update_camera(&camera);
                let _ = renderer.render();
                window.request_redraw();
            }
            _ => {}
        }
    })?;
    // Ok(())
}
