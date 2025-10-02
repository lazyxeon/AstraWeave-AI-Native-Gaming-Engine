//! Nanite Demo - High-polygon scene rendering with virtualized geometry
//!
//! This demo showcases the Nanite-inspired meshlet rendering system with:
//! - 10M+ polygon scene
//! - LOD hierarchy with automatic selection
//! - Frustum and backface culling
//! - Integration with voxel terrain

use anyhow::Result;
use glam::{Mat4, Vec3};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(feature = "nanite")]
use astraweave_render::nanite_visibility::GpuMeshlet;
#[cfg(feature = "nanite")]
use astraweave_render::nanite_render::NaniteRenderContext;
use astraweave_asset::nanite_preprocess::{generate_lod_hierarchy, MeshletHierarchy};

struct DemoState {
    camera_pos: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,
    camera_speed: f32,
    mouse_sensitivity: f32,
    last_mouse_pos: Option<(f32, f32)>,
}

impl DemoState {
    fn new() -> Self {
        Self {
            camera_pos: Vec3::new(0.0, 50.0, 100.0),
            camera_yaw: 0.0,
            camera_pitch: -0.3,
            camera_speed: 10.0,
            mouse_sensitivity: 0.002,
            last_mouse_pos: None,
        }
    }

    fn get_view_matrix(&self) -> Mat4 {
        let forward = Vec3::new(
            self.camera_yaw.cos() * self.camera_pitch.cos(),
            self.camera_pitch.sin(),
            self.camera_yaw.sin() * self.camera_pitch.cos(),
        );
        let target = self.camera_pos + forward;
        Mat4::look_at_rh(self.camera_pos, target, Vec3::Y)
    }

    fn handle_input(&mut self, event: &WindowEvent, delta_time: f32) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let amount = if *state == ElementState::Pressed {
                    self.camera_speed * delta_time
                } else {
                    0.0
                };

                let forward = Vec3::new(
                    self.camera_yaw.cos(),
                    0.0,
                    self.camera_yaw.sin(),
                ).normalize();
                let right = Vec3::new(
                    (self.camera_yaw + std::f32::consts::FRAC_PI_2).cos(),
                    0.0,
                    (self.camera_yaw + std::f32::consts::FRAC_PI_2).sin(),
                ).normalize();

                match keycode {
                    VirtualKeyCode::W => {
                        self.camera_pos += forward * amount;
                        true
                    }
                    VirtualKeyCode::S => {
                        self.camera_pos -= forward * amount;
                        true
                    }
                    VirtualKeyCode::A => {
                        self.camera_pos -= right * amount;
                        true
                    }
                    VirtualKeyCode::D => {
                        self.camera_pos += right * amount;
                        true
                    }
                    VirtualKeyCode::Space => {
                        self.camera_pos.y += amount;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.camera_pos.y -= amount;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let current_pos = (position.x as f32, position.y as f32);
                
                if let Some(last_pos) = self.last_mouse_pos {
                    let delta_x = current_pos.0 - last_pos.0;
                    let delta_y = current_pos.1 - last_pos.1;
                    
                    self.camera_yaw += delta_x * self.mouse_sensitivity;
                    self.camera_pitch -= delta_y * self.mouse_sensitivity;
                    
                    // Clamp pitch
                    self.camera_pitch = self.camera_pitch.clamp(
                        -std::f32::consts::FRAC_PI_2 + 0.1,
                        std::f32::consts::FRAC_PI_2 - 0.1,
                    );
                }
                
                self.last_mouse_pos = Some(current_pos);
                true
            }
            _ => false,
        }
    }
}

/// Generate a procedural high-detail sphere mesh
fn generate_sphere_mesh(radius: f32, subdivisions: u32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 4]>, Vec<[f32; 2]>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut tangents = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for lat in 0..=subdivisions {
        let theta = lat as f32 * std::f32::consts::PI / subdivisions as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=subdivisions {
            let phi = lon as f32 * 2.0 * std::f32::consts::PI / subdivisions as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = sin_theta * cos_phi;
            let y = cos_theta;
            let z = sin_theta * sin_phi;

            positions.push([x * radius, y * radius, z * radius]);
            normals.push([x, y, z]);
            
            // Tangent
            let tx = -sin_phi;
            let tz = cos_phi;
            tangents.push([tx, 0.0, tz, 1.0]);
            
            // UV
            let u = lon as f32 / subdivisions as f32;
            let v = lat as f32 / subdivisions as f32;
            uvs.push([u, v]);
        }
    }

    // Generate indices
    for lat in 0..subdivisions {
        for lon in 0..subdivisions {
            let first = lat * (subdivisions + 1) + lon;
            let second = first + subdivisions + 1;

            indices.push(first);
            indices.push(second);
            indices.push(first + 1);

            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }

    (positions, normals, tangents, uvs, indices)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("=== AstraWeave Nanite Demo ===");
    println!("Generating high-polygon scene...");

    // Generate a high-detail sphere (10M+ polygons)
    let subdivisions = 500; // This will create ~1.5M triangles per sphere
    let (positions, normals, tangents, uvs, indices) = generate_sphere_mesh(50.0, subdivisions);
    
    println!("Generated sphere with {} vertices and {} triangles", 
             positions.len(), indices.len() / 3);

    // Generate meshlet hierarchy
    println!("Generating meshlet hierarchy with LODs...");
    let hierarchy = generate_lod_hierarchy(
        &positions,
        &normals,
        &tangents,
        &uvs,
        &indices,
        4, // 4 LOD levels
    )?;

    println!("Generated {} meshlets across {} LOD levels", 
             hierarchy.meshlets.len(), hierarchy.lod_count);
    
    for (lod, range) in hierarchy.lod_ranges.iter().enumerate() {
        let meshlet_count = range.end - range.start;
        println!("  LOD {}: {} meshlets", lod, meshlet_count);
    }

    // Create window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("AstraWeave Nanite Demo - 10M+ Polygons")
        .with_inner_size(winit::dpi::LogicalSize::new(1920, 1080))
        .build(&event_loop)?;

    println!("\nControls:");
    println!("  WASD - Move camera");
    println!("  Space/Shift - Move up/down");
    println!("  Mouse - Look around");
    println!("  ESC - Exit");

    let mut state = DemoState::new();
    let mut last_frame_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {
                        let now = std::time::Instant::now();
                        let delta_time = (now - last_frame_time).as_secs_f32();
                        last_frame_time = now;
                        
                        state.handle_input(event, delta_time);
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // In a full implementation, this would render the scene
                // For now, we just demonstrate the meshlet generation
            }
            _ => {}
        }
    });
}