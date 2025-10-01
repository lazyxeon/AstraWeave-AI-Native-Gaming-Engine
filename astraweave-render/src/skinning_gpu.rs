//! GPU Skinning Pipeline for AstraWeave
//!
//! Phase 2 Task 5 (Phase D): GPU-accelerated skeletal animation skinning.
//! Feature-gated with `skinning-gpu` - optional for performance, CPU path is default.

use crate::animation::{JointMatrixGPU, JointPalette, MAX_JOINTS};
use anyhow::Result;
use glam::Mat4;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Handle for a joint palette buffer (per-skeleton GPU buffer)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JointPaletteHandle(pub u32);

/// GPU buffer pool for joint palettes
pub struct JointPaletteManager {
    device: wgpu::Device,
    queue: wgpu::Queue,

    /// Active joint palette buffers (handle -> buffer)
    buffers: HashMap<JointPaletteHandle, wgpu::Buffer>,

    /// Next available handle
    next_handle: u32,

    /// Bind group layout for skinning storage buffer
    pub bind_group_layout: wgpu::BindGroupLayout,

    /// Cached bind groups (handle -> bind group)
    bind_groups: HashMap<JointPaletteHandle, wgpu::BindGroup>,
}

impl JointPaletteManager {
    /// Create a new joint palette manager
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("joint_palette_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        Self {
            device: device.clone(),
            queue: queue.clone(),
            buffers: HashMap::new(),
            next_handle: 0,
            bind_group_layout,
            bind_groups: HashMap::new(),
        }
    }

    /// Allocate a new joint palette buffer
    pub fn allocate(&mut self) -> JointPaletteHandle {
        let handle = JointPaletteHandle(self.next_handle);
        self.next_handle += 1;

        // Create storage buffer for joint matrices
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("joint_palette_{}", handle.0)),
            size: std::mem::size_of::<JointPalette>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("joint_palette_bind_group_{}", handle.0)),
            layout: &self.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        self.buffers.insert(handle, buffer);
        self.bind_groups.insert(handle, bind_group);

        handle
    }

    /// Upload joint matrices to GPU buffer (from Mat4 array)
    pub fn upload_matrices(&mut self, handle: JointPaletteHandle, matrices: &[Mat4]) -> Result<()> {
        let palette = JointPalette::from_matrices(matrices);
        self.upload_palette(handle, &palette)
    }

    /// Upload joint palette to GPU buffer (from JointPalette struct)
    pub fn upload_palette(
        &mut self,
        handle: JointPaletteHandle,
        palette: &JointPalette,
    ) -> Result<()> {
        let buffer = self
            .buffers
            .get(&handle)
            .ok_or_else(|| anyhow::anyhow!("Invalid joint palette handle: {:?}", handle))?;

        // Convert to bytes and upload
        let data = bytemuck::cast_slice(&[*palette]);
        self.queue.write_buffer(buffer, 0, data);

        Ok(())
    }

    /// Get bind group for skinning shader
    pub fn get_bind_group(&self, handle: JointPaletteHandle) -> Option<&wgpu::BindGroup> {
        self.bind_groups.get(&handle)
    }

    /// Free a joint palette buffer
    pub fn free(&mut self, handle: JointPaletteHandle) {
        self.buffers.remove(&handle);
        self.bind_groups.remove(&handle);
    }

    /// Get number of active buffers
    pub fn active_count(&self) -> usize {
        self.buffers.len()
    }

    /// Clear all buffers (for cleanup)
    pub fn clear(&mut self) {
        self.buffers.clear();
        self.bind_groups.clear();
        self.next_handle = 0;
    }
}

// ============================================================================
// GPU Skinning Shader Module (embedded WGSL)
// ============================================================================

/// WGSL shader for GPU skinning vertex transformation
pub const SKINNING_GPU_SHADER: &str = r#"
// Joint palette storage buffer
struct JointMatrix {
    matrix: mat4x4<f32>,
}

struct JointPalette {
    joints: array<JointMatrix, 256>,
    joint_count: u32,
    _padding: vec3<u32>,
}

@group(4) @binding(0) var<storage, read> joint_palette: JointPalette;

