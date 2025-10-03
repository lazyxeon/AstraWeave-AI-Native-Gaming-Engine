//! Voxelization Pipeline - Converts Voxel Terrain Meshes to Radiance Field
//!
//! This module implements conservative rasterization-based voxelization of
//! voxel terrain meshes (from Marching Cubes) into a 3D radiance texture
//! for use with VXGI cone tracing.

use glam::Vec3;
use wgpu::util::DeviceExt;

/// Configuration for voxelization
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelizationConfig {
    /// Voxel grid resolution (power of 2)
    pub voxel_resolution: u32,
    /// World space size covered by voxel grid
    pub world_size: f32,
    /// Number of triangles to voxelize
    pub triangle_count: u32,
    /// Intensity of direct lighting
    pub light_intensity: f32,
}

impl Default for VoxelizationConfig {
    fn default() -> Self {
        Self {
            voxel_resolution: 256,
            world_size: 1000.0,
            triangle_count: 0,
            light_intensity: 1.0,
        }
    }
}

/// Vertex data for voxelization
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelVertex {
    /// World-space position
    pub position: [f32; 3],
    /// Normal vector
    pub normal: [f32; 3],
}

impl VoxelVertex {
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
        }
    }
}

/// Material data for voxelization
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelMaterial {
    /// Base color (RGB)
    pub albedo: [f32; 3],
    /// Metallic factor
    pub metallic: f32,
    /// Roughness factor
    pub roughness: f32,
    /// Emissive radiance (RGB)
    pub emissive: [f32; 3],
}

impl Default for VoxelMaterial {
    fn default() -> Self {
        Self {
            albedo: [0.8, 0.8, 0.8],
            metallic: 0.0,
            roughness: 0.8,
            emissive: [0.0, 0.0, 0.0],
        }
    }
}

impl VoxelMaterial {
    pub fn from_albedo(albedo: Vec3) -> Self {
        Self {
            albedo: albedo.to_array(),
            ..Default::default()
        }
    }

    pub fn emissive(emissive: Vec3) -> Self {
        Self {
            emissive: emissive.to_array(),
            ..Default::default()
        }
    }
}

/// Mesh data for voxelization
pub struct VoxelizationMesh {
    pub vertices: Vec<VoxelVertex>,
    pub indices: Vec<u32>,
    pub material: VoxelMaterial,
}

impl VoxelizationMesh {
    pub fn new(vertices: Vec<VoxelVertex>, indices: Vec<u32>, material: VoxelMaterial) -> Self {
        Self {
            vertices,
            indices,
            material,
        }
    }

    pub fn triangle_count(&self) -> u32 {
        (self.indices.len() / 3) as u32
    }
}

/// Voxelization compute pipeline
pub struct VoxelizationPipeline {
    config: VoxelizationConfig,

    // Shader module
    shader_module: wgpu::ShaderModule,

    // Compute pipelines
    voxelize_pipeline: wgpu::ComputePipeline,
    clear_pipeline: wgpu::ComputePipeline,

    // Bind group layout
    bind_group_layout: wgpu::BindGroupLayout,

    // GPU buffers (reusable)
    config_buffer: wgpu::Buffer,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    material_buffer: wgpu::Buffer,

