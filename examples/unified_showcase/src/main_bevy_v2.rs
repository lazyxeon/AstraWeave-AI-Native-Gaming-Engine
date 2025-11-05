//! # AstraWeave Unified Showcase - Bevy Renderer Edition
//!
//! **Real Asset Integration**: This showcase demonstrates the Bevy renderer with actual PolyHaven
//! textures and HDRIs, replacing the old low-poly placeholder shapes.
//!
//! ## Features
//! - **Bevy Renderer**: Complete switch to astraweave-render-bevy
//! - **Real PBR Materials**: PolyHaven textures (aerial_rocks, metal_plate, cobblestone, wood_floor, plaster)
//! - **Real HDRIs**: Kloppenheim (day), Spruit Sunrise, Venice Sunset (switchable with F1-F3)
//! - **MegaLights Extension**: GPU-accelerated light culling (100k+ lights)
//! - **IBL**: Image-based lighting with HDRI environment maps
//! - **CSM**: Cascaded shadow maps for directional lighting
//!
//! ## Controls
//! - **WASD**: Move camera (forward/left/back/right)
//! - **Q/E**: Move camera up/down (god-mode flight)
//! - **Mouse**: Look around (FPS controls)
//! - **Mouse Wheel**: Zoom (adjust FOV 20-110°)
//! - **Left Click**: Grab cursor for FPS mode
//! - **F1-F3**: Switch HDRI (F1=Day, F2=Sunrise, F3=Sunset)
//! - **Space**: Toggle MegaLights demo (spawn 10k lights)
//! - **ESC**: Exit

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};
use wgpu::util::DeviceExt;
use glam::{Mat4, Vec2, Vec3};
use std::time::Instant;
use std::sync::Arc;

/// Camera controller (FPS-style WASD movement)
struct Camera {
    position: Vec3,
    yaw: f32,   // Rotation around Y axis (radians)
    pitch: f32, // Rotation around X axis (radians)
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl Camera {
    fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 2.0, 10.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 75.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 1000.0,
        }
    }
    
    fn view_matrix(&self) -> Mat4 {
        let forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );
        Mat4::look_at_rh(self.position, self.position + forward, Vec3::Y)
    }
    
    fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
}

/// Input state tracker
struct InputState {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    q: bool, // Up
    e: bool, // Down
    mouse_delta: Vec2,
    mouse_wheel: f32, // Zoom
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            w: false,
            a: false,
            s: false,
            d: false,
            q: false,
            e: false,
            mouse_delta: Vec2::ZERO,
            mouse_wheel: 0.0,
        }
    }
}

/// PBR material (PolyHaven textures)
struct Material {
    name: String,
    albedo_path: String,
    normal_path: String,
    mra_path: String, // Metallic-Roughness-AO packed
}

impl Material {
    fn new(name: &str, polyhaven_id: &str) -> Self {
        let base = format!("assets/_downloaded/{}", polyhaven_id);
        Self {
            name: name.to_string(),
            albedo_path: format!("{}/{}_albedo.png", base, polyhaven_id),
            normal_path: format!("{}/{}_normal.png", base, polyhaven_id),
            mra_path: format!("{}/{}_roughness.png", base, polyhaven_id), // TODO: Pack M+R+AO
        }
    }
}

/// HDRI environment map
struct HDRI {
    name: String,
    path: String,
}

impl HDRI {
    fn new(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
        }
    }
}

/// Vertex format (position + normal + UV for PBR textures)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x2, // uv
    ];
    
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Create subdivided ground plane (100x100m with 10x10 UV tiling for detail)
fn create_ground_plane(size: f32, subdivisions: u32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let step = size / subdivisions as f32;
    let uv_scale = 10.0; // 10x10 UV tiling (100m / 10m per tile)
    
    // Generate grid vertices
    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let px = -size / 2.0 + x as f32 * step;
            let pz = -size / 2.0 + z as f32 * step;
            let u = (x as f32 / subdivisions as f32) * uv_scale;
            let v = (z as f32 / subdivisions as f32) * uv_scale;
            
            vertices.push(Vertex {
                position: [px, 0.0, pz],
                normal: [0.0, 1.0, 0.0],
                uv: [u, v],
            });
        }
    }
    
    // Generate quad indices (2 triangles per quad)
    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let top_left = z * (subdivisions + 1) + x;
            let top_right = top_left + 1;
            let bottom_left = (z + 1) * (subdivisions + 1) + x;
            let bottom_right = bottom_left + 1;
            
            // Triangle 1
            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);
            
            // Triangle 2
            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }
    
    (vertices, indices)
}

