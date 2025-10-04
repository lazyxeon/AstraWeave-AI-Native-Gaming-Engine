//! Test utilities for headless rendering and golden image tests
//!
//! Provides wgpu context initialization for CI-safe headless tests

use wgpu::{Device, Queue};

/// Initialize headless wgpu device for testing
/// Uses software adapter for CI compatibility
pub async fn create_headless_device() -> (Device, Queue) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // Request adapter with fallback preference for CI
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: true, // Use software adapter for CI
            compatible_surface: None,
        })
        .await
        .expect("Failed to find adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("test_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: None,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    (device, queue)
}

/// Helper to create a simple test skeleton
pub fn create_simple_skeleton() -> (Vec<glam::Mat4>, Vec<glam::Vec3>) {
    use glam::{Mat4, Vec3};

    // 3-joint skeleton: root -> child1 -> child2
    let inverse_bind_matrices = vec![
        Mat4::IDENTITY,
        Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)),
        Mat4::from_translation(Vec3::new(0.0, -2.0, 0.0)),
    ];

    let bind_translations = vec![
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ];

    (inverse_bind_matrices, bind_translations)
}

/// Helper to compute local poses for test skeleton
pub fn create_test_local_poses(rotation_angle: f32) -> Vec<astraweave_render::Transform> {
    use astraweave_render::Transform;
    use glam::{Quat, Vec3};

    vec![
        Transform::default(), // Root
        Transform {
            translation: Vec3::new(0.0, 1.0, 0.0),
            rotation: Quat::from_rotation_z(rotation_angle),
            scale: Vec3::ONE,
        },
        Transform {
            translation: Vec3::new(0.0, 1.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
    ]
}

/// Compute joint matrices for test skeleton
pub fn compute_test_joint_matrices(
    inverse_bind: &[glam::Mat4],
    local_poses: &[astraweave_render::Transform],
) -> Vec<glam::Mat4> {
    use glam::Mat4;

    assert_eq!(inverse_bind.len(), local_poses.len());
    let joint_count = inverse_bind.len();

    // Compute world transforms (hierarchical)
    let mut world_transforms = vec![Mat4::IDENTITY; joint_count];
    world_transforms[0] = local_poses[0].to_matrix();

    for i in 1..joint_count {
        let parent_idx = i - 1; // Simple linear hierarchy for test
        world_transforms[i] = world_transforms[parent_idx] * local_poses[i].to_matrix();
    }

    // Compute skinning matrices: world * inverse_bind
    let mut skinning_matrices = vec![Mat4::IDENTITY; joint_count];
    for i in 0..joint_count {
        skinning_matrices[i] = world_transforms[i] * inverse_bind[i];
    }

    skinning_matrices
}

/// Compare two sets of joint matrices with tolerance
pub fn assert_matrices_close(a: &[glam::Mat4], b: &[glam::Mat4], tolerance: f32) {
    assert_eq!(a.len(), b.len(), "Matrix array lengths differ");

    for (i, (m1, m2)) in a.iter().zip(b.iter()).enumerate() {
        let diff = (*m1 - *m2).abs();
        let max_diff = diff
            .to_cols_array()
            .iter()
            .fold(0.0f32, |acc, &x| acc.max(x));

        assert!(
            max_diff < tolerance,
            "Matrix {} differs by {} (tolerance: {})\nA: {:?}\nB: {:?}",
            i,
            max_diff,
            tolerance,
            m1,
            m2
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_skeleton() {
        let (inverse_bind, translations) = create_simple_skeleton();
        assert_eq!(inverse_bind.len(), 3);
        assert_eq!(translations.len(), 3);
    }

    #[test]
    fn test_compute_matrices() {
        let (inverse_bind, _) = create_simple_skeleton();
        let local_poses = create_test_local_poses(0.0);
        let matrices = compute_test_joint_matrices(&inverse_bind, &local_poses);
        assert_eq!(matrices.len(), 3);
    }
}
