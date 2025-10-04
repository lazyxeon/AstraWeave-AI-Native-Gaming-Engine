//! Complete Clustered Forward Rendering Implementation
//!
//! This module implements a full clustered forward rendering pipeline that supports
//! 100+ dynamic lights efficiently by dividing the screen into 3D clusters and
//! assigning lights to clusters for culling.

use glam::{Mat4, Vec3};

/// Configuration for clustered rendering
#[derive(Debug, Clone, Copy)]
pub struct ClusterConfig {
    /// Number of clusters in X dimension (screen width)
    pub cluster_x: u32,
    /// Number of clusters in Y dimension (screen height)
    pub cluster_y: u32,
    /// Number of clusters in Z dimension (depth)
    pub cluster_z: u32,
    /// Near plane distance
    pub near: f32,
    /// Far plane distance
    pub far: f32,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            cluster_x: 16,
            cluster_y: 16,
            cluster_z: 32,
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// GPU-compatible light structure
/// Uses arrays instead of Vec4 for Pod/Zeroable compatibility
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLight {
    /// Position in world space (w = radius)
    pub position: [f32; 4],
    /// Color and intensity (w = intensity)
    pub color: [f32; 4],
}

impl GpuLight {
    pub fn new(position: Vec3, radius: f32, color: Vec3, intensity: f32) -> Self {
        Self {
            position: [position.x, position.y, position.z, radius],
            color: [color.x, color.y, color.z, intensity],
        }
    }
}

/// Cluster data for GPU
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuCluster {
    /// Min bounds of cluster in view space
    pub min_bounds: [f32; 4],
    /// Max bounds of cluster in view space
    pub max_bounds: [f32; 4],
    /// Offset into light index list
    pub light_offset: u32,
    /// Number of lights in this cluster
    pub light_count: u32,
    /// Padding
    pub _padding: [u32; 2],
}

/// Clustered forward renderer
pub struct ClusteredForwardRenderer {
    config: ClusterConfig,

    // GPU resources
    light_buffer: wgpu::Buffer,
    cluster_buffer: wgpu::Buffer,
    light_indices_buffer: wgpu::Buffer,

    // Bind groups
    cluster_bind_group_layout: wgpu::BindGroupLayout,
    cluster_bind_group: wgpu::BindGroup,

    // CPU-side data
    lights: Vec<GpuLight>,
    clusters: Vec<GpuCluster>,
    light_indices: Vec<u32>,

