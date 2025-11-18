// Transparency Depth Sorting System
// Implements back-to-front sorting for transparent objects and separate render pass

use glam::Vec3;
use std::cmp::Ordering;
use wgpu;

/// Transparent instance data for depth sorting
#[derive(Clone, Copy, Debug)]
pub struct TransparentInstance {
    /// Instance index in the main instance buffer
    pub instance_index: u32,
    /// World position of the instance
    pub world_position: Vec3,
    /// Distance from camera (for sorting)
    pub camera_distance: f32,
    /// Alpha blend mode
    pub blend_mode: BlendMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    /// Standard alpha blending (src_alpha, one_minus_src_alpha)
    Alpha,
    /// Additive blending (src_alpha, one)
    Additive,
    /// Multiplicative blending (zero, src)
    Multiplicative,
}

/// Manages transparent object sorting and rendering
pub struct TransparencyManager {
    /// List of transparent instances to render this frame
    transparent_instances: Vec<TransparentInstance>,
    /// Sorted indices (back-to-front order)
    sorted_indices: Vec<usize>,
    /// Camera position from last update
    camera_position: Vec3,
}

impl TransparencyManager {
    pub fn new() -> Self {
        Self {
            transparent_instances: Vec::with_capacity(256),
            sorted_indices: Vec::with_capacity(256),
            camera_position: Vec3::ZERO,
        }
    }

    /// Clear all transparent instances for the next frame
    pub fn clear(&mut self) {
        self.transparent_instances.clear();
        self.sorted_indices.clear();
    }

    /// Add a transparent instance
    pub fn add_instance(
        &mut self,
        instance_index: u32,
        world_position: Vec3,
        blend_mode: BlendMode,
    ) {
        let camera_distance = (world_position - self.camera_position).length();

        self.transparent_instances.push(TransparentInstance {
            instance_index,
            world_position,
            camera_distance,
            blend_mode,
        });
    }

    /// Update camera position and perform depth sorting
    pub fn update(&mut self, camera_position: Vec3) {
        self.camera_position = camera_position;

        // Recalculate distances
        for instance in &mut self.transparent_instances {
            instance.camera_distance = (instance.world_position - camera_position).length();
        }

        // Sort back-to-front (furthest first)
        self.sorted_indices = (0..self.transparent_instances.len()).collect();
        self.sorted_indices.sort_by(|&a, &b| {
            let dist_a = self.transparent_instances[a].camera_distance;
            let dist_b = self.transparent_instances[b].camera_distance;

            // Reverse order for back-to-front
            dist_b.partial_cmp(&dist_a).unwrap_or(Ordering::Equal)
        });
    }

    /// Get sorted instances in back-to-front order
    pub fn sorted_instances(&self) -> impl Iterator<Item = &TransparentInstance> + '_ {
        self.sorted_indices
            .iter()
            .map(move |&idx| &self.transparent_instances[idx])
    }

    /// Get the number of transparent instances
    pub fn count(&self) -> usize {
        self.transparent_instances.len()
    }

    /// Group instances by blend mode for optimized rendering
    pub fn instances_by_blend_mode(
        &self,
        blend_mode: BlendMode,
    ) -> impl Iterator<Item = &TransparentInstance> + '_ {
        self.sorted_instances()
            .filter(move |inst| inst.blend_mode == blend_mode)
    }
}

impl Default for TransparencyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a wgpu blend state for a given blend mode
pub fn create_blend_state(blend_mode: BlendMode) -> wgpu::BlendState {
    match blend_mode {
        BlendMode::Alpha => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        },
        BlendMode::Additive => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        },
        BlendMode::Multiplicative => wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::Src,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transparency_manager_new() {
        let manager = TransparencyManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_add_instance() {
        let mut manager = TransparencyManager::new();
        manager.add_instance(0, Vec3::new(0.0, 0.0, 0.0), BlendMode::Alpha);
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_depth_sorting() {
        let mut manager = TransparencyManager::new();

        // Add instances at different distances
        manager.add_instance(0, Vec3::new(0.0, 0.0, -5.0), BlendMode::Alpha);
        manager.add_instance(1, Vec3::new(0.0, 0.0, -10.0), BlendMode::Alpha);
        manager.add_instance(2, Vec3::new(0.0, 0.0, -2.0), BlendMode::Alpha);

        // Update with camera at origin
        manager.update(Vec3::ZERO);

        // Check back-to-front order (furthest first)
        let sorted: Vec<u32> = manager
            .sorted_instances()
            .map(|inst| inst.instance_index)
            .collect();

        assert_eq!(sorted, vec![1, 0, 2]); // -10, -5, -2
    }

    #[test]
    fn test_clear() {
        let mut manager = TransparencyManager::new();
        manager.add_instance(0, Vec3::ZERO, BlendMode::Alpha);
        manager.clear();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_filter_by_blend_mode() {
        let mut manager = TransparencyManager::new();
        manager.add_instance(0, Vec3::ZERO, BlendMode::Alpha);
        manager.add_instance(1, Vec3::ZERO, BlendMode::Additive);
        manager.add_instance(2, Vec3::ZERO, BlendMode::Alpha);
        manager.update(Vec3::ZERO);

        let alpha_count = manager.instances_by_blend_mode(BlendMode::Alpha).count();
        let additive_count = manager.instances_by_blend_mode(BlendMode::Additive).count();

        assert_eq!(alpha_count, 2);
        assert_eq!(additive_count, 1);
    }
}
