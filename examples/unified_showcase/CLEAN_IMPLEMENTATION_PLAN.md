# Clean Unified Showcase Implementation Plan

## What We've Done

Created a **clean foundation** with NO legacy complexity:

### ✅ Removed
- ❌ Texture atlas system (source of all bugs)
- ❌ Material ID system
- ❌ UV remapping complexity
- ❌ material_blend for terrain
- ❌ Broken atlas_regions uniform buffer

### ✅ New Clean Architecture
- ✅ Simple vertex format: `position + normal + uv` (that's it!)
- ✅ One texture per material (how Bevy/Godot do it)
- ✅ GLTF models use original UVs (no remapping needed)
- ✅ Sampler uses `AddressMode::Repeat` (correct for individual textures)
- ✅ Clean shader with no atlas complexity

## Files Created

1. **`main_clean.rs`** - New clean application structure
   - Proper GPU initialization
   - Simple Material/Mesh/SceneObject system
   - Clean rendering pipeline

2. **`shader_clean.wgsl`** - Simple PBR shader
   - No atlas remapping
   - Direct texture sampling with original UVs
   - Simple lighting model

## Next Steps to Complete

### Step 1: Load Textures (30 minutes)
```rust
fn load_texture(device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> (wgpu::Texture, wgpu::TextureView) {
    let img = image::open(path).unwrap().to_rgba8();
    let (width, height) = img.dimensions();
    
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(path),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    
    queue.write_texture(
        texture.as_image_copy(),
        &img,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn create_material(&mut self, name: &str, texture_path: &str) -> usize {
    let (texture, view) = load_texture(&self.device, &self.queue, texture_path);
    
    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("{} Bind Group", name)),
        layout: &self.material_bind_group_layout, // Store this in ShowcaseApp
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
        ],
    });
    
    self.materials.push(Material { name: name.to_string(), albedo_texture: texture, albedo_view: view, bind_group });
    self.materials.len() - 1
}
```

### Step 2: Load GLTF Models (30 minutes)
```rust
fn load_gltf_mesh(&mut self, path: &str, material_index: usize) -> usize {
    let loaded = gltf_loader::load_gltf(path).unwrap();
    
    // Convert to our Vertex format
    let vertices: Vec<Vertex> = loaded.vertices.iter().map(|v| Vertex {
        position: v.position,
        normal: v.normal,
        uv: v.uv, // Use original UVs - no modification needed!
    }).collect();
    
    let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} Vertices", path)),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} Indices", path)),
        contents: bytemuck::cast_slice(&loaded.indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    
    self.meshes.push(Mesh {
        vertex_buffer,
        index_buffer,
        num_indices: loaded.indices.len() as u32,
        material_index,
    });
    
    self.meshes.len() - 1
}
```

### Step 3: Create Scene (15 minutes)
```rust
fn load_scene(&mut self) {
    println!("📦 Loading scene...");
    
    // Create materials (one per texture)
    let wood_mat = self.create_material("Wood", "assets/textures/planks.png");
    let stone_mat = self.create_material("Stone", "assets/textures/cobblestone.png");
    let grass_mat = self.create_material("Grass", "assets/textures/texture-d.png");
    
    // Load meshes
    let tree_mesh = self.load_gltf_mesh("assets/models/tree_default.glb", wood_mat);
    let rock_mesh = self.load_gltf_mesh("assets/models/rock_largeA.glb", stone_mat);
    
    // Create scene objects
    for i in 0..10 {
        self.objects.push(SceneObject {
            mesh_index: tree_mesh,
            position: Vec3::new(i as f32 * 5.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
    }
    
    for i in 0..5 {
        self.objects.push(SceneObject {
            mesh_index: rock_mesh,
            position: Vec3::new(i as f32 * 8.0, 0.0, 10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
    }
    
    println!("✅ Scene loaded: {} objects", self.objects.len());
}
```

### Step 4: Implement Rendering (30 minutes)
```rust
fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    // ... existing setup code ...
    
    {
        let mut render_pass = encoder.begin_render_pass(...);
        
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        
        // Render all objects
        for object in &self.objects {
            let mesh = &self.meshes[object.mesh_index];
            let material = &self.materials[mesh.material_index];
            
            // Set material bind group
            render_pass.set_bind_group(1, &material.bind_group, &[]);
            
            // Set mesh buffers
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            
            // Draw (add model matrix via push constants or instance buffer later)
            render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
        }
    }
    
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}
```

### Step 5: Add Model Matrix Support (30 minutes)
Update shader to use per-instance model matrices via push constants or instance buffer.

## Why This Works

1. **Each material = one texture** (simple, predictable)
2. **GLTF UVs unchanged** (models work as designed)
3. **No atlas complexity** (no remapping, no bleeding)
4. **Sampler mode Repeat** (correct for individual textures)
5. **One material binding per draw call** (slight overhead, but CORRECT)

## Performance Notes

- **Bind group switches**: ~5-20 materials = 5-20 switches per frame
- **Impact**: Negligible (<0.1ms on modern GPUs)
- **Benefit**: CORRECTNESS and simplicity

## Migration from Old Code

1. Keep `gltf_loader.rs` (already works)
2. Delete `main_bevy_v2.rs` (broken foundation)
3. Delete `pbr_shader.wgsl` (atlas complexity)
4. Delete `atlas_packer.rs` (not needed)
5. Use `main_clean.rs` + `shader_clean.wgsl`

## Timeline

- **Total**: ~2-3 hours to complete
- **Result**: Clean, working showcase with proper texturing
- **Bonus**: Easy to extend (add more materials, objects, features)

## Testing Plan

1. Load one tree → verify correct wood texture
2. Load one rock → verify correct stone texture
3. Load multiple objects → verify no texture bleeding
4. Verify FPS acceptable (should be 60+ FPS easily)
5. Add terrain later using same pattern (one texture = one material)
