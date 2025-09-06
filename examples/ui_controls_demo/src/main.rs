use std::time::Instant;
use glam::{vec3, Vec2};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent, ElementState, KeyEvent, MouseButton},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

use astraweave_render::{Renderer, Camera, CameraController, Instance};
use astraweave_physics::PhysicsWorld;

use astraweave_input::{InputManager, InputContext, BindingSet};
use astraweave_ui::{UiLayer, UiFlags, Accessibility, UiData, draw_ui};

use astraweave_gameplay::stats::Stats;
use astraweave_gameplay::{Inventory};
use astraweave_gameplay::quests::{QuestLog};
use astraweave_gameplay::crafting::{RecipeBook, CraftRecipe, CraftCost};
use astraweave_gameplay::{DamageType, items::ItemKind, ResourceKind};

fn build_sample_recipe_book() -> RecipeBook {
    RecipeBook {
        recipes: vec![
            CraftRecipe {
                name: "Echo Blade".into(),
                output_item: ItemKind::Weapon{ base_damage: 12, dtype: DamageType::Physical },
                costs: vec![ CraftCost{ kind: ResourceKind::Ore, count: 2 }, CraftCost{ kind: ResourceKind::Crystal, count: 1 } ],
            },
            CraftRecipe {
                name: "Health Tonic".into(),
                output_item: ItemKind::Consumable { heal: 25 },
                costs: vec![ CraftCost{ kind: ResourceKind::Essence, count: 1 }, CraftCost{ kind: ResourceKind::Fiber, count: 2 } ],
            },
        ]
    }
}

fn main() -> anyhow::Result<()> {
    // Window + renderer
    let event_loop = EventLoop::new()?;
    let window = winit::window::WindowBuilder::new()
        .with_title("UI & Controls Demo")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop)?;
    let window = std::sync::Arc::new(window);
    let mut renderer = pollster::block_on(Renderer::new(window.clone()))?;
    let mut camera = Camera {
        position: vec3(0.0, 6.0, 14.0),
        yaw: -1.57, pitch: -0.35,
        fovy: 60f32.to_radians(), aspect: 16.0/9.0, znear: 0.1, zfar: 300.0
    };
    let mut cam_ctl = CameraController::new(12.0, 0.005);

    // Physics (for ground plane visual)
    let mut phys = PhysicsWorld::new(vec3(0.0, -9.81, 0.0));
    phys.create_ground_plane(vec3(100.0, 0.0, 100.0), 1.0);

    // Input system
    let bindings = BindingSet::default();
    let mut input = InputManager::new(InputContext::Gameplay, bindings);

    // UI layer

    let mut ui = UiLayer::new(&window, renderer.device(), renderer.surface_format());
    // NOTE: Replace the above with actual format from renderer; if you kept it private, expose a getter for format.
    // For clarity in this snippet, we’ll assume you’ve added:
    //   pub fn surface_format(&self) -> wgpu::TextureFormat
    // and then:
    // let mut ui = UiLayer::new(&window, renderer.device(), renderer.surface_format());

    // Demo data
    let mut flags = UiFlags::default();
    let mut acc = Accessibility::default();

    let mut player_stats = Stats::new(120);
    let mut inventory = Inventory::default();
    inventory.add_resource(ResourceKind::Ore, 3);
    inventory.add_resource(ResourceKind::Crystal, 2);
    inventory.add_resource(ResourceKind::Essence, 1);
    inventory.add_resource(ResourceKind::Fiber, 3);

    let recipes = build_sample_recipe_book();
    let mut quest_log = QuestLog::default();

    let mut last = Instant::now();
    let mut instances: Vec<Instance> = vec![];

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => {
                let consumed_by_ui = ui.on_event(&window, &event);
                input.process_window_event(&event);

                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(s) => {
                        renderer.resize(s.width, s.height);
                        camera.aspect = s.width as f32 / s.height.max(1) as f32;
                    }
                    WindowEvent::KeyboardInput { event: KeyEvent{ state, physical_key: PhysicalKey::Code(code), .. }, .. } => {
                        // Camera + toggles
                        cam_ctl.process_keyboard(code, state == ElementState::Pressed);
                        if state == ElementState::Pressed {
                            match code {
                                KeyCode::KeyI => { flags.show_inventory = !flags.show_inventory; }
                                KeyCode::KeyC => { flags.show_crafting  = !flags.show_crafting; }
                                KeyCode::KeyM => { flags.show_map       = !flags.show_map; }
                                KeyCode::KeyJ => { flags.show_quests    = !flags.show_quests; }
                                KeyCode::Escape => { flags.show_menu    = !flags.show_menu; }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Right { cam_ctl.process_mouse_button(MouseButton::Right, state == ElementState::Pressed); }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        cam_ctl.process_mouse_move(&mut camera, Vec2::new(position.x as f32, position.y as f32));
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                let dt = (Instant::now() - last).as_secs_f32(); last = Instant::now();
                input.clear_frame();
                input.poll_gamepads();

                cam_ctl.update_camera(&mut camera, dt);
                renderer.update_camera(&camera);

                // Build UI frame
                ui.begin(&window);

                let mut ui_data = UiData {
                    player_stats: &player_stats,
                    player_pos: camera.position,
                    inventory: &mut inventory,
                    recipe_book: Some(&recipes),
                    quest_log: Some(&mut quest_log),
                };
                let _ui_out = draw_ui(
                    ui.ctx(),
                    &mut flags, &mut acc,
                    ui_data.player_stats,
                    ui_data.player_pos,
                    ui_data.inventory,
                    ui_data.recipe_book,
                    ui_data.quest_log.as_deref_mut(),
                );

                // Draw scene instances (simple markers)
                instances.clear();
                // Add a couple of cubes to show something on screen
                for z in -3..=3 {
                    instances.push(Instance::from_pos_scale_color(vec3(z as f32*1.5, 0.5, 0.0), vec3(0.6,1.0,0.6), [0.6,0.7,0.9,1.0]));
                }
                renderer.update_instances(&instances);

                // Render 3D + UI
                let size = renderer.surface_size();
                let _ = renderer.render_with(|view, enc, dev, queue, size| {
                    ui.end_and_paint(&window, view, enc, dev, queue, size);
                });

                window.request_redraw();
            }
            _ => {}
        }
    })?;
    Ok(())
}