    // Capacity
    _max_lights: usize,
    max_lights_per_cluster: usize,
}

impl ClusteredForwardRenderer {
    /// Create a new clustered forward renderer
    pub fn new(device: &wgpu::Device, config: ClusterConfig) -> Self {
        let max_lights = 256;
        let max_lights_per_cluster = 128;
        let total_clusters = (config.cluster_x * config.cluster_y * config.cluster_z) as usize;

        // Create buffers
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Clustered Light Buffer"),
            size: (max_lights * std::mem::size_of::<GpuLight>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cluster_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cluster Buffer"),
            size: (total_clusters * std::mem::size_of::<GpuCluster>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Indices Buffer"),
            size: (total_clusters * max_lights_per_cluster * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let cluster_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cluster Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Create bind group
        let cluster_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cluster Bind Group"),
            layout: &cluster_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: cluster_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_indices_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            config,
            light_buffer,
            cluster_buffer,
            light_indices_buffer,
            cluster_bind_group_layout,
            cluster_bind_group,
            lights: Vec::new(),
            clusters: vec![
                GpuCluster {
                    min_bounds: [0.0; 4],
                    max_bounds: [0.0; 4],
                    light_offset: 0,
                    light_count: 0,
                    _padding: [0; 2],
                };
                total_clusters
            ],
            light_indices: Vec::new(),
            _max_lights: max_lights,
            max_lights_per_cluster,
        }
    }

    /// Update lights for the current frame
    pub fn update_lights(&mut self, lights: Vec<GpuLight>) {
        self.lights = lights;
    }

    /// Build clusters and assign lights
    pub fn build_clusters(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: Mat4,
        _proj_matrix: Mat4,
        screen_size: (u32, u32),
    ) {
        // Clear previous data
        self.light_indices.clear();

        let (_width, _height) = screen_size;
        let _total_clusters =
            (self.config.cluster_x * self.config.cluster_y * self.config.cluster_z) as usize;

        // Calculate cluster bounds in view space
        for z in 0..self.config.cluster_z {
            for y in 0..self.config.cluster_y {
                for x in 0..self.config.cluster_x {
                    let cluster_idx = self.cluster_index(x, y, z);

                    // Calculate cluster bounds
                    let min_x = (x as f32 / self.config.cluster_x as f32) * 2.0 - 1.0;
                    let max_x = ((x + 1) as f32 / self.config.cluster_x as f32) * 2.0 - 1.0;
                    let min_y = (y as f32 / self.config.cluster_y as f32) * 2.0 - 1.0;
                    let max_y = ((y + 1) as f32 / self.config.cluster_y as f32) * 2.0 - 1.0;

                    // Exponential depth slicing for better distribution
                    let near = self.config.near;
                    let far = self.config.far;
                    let min_z = near * (far / near).powf(z as f32 / self.config.cluster_z as f32);
                    let max_z =
                        near * (far / near).powf((z + 1) as f32 / self.config.cluster_z as f32);

                    self.clusters[cluster_idx].min_bounds = [min_x, min_y, min_z, 1.0];
                    self.clusters[cluster_idx].max_bounds = [max_x, max_y, max_z, 1.0];
                    self.clusters[cluster_idx].light_offset = self.light_indices.len() as u32;
                    self.clusters[cluster_idx].light_count = 0;

                    // Assign lights to this cluster
                    for (light_idx, light) in self.lights.iter().enumerate() {
                        if self.light_intersects_cluster(
                            light,
                            &self.clusters[cluster_idx],
                            view_matrix,
                        ) {
                            if self.clusters[cluster_idx].light_count
                                < self.max_lights_per_cluster as u32
                            {
                                self.light_indices.push(light_idx as u32);
                                self.clusters[cluster_idx].light_count += 1;
                            }
                        }
                    }
                }
            }
        }

        // Upload to GPU
        if !self.lights.is_empty() {
            queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&self.lights));
        }
        queue.write_buffer(
            &self.cluster_buffer,
            0,
            bytemuck::cast_slice(&self.clusters),
        );
        if !self.light_indices.is_empty() {
            queue.write_buffer(
                &self.light_indices_buffer,
                0,
                bytemuck::cast_slice(&self.light_indices),
            );
        }
    }

    /// Get cluster index from 3D coordinates
    fn cluster_index(&self, x: u32, y: u32, z: u32) -> usize {
        (x + y * self.config.cluster_x + z * self.config.cluster_x * self.config.cluster_y) as usize
    }

    /// Check if a light intersects a cluster
    fn light_intersects_cluster(
        &self,
        light: &GpuLight,
        cluster: &GpuCluster,
        view_matrix: Mat4,
    ) -> bool {
        // Transform light position to view space
        let light_pos_4 =
            Vec3::new(light.position[0], light.position[1], light.position[2]).extend(1.0);
        let light_pos_view_4 = view_matrix * light_pos_4;
        let light_pos_view = Vec3::new(light_pos_view_4.x, light_pos_view_4.y, light_pos_view_4.z);
        let radius = light.position[3];

        // Simple sphere-AABB intersection test
        let closest_point = Vec3::new(
            light_pos_view
                .x
                .clamp(cluster.min_bounds[0], cluster.max_bounds[0]),
            light_pos_view
                .y
                .clamp(cluster.min_bounds[1], cluster.max_bounds[1]),
            light_pos_view
                .z
                .clamp(cluster.min_bounds[2], cluster.max_bounds[2]),
        );

        let distance_sq = (light_pos_view - closest_point).length_squared();
        distance_sq <= radius * radius
    }

