//! Voxel Global Illumination (VXGI) using Voxel Cone Tracing
//!
//! This module implements VXGI for dynamic global illumination on voxel terrain.
//! It uses voxel cone tracing to sample indirect lighting from a sparse voxel
//! radiance field built from the terrain SVO.

/// Configuration for VXGI
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VxgiConfig {
    /// Voxel grid resolution (power of 2)
    pub voxel_resolution: u32,
    /// World space size covered by voxel grid
    pub world_size: f32,
    /// Number of cone samples per pixel
    pub cone_count: u32,
    /// Maximum cone tracing distance
    pub max_trace_distance: f32,
    /// Cone aperture angle in radians
    pub cone_aperture: f32,
    /// Padding for 16-byte alignment (total 32 bytes)
    pub _pad: [u32; 3],
}

impl Default for VxgiConfig {
    fn default() -> Self {
        Self {
            voxel_resolution: 256,
            world_size: 1000.0,
            cone_count: 6,
            max_trace_distance: 100.0,
            cone_aperture: 0.577, // ~33 degrees
            _pad: [0; 3],
        }
    }
}

/// Voxel data for radiance field
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VoxelRadiance {
    /// RGB radiance + opacity (stored as array instead of Vec4)
    pub radiance: [f32; 4],
}

/// VXGI renderer
pub struct VxgiRenderer {
    config: VxgiConfig,

    // GPU resources
    _voxel_texture: wgpu::Texture,
    _voxel_texture_view: wgpu::TextureView,
    _voxel_sampler: wgpu::Sampler,
    _config_buffer: wgpu::Buffer,

    // Bind groups
    vxgi_bind_group_layout: wgpu::BindGroupLayout,
    vxgi_bind_group: wgpu::BindGroup,
    voxelization_bind_group: wgpu::BindGroup,

    // Compute pipeline for voxelization
    voxelization_pipeline: wgpu::ComputePipeline,

    // Dirty flag
    needs_update: bool,
}

impl VxgiRenderer {
    /// Create a new VXGI renderer
    pub fn new(device: &wgpu::Device, config: VxgiConfig) -> Self {
        // Create 3D texture for voxel radiance
        let voxel_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("VXGI Voxel Texture"),
            size: wgpu::Extent3d {
                width: config.voxel_resolution,
                height: config.voxel_resolution,
                depth_or_array_layers: config.voxel_resolution,
            },
            mip_level_count: (config.voxel_resolution as f32).log2() as u32 + 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let voxel_texture_view = voxel_texture.create_view(&wgpu::TextureViewDescriptor {
            usage: None,
            label: Some("VXGI Voxel Texture View"),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::D3),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let voxel_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("VXGI Voxel Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Config Uniform Buffer
        let config_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("VXGI Config Buffer"),
            size: std::mem::size_of::<VxgiConfig>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
        config_buffer
            .slice(..)
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::bytes_of(&config));
        config_buffer.unmap();

        // Create bind group layout
        let vxgi_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("VXGI Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D3,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let vxgi_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VXGI Bind Group"),
            layout: &vxgi_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&voxel_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&voxel_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: config_buffer.as_entire_binding(),
                },
            ],
        });

        // Create voxelization compute pipeline
        let voxelization_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("VXGI Voxelization Shader"),
            source: wgpu::ShaderSource::Wgsl(VOXELIZATION_SHADER.into()),
        });

        let voxelization_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("VXGI Voxelization Pipeline"),
                layout: None,
                module: &voxelization_shader,
                entry_point: Some("voxelize"),
                compilation_options: Default::default(),
                cache: None,
            });

        // Create a single-mip view for storage writing
        let voxelization_view = voxel_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("VXGI Voxelization View"),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::D3),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: None,
            usage: None,
        });

        let voxelization_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VXGI Voxelization Bind Group"),
            layout: &voxelization_pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&voxelization_view),
            }],
        });

        Self {
            config,
            _voxel_texture: voxel_texture,
            _voxel_texture_view: voxel_texture_view,
            _voxel_sampler: voxel_sampler,
            _config_buffer: config_buffer,
            vxgi_bind_group_layout,
            vxgi_bind_group,
            voxelization_bind_group,
            voxelization_pipeline,
            needs_update: true,
        }
    }

    /// Mark voxel grid as needing update
    pub fn mark_dirty(&mut self) {
        self.needs_update = true;
    }

    /// Update voxel radiance field from terrain
    pub fn update_voxel_field(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if !self.needs_update {
            return;
        }

        // Run voxelization compute shader
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("VXGI Voxelization Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.voxelization_pipeline);
        compute_pass.set_bind_group(0, &self.voxelization_bind_group, &[]);

        let workgroup_size_xy = 8;
        let workgroup_size_z = 4;
        let dispatch_size_xy = self.config.voxel_resolution.div_ceil(workgroup_size_xy);
        let dispatch_size_z = self.config.voxel_resolution.div_ceil(workgroup_size_z);
        compute_pass.dispatch_workgroups(dispatch_size_xy, dispatch_size_xy, dispatch_size_z);

        drop(compute_pass);

        self.needs_update = false;
    }

    /// Get bind group layout
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.vxgi_bind_group_layout
    }

    /// Get bind group
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.vxgi_bind_group
    }

    /// Get configuration
    pub fn config(&self) -> &VxgiConfig {
        &self.config
    }
}