    // Statistics
    stats: VoxelizationStats,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct VoxelizationStats {
    pub total_triangles: u32,
    pub total_vertices: u32,
    pub voxelization_time_ms: f32,
    pub clear_time_ms: f32,
}

impl VoxelizationPipeline {
    /// Create a new voxelization pipeline
    pub fn new(device: &wgpu::Device, config: VoxelizationConfig) -> Self {
        // Load shader
        let shader_source = include_str!("../shaders/vxgi_voxelize.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("VXGI Voxelization Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Voxelization Bind Group Layout"),
            entries: &[
                // Config uniform buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Vertex storage buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Index storage buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Material storage buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Voxel texture (read-write storage)
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D3,
                    },
                    count: None,
                },
            ],
        });

        // Create compute pipeline for voxelization
        let voxelize_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Voxelization Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let voxelize_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Voxelization Compute Pipeline"),
            layout: Some(&voxelize_pipeline_layout),
            module: &shader_module,
            entry_point: "voxelize",
            compilation_options: Default::default(),
        });

        // Create compute pipeline for clearing voxels
        let clear_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Voxel Clear Compute Pipeline"),
            layout: Some(&voxelize_pipeline_layout),
            module: &shader_module,
            entry_point: "clear_voxels",
            compilation_options: Default::default(),
        });

        // Create config buffer
        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxelization Config Buffer"),
            contents: bytemuck::cast_slice(&[config]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create material buffer with default material
        let default_material = VoxelMaterial::default();
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Voxelization Material Buffer"),
            contents: bytemuck::cast_slice(&[default_material]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            config,
            shader_module,
            voxelize_pipeline,
            clear_pipeline,
            bind_group_layout,
            config_buffer,
            vertex_buffer: None,
            index_buffer: None,
            material_buffer,
            stats: VoxelizationStats::default(),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, queue: &wgpu::Queue, config: VoxelizationConfig) {
        self.config = config;
        queue.write_buffer(&self.config_buffer, 0, bytemuck::cast_slice(&[config]));
    }

    /// Upload mesh data to GPU
    fn upload_mesh(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &VoxelizationMesh) {
        // Create or recreate vertex buffer
        if self.vertex_buffer.is_none()
            || self.vertex_buffer.as_ref().unwrap().size()
                < (mesh.vertices.len() * std::mem::size_of::<VoxelVertex>()) as u64
        {
            self.vertex_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Voxelization Vertex Buffer"),
                    contents: bytemuck::cast_slice(&mesh.vertices),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else {
            queue.write_buffer(
                self.vertex_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&mesh.vertices),
            );
        }

        // Create or recreate index buffer
        if self.index_buffer.is_none()
            || self.index_buffer.as_ref().unwrap().size()
                < (mesh.indices.len() * std::mem::size_of::<u32>()) as u64
        {
            self.index_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Voxelization Index Buffer"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                },
            ));
        } else {
            queue.write_buffer(
                self.index_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&mesh.indices),
            );
        }

        // Update material buffer
        queue.write_buffer(
            &self.material_buffer,
            0,
            bytemuck::cast_slice(&[mesh.material]),
        );

        // Update config with triangle count
        let mut updated_config = self.config;
        updated_config.triangle_count = mesh.triangle_count();
        self.update_config(queue, updated_config);

        // Update stats
        self.stats.total_triangles = mesh.triangle_count();
        self.stats.total_vertices = mesh.vertices.len() as u32;
    }

    /// Clear voxel texture to prepare for voxelization
    pub fn clear_voxels(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        voxel_texture_view: &wgpu::TextureView,
    ) {
        let _start_time = std::time::Instant::now();

        // Create bind group for clear pass
        let bind_group = self.create_bind_group_for_clear(device, voxel_texture_view);

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Voxel Clear Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.clear_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        // Dispatch clear shader (8x8x8 workgroups)
        let workgroup_size = 8;
        let dispatch_size = (self.config.voxel_resolution + workgroup_size - 1) / workgroup_size;
        compute_pass.dispatch_workgroups(dispatch_size, dispatch_size, dispatch_size);

        drop(compute_pass);
    }

    /// Voxelize a mesh into the voxel texture
    pub fn voxelize_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        mesh: &VoxelizationMesh,
        voxel_texture_view: &wgpu::TextureView,
    ) {
        let _start_time = std::time::Instant::now();

        // Upload mesh data
        self.upload_mesh(device, queue, mesh);

        // Create bind group
        let bind_group = self.create_bind_group(device, voxel_texture_view);

        // Run voxelization compute pass
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("VXGI Voxelization Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.voxelize_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);

        // Dispatch one thread per triangle (workgroup size 64x1x1)
        let triangle_count = mesh.triangle_count();
        let workgroup_size = 64;
        let dispatch_size = (triangle_count + workgroup_size - 1) / workgroup_size;
        compute_pass.dispatch_workgroups(dispatch_size, 1, 1);

        drop(compute_pass);

        self.stats.voxelization_time_ms = _start_time.elapsed().as_secs_f32() * 1000.0;
    }

    /// Create bind group for voxelization
    fn create_bind_group(
        &self,
        device: &wgpu::Device,
        voxel_texture_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Voxelization Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.config_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.vertex_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.index_buffer.as_ref().unwrap().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(voxel_texture_view),
                },
            ],
        })
    }

    /// Create bind group for clear pass (minimal bindings)
    fn create_bind_group_for_clear(
        &self,
        device: &wgpu::Device,
        voxel_texture_view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        // For clear pass, we need dummy buffers for unused bindings
        let dummy_vertex_buffer = self.vertex_buffer.as_ref().unwrap_or(&self.config_buffer);
        let dummy_index_buffer = self.index_buffer.as_ref().unwrap_or(&self.config_buffer);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Voxel Clear Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.config_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: dummy_vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: dummy_index_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(voxel_texture_view),
                },
            ],
        })
    }

    /// Get configuration
    pub fn config(&self) -> &VoxelizationConfig {
        &self.config
    }

    /// Get statistics
    pub fn stats(&self) -> &VoxelizationStats {
        &self.stats
    }

    /// Get bind group layout
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voxelization_config_default() {
        let config = VoxelizationConfig::default();
        assert_eq!(config.voxel_resolution, 256);
        assert_eq!(config.world_size, 1000.0);
        assert_eq!(config.light_intensity, 1.0);
    }

    #[test]
    fn test_voxel_vertex_size() {
        assert_eq!(std::mem::size_of::<VoxelVertex>(), 24); // 3 floats pos + 3 floats normal
    }

    #[test]
    fn test_voxel_material_size() {
        assert_eq!(std::mem::size_of::<VoxelMaterial>(), 32); // 3 + 1 + 1 + 3 floats + padding
    }

    #[test]
    fn test_voxelization_mesh() {
        let vertices = vec![
            VoxelVertex::new(Vec3::ZERO, Vec3::Y),
            VoxelVertex::new(Vec3::X, Vec3::Y),
            VoxelVertex::new(Vec3::Z, Vec3::Y),
        ];
        let indices = vec![0, 1, 2];
        let material = VoxelMaterial::default();

        let mesh = VoxelizationMesh::new(vertices, indices, material);
        assert_eq!(mesh.triangle_count(), 1);
    }
}
