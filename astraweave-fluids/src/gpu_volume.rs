//! GPU-accelerated water volume rendering
//!
//! This module provides GPU integration for the volumetric water system,
//! including 3D texture upload and surface mesh generation using a
//! heightfield-based approach.

use crate::volume_grid::{WaterVolumeGrid, WaterCell};
use bytemuck::{Pod, Zeroable};
use glam::{IVec3, Vec2, Vec3, UVec3};
use wgpu::util::DeviceExt;

/// Minimum water level to consider a cell as containing water
const MIN_WATER_LEVEL: f32 = 0.01;

/// GPU-friendly water cell data (16 bytes, aligned for GPU)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuWaterCell {
    /// Water level (0.0-1.0)
    pub level: f32,
    /// Flow velocity X
    pub velocity_x: f32,
    /// Flow velocity Y
    pub velocity_y: f32,
    /// Flow velocity Z
    pub velocity_z: f32,
}

impl GpuWaterCell {
    /// Create from a water cell
    pub fn from_cell(cell: &WaterCell) -> Self {
        Self {
            level: cell.level,
            velocity_x: cell.velocity.x,
            velocity_y: cell.velocity.y,
            velocity_z: cell.velocity.z,
        }
    }
}

/// Vertex format for water surface mesh (32 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct WaterSurfaceVertex {
    /// World position
    pub position: [f32; 3],
    /// Surface normal
    pub normal: [f32; 3],
    /// UV coordinates for flow effects
    pub uv: [f32; 2],
}

