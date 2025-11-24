# Infinite Ocean Port Plan

## Overview
Port the Godot "Infinite Ocean" asset pack to the `fluids_demo` example using WGSL shaders and Rust/WGPU rendering.

---

## 1. Shader Analysis (Water.gdshader)

### Vertex Shader Logic
**Key Features:**
- **Texture-based wave displacement**: Uses noise texture (`wave`) sampled at two different UV offsets/directions
- **Distance-based LOD**: Wave height diminishes with distance from camera (0-85 units)
- **Dual wave system**: Two wave directions mixed together for variety
- **Tile edge smoothing**: Special handling for UV edges to prevent seams (lines 82-93)

**Math Breakdown:**
```glsl
// Two waves sampled with time-animated UVs
h1 = texture(wave, (world_pos.xz + offset) / noise_scale + (TIME * wave_direction) * time_scale).r
h2 = texture(wave, (world_pos.xz + offset) / noise_scale + (TIME * wave_direction2) * time_scale).r
hmix = mix(h1, h2, 0.5)

// Vertex displacement with distance attenuation
VERTEX.y += hmix * height_scale * (1.0 - vertex_distance_clamped)
```

**Parameters:**
- `noise_scale = 20.0` - UV tiling scale
- `height_scale = 2.0` - Wave amplitude
- `time_scale = 0.1-0.2` - Animation speed
- `wave_direction = (0.5, -0.2)` - Wave 1 direction
- `wave_direction2 = (-0.5, 0.5)` - Wave 2 direction

### Fragment Shader Logic
**Key Features:**
- **Depth-based coloring**: Beer's law for underwater color blending
- **Edge foam**: White foam at shallow water edges
- **Dual normal maps**: Two normal textures mixed for surface detail
- **Fresnel effect**: View-angle-dependent albedo mixing

**Math Breakdown:**
```glsl
// Depth reading from depth buffer
depth = PROJECTION_MATRIX[3][2] / (depth_texture + PROJECTION_MATRIX[2][2])
depth_blend = exp((depth + VERTEX.z + depth_offset) * -beers_law)

// Color mixing
depth_color = mix(color_shallow, color_deep, depth_blend_power)
color = mix(screen_color * depth_color, depth_color * 0.25, depth_blend_power * 0.5)

// Normal blending (two animated layers)
normal_blend = mix(texture(normal1, world_pos.xz/10 + time/10), texture(normal2, world_pos.xz/10 + time2/10), 0.5)
wave_normal = mix(wave_bump at time1, wave_bump at time2, 0.3)
NORMAL_MAP = mix(wave_normal, normal_blend, 0.5)
```

**Parameters:**
- `beers_law = 2.0` - Underwater color falloff
- `depth_offset = 1.5` - Depth calculation offset
- `color_deep = (0.11, 0.29, 0.33)` - Deep water color
- `color_shallow = (0.0, 0.47, 0.76)` - Shallow water color
- `edge_scale = 0.3` - Foam edge threshold

---

## 2. Mesh Tiling System (Ocean.gd)

### Grid Layout
The ocean uses a **17-tile grid** centered on the player/camera:

```
Tile Layout (XZ plane):
  9  10  11  12
     2   3   4
13  5   0   6  14
     7   8   1
 15  16  17   9

Tile 0: Center (high detail, 199 subdivisions)
Tiles 1-8: Ring 1 (medium detail, 99 subdivisions)
Tiles 9-16: Ring 2 (low detail, 0 subdivisions = coarse)
```

### Spawn Data (from GridSpawnInfo.tres)
```rust
// Positions (in tile units, multiply by 10.05m)
spawnPoints = [
    (0,0),           // Center
    (-1,-1), (-1,0), (-1,1), (0,1), (1,1), (1,0), (1,-1), (0,-1),  // Ring 1
    (-3,-3), (-3,0), (-3,3), (0,3), (3,3), (3,0), (3,-3), (0,-3)   // Ring 2
]

// Mesh subdivision levels
subdivision = [199, 99,99,99,99,99,99,99,99, 0,0,0,0,0,0,0,0]

// Tile scale factors
scale = [1, 1,1,1,1,1,1,1,1, 3,3,3,3,3,3,3,3]
```

