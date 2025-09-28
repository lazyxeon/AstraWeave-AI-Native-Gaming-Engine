use astraweave_core::{IVec2, Team, World};
use astraweave_render::types::SkinnedVertex;
use astraweave_render::{Camera, CameraController, Instance, Renderer, SkyConfig};
use egui::{Context as EguiContext, Slider};
use egui_wgpu::Renderer as EguiRenderer;
use egui_winit::State as EguiWinitState;
use glam::{vec3, Vec2};
use serde::Deserialize;
use std::{sync::Arc, time::Instant};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::PhysicalKey,
};

fn world_to_instances(world: &World, scale: f32) -> Vec<Instance> {
    let mut v = Vec::new();
    for (x, y) in world.obstacles.iter() {
        let pos = vec3(*x as f32 * scale, 0.5, *y as f32 * scale);
        v.push(Instance::from_pos_scale_color(
            pos,
            vec3(0.9, 1.0, 0.9) * 0.9,
            [0.6, 0.6, 0.7, 1.0],
        ));
    }
    for e in world.all_of_team(0) {
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(
            vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale),
            vec3(0.7, 1.0, 0.7),
            [0.2, 0.4, 1.0, 1.0],
        ));
    }
    for e in world.all_of_team(1) {
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(
            vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale),
            vec3(0.7, 1.0, 0.7),
            [0.2, 1.0, 0.4, 1.0],
        ));
    }
    for e in world.all_of_team(2) {
        let p = world.pos_of(e).unwrap();
        v.push(Instance::from_pos_scale_color(
            vec3(p.x as f32 * scale, 0.5, p.y as f32 * scale),
            vec3(0.7, 1.0, 0.7),
            [1.0, 0.2, 0.2, 1.0],
        ));
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
    let mut egui_state = EguiWinitState::new(
        egui_ctx.clone(),
        egui::ViewportId::default(),
        &*window,
        None,
        None,
    );
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

    // Simple material state for live editing
    let mut base_color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    let mut metallic: f32 = 0.1;
    let mut roughness: f32 = 0.6;
    renderer.set_material_params(base_color, metallic, roughness);
    let mut use_checkerboard: bool = false;
    let mut last_checkerboard: bool = false;
    #[derive(Debug, Deserialize)]
    struct MaterialLiveDoc {
        base_color: [f32; 4],
        metallic: f32,
        roughness: f32,
        texture_path: Option<String>,
    }

    let mut last = Instant::now();
    let mut load_mesh: bool = false;
    let mut mesh_loaded: bool = false;
    let mut skinned_demo: bool = false;
    let mut skinned_setup_done: bool = false;
    let mut load_skinned_gltf: bool = false;
    let mut skinned_gltf_loaded: bool = false;
    let mut skinned_gltf_clip: Option<(Vec<f32>, Vec<[f32; 4]>)> = None; // (times, rotations)
    let mut skinned_gltf_joint_count: usize = 0;

    println!("✅ Setup complete! Note: This demo uses basic colored primitives.");
    println!("   For full texture rendering, use the unified_showcase example.");
    println!("   Controls: WASD + mouse to move camera");

    // Note: glTF loading stub is available in astraweave-asset, but not wired here to avoid feature churn

    event_loop.run(move |event, elwt| {
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
                        ui.add(
                            Slider::new(&mut sky_cfg.cloud_speed, 0.0..=5.0).text("Cloud Speed"),
                        );
                    });
                    egui::Window::new("Material").show(ctx, |ui| {
                        ui.label("Base Color (RGB)");
                        ui.add(Slider::new(&mut base_color[0], 0.0..=1.0).text("R"));
                        ui.add(Slider::new(&mut base_color[1], 0.0..=1.0).text("G"));
                        ui.add(Slider::new(&mut base_color[2], 0.0..=1.0).text("B"));
                        ui.add(Slider::new(&mut metallic, 0.0..=1.0).text("Metallic"));
                        ui.add(Slider::new(&mut roughness, 0.04..=1.0).text("Roughness"));
                        ui.checkbox(&mut use_checkerboard, "Checkerboard Albedo");
                        ui.separator();
                        ui.checkbox(&mut load_mesh, "Load demo glTF plane");
                        ui.separator();
                        ui.checkbox(&mut skinned_demo, "Skinned quad (idle) demo");
                        ui.separator();
                        ui.checkbox(&mut load_skinned_gltf, "Load skinned glTF (idle)");
                    });
                });

                // apply any UI-driven changes
                renderer.time_of_day_mut().current_time = hour;
                renderer.set_sky_config(sky_cfg);
                renderer.set_material_params(base_color, metallic, roughness);

                if use_checkerboard != last_checkerboard {
                    if use_checkerboard {
                        // Generate a simple 256x256 checkerboard (8x8 tiles)
                        let w: u32 = 256;
                        let h: u32 = 256;
                        let tile: u32 = 32;
                        let mut data = vec![0u8; (w * h * 4) as usize];
                        for y in 0..h {
                            for x in 0..w {
                                let tx = (x / tile) % 2;
                                let ty = (y / tile) % 2;
                                let c: u8 = if (tx ^ ty) == 0 { 220 } else { 40 };
                                let idx = ((y * w + x) * 4) as usize;
                                data[idx + 0] = c;
                                data[idx + 1] = c;
                                data[idx + 2] = c;
                                data[idx + 3] = 255;
                            }
                        }
                        renderer.set_albedo_from_rgba8(w, h, &data);
                    } else {
                        renderer.set_albedo_from_rgba8(1, 1, &[255, 255, 255, 255]);
                    }
                    last_checkerboard = use_checkerboard;
                }

                // On demand: load a minimal glTF demo plane and apply material
                if load_mesh && !mesh_loaded {
                    use astraweave_asset::gltf_loader as gl;
                    if let Ok(bytes) = std::fs::read("assets/demo_plane.gltf") {
                        if let Ok((mesh, mat)) = gl::load_first_mesh_and_material(&bytes) {
                            // Use full vertex data (positions, normals, tangents, uvs)
                            let gpu_mesh = renderer.create_mesh_from_full_arrays(
                                &mesh.positions,
                                &mesh.normals,
                                &mesh.tangents,
                                &mesh.texcoords,
                                &mesh.indices,
                            );
                            renderer.set_external_mesh(gpu_mesh);
                            renderer.set_material_params(
                                mat.base_color_factor,
                                mat.metallic_factor,
                                mat.roughness_factor,
                            );
                            if let Some(img) = mat.base_color_texture {
                                renderer.set_albedo_from_rgba8(img.width, img.height, &img.rgba8);
                            }
                            if let Some(img) = mat.metallic_roughness_texture {
                                renderer.set_metallic_roughness_from_rgba8(
                                    img.width, img.height, &img.rgba8,
                                );
                            }
                            if let Some(img) = mat.normal_texture {
                                renderer.set_normal_from_rgba8(img.width, img.height, &img.rgba8);
                            }
                            mesh_loaded = true;
                        } else {
                            eprintln!("Failed to parse demo_plane.gltf");
                        }
                    } else {
                        eprintln!("Missing assets/demo_plane.gltf");
                    }
                }

                // Simple skinned quad demo: a 2-triangle plane with 1 bone influencing all verts
                if skinned_demo && !skinned_setup_done {
                    // Build a unit quad in XZ plane centered at origin
                    let verts = [
                        SkinnedVertex {
                            position: [-0.5, 0.0, -0.5],
                            normal: [0.0, 1.0, 0.0],
                            tangent: [1.0, 0.0, 0.0, 1.0],
                            joints: [0, 0, 0, 0],
                            weights: [1.0, 0.0, 0.0, 0.0],
                        },
                        SkinnedVertex {
                            position: [0.5, 0.0, -0.5],
                            normal: [0.0, 1.0, 0.0],
                            tangent: [1.0, 0.0, 0.0, 1.0],
                            joints: [0, 0, 0, 0],
                            weights: [1.0, 0.0, 0.0, 0.0],
                        },
                        SkinnedVertex {
                            position: [0.5, 0.0, 0.5],
                            normal: [0.0, 1.0, 0.0],
                            tangent: [1.0, 0.0, 0.0, 1.0],
                            joints: [0, 0, 0, 0],
                            weights: [1.0, 0.0, 0.0, 0.0],
                        },
                        SkinnedVertex {
                            position: [-0.5, 0.0, 0.5],
                            normal: [0.0, 1.0, 0.0],
                            tangent: [1.0, 0.0, 0.0, 1.0],
                            joints: [0, 0, 0, 0],
                            weights: [1.0, 0.0, 0.0, 0.0],
                        },
                    ];
                    let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
                    renderer.set_skinned_mesh(&verts, &indices);
                    // Set a colored material for visibility
                    renderer.set_material_params([0.9, 0.2, 0.2, 1.0], 0.0, 0.8);
                    skinned_setup_done = true;
                }

                // Animate the single bone with a subtle sway (idle)
                if skinned_demo && skinned_setup_done {
                    let t = now.elapsed().as_secs_f32();
                    let angle = (t * 1.2).sin() * 0.3; // radians
                    let rot = glam::Mat4::from_rotation_z(angle);
                    let trans = glam::Mat4::from_translation(glam::vec3(0.0, 0.0, 0.0));
                    let m = trans * rot; // simple idle motion
                    renderer.update_skin_palette(&[m]);
                }

                // Load a skinned glTF (first skinned node) and an optional idle clip
                if load_skinned_gltf && !skinned_gltf_loaded {
                    use astraweave_asset::gltf_loader as gl;
                    let path = "assets/skinned_demo.gltf"; // expected to be placed by user in assets/
                    if let Ok(bytes) = std::fs::read(path) {
                        if let Ok((mesh, clip, mat_opt)) =
                            gl::load_first_skinned_mesh_and_idle(&bytes)
                        {
                            // Convert to renderer format
                            let verts: Vec<SkinnedVertex> = mesh
                                .vertices
                                .into_iter()
                                .map(|v| SkinnedVertex {
                                    position: v.position,
                                    normal: v.normal,
                                    tangent: v.tangent,
                                    joints: v.joints,
                                    weights: v.weights,
                                })
                                .collect();
                            renderer.set_skinned_mesh(&verts, &mesh.indices);
                            if let Some(mat) = mat_opt {
                                renderer.set_material_params(
                                    mat.base_color_factor,
                                    mat.metallic_factor,
                                    mat.roughness_factor,
                                );
                                if let Some(img) = mat.base_color_texture {
                                    renderer
                                        .set_albedo_from_rgba8(img.width, img.height, &img.rgba8);
                                }
                                if let Some(img) = mat.metallic_roughness_texture {
                                    renderer.set_metallic_roughness_from_rgba8(
                                        img.width, img.height, &img.rgba8,
                                    );
                                }
                                if let Some(img) = mat.normal_texture {
                                    renderer
                                        .set_normal_from_rgba8(img.width, img.height, &img.rgba8);
                                }
                            }
                            // Default palette: identity for all joints
                            skinned_gltf_joint_count = mesh.joint_count as usize;
                            let mut mats = Vec::with_capacity(skinned_gltf_joint_count);
                            for _ in 0..skinned_gltf_joint_count {
                                mats.push(glam::Mat4::IDENTITY);
                            }
                            renderer.update_skin_palette(&mats);
                            // Store clip if present
                            if let Some(c) = clip {
                                skinned_gltf_clip = Some((c.times, c.rotations));
                            }
                            skinned_gltf_loaded = true;
                        } else {
                            eprintln!("Failed to parse skinned glTF: {path}");
                        }
                    } else {
                        eprintln!("Missing {path}");
                    }
                }

                // Animate first joint from clip if present
                if skinned_gltf_loaded {
                    if let Some((times, rotations)) = &skinned_gltf_clip {
                        if !times.is_empty() && !rotations.is_empty() {
                            // Loop time in clip range
                            let t = now.elapsed().as_secs_f32();
                            let max_t = *times.last().unwrap();
                            let lt = (t % max_t.max(0.0001)).max(0.0);
                            // Find the first time >= lt
                            let mut idx = 0usize;
                            while idx + 1 < times.len() && times[idx + 1] < lt {
                                idx += 1;
                            }
                            let i0 = idx;
                            let i1 = (idx + 1).min(times.len() - 1);
                            let t0 = times[i0];
                            let t1 = times[i1].max(t0 + 1e-5);
                            let a = ((lt - t0) / (t1 - t0)).clamp(0.0, 1.0);
                            let q0 = glam::Quat::from_xyzw(
                                rotations[i0][0],
                                rotations[i0][1],
                                rotations[i0][2],
                                rotations[i0][3],
                            );
                            let q1 = glam::Quat::from_xyzw(
                                rotations[i1][0],
                                rotations[i1][1],
                                rotations[i1][2],
                                rotations[i1][3],
                            );
                            let q = q0.slerp(q1, a);
                            // Build palette: rotate first joint; rest identity
                            let mut mats = Vec::with_capacity(skinned_gltf_joint_count.max(1));
                            if skinned_gltf_joint_count == 0 {
                                mats.push(glam::Mat4::from_quat(q));
                            } else {
                                mats.push(glam::Mat4::from_quat(q));
                                for _ in 1..skinned_gltf_joint_count {
                                    mats.push(glam::Mat4::IDENTITY);
                                }
                            }
                            // Keep it at origin for demo
                            renderer.update_skin_palette(&mats);
                        }
                    }
                }

                // Apply live material if available
                if let Ok(txt) = std::fs::read_to_string("assets/material_live.json") {
                    if let Ok(doc) = serde_json::from_str::<MaterialLiveDoc>(&txt) {
                        renderer.set_material_params(doc.base_color, doc.metallic, doc.roughness);
                        if let Some(path) = doc.texture_path {
                            if let Ok(bytes) = std::fs::read(&path) {
                                if let Ok(img) = image::load_from_memory(&bytes) {
                                    let img = img.to_rgba8();
                                    let (w, h) = img.dimensions();
                                    renderer.set_albedo_from_rgba8(w, h, &img);
                                }
                            }
                        }
                    }
                }

                egui_state.handle_platform_output(&*window, egui_output.platform_output.clone());

                let shapes =
                    egui_ctx.tessellate(egui_output.shapes.clone(), egui_output.pixels_per_point);

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
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
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
    })?;
    Ok(())
}