impl WaterSurfaceVertex {
    /// Vertex buffer layout for wgpu
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // UV
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Uniform buffer for water volume shader (64 bytes, 16-byte aligned)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct WaterVolumeUniforms {
    /// Volume dimensions (width, height, depth, padding)
    pub dimensions: [u32; 4],
    /// World-space origin of the volume
    pub origin: [f32; 4],
    /// Cell size in world units (x, y, z, padding)
    pub cell_size: [f32; 4],
    /// Time for animation (time, flow_speed, wave_height, wave_frequency)
    pub animation: [f32; 4],
}

impl Default for WaterVolumeUniforms {
    fn default() -> Self {
        Self {
            dimensions: [64, 64, 64, 0],
            origin: [0.0, 0.0, 0.0, 0.0],
            cell_size: [1.0, 1.0, 1.0, 0.0],
            animation: [0.0, 1.0, 0.1, 2.0],
        }
    }
}

/// GPU resources for water volume rendering
pub struct WaterVolumeGpu {
    /// 3D texture containing water cell data
    volume_texture: wgpu::Texture,
    /// View for sampling the volume texture (used by render pass)
    #[allow(dead_code)]
    volume_view: wgpu::TextureView,
    /// Sampler for the volume texture (used by render pass)
    #[allow(dead_code)]
    volume_sampler: wgpu::Sampler,
    /// Uniform buffer for shader parameters
    uniform_buffer: wgpu::Buffer,
    /// Bind group for the water volume
    bind_group: wgpu::BindGroup,
    /// Bind group layout for the water volume
    bind_group_layout: wgpu::BindGroupLayout,
    /// Volume dimensions
    dimensions: UVec3,
    /// Current uniform values
    uniforms: WaterVolumeUniforms,
    /// Staging buffer for CPU -> GPU uploads (used during upload)
    #[allow(dead_code)]
    staging_buffer: wgpu::Buffer,
    /// Whether the volume needs re-upload
    dirty: bool,
}

impl WaterVolumeGpu {
    /// Create a new GPU water volume
    pub fn new(device: &wgpu::Device, dimensions: UVec3) -> Self {
        let volume_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Water Volume 3D Texture"),
            size: wgpu::Extent3d {
                width: dimensions.x,
                height: dimensions.y,
                depth_or_array_layers: dimensions.z,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let volume_view = volume_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let volume_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Water Volume Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniforms = WaterVolumeUniforms {
            dimensions: [dimensions.x, dimensions.y, dimensions.z, 0],
            ..Default::default()
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Water Volume Uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Water Volume Bind Group Layout"),
            entries: &[
                // Volume texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Water Volume Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&volume_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&volume_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Create staging buffer for uploads
        let staging_size = (dimensions.x * dimensions.y * dimensions.z) as usize
            * std::mem::size_of::<GpuWaterCell>();
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Water Volume Staging Buffer"),
            size: staging_size as u64,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });

        Self {
            volume_texture,
            volume_view,
            volume_sampler,
            uniform_buffer,
            bind_group,
            bind_group_layout,
            dimensions,
            uniforms,
            staging_buffer,
            dirty: true,
        }
    }

    /// Upload water volume data to the GPU
    pub fn upload(&mut self, queue: &wgpu::Queue, grid: &WaterVolumeGrid) {
        // Convert grid data to GPU format
        let mut gpu_cells = Vec::with_capacity(
            (self.dimensions.x * self.dimensions.y * self.dimensions.z) as usize,
        );

        for z in 0..self.dimensions.z as i32 {
            for y in 0..self.dimensions.y as i32 {
                for x in 0..self.dimensions.x as i32 {
                    let pos = IVec3::new(x, y, z);
                    let cell = grid.get_cell(pos).cloned().unwrap_or_default();
                    gpu_cells.push(GpuWaterCell::from_cell(&cell));
                }
            }
        }

        // Upload to GPU
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.volume_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&gpu_cells),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.dimensions.x * std::mem::size_of::<GpuWaterCell>() as u32),
                rows_per_image: Some(self.dimensions.y),
            },
            wgpu::Extent3d {
                width: self.dimensions.x,
                height: self.dimensions.y,
                depth_or_array_layers: self.dimensions.z,
            },
        );

        self.dirty = false;
    }

    /// Update uniforms
    pub fn update_uniforms(&mut self, queue: &wgpu::Queue, time: f32) {
        self.uniforms.animation[0] = time;
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }

    /// Set world origin
    pub fn set_origin(&mut self, origin: Vec3) {
        self.uniforms.origin = [origin.x, origin.y, origin.z, 0.0];
    }

    /// Set cell size
    pub fn set_cell_size(&mut self, size: Vec3) {
        self.uniforms.cell_size = [size.x, size.y, size.z, 0.0];
    }

    /// Set flow animation speed
    pub fn set_flow_speed(&mut self, speed: f32) {
        self.uniforms.animation[1] = speed;
    }

    /// Set wave height
    pub fn set_wave_height(&mut self, height: f32) {
        self.uniforms.animation[2] = height;
    }

    /// Set wave frequency
    pub fn set_wave_frequency(&mut self, frequency: f32) {
        self.uniforms.animation[3] = frequency;
    }

    /// Get the bind group layout
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Get the bind group
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Get volume dimensions
    pub fn dimensions(&self) -> UVec3 {
        self.dimensions
    }

    /// Check if the volume needs re-upload
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the volume as dirty (needs re-upload)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Generate a surface mesh from the water volume using heightfield approach
    ///
    /// This scans columns (x, z) and finds the water surface height for each,
    /// then generates a quad mesh representing the water surface.
    pub fn generate_surface_mesh(&self, grid: &WaterVolumeGrid) -> (Vec<WaterSurfaceVertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let cell_size = Vec3::new(
            self.uniforms.cell_size[0],
            self.uniforms.cell_size[1],
            self.uniforms.cell_size[2],
        );
        let origin = Vec3::new(
            self.uniforms.origin[0],
            self.uniforms.origin[1],
            self.uniforms.origin[2],
        );

        // For each column (x, z), find the water surface height
        for z in 0..self.dimensions.z as i32 - 1 {
            for x in 0..self.dimensions.x as i32 - 1 {
                // Sample heights at the four corners of this grid cell
                let h00 = self.sample_column_height(grid, x, z);
                let h10 = self.sample_column_height(grid, x + 1, z);
                let h01 = self.sample_column_height(grid, x, z + 1);
                let h11 = self.sample_column_height(grid, x + 1, z + 1);

                // Skip if no water in any corner
                if h00.is_none() && h10.is_none() && h01.is_none() && h11.is_none() {
                    continue;
                }

                // Use a default height for corners without water (slightly below neighbors)
                let avg_height = [h00, h10, h01, h11]
                    .iter()
                    .filter_map(|h| *h)
                    .sum::<f32>()
                    / [h00, h10, h01, h11]
                        .iter()
                        .filter(|h| h.is_some())
                        .count()
                        .max(1) as f32;

                let y00 = h00.unwrap_or(avg_height - 0.1);
                let y10 = h10.unwrap_or(avg_height - 0.1);
                let y01 = h01.unwrap_or(avg_height - 0.1);
                let y11 = h11.unwrap_or(avg_height - 0.1);

                // Calculate world positions
                let p00 = origin + Vec3::new(x as f32 * cell_size.x, y00, z as f32 * cell_size.z);
                let p10 = origin + Vec3::new((x + 1) as f32 * cell_size.x, y10, z as f32 * cell_size.z);
                let p01 = origin + Vec3::new(x as f32 * cell_size.x, y01, (z + 1) as f32 * cell_size.z);
                let p11 = origin + Vec3::new((x + 1) as f32 * cell_size.x, y11, (z + 1) as f32 * cell_size.z);

                // Calculate normals using gradient
                let n00 = self.calculate_surface_normal(grid, x, z);
                let n10 = self.calculate_surface_normal(grid, x + 1, z);
                let n01 = self.calculate_surface_normal(grid, x, z + 1);
                let n11 = self.calculate_surface_normal(grid, x + 1, z + 1);

                // UV coordinates (for flow effects)
                let uv00 = Vec2::new(x as f32 / self.dimensions.x as f32, z as f32 / self.dimensions.z as f32);
                let uv10 = Vec2::new((x + 1) as f32 / self.dimensions.x as f32, z as f32 / self.dimensions.z as f32);
                let uv01 = Vec2::new(x as f32 / self.dimensions.x as f32, (z + 1) as f32 / self.dimensions.z as f32);
                let uv11 = Vec2::new((x + 1) as f32 / self.dimensions.x as f32, (z + 1) as f32 / self.dimensions.z as f32);

                // Add vertices
                let base_index = vertices.len() as u32;
                vertices.push(WaterSurfaceVertex {
                    position: p00.into(),
                    normal: n00.into(),
                    uv: uv00.into(),
                });
                vertices.push(WaterSurfaceVertex {
                    position: p10.into(),
                    normal: n10.into(),
                    uv: uv10.into(),
                });
                vertices.push(WaterSurfaceVertex {
                    position: p01.into(),
                    normal: n01.into(),
                    uv: uv01.into(),
                });
                vertices.push(WaterSurfaceVertex {
                    position: p11.into(),
                    normal: n11.into(),
                    uv: uv11.into(),
                });

                // Two triangles for the quad
                indices.extend_from_slice(&[
                    base_index,
                    base_index + 1,
                    base_index + 2,
                    base_index + 1,
                    base_index + 3,
                    base_index + 2,
                ]);
            }
        }

        (vertices, indices)
    }

    /// Calculate the surface normal at a grid column using finite differences
    fn calculate_surface_normal(&self, grid: &WaterVolumeGrid, x: i32, z: i32) -> Vec3 {
        let h_center = self.sample_column_height(grid, x, z).unwrap_or(0.0);

        // Sample neighboring heights
        let h_left = if x > 0 {
            self.sample_column_height(grid, x - 1, z).unwrap_or(h_center)
        } else {
            h_center
        };
        let h_right = if x < self.dimensions.x as i32 - 1 {
            self.sample_column_height(grid, x + 1, z).unwrap_or(h_center)
        } else {
            h_center
        };
        let h_back = if z > 0 {
            self.sample_column_height(grid, x, z - 1).unwrap_or(h_center)
        } else {
            h_center
        };
        let h_front = if z < self.dimensions.z as i32 - 1 {
            self.sample_column_height(grid, x, z + 1).unwrap_or(h_center)
        } else {
            h_center
        };

        // Calculate gradient
        let dx = (h_right - h_left) / (2.0 * self.uniforms.cell_size[0]);
        let dz = (h_front - h_back) / (2.0 * self.uniforms.cell_size[2]);

        // Normal from gradient
        Vec3::new(-dx, 1.0, -dz).normalize()
    }

    /// Sample the water surface height at a grid column (x, z)
    /// Returns None if the column has no water
    fn sample_column_height(&self, grid: &WaterVolumeGrid, x: i32, z: i32) -> Option<f32> {
        // Scan from top to bottom to find the first cell with water
        for y in (0..self.dimensions.y as i32).rev() {
            let pos = IVec3::new(x, y, z);
            if let Some(cell) = grid.get_cell(pos) {
                if cell.level > MIN_WATER_LEVEL {
                    // Interpolate height based on water level within the cell
                    let base_height = y as f32 * self.uniforms.cell_size[1];
                    let water_height = base_height + cell.level * self.uniforms.cell_size[1];
                    return Some(water_height);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_water_cell_size() {
        assert_eq!(std::mem::size_of::<GpuWaterCell>(), 16); // 4 floats * 4 bytes
    }

    #[test]
    fn test_water_surface_vertex_size() {
        assert_eq!(std::mem::size_of::<WaterSurfaceVertex>(), 32); // 8 floats * 4 bytes
    }

    #[test]
    fn test_uniforms_size() {
        // Ensure proper alignment for GPU
        let size = std::mem::size_of::<WaterVolumeUniforms>();
        assert!(size % 16 == 0, "Uniforms must be 16-byte aligned, got {}", size);
    }
}