### Dynamic Positioning
```gdscript
func _process(delta):
    RenderingServer.global_shader_parameter_set("ocean_pos", self.position)
```
- The `ocean_pos` uniform tracks camera/player position
- Used in shader for distance-based LOD
- Tiles should move with camera for infinite effect (not implemented in base pack)

---

## 3. Texture Generation

### No External Textures!
The pack uses **procedural noise textures** generated in Godot (from WaterPlane.tscn):

#### Wave Texture (height map)
```gdscript
FastNoiseLite:
  noise_type = 4 (Cellular)
  fractal_gain = 0.34
  fractal_weighted_strength = 0.6
NoiseTexture2D:
  seamless = true
```

#### Wave Bump (normal map from same noise)
```gdscript
FastNoiseLite:
  noise_type = 4 (Cellular)
  fractal_gain = 0.34
  fractal_weighted_strength = 0.6
NoiseTexture2D:
  seamless = true
  as_normal_map = true
```

#### Texture Normal 1
```gdscript
FastNoiseLite:
  noise_type = 3 (Simplex)
  frequency = 0.032
NoiseTexture2D:
  seamless = true
  as_normal_map = true
  bump_strength = 1.6
```

#### Texture Normal 2
```gdscript
FastNoiseLite:
  noise_type = 0 (Perlin)
  seed = 22
  frequency = 0.003
  fractal_lacunarity = 1.6
  fractal_gain = 0.47
  fractal_weighted_strength = 0.53
NoiseTexture2D:
  seamless = true
  as_normal_map = true
  bump_strength = 21.8
```

**Port Strategy:**
- Generate noise textures using a Rust noise library (e.g., `noise-rs`)
- Create 512x512 or 1024x1024 RGBA8 textures
- Upload to WGPU as `Texture2D` with sampler set to `repeat` and `linear` filtering

---

## 4. Implementation Plan

### File Structure
```
examples/fluids_demo/src/
├── main.rs                 (modify to add ocean mode)
├── fluid_renderer.rs       (keep existing)
├── ocean_renderer.rs       (NEW)
├── ocean_mesh.rs           (NEW)
├── ocean_noise.rs          (NEW)
└── shaders/
    ├── fluid.wgsl          (existing)
    ├── ocean_vertex.wgsl   (NEW)
    └── ocean_fragment.wgsl (NEW)
```

### Dependencies to Add
```toml
[dependencies]
noise = "0.9"  # For procedural noise generation
```

### Step-by-Step Implementation

#### Step 1: Noise Texture Generation (`ocean_noise.rs`)
```rust
pub struct OceanNoiseTextures {
    pub wave: wgpu::Texture,
    pub wave_bump: wgpu::Texture,
    pub normal1: wgpu::Texture,
    pub normal2: wgpu::Texture,
}

pub fn generate_ocean_textures(device: &wgpu::Device, queue: &wgpu::Queue) -> OceanNoiseTextures {
    // Use noise-rs to generate:
    // 1. Cellular noise for wave texture
    // 2. Convert to normal map for wave_bump
    // 3. Simplex noise for normal1
    // 4. Perlin noise for normal2
}
```

#### Step 2: Mesh Generation (`ocean_mesh.rs`)
```rust
pub struct OceanTile {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub position: Vec2,  // Grid position
    pub scale: f32,
    pub subdivision: u32,
}

pub fn generate_ocean_grid(device: &wgpu::Device) -> Vec<OceanTile> {
    // Generate 17 plane meshes with varying subdivision
    // Based on spawn data from GridSpawnInfo
}

fn create_plane_mesh(subdivision: u32, size: f32) -> (Vec<Vertex>, Vec<u16>) {
    // Generate plane with UV coordinates
}
```