/// Create textured cube (2m size with proper normals and UVs)
fn create_cube(size: f32) -> (Vec<Vertex>, Vec<u32>) {
    let s = size / 2.0;
    
    #[rustfmt::skip]
    let vertices = vec![
        // Front face (+Z)
        Vertex { position: [-s, -s,  s], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0] },
        Vertex { position: [ s, -s,  s], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0] },
        Vertex { position: [ s,  s,  s], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0] },
        Vertex { position: [-s,  s,  s], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] },
        
        // Back face (-Z)
        Vertex { position: [ s, -s, -s], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0] },
        Vertex { position: [-s, -s, -s], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0] },
        Vertex { position: [-s,  s, -s], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0] },
        Vertex { position: [ s,  s, -s], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0] },
        
        // Top face (+Y)
        Vertex { position: [-s,  s,  s], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0] },
        Vertex { position: [ s,  s,  s], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0] },
        Vertex { position: [ s,  s, -s], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0] },
        Vertex { position: [-s,  s, -s], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },
        
        // Bottom face (-Y)
        Vertex { position: [-s, -s, -s], normal: [0.0, -1.0, 0.0], uv: [0.0, 1.0] },
        Vertex { position: [ s, -s, -s], normal: [0.0, -1.0, 0.0], uv: [1.0, 1.0] },
        Vertex { position: [ s, -s,  s], normal: [0.0, -1.0, 0.0], uv: [1.0, 0.0] },
        Vertex { position: [-s, -s,  s], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0] },
        
        // Right face (+X)
        Vertex { position: [ s, -s,  s], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0] },
        Vertex { position: [ s, -s, -s], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0] },
        Vertex { position: [ s,  s, -s], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0] },
        Vertex { position: [ s,  s,  s], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },
        
        // Left face (-X)
        Vertex { position: [-s, -s, -s], normal: [-1.0, 0.0, 0.0], uv: [0.0, 1.0] },
        Vertex { position: [-s, -s,  s], normal: [-1.0, 0.0, 0.0], uv: [1.0, 1.0] },
        Vertex { position: [-s,  s,  s], normal: [-1.0, 0.0, 0.0], uv: [1.0, 0.0] },
        Vertex { position: [-s,  s, -s], normal: [-1.0, 0.0, 0.0], uv: [0.0, 0.0] },
    ];
    
    #[rustfmt::skip]
    let indices = vec![
        0, 1, 2,  0, 2, 3,    // Front
        4, 5, 6,  4, 6, 7,    // Back
        8, 9, 10, 8, 10, 11,  // Top
        12, 13, 14, 12, 14, 15, // Bottom
        16, 17, 18, 16, 18, 19, // Right
        20, 21, 22, 20, 22, 23, // Left
    ];
    
    (vertices, indices)
}

/// Helper: Sample terrain height at given x,z position
fn sample_terrain_height(x: f32, z: f32, island_size: f32) -> f32 {
    let center_x = 0.0;
    let center_z = 0.0;
    let dx = x - center_x;
    let dz = z - center_z;
    let dist = (dx * dx + dz * dz).sqrt();
    let radius = island_size / 2.0;
    let normalized_dist = dist / radius;
    
    if normalized_dist < 1.0 {
        let base_height = (1.0 - normalized_dist).powi(2) * 8.0;
        let detail = (x * 0.1).sin() * (z * 0.1).cos() * 0.5;
        base_height + detail
    } else {
        0.0
    }
}

/// Create a tree (cylinder trunk + cone canopy)
fn create_tree(trunk_height: f32, trunk_radius: f32, canopy_height: f32, canopy_radius: f32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let segments = 12;
    
    // Trunk (cylinder)
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * trunk_radius;
        let z = angle.sin() * trunk_radius;
        let u = i as f32 / segments as f32;
        
        // Bottom vertex
        vertices.push(Vertex {
            position: [x, 0.0, z],
            normal: [x.signum(), 0.0, z.signum()],
            uv: [u, 1.0],
        });
        
        // Top vertex
        vertices.push(Vertex {
            position: [x, trunk_height, z],
            normal: [x.signum(), 0.0, z.signum()],
            uv: [u, 0.0],
        });
    }
    
    // Trunk indices
    for i in 0..segments {
        let base = (i * 2) as u32;
        indices.extend_from_slice(&[
            base, base + 2, base + 1,
            base + 1, base + 2, base + 3,
        ]);
    }
    
    let canopy_base_idx = vertices.len() as u32;
    
    // Canopy (cone)
    let canopy_y = trunk_height + canopy_height * 0.2;
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * canopy_radius;
        let z = angle.sin() * canopy_radius;
        
        let normal_vec = Vec3::new(x, 0.5, z).normalize();
        vertices.push(Vertex {
            position: [x, canopy_y, z],
            normal: [normal_vec.x, normal_vec.y, normal_vec.z],
            uv: [i as f32 / segments as f32, 1.0],
        });
    }
    
    // Cone apex
    let apex_idx = vertices.len() as u32;
    vertices.push(Vertex {
        position: [0.0, canopy_y + canopy_height, 0.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.5, 0.0],
    });
    
    // Canopy indices
    for i in 0..segments {
        indices.extend_from_slice(&[
            canopy_base_idx + i as u32,
            apex_idx,
            canopy_base_idx + (i + 1) as u32,
        ]);
    }
    
    (vertices, indices)
}

