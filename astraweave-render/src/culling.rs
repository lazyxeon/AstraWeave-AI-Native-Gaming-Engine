//! GPU-driven frustum culling implementation for Phase 2 Task 3
//!
//! This module provides compute-based frustum culling with CPU fallback for determinism.
//! The compute path writes a compacted list of visible instance indices, enabling indirect draws.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Per-instance data for culling compute shader
/// Represents an axis-aligned bounding box (AABB) in world space
/// Layout must match WGSL std140: vec3 aligned to 16 bytes
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct InstanceAABB {
    /// Center of the AABB in world space (vec3<f32> in WGSL = 16 bytes with padding)
    pub center: [f32; 3],
    pub _pad0: u32, // Padding to align extent to 16-byte boundary
    /// Half-extents (radius) from center (vec3<f32> in WGSL = 16 bytes with padding)
    pub extent: [f32; 3],
    /// Original instance index in the instance buffer
    pub instance_index: u32,
}

impl InstanceAABB {
    pub fn new(center: Vec3, extent: Vec3, instance_index: u32) -> Self {
        Self {
            center: center.to_array(),
            _pad0: 0,
            extent: extent.to_array(),
            instance_index,
        }
    }

    /// Compute AABB from world transform matrix and local bounds
    pub fn from_transform(
        transform: &Mat4,
        local_min: Vec3,
        local_max: Vec3,
        instance_index: u32,
    ) -> Self {
        let local_center = (local_min + local_max) * 0.5;
        let local_extent = (local_max - local_min) * 0.5;

        // Transform center to world space
        let world_center = transform.transform_point3(local_center);

        // For extent, transform all 8 corners and compute new AABB
        // This handles rotation properly
        let corners = [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
        ];

        let mut world_min = Vec3::splat(f32::MAX);
        let mut world_max = Vec3::splat(f32::MIN);

        for corner in &corners {
            let local_point = local_center + *corner * local_extent;
            let world_point = transform.transform_point3(local_point);
            world_min = world_min.min(world_point);
            world_max = world_max.max(world_point);
        }

        let world_extent = (world_max - world_min) * 0.5;

        Self {
            center: world_center.to_array(),
            _pad0: 0,
            extent: world_extent.to_array(),
            instance_index,
        }
    }
}

/// Frustum planes in world space for culling
/// Planes are stored as (nx, ny, nz, d) where dot(normal, point) + d = 0 defines the plane
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FrustumPlanes {
    /// Six planes: left, right, bottom, top, near, far
    /// Each plane: [nx, ny, nz, d]
    pub planes: [[f32; 4]; 6],
}