#### Step 3: Ocean Vertex Shader (`ocean_vertex.wgsl`)
```wgsl
struct Uniforms {
    view_proj: mat4x4<f32>,
    ocean_pos: vec3<f32>,
    time: f32,
    wave_direction: vec2<f32>,
    wave_direction2: vec2<f32>,
    noise_scale: f32,
    height_scale: f32,
    time_scale: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var wave_texture: texture_2d<f32>;
@group(0) @binding(2) var wave_sampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    // Calculate world position
    let world_pos = position;
    
    // Distance-based LOD
    let vertex_distance = distance(world_pos, uniforms.ocean_pos);
    let clamped_dist = clamp(vertex_distance, 0.0, 85.0) / 85.0;
    
    // Sample wave textures
    let uv1 = world_pos.xz / uniforms.noise_scale + uniforms.time * uniforms.wave_direction * uniforms.time_scale;
    let uv2 = world_pos.xz / uniforms.noise_scale + uniforms.time * uniforms.wave_direction2 * uniforms.time_scale;
    
    let h1 = textureSampleLevel(wave_texture, wave_sampler, uv1, 0.0).r;
    let h2 = textureSampleLevel(wave_texture, wave_sampler, uv2, 0.0).r;
    let hmix = mix(h1, h2, 0.5);
    
    // Apply displacement
    var displaced_pos = world_pos;
    displaced_pos.y += hmix * uniforms.height_scale * (1.0 - clamped_dist);
    
    // Output
    var output: VertexOutput;
    output.position = uniforms.view_proj * vec4<f32>(displaced_pos, 1.0);
    output.world_pos = world_pos;
    output.uv = uv;
    return output;
}
```

#### Step 4: Ocean Fragment Shader (`ocean_fragment.wgsl`)
```wgsl
@group(1) @binding(0) var normal1_texture: texture_2d<f32>;
@group(1) @binding(1) var normal2_texture: texture_2d<f32>;
@group(1) @binding(2) var wave_bump_texture: texture_2d<f32>;
@group(1) @binding(3) var texture_sampler: sampler;
@group(1) @binding(4) var depth_texture: texture_depth_2d;

struct FragmentUniforms {
    albedo: vec3<f32>,
    albedo2: vec3<f32>,
    color_deep: vec3<f32>,
    color_shallow: vec3<f32>,
    beers_law: f32,
    depth_offset: f32,
    // ... other params
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample normal maps with time animation
    let time1 = uniforms.time * uniforms.wave_direction * uniforms.time_scale;
    let time2 = uniforms.time * uniforms.wave_direction2 * uniforms.time_scale;
    
    let normal1 = textureSample(normal1_texture, texture_sampler, in.world_pos.xz / 10.0 + time1 / 10.0).rgb;
    let normal2 = textureSample(normal2_texture, texture_sampler, in.world_pos.xz / 10.0 + time2 / 10.0).rgb;
    let normal_blend = mix(normal1, normal2, 0.5);
    
    // Depth-based color (requires depth buffer access - may need separate pass)
    // For now, use a simple ocean color
    
    // Fresnel
    let fresnel = pow(1.0 - max(dot(normalize(normal_blend), normalize(view_dir)), 0.0), 5.0);
    let surface_color = mix(uniforms.albedo, uniforms.albedo2, fresnel);
    
    return vec4<f32>(surface_color, 1.0);
}
```

#### Step 5: Ocean Renderer (`ocean_renderer.rs`)
```rust
pub struct OceanRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    textures: OceanNoiseTextures,
    tiles: Vec<OceanTile>,
}

impl OceanRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, surface_format: wgpu::TextureFormat) -> Self {
        // Create pipeline
        // Generate textures
        // Generate mesh tiles
    }
    
    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera_pos: Vec3,
        view_proj: Mat4,
        time: f32,
    ) {
        // Update uniforms
        // Render 17 tiles
    }
}
```