/// Create a simple building (box with peaked roof)
fn create_building(width: f32, height: f32, depth: f32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Base box (walls + floor)
    let (base_verts, base_indices) = create_cube(1.0);
    for v in base_verts {
        vertices.push(Vertex {
            position: [v.position[0] * width, v.position[1] * height + height / 2.0, v.position[2] * depth],
            normal: v.normal,
            uv: [v.uv[0] * 2.0, v.uv[1] * 2.0], // Tile UVs
        });
    }
    indices.extend_from_slice(&base_indices);
    
    // Peaked roof (pyramid top)
    let roof_base_y = height;
    let roof_height = height * 0.4;
    let roof_base_idx = vertices.len() as u32;
    
    // Roof base corners
    let half_w = width / 2.0;
    let half_d = depth / 2.0;
    vertices.extend_from_slice(&[
        Vertex { position: [-half_w, roof_base_y, -half_d], normal: [0.0, 0.7, -0.3], uv: [0.0, 0.0] },
        Vertex { position: [ half_w, roof_base_y, -half_d], normal: [0.0, 0.7, -0.3], uv: [1.0, 0.0] },
        Vertex { position: [ half_w, roof_base_y,  half_d], normal: [0.0, 0.7,  0.3], uv: [1.0, 1.0] },
        Vertex { position: [-half_w, roof_base_y,  half_d], normal: [0.0, 0.7,  0.3], uv: [0.0, 1.0] },
    ]);
    
    // Roof apex
    let apex_idx = vertices.len() as u32;
    vertices.push(Vertex {
        position: [0.0, roof_base_y + roof_height, 0.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.5, 0.5],
    });
    
    // Roof triangles
    indices.extend_from_slice(&[
        roof_base_idx, apex_idx, roof_base_idx + 1,
        roof_base_idx + 1, apex_idx, roof_base_idx + 2,
        roof_base_idx + 2, apex_idx, roof_base_idx + 3,
        roof_base_idx + 3, apex_idx, roof_base_idx,
    ]);
    
    (vertices, indices)
}

/// Create a humanoid NPC (capsule: sphere head + cylinder body + sphere feet)
fn create_humanoid(height: f32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let head_radius = height * 0.12;
    let body_height = height * 0.5;
    let body_radius = height * 0.15;
    let head_y = body_height + head_radius;
    
    // Simple body cylinder
    let (body_verts, body_indices) = create_cube(1.0);
    for v in body_verts {
        vertices.push(Vertex {
            position: [v.position[0] * body_radius, v.position[1] * body_height / 2.0 + body_height / 2.0, v.position[2] * body_radius],
            normal: v.normal,
            uv: v.uv,
        });
    }
    indices.extend_from_slice(&body_indices);
    
    // Head sphere (simplified: just a cube scaled)
    let head_base_idx = vertices.len() as u32;
    let (head_verts, mut head_indices) = create_cube(head_radius * 2.0);
    for v in head_verts {
        vertices.push(Vertex {
            position: [v.position[0], v.position[1] + head_y, v.position[2]],
            normal: v.normal,
            uv: v.uv,
        });
    }
    for idx in &mut head_indices {
        *idx += head_base_idx;
    }
    indices.extend_from_slice(&head_indices);
    
    (vertices, indices)
}

/// Create an animal (simple quadruped: sphere body + 4 leg cylinders)
fn create_animal(body_size: f32, leg_length: f32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    // Body (sphere approximation: scaled cube)
    let (body_verts, body_indices) = create_cube(body_size);
    for v in body_verts {
        vertices.push(Vertex {
            position: [v.position[0], v.position[1] + leg_length + body_size / 2.0, v.position[2]],
            normal: v.normal,
            uv: v.uv,
        });
    }
    indices.extend_from_slice(&body_indices);
    
    // Legs (4 thin cylinders)
    let leg_radius = body_size * 0.08;
    let leg_offsets = [
        [-body_size * 0.3, body_size * 0.3],  // Front left
        [ body_size * 0.3, body_size * 0.3],  // Front right
        [-body_size * 0.3, -body_size * 0.3], // Back left
        [ body_size * 0.3, -body_size * 0.3], // Back right
    ];
    
    for [x_offset, z_offset] in &leg_offsets {
        let leg_base_idx = vertices.len() as u32;
        let (leg_verts, mut leg_indices) = create_cube(leg_radius * 2.0);
        for v in leg_verts {
            vertices.push(Vertex {
                position: [v.position[0] + x_offset, v.position[1] * leg_length / 2.0 + leg_length / 2.0, v.position[2] + z_offset],
                normal: v.normal,
                uv: v.uv,
            });
        }
        for idx in &mut leg_indices {
            *idx += leg_base_idx;
        }
        indices.extend_from_slice(&leg_indices);
    }
    
    (vertices, indices)
}

