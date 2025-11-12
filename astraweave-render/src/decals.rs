// Screen-Space Decal System
// Implements deferred decal rendering with projection matrices and atlas management

use anyhow::{Context, Result};
use glam::{Mat4, Quat, Vec3};
use wgpu;

/// Decal instance on GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuDecal {
    /// Inverse projection matrix (world to decal space)
    pub inv_projection: [[f32; 4]; 4],
    /// Albedo color tint
    pub albedo_tint: [f32; 4],
    /// Normal strength and roughness/metallic override
    pub params: [f32; 4], // x=normal_strength, y=roughness, z=metallic, w=blend_mode
    /// Atlas UV offset and scale
    pub atlas_uv: [f32; 4], // xy=offset, zw=scale
}

/// Decal blend mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecalBlendMode {
    /// Multiply albedo, blend normals
    Multiply = 0,
    /// Additive blend
    Additive = 1,
    /// Alpha blend
    AlphaBlend = 2,
    /// Stain (darken only)
    Stain = 3,
}

/// Decal definition (CPU-side)
#[derive(Debug, Clone)]
pub struct Decal {
    /// Position in world space
    pub position: Vec3,
    /// Rotation quaternion
    pub rotation: Quat,
    /// Scale (half-extents of decal box)
    pub scale: Vec3,
    /// Albedo tint color
    pub albedo_tint: [f32; 4],
    /// Normal strength (0-1)
    pub normal_strength: f32,
    /// Roughness override
    pub roughness: f32,
    /// Metallic override
    pub metallic: f32,
    /// Blend mode
    pub blend_mode: DecalBlendMode,
    /// Atlas UV coordinates (offset and scale)
    pub atlas_uv: ([f32; 2], [f32; 2]),
    /// Fade-out timer (0 = permanent, >0 = fade duration)
    pub fade_duration: f32,
    /// Current fade time
    pub fade_time: f32,
}

impl Decal {
    /// Create a new decal
    pub fn new(
        position: Vec3,
        rotation: Quat,
        scale: Vec3,
        atlas_uv: ([f32; 2], [f32; 2]),
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
            albedo_tint: [1.0, 1.0, 1.0, 1.0],
            normal_strength: 1.0,
            roughness: 0.5,
            metallic: 0.0,
            blend_mode: DecalBlendMode::AlphaBlend,
            atlas_uv,
            fade_duration: 0.0,
            fade_time: 0.0,
        }
    }

    /// Update fade-out
    pub fn update(&mut self, dt: f32) -> bool {
        if self.fade_duration > 0.0 {
            self.fade_time += dt;
            if self.fade_time >= self.fade_duration {
                return false; // Decal should be removed
            }
            // Update alpha based on fade
            let fade_alpha = 1.0 - (self.fade_time / self.fade_duration);
            self.albedo_tint[3] = fade_alpha;
        }
        true
    }

    /// Convert to GPU representation
    pub fn to_gpu(&self) -> GpuDecal {
        // Build projection matrix (world to decal space)
        let transform = Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
        let inv_projection = transform.inverse();

        GpuDecal {
            inv_projection: inv_projection.to_cols_array_2d(),
            albedo_tint: self.albedo_tint,
            params: [
                self.normal_strength,
                self.roughness,
                self.metallic,
                self.blend_mode as u32 as f32,
            ],
            atlas_uv: [
                self.atlas_uv.0[0],
                self.atlas_uv.0[1],
                self.atlas_uv.1[0],
                self.atlas_uv.1[1],
            ],
        }
    }
}

/// Decal atlas manager
pub struct DecalAtlas {
    /// Atlas texture
    pub texture: wgpu::Texture,
    /// Atlas texture view
    pub view: wgpu::TextureView,
    /// Atlas sampler
    pub sampler: wgpu::Sampler,
    /// Atlas size
    pub size: u32,
    /// Grid size (decals per row/column)
    pub grid_size: u32,
}