impl FrustumPlanes {
    /// Extract frustum planes from view-projection matrix
    /// Uses Gribb-Hartmann method
    pub fn from_view_proj(view_proj: &Mat4) -> Self {
        let m = view_proj.to_cols_array();

        // Extract planes from matrix rows
        let left = Self::normalize_plane([m[3] + m[0], m[7] + m[4], m[11] + m[8], m[15] + m[12]]);
        let right = Self::normalize_plane([m[3] - m[0], m[7] - m[4], m[11] - m[8], m[15] - m[12]]);
        let bottom = Self::normalize_plane([m[3] + m[1], m[7] + m[5], m[11] + m[9], m[15] + m[13]]);
        let top = Self::normalize_plane([m[3] - m[1], m[7] - m[5], m[11] - m[9], m[15] - m[13]]);
        let near = Self::normalize_plane([m[3] + m[2], m[7] + m[6], m[11] + m[10], m[15] + m[14]]);
        let far = Self::normalize_plane([m[3] - m[2], m[7] - m[6], m[11] - m[10], m[15] - m[14]]);

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    fn normalize_plane(plane: [f32; 4]) -> [f32; 4] {
        let len = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
        if len > 0.0 {
            [
                plane[0] / len,
                plane[1] / len,
                plane[2] / len,
                plane[3] / len,
            ]
        } else {
            plane
        }
    }

    /// CPU-based frustum culling (fallback path)
    /// Returns true if AABB intersects frustum
    pub fn test_aabb(&self, center: Vec3, extent: Vec3) -> bool {
        for plane in &self.planes {
            let normal = Vec3::new(plane[0], plane[1], plane[2]);
            let d = plane[3];

            // Compute signed distance from center to plane
            let dist = normal.dot(center) + d;

            // Compute the effective radius along the plane normal
            let radius = extent.x.abs() * normal.x.abs()
                + extent.y.abs() * normal.y.abs()
                + extent.z.abs() * normal.z.abs();

            // If center is more than radius away on the negative side, AABB is outside
            if dist < -radius {
                return false;
            }
        }
        true
    }
}

/// Indirect draw command structure matching wgpu::DrawIndirect
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct DrawIndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

impl DrawIndirectCommand {
    /// Create a new indirect draw command for a mesh batch
    pub fn new(
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Self {
        Self {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        }
    }
}

impl Default for DrawIndirectCommand {
    fn default() -> Self {
        Self {
            vertex_count: 0,
            instance_count: 0,
            first_vertex: 0,
            first_instance: 0,
        }
    }
}

/// Batch identifier for grouping instances by mesh+material
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BatchId {
    pub mesh_id: u32,
    pub material_id: u32,
}

impl BatchId {
    pub fn new(mesh_id: u32, material_id: u32) -> Self {
        Self {
            mesh_id,
            material_id,
        }
    }
}

/// A batch of instances sharing the same mesh and material
#[derive(Debug, Clone)]
pub struct DrawBatch {
    pub batch_id: BatchId,
    pub vertex_count: u32,
    pub first_vertex: u32,
    pub instances: Vec<u32>, // Instance indices
}

impl DrawBatch {
    pub fn new(batch_id: BatchId, vertex_count: u32, first_vertex: u32) -> Self {
        Self {
            batch_id,
            vertex_count,
            first_vertex,
            instances: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, instance_idx: u32) {
        self.instances.push(instance_idx);
    }

    pub fn instance_count(&self) -> u32 {
        self.instances.len() as u32
    }
}

/// CPU fallback culling implementation
/// Returns list of visible instance indices
pub fn cpu_frustum_cull(instances: &[InstanceAABB], frustum: &FrustumPlanes) -> Vec<u32> {
    instances
        .iter()
        .filter(|inst| {
            let center = Vec3::from(inst.center);
            let extent = Vec3::from(inst.extent);
            frustum.test_aabb(center, extent)
        })
        .map(|inst| inst.instance_index)
        .collect()
}

/// Build indirect draw commands from visible instances and batch info (CPU path)
/// Returns vector of DrawIndirectCommand structs, one per batch
pub fn build_indirect_commands_cpu(batches: &[DrawBatch]) -> Vec<DrawIndirectCommand> {
    batches
        .iter()
        .map(|batch| {
            DrawIndirectCommand::new(
                batch.vertex_count,
                batch.instance_count(),
                batch.first_vertex,
                0, // first_instance typically 0 for instanced draws
            )
        })
        .collect()
}

/// Group visible instances into batches by mesh/material (CPU path)
/// This is a simplified version - real implementation would use scene data
pub fn batch_visible_instances(
    visible_indices: &[u32],
    get_batch_id: impl Fn(u32) -> BatchId,
    get_mesh_info: impl Fn(BatchId) -> (u32, u32), // Returns (vertex_count, first_vertex)
) -> Vec<DrawBatch> {
    use std::collections::BTreeMap;

    let mut batch_map: BTreeMap<BatchId, DrawBatch> = BTreeMap::new();

    for &instance_idx in visible_indices {
        let batch_id = get_batch_id(instance_idx);

        batch_map
            .entry(batch_id)
            .or_insert_with(|| {
                let (vertex_count, first_vertex) = get_mesh_info(batch_id);
                DrawBatch::new(batch_id, vertex_count, first_vertex)
            })
            .add_instance(instance_idx);
    }

    batch_map.into_values().collect()
}

/// Compute shader source for frustum culling
pub const CULLING_SHADER: &str = r#"
// Per-instance AABB data (std140 layout: vec3 is 16-byte aligned)
struct InstanceAABB {
    center: vec3<f32>,      // offset 0, size 12, align 16
    _pad0: u32,             // offset 12
    extent: vec3<f32>,      // offset 16, size 12, align 16
    instance_index: u32,    // offset 28
}
// Total size: 32 bytes

// Frustum planes (6 planes)
struct FrustumPlanes {
    planes: array<vec4<f32>, 6>,
}

// Input/output bindings
@group(0) @binding(0) var<storage, read> instance_aabbs: array<InstanceAABB>;
@group(0) @binding(1) var<uniform> frustum: FrustumPlanes;
@group(0) @binding(2) var<storage, read_write> visible_instances: array<u32>;
@group(0) @binding(3) var<storage, read_write> visible_count: atomic<u32>;

// Test if AABB is visible against frustum
fn is_aabb_visible(center: vec3<f32>, extent: vec3<f32>, frustum: FrustumPlanes) -> bool {
    // Test AABB against all 6 frustum planes (manually unrolled - WGSL requires constant indices)
    // Plane 0 (left)
    var plane = frustum.planes[0];
    var normal = plane.xyz;
    var d = plane.w;
    var dist = dot(center, normal) + d;
    var radius = abs(extent.x) * abs(normal.x) + abs(extent.y) * abs(normal.y) + abs(extent.z) * abs(normal.z);
    if (dist < -radius) { return false; }
    
    // Plane 1 (right)
    plane = frustum.planes[1];
    normal = plane.xyz;
    d = plane.w;
    dist = dot(center, normal) + d;
    radius = abs(extent.x) * abs(normal.x) + abs(extent.y) * abs(normal.y) + abs(extent.z) * abs(normal.z);
    if (dist < -radius) { return false; }
    
    // Plane 2 (bottom)
    plane = frustum.planes[2];
    normal = plane.xyz;
    d = plane.w;
    dist = dot(center, normal) + d;
    radius = abs(extent.x) * abs(normal.x) + abs(extent.y) * abs(normal.y) + abs(extent.z) * abs(normal.z);
    if (dist < -radius) { return false; }
    
    // Plane 3 (top)
    plane = frustum.planes[3];
    normal = plane.xyz;
    d = plane.w;
    dist = dot(center, normal) + d;
    radius = abs(extent.x) * abs(normal.x) + abs(extent.y) * abs(normal.y) + abs(extent.z) * abs(normal.z);
    if (dist < -radius) { return false; }
    
    // Plane 4 (near)
    plane = frustum.planes[4];
    normal = plane.xyz;
    d = plane.w;
    dist = dot(center, normal) + d;
    radius = abs(extent.x) * abs(normal.x) + abs(extent.y) * abs(normal.y) + abs(extent.z) * abs(normal.z);
    if (dist < -radius) { return false; }
    
    // Plane 5 (far)
    plane = frustum.planes[5];
    normal = plane.xyz;
    d = plane.w;
    dist = dot(center, normal) + d;
    radius = abs(extent.x) * abs(normal.x) + abs(extent.y) * abs(normal.y) + abs(extent.z) * abs(normal.z);
    if (dist < -radius) { return false; }
    
    return true;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if (idx >= arrayLength(&instance_aabbs)) { 
        return; 
    }
    
    let aabb = instance_aabbs[idx];
    if (is_aabb_visible(aabb.center, aabb.extent, frustum)) {
        let slot = atomicAdd(&visible_count, 1u);
        visible_instances[slot] = aabb.instance_index;
    }
}
"#;

/// GPU culling manager
pub struct CullingPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl CullingPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("frustum_culling_shader"),
            source: wgpu::ShaderSource::Wgsl(CULLING_SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("culling_bind_group_layout"),
            entries: &[
                // Instance AABBs (storage buffer)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Frustum planes (uniform buffer)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Visible instances output (storage buffer)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Visible count (atomic storage buffer)
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("culling_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("frustum_culling_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Create buffers and bind group for culling
    pub fn create_culling_resources(
        &self,
        device: &wgpu::Device,
        instances: &[InstanceAABB],
        frustum: &FrustumPlanes,
    ) -> CullingResources {
        // Handle empty instance list with minimum buffer size
        let instance_data = if instances.is_empty() {
            vec![InstanceAABB::new(Vec3::ZERO, Vec3::ZERO, 0)]
        } else {
            instances.to_vec()
        };

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("culling_instance_buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let frustum_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("culling_frustum_buffer"),
            contents: bytemuck::bytes_of(frustum),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Allocate buffer for visible instances (max size = input size, min 1 element)
        let buffer_size = (instances.len().max(1) * std::mem::size_of::<u32>()) as u64;
        let visible_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("culling_visible_buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Counter for visible instances (atomic)
        let count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("culling_count_buffer"),
            size: std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("culling_bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: frustum_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: visible_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: count_buffer.as_entire_binding(),
                },
            ],
        });

        CullingResources {
            instance_buffer,
            frustum_buffer,
            visible_buffer,
            count_buffer,
            bind_group,
        }
    }

    /// Execute culling compute pass
    /// Note: Caller must ensure count_buffer is cleared to 0 before calling this
    pub fn execute(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bind_group: &wgpu::BindGroup,
        instance_count: u32,
    ) {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("frustum_culling_pass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, bind_group, &[]);
        let workgroup_count = (instance_count + 63) / 64;
        cpass.dispatch_workgroups(workgroup_count, 1, 1);
    }

    /// Execute culling with automatic buffer setup (clears count buffer first)
    pub fn execute_with_clear(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        resources: &CullingResources,
        instance_count: u32,
    ) {
        // Clear count buffer to 0
        encoder.clear_buffer(&resources.count_buffer, 0, None);

        // Execute culling
        self.execute(encoder, &resources.bind_group, instance_count);
    }
}

/// GPU buffers for culling
pub struct CullingResources {
    pub instance_buffer: wgpu::Buffer,
    pub frustum_buffer: wgpu::Buffer,
    pub visible_buffer: wgpu::Buffer,
    pub count_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frustum_extraction() {
        // Simple orthographic-like projection
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = FrustumPlanes::from_view_proj(&view_proj);

        // All planes should be normalized
        for plane in &frustum.planes {
            let normal_len =
                (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
            assert!(
                (normal_len - 1.0).abs() < 0.01,
                "plane normal not normalized"
            );
        }
    }

    #[test]
    fn test_aabb_inside_frustum() {
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = FrustumPlanes::from_view_proj(&view_proj);

        // AABB at origin, small size - should be visible
        let center = Vec3::ZERO;
        let extent = Vec3::splat(1.0);
        assert!(
            frustum.test_aabb(center, extent),
            "AABB at origin should be visible"
        );
    }

    #[test]
    fn test_aabb_outside_frustum() {
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = FrustumPlanes::from_view_proj(&view_proj);

        // AABB far beyond frustum bounds - should not be visible
        let center = Vec3::new(100.0, 0.0, -50.0);
        let extent = Vec3::splat(1.0);
        assert!(
            !frustum.test_aabb(center, extent),
            "AABB far outside should not be visible"
        );
    }

    #[test]
    fn test_cpu_culling() {
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = FrustumPlanes::from_view_proj(&view_proj);

        let instances = vec![
            InstanceAABB::new(Vec3::ZERO, Vec3::splat(1.0), 0), // Visible
            InstanceAABB::new(Vec3::new(100.0, 0.0, -50.0), Vec3::splat(1.0), 1), // Not visible
            InstanceAABB::new(Vec3::new(0.0, 5.0, -10.0), Vec3::splat(1.0), 2), // Visible
        ];

        let visible = cpu_frustum_cull(&instances, &frustum);
        assert!(visible.contains(&0), "Instance 0 should be visible");
        assert!(!visible.contains(&1), "Instance 1 should not be visible");
        assert!(visible.contains(&2), "Instance 2 should be visible");
    }

    #[test]
    fn test_aabb_from_transform() {
        let transform = Mat4::from_translation(Vec3::new(5.0, 10.0, -15.0));
        let local_min = Vec3::new(-1.0, -1.0, -1.0);
        let local_max = Vec3::new(1.0, 1.0, 1.0);

        let aabb = InstanceAABB::from_transform(&transform, local_min, local_max, 0);

        // Center should be translated
        assert!((aabb.center[0] - 5.0).abs() < 0.01);
        assert!((aabb.center[1] - 10.0).abs() < 0.01);
        assert!((aabb.center[2] + 15.0).abs() < 0.01);

        // Extent should remain same (no rotation)
        assert!((aabb.extent[0] - 1.0).abs() < 0.01);
        assert!((aabb.extent[1] - 1.0).abs() < 0.01);
        assert!((aabb.extent[2] - 1.0).abs() < 0.01);
    }
}