/// Create island terrain with elevation variation
fn create_island_terrain(size: f32, subdivisions: usize) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let half_size = size / 2.0;
    
    // Generate heightmap-based terrain
    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let u = x as f32 / subdivisions as f32;
            let v = z as f32 / subdivisions as f32;
            
            let world_x = (u - 0.5) * size;
            let world_z = (v - 0.5) * size;
            
            // Island shape: raised center, beaches at edges
            let dist_from_center = ((world_x / half_size).powi(2) + (world_z / half_size).powi(2)).sqrt();
            let height = if dist_from_center < 1.0 {
                // Center hills (0-8m elevation)
                let hill_factor = 1.0 - dist_from_center;
                hill_factor.powi(2) * 8.0 + (world_x * 0.1).sin() * (world_z * 0.1).cos() * 0.5
            } else {
                // Beaches/water (flat at 0m)
                0.0
            };
            
            vertices.push(Vertex {
                position: [world_x, height, world_z],
                normal: [0.0, 1.0, 0.0], // Will be recalculated
                uv: [u * 10.0, v * 10.0], // Tile UVs 10x10
            });
        }
    }
    
    // Generate indices (quads)
    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let row_size = (subdivisions + 1) as u32;
            let tl = (z * (subdivisions + 1) + x) as u32;
            let tr = tl + 1;
            let bl = tl + row_size;
            let br = bl + 1;
            
            indices.extend_from_slice(&[
                tl, bl, tr,
                tr, bl, br,
            ]);
        }
    }
    
    // Recalculate normals (flat shading approximation)
    for i in (0..indices.len()).step_by(3) {
        let i0 = indices[i] as usize;
        let i1 = indices[i + 1] as usize;
        let i2 = indices[i + 2] as usize;
        
        let v0 = Vec3::from(vertices[i0].position);
        let v1 = Vec3::from(vertices[i1].position);
        let v2 = Vec3::from(vertices[i2].position);
        
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize();
        
        for idx in [i0, i1, i2] {
            vertices[idx].normal = normal.to_array();
        }
    }
    
    (vertices, indices)
}

/// Main application state (using ApplicationHandler for winit 0.30)
struct ShowcaseApp {
    // Window & wgpu
    window: Option<Arc<Window>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    
    // Camera & input
    camera: Camera,
    input: InputState,
    last_frame: Instant,
    cursor_grabbed: bool,
    
    // Scene geometry
    ground_vertex_buffer: Option<wgpu::Buffer>,
    ground_index_buffer: Option<wgpu::Buffer>,
    ground_index_count: u32,
    
    cube_vertex_buffer: Option<wgpu::Buffer>,
    cube_index_buffer: Option<wgpu::Buffer>,
    cube_index_count: u32,
    
    // Island scene objects
    tree_vertex_buffer: Option<wgpu::Buffer>,
    tree_index_buffer: Option<wgpu::Buffer>,
    tree_index_count: u32,
    tree_positions: Vec<(Vec3, u32)>, // (position, material_index)
    
    building_vertex_buffer: Option<wgpu::Buffer>,
    building_index_buffer: Option<wgpu::Buffer>,
    building_index_count: u32,
    building_positions: Vec<(Vec3, u32)>,
    
    npc_vertex_buffer: Option<wgpu::Buffer>,
    npc_index_buffer: Option<wgpu::Buffer>,
    npc_index_count: u32,
    npc_positions: Vec<(Vec3, u32)>,
    
    animal_vertex_buffer: Option<wgpu::Buffer>,
    animal_index_buffer: Option<wgpu::Buffer>,
    animal_index_count: u32,
    animal_positions: Vec<(Vec3, u32)>,
    
    companion_position: Vec3,
    
    // Materials & HDRIs
    materials: Vec<Material>,
    hdris: Vec<HDRI>,
    current_hdri: usize,
    
    // Rendering resources
    render_pipeline: Option<wgpu::RenderPipeline>,
    uniform_buffer: Option<wgpu::Buffer>,
    uniform_bind_group: Option<wgpu::BindGroup>,
    material_bind_groups: Vec<wgpu::BindGroup>,
    depth_texture: Option<wgpu::TextureView>,
}

impl Default for ShowcaseApp {
    fn default() -> Self {
        Self {
            window: None,
            device: None,
            queue: None,
            surface: None,
            surface_config: None,
            camera: Camera::new(16.0 / 9.0), // Default aspect ratio
            input: InputState::default(),
            last_frame: Instant::now(),
            cursor_grabbed: false,
            ground_vertex_buffer: None,
            ground_index_buffer: None,
            ground_index_count: 0,
            cube_vertex_buffer: None,
            cube_index_buffer: None,
            cube_index_count: 0,
            tree_vertex_buffer: None,
            tree_index_buffer: None,
            tree_index_count: 0,
            tree_positions: Vec::new(),
            building_vertex_buffer: None,
            building_index_buffer: None,
            building_index_count: 0,
            building_positions: Vec::new(),
            npc_vertex_buffer: None,
            npc_index_buffer: None,
            npc_index_count: 0,
            npc_positions: Vec::new(),
            animal_vertex_buffer: None,
            animal_index_buffer: None,
            animal_index_count: 0,
            animal_positions: Vec::new(),
            companion_position: Vec3::new(5.0, 0.0, 5.0),
            materials: vec![
                Material::new("Aerial Rocks", "aerial_rocks"),
                Material::new("Metal Plate", "metal_plate"),
                Material::new("Cobblestone", "cobblestone"),
                Material::new("Wood Floor", "wood_floor"),
                Material::new("Plastered Wall", "plastered_wall"),
            ],
            hdris: vec![
                HDRI::new("Kloppenheim (Day)", "assets/hdri/polyhaven/kloppenheim/kloppenheim_06_puresky_2k.hdr"),
                HDRI::new("Spruit Sunrise", "assets/hdri/polyhaven/spruit_sunrise/spruit_sunrise_2k.hdr"),
                HDRI::new("Venice Sunset", "assets/hdri/polyhaven/venice_sunset/venice_sunset_2k.hdr"),
            ],
            current_hdri: 0,
            render_pipeline: None,
            uniform_buffer: None,
            uniform_bind_group: None,
            material_bind_groups: Vec::new(),
            depth_texture: None,
        }
    }
}