impl DecalAtlas {
    /// Create a new decal atlas
    pub fn new(device: &wgpu::Device, size: u32, grid_size: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Decal Atlas"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Decal Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            size,
            grid_size,
        }
    }

    /// Get UV coordinates for a decal in the atlas
    pub fn get_uv(&self, atlas_x: u32, atlas_y: u32) -> ([f32; 2], [f32; 2]) {
        let cell_size = 1.0 / self.grid_size as f32;
        let offset = [
            atlas_x as f32 * cell_size,
            atlas_y as f32 * cell_size,
        ];
        let scale = [cell_size, cell_size];
        (offset, scale)
    }
}

/// Decal system manager
pub struct DecalSystem {
    /// Active decals
    decals: Vec<Decal>,
    /// GPU buffer for decal instances
    decal_buffer: wgpu::Buffer,
    /// Decal atlas
    atlas: DecalAtlas,
    /// Maximum decals
    max_decals: usize,
}

impl DecalSystem {
    /// Create a new decal system
    pub fn new(device: &wgpu::Device, max_decals: usize, atlas_size: u32, atlas_grid: u32) -> Self {
        let decal_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Decal Buffer"),
            size: (max_decals * std::mem::size_of::<GpuDecal>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let atlas = DecalAtlas::new(device, atlas_size, atlas_grid);

        Self {
            decals: Vec::with_capacity(max_decals),
            decal_buffer,
            atlas,
            max_decals,
        }
    }

    /// Add a decal
    pub fn add_decal(&mut self, decal: Decal) -> Result<()> {
        anyhow::ensure!(
            self.decals.len() < self.max_decals,
            "Maximum decal count reached"
        );
        self.decals.push(decal);
        Ok(())
    }

    /// Update all decals
    pub fn update(&mut self, queue: &wgpu::Queue, dt: f32) {
        // Update and remove faded decals
        self.decals.retain_mut(|decal| decal.update(dt));

        // Upload to GPU
        let gpu_decals: Vec<GpuDecal> = self.decals.iter().map(|d| d.to_gpu()).collect();
        queue.write_buffer(&self.decal_buffer, 0, bytemuck::cast_slice(&gpu_decals));
    }

    /// Get decal count
    pub fn count(&self) -> usize {
        self.decals.len()
    }

    /// Get decal buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.decal_buffer
    }

    /// Get atlas
    pub fn atlas(&self) -> &DecalAtlas {
        &self.atlas
    }
}

// Decal rendering shader (would be used in deferred pass)
pub const DECAL_SHADER: &str = r#"
struct Decal {
    inv_projection: mat4x4<f32>,
    albedo_tint: vec4<f32>,
    params: vec4<f32>,  // normal_strength, roughness, metallic, blend_mode
    atlas_uv: vec4<f32>, // offset.xy, scale.zw
}

@group(0) @binding(0) var<storage, read> decals: array<Decal>;
@group(0) @binding(1) var decal_atlas: texture_2d<f32>;
@group(0) @binding(2) var decal_sampler: sampler;
@group(0) @binding(3) var depth_texture: texture_2d<f32>;
@group(0) @binding(4) var gbuffer_normal: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    // Read depth and reconstruct world position
    let depth = textureLoad(depth_texture, vec2<i32>(frag_coord.xy), 0).r;
    // ... reconstruct world pos from depth ...
    
    // For each decal, check if pixel is inside decal box
    // If inside, sample atlas and blend
    
    var output_color = vec4<f32>(0.0);
    
    // ... decal projection and blending logic ...
    
    return output_color;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_decal_size() {
        assert_eq!(std::mem::size_of::<GpuDecal>(), 96);
    }

    #[test]
    fn test_decal_new() {
        let decal = Decal::new(
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
            ([0.0, 0.0], [1.0, 1.0]),
        );
        assert_eq!(decal.albedo_tint, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_decal_fade() {
        let mut decal = Decal::new(
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE,
            ([0.0, 0.0], [1.0, 1.0]),
        );
        decal.fade_duration = 2.0;
        
        assert!(decal.update(1.0)); // Still alive at 50%
        assert!(decal.update(0.5)); // Still alive at 75%
        assert!(!decal.update(0.6)); // Dead after exceeding duration
    }
}