// Vertex input with skinning data
struct SkinnedVertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(12) tangent: vec4<f32>,
    @location(10) joints: vec4<u32>,
    @location(11) weights: vec4<f32>,
}

// Apply GPU skinning to position and normal
fn apply_skinning(input: SkinnedVertexInput) -> vec4<f32> {
    let j = input.joints;
    let w = input.weights;
    
    // Fetch joint matrices
    let m0 = joint_palette.joints[j.x].matrix;
    let m1 = joint_palette.joints[j.y].matrix;
    let m2 = joint_palette.joints[j.z].matrix;
    let m3 = joint_palette.joints[j.w].matrix;
    
    // Blend position
    let pos4 = vec4<f32>(input.position, 1.0);
    let skinned_pos = (m0 * pos4) * w.x 
                    + (m1 * pos4) * w.y 
                    + (m2 * pos4) * w.z 
                    + (m3 * pos4) * w.w;
    
    return skinned_pos;
}

// Apply skinning to normal (for lighting)
fn apply_skinning_normal(input: SkinnedVertexInput) -> vec3<f32> {
    let j = input.joints;
    let w = input.weights;
    
    let m0 = joint_palette.joints[j.x].matrix;
    let m1 = joint_palette.joints[j.y].matrix;
    let m2 = joint_palette.joints[j.z].matrix;
    let m3 = joint_palette.joints[j.w].matrix;
    
    let nrm4 = vec4<f32>(input.normal, 0.0);
    let skinned_nrm = (m0 * nrm4) * w.x 
                    + (m1 * nrm4) * w.y 
                    + (m2 * nrm4) * w.z 
                    + (m3 * nrm4) * w.w;
    
    return normalize(skinned_nrm.xyz);
}

// Apply skinning to tangent (for normal mapping)
fn apply_skinning_tangent(input: SkinnedVertexInput) -> vec3<f32> {
    let j = input.joints;
    let w = input.weights;
    
    let m0 = joint_palette.joints[j.x].matrix;
    let m1 = joint_palette.joints[j.y].matrix;
    let m2 = joint_palette.joints[j.z].matrix;
    let m3 = joint_palette.joints[j.w].matrix;
    
    let tan4 = vec4<f32>(input.tangent.xyz, 0.0);
    let skinned_tan = (m0 * tan4) * w.x 
                    + (m1 * tan4) * w.y 
                    + (m2 * tan4) * w.z 
                    + (m3 * tan4) * w.w;
    
    return normalize(skinned_tan.xyz);
}
"#;

// ============================================================================
// Integration Helpers
// ============================================================================

/// Helper to create skinned mesh pipeline with GPU skinning enabled
pub fn create_skinned_pipeline_descriptor(
    device: &wgpu::Device,
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    material_bind_group_layout: &wgpu::BindGroupLayout,
    light_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    joint_palette_bind_group_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipelineDescriptor {
    // This would be implemented with the full pipeline descriptor
    // For now, return a minimal descriptor structure
    todo!("Pipeline descriptor creation - integrate with existing renderer pipelines")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joint_palette_handle() {
        let handle1 = JointPaletteHandle(0);
        let handle2 = JointPaletteHandle(1);
        assert_ne!(handle1, handle2);
    }

    #[test]
    fn test_joint_palette_from_matrices() {
        let matrices = vec![
            Mat4::from_translation(glam::Vec3::new(1.0, 0.0, 0.0)),
            Mat4::from_translation(glam::Vec3::new(0.0, 2.0, 0.0)),
        ];

        let palette = JointPalette::from_matrices(&matrices);
        assert_eq!(palette.joint_count, 2);

        // Verify first matrix
        let m0 = palette.joints[0].matrix;
        assert_eq!(m0[3][0], 1.0); // Translation X
    }

    #[test]
    fn test_max_joints_limit() {
        let matrices = vec![Mat4::IDENTITY; 300]; // More than MAX_JOINTS

        let palette = JointPalette::from_matrices(&matrices);
        assert_eq!(palette.joint_count, MAX_JOINTS as u32);
    }

    // GPU tests require wgpu instance - add integration tests later
}