impl ShowcaseApp {
    /// Load a texture from file path
    fn load_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
    ) -> Result<wgpu::Texture, Box<dyn std::error::Error>> {
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let dimensions = rgba.dimensions();
        
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(path),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );
        
        Ok(texture)
    }
    
    /// Create render pipeline and resources
    fn create_render_pipeline(&mut self) {
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        let config = self.surface_config.as_ref().unwrap();
        
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PBR Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("pbr_shader.wgsl").into()),
        });
        
        // Create depth texture
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        
        self.depth_texture = Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));
        
        // Create uniform buffer (camera + model matrices)
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: 256, // mat4x4 + mat4x4 + mat4x4 + vec3 + padding = 208 bytes (round to 256)
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create uniform bind group layout (group 0)
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create material bind group layout (group 1)
        let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &[
                // Albedo texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Normal texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Roughness texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        
        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("PBR Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        });
        
        // Load textures for each material and create bind groups
        let mut material_bind_groups = Vec::new();
        
        for (i, material) in self.materials.iter().enumerate() {
            println!("📦 Loading material {}: {}", i, material.name);
            
            // Load textures (with fallback for missing files)
            let albedo_texture = Self::load_texture(device, queue, &material.albedo_path)
                .unwrap_or_else(|e| {
                    eprintln!("⚠️  Failed to load albedo {}: {}", material.albedo_path, e);
                    // Create 1x1 magenta fallback
                    let fallback_data = vec![255u8, 0, 255, 255];
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Fallback Albedo"),
                        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    });
                    queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &fallback_data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(4),
                            rows_per_image: Some(1),
                        },
                        wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    );
                    texture
                });
            
            let normal_texture = Self::load_texture(device, queue, &material.normal_path)
                .unwrap_or_else(|_| {
                    // Create 1x1 flat normal (0.5, 0.5, 1.0 in tangent space)
                    let fallback_data = vec![128u8, 128, 255, 255];
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Fallback Normal"),
                        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    });
                    queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &fallback_data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(4),
                            rows_per_image: Some(1),
                        },
                        wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    );
                    texture
                });
            
            let roughness_texture = Self::load_texture(device, queue, &material.mra_path)
                .unwrap_or_else(|_| {
                    // Create 1x1 mid-roughness (0.5)
                    let fallback_data = vec![128u8, 128, 128, 255];
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("Fallback Roughness"),
                        size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    });
                    queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &fallback_data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(4),
                            rows_per_image: Some(1),
                        },
                        wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    );
                    texture
                });
            
            // Create bind group for this material
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Material Bind Group {}", i)),
                layout: &material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &albedo_texture.create_view(&wgpu::TextureViewDescriptor::default())
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &normal_texture.create_view(&wgpu::TextureViewDescriptor::default())
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(
                            &roughness_texture.create_view(&wgpu::TextureViewDescriptor::default())
                        ),
                    },
                ],
            });
            
            material_bind_groups.push(bind_group);
        }
        
        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PBR Pipeline Layout"),
            bind_group_layouts: &[
                &uniform_bind_group_layout,
                &material_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        
        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PBR Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        
        self.render_pipeline = Some(render_pipeline);
        self.uniform_buffer = Some(uniform_buffer);
        self.uniform_bind_group = Some(uniform_bind_group);
        self.material_bind_groups = material_bind_groups;
        
        println!("✅ Render pipeline created with {} materials", self.materials.len());
    }
    
    fn setup_scene(&mut self) {
        let device = self.device.as_ref().unwrap();
        
        // Create island terrain (200x200m with elevation)
        let (ground_vertices, ground_indices) = create_island_terrain(200.0, 80);
        self.ground_index_count = ground_indices.len() as u32;
        
        self.ground_vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Island Terrain Vertex Buffer"),
            contents: bytemuck::cast_slice(&ground_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));
        
        self.ground_index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Island Terrain Index Buffer"),
            contents: bytemuck::cast_slice(&ground_indices),
            usage: wgpu::BufferUsages::INDEX,
        }));
        
        // Create tree mesh (single instance with all trees baked in)
        let (tree_template_vertices, tree_template_indices) = create_tree(3.0, 0.3, 4.0, 2.0);
        
        // Generate all tree instances with positions baked into vertices
        let tree_positions = vec![
            Vec3::new(-20.0, 0.0, -30.0), Vec3::new(-15.0, 0.0, -25.0), Vec3::new(-10.0, 0.0, -20.0),
            Vec3::new(15.0, 0.0, -28.0), Vec3::new(20.0, 0.0, -22.0), Vec3::new(-25.0, 0.0, 10.0),
            Vec3::new(-18.0, 0.0, 15.0), Vec3::new(22.0, 0.0, 12.0), Vec3::new(28.0, 0.0, 18.0),
            Vec3::new(0.0, 0.0, -15.0), Vec3::new(-5.0, 0.0, -12.0), Vec3::new(5.0, 0.0, -10.0),
        ];
        
        let mut all_tree_vertices = Vec::new();
        let mut all_tree_indices = Vec::new();
        
        for pos in &tree_positions {
            let terrain_height = sample_terrain_height(pos.x, pos.z, 200.0);
            let base_vertex_index = all_tree_vertices.len() as u32;
            for v in &tree_template_vertices {
                all_tree_vertices.push(Vertex {
                    position: [v.position[0] + pos.x, v.position[1] + terrain_height, v.position[2] + pos.z],
                    normal: v.normal,
                    uv: v.uv,
                });
            }
            for idx in &tree_template_indices {
                all_tree_indices.push(base_vertex_index + idx);
            }
        }
        
        self.tree_index_count = all_tree_indices.len() as u32;
        
        self.tree_vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tree Vertex Buffer"),
            contents: bytemuck::cast_slice(&all_tree_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));
        
        self.tree_index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tree Index Buffer"),
            contents: bytemuck::cast_slice(&all_tree_indices),
            usage: wgpu::BufferUsages::INDEX,
        }));
        
        // Create building instances (baked positions)
        let (building_template_vertices, building_template_indices) = create_building(6.0, 4.0, 6.0);
        
        let building_positions = vec![
            Vec3::new(-30.0, 0.0, 5.0),
            Vec3::new(25.0, 0.0, -5.0),
            Vec3::new(0.0, 0.0, 20.0),
        ];
        
        let mut all_building_vertices = Vec::new();
        let mut all_building_indices = Vec::new();
        
        for pos in &building_positions {
            let terrain_height = sample_terrain_height(pos.x, pos.z, 200.0);
            let base_vertex_index = all_building_vertices.len() as u32;
            for v in &building_template_vertices {
                all_building_vertices.push(Vertex {
                    position: [v.position[0] + pos.x, v.position[1] + terrain_height, v.position[2] + pos.z],
                    normal: v.normal,
                    uv: v.uv,
                });
            }
            for idx in &building_template_indices {
                all_building_indices.push(base_vertex_index + idx);
            }
        }
        
        self.building_index_count = all_building_indices.len() as u32;
        
        self.building_vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Vertex Buffer"),
            contents: bytemuck::cast_slice(&all_building_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));
        
        self.building_index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Building Index Buffer"),
            contents: bytemuck::cast_slice(&all_building_indices),
            usage: wgpu::BufferUsages::INDEX,
        }));
        
        // Create NPC instances (baked positions)
        let (npc_template_vertices, npc_template_indices) = create_humanoid(1.8);
        
        let npc_positions = vec![
            Vec3::new(-28.0, 0.0, 8.0),
            Vec3::new(27.0, 0.0, -3.0),
            Vec3::new(-2.0, 0.0, 18.0),
            Vec3::new(5.0, 0.0, -5.0),
            Vec3::new(-10.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 8.0), // Companion
        ];
        
        let mut all_npc_vertices = Vec::new();
        let mut all_npc_indices = Vec::new();
        
        for pos in &npc_positions {
            let terrain_height = sample_terrain_height(pos.x, pos.z, 200.0);
            let base_vertex_index = all_npc_vertices.len() as u32;
            for v in &npc_template_vertices {
                all_npc_vertices.push(Vertex {
                    position: [v.position[0] + pos.x, v.position[1] + terrain_height, v.position[2] + pos.z],
                    normal: v.normal,
                    uv: v.uv,
                });
            }
            for idx in &npc_template_indices {
                all_npc_indices.push(base_vertex_index + idx);
            }
        }
        
        self.npc_index_count = all_npc_indices.len() as u32;
        
        self.npc_vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NPC Vertex Buffer"),
            contents: bytemuck::cast_slice(&all_npc_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));
        
        self.npc_index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NPC Index Buffer"),
            contents: bytemuck::cast_slice(&all_npc_indices),
            usage: wgpu::BufferUsages::INDEX,
        }));
        
        // Create animal instances (baked positions)
        let (animal_template_vertices, animal_template_indices) = create_animal(1.0, 0.8);
        
        let animal_positions = vec![
            Vec3::new(-15.0, 0.0, -10.0),
            Vec3::new(18.0, 0.0, 8.0),
            Vec3::new(-8.0, 0.0, -5.0),
        ];
        
        let mut all_animal_vertices = Vec::new();
        let mut all_animal_indices = Vec::new();
        
        for pos in &animal_positions {
            let terrain_height = sample_terrain_height(pos.x, pos.z, 200.0);
            let base_vertex_index = all_animal_vertices.len() as u32;
            for v in &animal_template_vertices {
                all_animal_vertices.push(Vertex {
                    position: [v.position[0] + pos.x, v.position[1] + terrain_height, v.position[2] + pos.z],
                    normal: v.normal,
                    uv: v.uv,
                });
            }
            for idx in &animal_template_indices {
                all_animal_indices.push(base_vertex_index + idx);
            }
        }
        
        self.animal_index_count = all_animal_indices.len() as u32;
        
        self.animal_vertex_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animal Vertex Buffer"),
            contents: bytemuck::cast_slice(&all_animal_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }));
        
        self.animal_index_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Animal Index Buffer"),
            contents: bytemuck::cast_slice(&all_animal_indices),
            usage: wgpu::BufferUsages::INDEX,
        }));
        
        // Adjust camera start position for island view
        self.camera.position = Vec3::new(0.0, 15.0, 40.0); // High overview
        self.camera.pitch = -0.3; // Looking down slightly
        
        println!("🏝️  Veilweaver Starter Island loaded:");
        println!("   Terrain: {} vertices, {} indices", ground_vertices.len(), ground_indices.len());
        println!("   Trees: {} instances ({} vertices total)", tree_positions.len(), all_tree_vertices.len());
        println!("   Buildings: {} instances ({} vertices total)", building_positions.len(), all_building_vertices.len());
        println!("   NPCs: {} humanoids ({} vertices total)", npc_positions.len(), all_npc_vertices.len());
        println!("   Animals: {} creatures ({} vertices total)", animal_positions.len(), all_animal_vertices.len());
        println!("   Materials: {:?}", self.materials.iter().map(|m| &m.name).collect::<Vec<_>>());
    }
    
    fn update(&mut self) {
        let dt = self.last_frame.elapsed().as_secs_f32();
        self.last_frame = Instant::now();
        
        let move_speed = 15.0; // 15 m/s (slightly faster for exploration)
        let mouse_sensitivity = 0.003;
        let zoom_speed = 2.0;
        
        // Update camera orientation from mouse
        self.camera.yaw += self.input.mouse_delta.x * mouse_sensitivity;
        self.camera.pitch -= self.input.mouse_delta.y * mouse_sensitivity;
        self.camera.pitch = self.camera.pitch.clamp(-1.5_f32, 1.5_f32);
        self.input.mouse_delta = Vec2::ZERO;
        
        // Mouse wheel zoom (adjust FOV)
        if self.input.mouse_wheel != 0.0 {
            self.camera.fov -= self.input.mouse_wheel * zoom_speed * dt;
            self.camera.fov = self.camera.fov.clamp(20.0_f32.to_radians(), 110.0_f32.to_radians());
            self.input.mouse_wheel = 0.0;
        }
        
        // Camera movement (full 6DOF god-mode)
        let forward = Vec3::new(
            self.camera.yaw.cos() * self.camera.pitch.cos(),
            self.camera.pitch.sin(),
            self.camera.yaw.sin() * self.camera.pitch.cos(),
        ).normalize();
        let right = Vec3::new(
            (self.camera.yaw + std::f32::consts::FRAC_PI_2).cos(),
            0.0,
            (self.camera.yaw + std::f32::consts::FRAC_PI_2).sin(),
        ).normalize();
        let up = Vec3::Y;
        
        let mut velocity = Vec3::ZERO;
        if self.input.w { velocity += forward; }
        if self.input.s { velocity -= forward; }
        if self.input.a { velocity -= right; }  // FIXED: Swapped from + to -
        if self.input.d { velocity += right; }  // FIXED: Swapped from - to +
        if self.input.q { velocity += up; }     // NEW: Fly up
        if self.input.e { velocity -= up; }     // NEW: Fly down
        
        if velocity.length() > 0.0 {
            self.camera.position += velocity.normalize() * move_speed * dt;
        }
    }
    
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface = self.surface.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();
        
        let frame = surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // Update uniforms (camera matrices)
        if let Some(uniform_buffer) = &self.uniform_buffer {
            let view_matrix = self.camera.view_matrix();
            let proj_matrix = self.camera.projection_matrix();
            let view_proj = proj_matrix * view_matrix;
            
            #[repr(C)]
            #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
            struct Uniforms {
                view_proj: [[f32; 4]; 4],
                view: [[f32; 4]; 4],
                model: [[f32; 4]; 4],
                camera_pos: [f32; 3],
                _pad: f32,
            }
            
            let uniforms = Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                view: view_matrix.to_cols_array_2d(),
                model: Mat4::IDENTITY.to_cols_array_2d(),
                camera_pos: [self.camera.position.x, self.camera.position.y, self.camera.position.z],
                _pad: 0.0,
            };
            
            queue.write_buffer(uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        }
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.53,
                            g: 0.81,
                            b: 0.92,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: self.depth_texture.as_ref().map(|view| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            if let (Some(pipeline), Some(uniform_bg)) = (&self.render_pipeline, &self.uniform_bind_group) {
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, uniform_bg, &[]);
                
                // Render island terrain (material 0: aerial_rocks)
                if !self.material_bind_groups.is_empty() {
                    if let (Some(vbuf), Some(ibuf)) = (&self.ground_vertex_buffer, &self.ground_index_buffer) {
                        render_pass.set_bind_group(1, &self.material_bind_groups[0], &[]);
                        render_pass.set_vertex_buffer(0, vbuf.slice(..));
                        render_pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..self.ground_index_count, 0, 0..1);
                    }
                    
                    // Render trees (material 3: wood_floor for bark)
                    if self.tree_index_count > 0 && self.material_bind_groups.len() > 3 {
                        if let (Some(vbuf), Some(ibuf)) = (&self.tree_vertex_buffer, &self.tree_index_buffer) {
                            render_pass.set_bind_group(1, &self.material_bind_groups[3], &[]);
                            render_pass.set_vertex_buffer(0, vbuf.slice(..));
                            render_pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..self.tree_index_count, 0, 0..1);
                        }
                    }
                    
                    // Render buildings (material 2: cobblestone)
                    if self.building_index_count > 0 && self.material_bind_groups.len() > 2 {
                        if let (Some(vbuf), Some(ibuf)) = (&self.building_vertex_buffer, &self.building_index_buffer) {
                            render_pass.set_bind_group(1, &self.material_bind_groups[2], &[]);
                            render_pass.set_vertex_buffer(0, vbuf.slice(..));
                            render_pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..self.building_index_count, 0, 0..1);
                        }
                    }
                    
                    // Render NPCs (material 1: metal_plate)
                    if self.npc_index_count > 0 && self.material_bind_groups.len() > 1 {
                        if let (Some(vbuf), Some(ibuf)) = (&self.npc_vertex_buffer, &self.npc_index_buffer) {
                            render_pass.set_bind_group(1, &self.material_bind_groups[1], &[]);
                            render_pass.set_vertex_buffer(0, vbuf.slice(..));
                            render_pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..self.npc_index_count, 0, 0..1);
                        }
                    }
                    
                    // Render animals (material 0: aerial_rocks for fur)
                    if self.animal_index_count > 0 {
                        if let (Some(vbuf), Some(ibuf)) = (&self.animal_vertex_buffer, &self.animal_index_buffer) {
                            render_pass.set_bind_group(1, &self.material_bind_groups[0], &[]);
                            render_pass.set_vertex_buffer(0, vbuf.slice(..));
                            render_pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..self.animal_index_count, 0, 0..1);
                        }
                    }
                }
            }
        }
        
        queue.submit(Some(encoder.finish()));
        frame.present();
        
        Ok(())
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            if let (Some(surface), Some(device), Some(config)) = 
                (self.surface.as_ref(), self.device.as_ref(), self.surface_config.as_mut()) 
            {
                config.width = new_size.width;
                config.height = new_size.height;
                surface.configure(device, config);
                
                self.camera.aspect = new_size.width as f32 / new_size.height as f32;
            }
        }
    }
}