/// WGSL shader for voxelization
const VOXELIZATION_SHADER: &str = r#"
@group(0) @binding(0) var voxel_texture: texture_storage_3d<rgba16float, write>;

@compute @workgroup_size(8, 8, 4)
fn voxelize(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let voxel_pos = vec3<f32>(global_id);
    
    // Sample terrain density and material at this voxel position
    // This would integrate with the terrain voxel system
    
    // For now, initialize with ambient lighting
    let radiance = vec4<f32>(0.1, 0.1, 0.1, 1.0);
    
    textureStore(voxel_texture, global_id, radiance);
}
"#;

/// WGSL shader for cone tracing
pub const CONE_TRACING_SHADER: &str = r#"
struct VxgiConfig {
    voxel_resolution: u32,
    world_size: f32,
    cone_count: u32,
    max_trace_distance: f32,
    cone_aperture: f32,
    _pad: vec3<u32>,
}

@group(5) @binding(0) var voxel_texture: texture_3d<f32>;
@group(5) @binding(1) var voxel_sampler: sampler;
@group(5) @binding(2) var<uniform> uVxgi: VxgiConfig;

// Cone directions for diffuse sampling (6 cones)
const CONE_DIRECTIONS = array<vec3<f32>, 6>(
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(0.0, 0.5, 0.866),
    vec3<f32>(0.823, 0.5, 0.267),
    vec3<f32>(0.509, 0.5, -0.7),
    vec3<f32>(-0.509, 0.5, -0.7),
    vec3<f32>(-0.823, 0.5, 0.267),
);

fn world_to_voxel(world_pos: vec3<f32>, config: VxgiConfig) -> vec3<f32> {
    return (world_pos / config.world_size + 0.5) * f32(config.voxel_resolution);
}

fn trace_cone(
    origin: vec3<f32>,
    direction: vec3<f32>,
    aperture: f32,
    config: VxgiConfig
) -> vec4<f32> {
    var accumulated_radiance = vec3<f32>(0.0);
    var accumulated_opacity = 0.0;
    
    let step_size = 1.0;
    let max_steps = u32(config.max_trace_distance / step_size);
    
    var current_pos = origin;
    var diameter = 0.0;
    
    for (var i = 0u; i < max_steps; i = i + 1u) {
        if (accumulated_opacity >= 0.95) {
            break;
        }
        
        // Calculate mip level based on cone diameter
        diameter = diameter + aperture * step_size;
        let mip_level = log2(diameter + 1.0);
        
        // Sample voxel texture
        let voxel_coord = world_to_voxel(current_pos, config) / f32(config.voxel_resolution);
        let sample = textureSampleLevel(voxel_texture, voxel_sampler, voxel_coord, mip_level);
        
        // Accumulate radiance
        let opacity = sample.a * (1.0 - accumulated_opacity);
        accumulated_radiance = accumulated_radiance + sample.rgb * opacity;
        accumulated_opacity = accumulated_opacity + opacity;
        
        // Step forward
        current_pos = current_pos + direction * step_size;
    }
    
    return vec4<f32>(accumulated_radiance, accumulated_opacity);
}

fn calculate_vxgi_lighting(
    world_pos: vec3<f32>,
    normal: vec3<f32>
) -> vec3<f32> {
    var total_radiance = vec3<f32>(0.0);
    
    // Trace multiple cones for diffuse indirect lighting
    for (var i = 0u; i < uVxgi.cone_count; i = i + 1u) {
        // Transform cone direction to world space aligned with normal
        let cone_dir = normalize(CONE_DIRECTIONS[i]);
        
        // Create tangent space
        let up = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(normal.y) > 0.9);
        let tangent = normalize(cross(up, normal));
        let bitangent = cross(normal, tangent);
        
        let world_cone_dir = tangent * cone_dir.x + normal * cone_dir.y + bitangent * cone_dir.z;
        
        // Trace cone
        let cone_result = trace_cone(
            world_pos + normal * 0.1, // Offset to avoid self-intersection
            world_cone_dir,
            uVxgi.cone_aperture,
            uVxgi
        );
        
        total_radiance = total_radiance + cone_result.rgb;
    }
    
    // Average over all cones
    return total_radiance / f32(uVxgi.cone_count);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vxgi_config_default() {
        let config = VxgiConfig::default();
        assert_eq!(config.voxel_resolution, 256);
        assert_eq!(config.cone_count, 6);
    }

    #[test]
    fn test_voxel_radiance_size() {
        assert_eq!(std::mem::size_of::<VoxelRadiance>(), 16);
    }
}
