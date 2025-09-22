// Patch for integrating texture system into unified_showcase main.rs
// To apply this patch, apply these changes to your main.rs file
// NOTE: This is not a standalone Rust file - it contains code snippets to be integrated
//       into the main.rs file. Do not try to compile this file directly.

// ----- Add these module imports at the top of main.rs -----
mod texture_manager;
mod texture_utils;
mod material;

use texture_manager::TextureManager;
use texture_utils::{find_normal_map, resolve_texture_path};
use material::{Material, MaterialLibrary, MaterialGpu};

// ----- Add these fields to the RenderStuff struct -----
// Inside the RenderStuff struct, add:
texture_manager: Option<TextureManager>,
material_library: Option<MaterialLibrary>,
material_buffer: Option<wgpu::Buffer>,
material_bind_group_layout: wgpu::BindGroupLayout,
material_bind_group: Option<wgpu::BindGroup>,
current_biome: String,

// ----- Add this code to the run() function, after renderer setup -----
// Initialize texture system
let texture_config_path = Path::new("assets/textures/atlas_config.toml");
if texture_config_path.exists() {
    println!("Initializing texture system from configuration: {}", texture_config_path.display());
    match TextureManager::new(texture_config_path) {
        Ok(texture_manager) => {
            render.texture_manager = Some(texture_manager);
            println!("Texture manager initialized successfully");
        },
        Err(e) => println!("Failed to initialize texture manager: {}", e),
    }
} else {
    println!("Texture atlas configuration not found, using direct textures");
}

// Initialize material library
let mut material_library = MaterialLibrary::new();

// Initial biome is grassland
render.current_biome = "grassland".to_string();
material_library.create_environment_materials(&render.current_biome);
render.material_library = Some(material_library);

// Initialize materials for the current biome
initialize_biome_materials(&mut render, &render.current_biome)?;

// ----- Add this function to create and initialize materials for biomes -----
fn initialize_biome_materials(render: &mut RenderStuff, biome_name: &str) -> Result<()> {
    println!("Initializing materials for biome: {}", biome_name);
    
    // Ensure material library exists
    let material_library = render.material_library.as_mut()
        .ok_or_else(|| anyhow::anyhow!("Material library not initialized"))?;
    
    // Create materials for the biome
    material_library.create_environment_materials(biome_name);
    
    // Get the ground material for this biome
    let ground_material_name = match biome_name {
        "desert" => "sand",
        "forest" => "forest_floor",
        _ => "grass", // Default to grass
    };
    
    let ground_material = material_library.get_material(ground_material_name);
    
    // Create material uniform buffer
    let material_gpu = MaterialGpu::from(ground_material);
    
    let material_buffer = render.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("material-buffer"),
        size: std::mem::size_of::<MaterialGpu>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    
    render.queue.write_buffer(&material_buffer, 0, bytemuck::cast_slice(&[material_gpu]));
    
    // Create material bind group
    let material_bind_group = render.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("material-bind-group"),
        layout: &render.material_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buffer.as_entire_binding(),
            },
        ],
    });
    
    // Update render state
    render.material_buffer = Some(material_buffer);
    render.material_bind_group = Some(material_bind_group);
    
    println!("Materials initialized for biome: {}", biome_name);
    Ok(())
}

// ----- Add this to the setup_renderer function -----
// Inside setup_renderer, after creating texture_bind_group_layout:

// Create material bind group layout
let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("material-bg-layout"),
    entries: &[
        // Material uniform buffer
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<MaterialGpu>() as u64),
            },
            count: None,
        },
    ],
});

// ----- Add this to the pipeline layout creation -----
// In the pipeline layout creation, add the material bind group layout:

let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: None,
    bind_group_layouts: &[
        &camera_bind_group_layout,
        &light_bind_group_layout, // If you have lights
        &texture_bind_group_layout,
        &material_bind_group_layout, // Add this line
    ],
    push_constant_ranges: &[],
});

// ----- Add these lines to the render pass in render_frame function -----
// Inside the render_frame function, in the render pass:

if let Some(bind_group) = &render.material_bind_group {
    render_pass.set_bind_group(3, bind_group, &[]);
}

// ----- Add this code to the switch_biome function -----
// Create or modify the switch_biome function:

fn switch_biome(render: &mut RenderStuff, physics: &mut Physics, biome_name: &str) -> Result<()> {
    println!("Switching to biome: {}", biome_name);
    
    // Update current biome
    render.current_biome = biome_name.to_string();
    
    // Reload texture pack
    reload_texture_pack(render, biome_name)?;
    
    // Regenerate environment objects
    let new_objects = generate_environment_objects(physics, biome_name);
    
    // Update material bindings
    initialize_biome_materials(render, biome_name)?;
    
    println!("Successfully switched to biome: {}", biome_name);
    Ok(())
}

// ----- Add this to the event handling in run() function -----
// Inside the event handling match in run():

Event::WindowEvent {
    event: WindowEvent::KeyboardInput { event, .. },
    ..
} => match event {
    // ... existing key handlers
    
    // Add these biome switching keys:
    KeyEvent {
        physical_key: PhysicalKey::Code(KeyCode::Digit1),
        state: ElementState::Pressed,
        ..
    } => {
        println!("Switching to grassland biome");
        if let Err(e) = switch_biome(&mut render, &mut physics, "grassland") {
            println!("Error switching to grassland: {}", e);
        }
    },
    KeyEvent {
        physical_key: PhysicalKey::Code(KeyCode::Digit2),
        state: ElementState::Pressed,
        ..
    } => {
        println!("Switching to desert biome");
        if let Err(e) = switch_biome(&mut render, &mut physics, "desert") {
            println!("Error switching to desert: {}", e);
        }
    },
    KeyEvent {
        physical_key: PhysicalKey::Code(KeyCode::Digit3),
        state: ElementState::Pressed,
        ..
    } => {
        println!("Switching to forest biome");
        if let Err(e) = switch_biome(&mut render, &mut physics, "forest") {
            println!("Error switching to forest: {}", e);
        }
    },
    // ... rest of key handlers
}