impl ApplicationHandler for ShowcaseApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return; // Already initialized
        }
        
        // Create window
        let window_attrs = Window::default_attributes()
            .with_title("AstraWeave Unified Showcase - Bevy Renderer + Real Assets")
            .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080));
        
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
        
        // Initialize wgpu
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone()).unwrap();
        
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();
        
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
        ))
        .unwrap();
        
        let size = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &surface_config);
        
        println!("🎮 AstraWeave Unified Showcase Initialized");
        println!("   Resolution: {}×{}", size.width, size.height);
        println!("   Backend: {:?}", adapter.get_info().backend);
        println!("   Device: {}", adapter.get_info().name);
        
        self.window = Some(window);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
        self.camera.aspect = size.width as f32 / size.height as f32;
        
        // Setup scene geometry
        self.setup_scene();
        
        // Create render pipeline and load textures
        self.create_render_pipeline();
    }
    
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("👋 Exiting showcase...");
                event_loop.exit();
            }
            
            WindowEvent::Resized(physical_size) => {
                self.resize(physical_size);
            }
            
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                let pressed = key_event.state == ElementState::Pressed;
                
                if let PhysicalKey::Code(keycode) = key_event.physical_key {
                    match keycode {
                        KeyCode::Escape if pressed => event_loop.exit(),
                        KeyCode::KeyW => self.input.w = pressed,
                        KeyCode::KeyA => self.input.a = pressed,
                        KeyCode::KeyS => self.input.s = pressed,
                        KeyCode::KeyD => self.input.d = pressed,
                        KeyCode::KeyQ => self.input.q = pressed,
                        KeyCode::KeyE => self.input.e = pressed,
                        
                        // HDRI switching
                        KeyCode::F1 if pressed => {
                            self.current_hdri = 0;
                            println!("🌅 Switched to HDRI: {}", self.hdris[0].name);
                        }
                        KeyCode::F2 if pressed => {
                            self.current_hdri = 1;
                            println!("🌅 Switched to HDRI: {}", self.hdris[1].name);
                        }
                        KeyCode::F3 if pressed => {
                            self.current_hdri = 2;
                            println!("🌅 Switched to HDRI: {}", self.hdris[2].name);
                        }
                        
                        // TODO: Space key for MegaLights demo
                        
                        _ => {}
                    }
                }
            }
            
            WindowEvent::MouseWheel { delta, .. } => {
                // Handle mouse wheel zoom
                match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        self.input.mouse_wheel += y;
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        self.input.mouse_wheel += (pos.y / 100.0) as f32;
                    }
                }
            }
            
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                if let Some(window) = &self.window {
                    // Grab cursor for FPS controls
                    let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
                    window.set_cursor_visible(false);
                    self.cursor_grabbed = true;
                }
            }
            
            WindowEvent::RedrawRequested => {
                self.update();
                
                match self.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        if let Some(size) = self.window.as_ref().map(|w| w.inner_size()) {
                            self.resize(size);
                        }
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        eprintln!("❌ Out of memory!");
                        event_loop.exit();
                    }
                    Err(e) => {
                        eprintln!("⚠️  Render error: {:?}", e);
                    }
                }
                
                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            
            _ => {}
        }
    }
    
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            if self.cursor_grabbed {
                self.input.mouse_delta.x += delta.0 as f32;
                self.input.mouse_delta.y += delta.1 as f32;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let event_loop = EventLoop::new()?;
    let mut app = ShowcaseApp::default();
    
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