    /// Get the bind group layout
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.cluster_bind_group_layout
    }

    /// Get the bind group
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.cluster_bind_group
    }

    /// Get configuration
    pub fn config(&self) -> &ClusterConfig {
        &self.config
    }

    /// Get light count
    pub fn light_count(&self) -> usize {
        self.lights.len()
    }
}

/// WGSL shader code for clustered forward rendering
pub const CLUSTERED_LIGHTING_SHADER: &str = r#"
struct Light {
    position: vec4<f32>,  // w = radius
    color: vec4<f32>,     // w = intensity
}

struct Cluster {
    min_bounds: vec4<f32>,
    max_bounds: vec4<f32>,
    light_offset: u32,
    light_count: u32,
    padding: vec2<u32>,
}

@group(2) @binding(0) var<storage, read> lights: array<Light>;
@group(2) @binding(1) var<storage, read> clusters: array<Cluster>;
@group(2) @binding(2) var<storage, read> light_indices: array<u32>;

struct ClusterConfig {
    cluster_x: u32,
    cluster_y: u32,
    cluster_z: u32,
    near: f32,
    far: f32,
}

fn get_cluster_index(frag_coord: vec3<f32>, config: ClusterConfig) -> u32 {
    let x = u32(frag_coord.x / config.cluster_x);
    let y = u32(frag_coord.y / config.cluster_y);
    
    // Exponential depth mapping
    let depth = frag_coord.z;
    let z_slice = log2(depth / config.near) / log2(config.far / config.near);
    let z = u32(z_slice * f32(config.cluster_z));
    
    return x + y * config.cluster_x + z * config.cluster_x * config.cluster_y;
}

fn calculate_clustered_lighting(
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    view_pos: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    frag_coord: vec3<f32>,
    config: ClusterConfig
) -> vec3<f32> {
    let cluster_idx = get_cluster_index(frag_coord, config);
    let cluster = clusters[cluster_idx];
    
    var total_light = vec3<f32>(0.0);
    
    // Iterate through lights in this cluster
    for (var i = 0u; i < cluster.light_count; i = i + 1u) {
        let light_idx = light_indices[cluster.light_offset + i];
        let light = lights[light_idx];
        
        let light_dir = light.position.xyz - world_pos;
        let distance = length(light_dir);
        let radius = light.position.w;
        
        // Skip if outside light radius
        if (distance > radius) {
            continue;
        }
        
        let L = normalize(light_dir);
        let V = normalize(view_pos - world_pos);
        let H = normalize(L + V);
        
        let NdotL = max(dot(normal, L), 0.0);
        let NdotV = max(dot(normal, V), 0.0);
        let NdotH = max(dot(normal, H), 0.0);
        
        // Attenuation
        let attenuation = 1.0 - pow(distance / radius, 4.0);
        let attenuation_clamped = max(attenuation, 0.0);
        
        // Simple Blinn-Phong for now (can be extended to PBR)
        let diffuse = albedo * NdotL;
        let specular = pow(NdotH, 32.0) * (1.0 - roughness);
        
        let light_contribution = (diffuse + specular) * light.color.rgb * light.color.w * attenuation_clamped;
        total_light = total_light + light_contribution;
    }
    
    return total_light;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_config_default() {
        let config = ClusterConfig::default();
        assert_eq!(config.cluster_x, 16);
        assert_eq!(config.cluster_y, 16);
        assert_eq!(config.cluster_z, 32);
    }

    #[test]
    fn test_gpu_light_creation() {
        let light = GpuLight::new(
            Vec3::new(1.0, 2.0, 3.0),
            10.0,
            Vec3::new(1.0, 0.5, 0.0),
            2.0,
        );

        assert_eq!(light.position[0], 1.0);
        assert_eq!(light.position[3], 10.0);
        assert_eq!(light.color[3], 2.0);
    }
}