#### Step 6: Integrate into `main.rs`
```rust
struct State {
    // ... existing fields
    ocean_renderer: OceanRenderer,  // NEW
    mode: DemoMode,  // NEW enum: Fluid or Ocean
}

enum DemoMode {
    Fluid,
    Ocean,
}

// Add keyboard input to toggle between modes
// Press 'O' for ocean, 'F' for fluid
```

---

## 5. Challenges & Solutions

### Challenge 1: Depth Buffer Access in Fragment Shader
**Problem:** WGSL doesn't have `DEPTH_TEXTURE` and `SCREEN_TEXTURE` like Godot.

**Solution:**
- Render ocean in a separate pass
- Use depth texture from previous pass as binding
- Or simplify: Skip depth-based edge foam for first iteration

### Challenge 2: Tile Edge Seams
**Problem:** UV-based displacement can cause cracks between tiles.

**Solution:**
- Ensure seamless noise textures
- Use same world-space UVs (not tile-local UVs)
- The Godot shader's edge smoothing (lines 82-93) may not be necessary with proper world-space UVs

### Challenge 3: Performance with 17 Meshes
**Problem:** 17 draw calls per frame.

**Solution:**
- Instance rendering: Single draw call with instance buffer for positions/scales
- Merge meshes into single vertex buffer with offsets
- Start simple (17 draws), optimize later if needed

### Challenge 4: Normal Map Conversion
**Problem:** Godot auto-generates normal maps from height maps.

**Solution:**
- Use Sobel filter to convert height texture to normal map:
```rust
fn height_to_normal(height_data: &[f32], width: usize, height: usize) -> Vec<u8> {
    // Sobel operator on height values
    // Convert to RGB normal map (R=X, G=Y, B=Z)
}
```

---

## 6. Testing Plan

### Phase 1: Static Ocean (No Animation)
- Generate noise textures
- Render single tile with flat shading
- Verify mesh generation and UVs

### Phase 2: Vertex Displacement
- Add wave texture sampling in vertex shader
- Test distance-based LOD
- Verify no seams between tiles

### Phase 3: Surface Shading
- Add normal map blending
- Implement Fresnel effect
- Test with moving camera

### Phase 4: Full Features
- Add depth-based coloring (if depth buffer accessible)
- Add edge foam
- Optimize performance

---

## 7. Parameter Tuning Guide

Once implemented, these parameters can be tweaked for different ocean styles:

### Calm Ocean
```rust
wave_direction: (0.3, -0.1),
wave_direction2: (-0.2, 0.3),
time_scale: 0.05,
height_scale: 0.5,
```

### Stormy Ocean
```rust
wave_direction: (1.0, -0.5),
wave_direction2: (-0.8, 1.0),
time_scale: 0.25,
height_scale: 4.0,
```

### Tropical Ocean
```rust
albedo: (0.0, 0.5, 0.7),
color_shallow: (0.0, 0.7, 0.9),
color_deep: (0.0, 0.2, 0.4),
```

---

## 8. Future Enhancements

1. **True Infinite Tiling**: Move tiles with camera to create endless ocean
2. **FFT-based waves**: Replace texture noise with ocean wave simulation
3. **Foam particles**: Add particle system for wave crests
4. **Caustics**: Underwater light patterns
5. **Buoyancy**: Physics interaction with fluid system

---

## Summary

**Core Requirements:**
- 4 procedural noise textures (512x512 recommended)
- 17 plane meshes with varying subdivision
- Vertex shader with dual-wave displacement and LOD
- Fragment shader with normal blending and Fresnel
- Uniform buffer with 15+ parameters

**Estimated Complexity:**
- Noise generation: ~100 LOC
- Mesh generation: ~150 LOC
- Shaders: ~200 LOC (WGSL)
- Renderer: ~300 LOC
- Integration: ~50 LOC

**Total:** ~800 lines of new code

**Time Estimate:** 6-8 hours for full implementation and testing
