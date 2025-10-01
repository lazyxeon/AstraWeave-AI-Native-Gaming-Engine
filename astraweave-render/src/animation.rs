//! Skeletal Animation System for AstraWeave
//!
//! Phase 2 Task 5: Implements CPU and GPU skinning paths with ECS integration.
//! - CPU skinning: Default, CI-safe, computes skinned vertices on CPU
//! - GPU skinning: Optional feature flag, computes skinning in vertex shader

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};

// ============================================================================
// Core Animation Datatypes
// ============================================================================

/// Joint in a skeleton hierarchy (imported from asset)
#[derive(Debug, Clone)]
pub struct Joint {
    pub name: String,
    pub parent_index: Option<usize>,
    pub inverse_bind_matrix: Mat4,
    pub local_transform: Transform,
}

/// Local transform (TRS)
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Linear interpolation between two transforms
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

/// Skeleton structure with joint hierarchy
#[derive(Debug, Clone)]
pub struct Skeleton {
    pub joints: Vec<Joint>,
    pub root_indices: Vec<usize>,
}

/// Animation keyframe interpolation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    Step,
    Linear,
    CubicSpline, // Not fully implemented
}

/// Animation channel data (one property per channel)
#[derive(Debug, Clone)]
pub enum ChannelData {
    Translation(Vec<Vec3>),
    Rotation(Vec<Quat>),
    Scale(Vec<Vec3>),
}

/// Animation channel targeting a specific joint property
#[derive(Debug, Clone)]
pub struct AnimationChannel {
    pub target_joint_index: usize,
    pub times: Vec<f32>,
    pub data: ChannelData,
    pub interpolation: Interpolation,
}

/// Animation clip with multiple channels
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
}

impl AnimationClip {
    /// Sample the animation at a given time and write joint transforms
    /// Returns joint local transforms for all joints
    pub fn sample(&self, time: f32, skeleton: &Skeleton) -> Vec<Transform> {
        let mut transforms = Vec::with_capacity(skeleton.joints.len());

        // Initialize with bind pose (local transforms from skeleton)
        for joint in &skeleton.joints {
            transforms.push(joint.local_transform);
        }

        // Apply animation channels
        for channel in &self.channels {
            let joint_idx = channel.target_joint_index;
            if joint_idx >= transforms.len() {
                continue; // Skip invalid joints
            }

            // Find keyframe indices
            let (idx0, idx1, t) = Self::find_keyframes(&channel.times, time);

            // Sample and interpolate
            let transform = match &channel.data {
                ChannelData::Translation(values) => {
                    let v0 = values[idx0];
                    let v1 = values[idx1];
                    let translation = match channel.interpolation {
                        Interpolation::Step => v0,
                        Interpolation::Linear => v0.lerp(v1, t),
                        Interpolation::CubicSpline => v0.lerp(v1, t), // Fallback to linear
                    };
                    Transform {
                        translation,
                        ..transforms[joint_idx]
                    }
                }
                ChannelData::Rotation(values) => {
                    let q0 = values[idx0];
                    let q1 = values[idx1];
                    let rotation = match channel.interpolation {
                        Interpolation::Step => q0,
                        Interpolation::Linear => q0.slerp(q1, t),
                        Interpolation::CubicSpline => q0.slerp(q1, t), // Fallback to slerp
                    };
                    Transform {
                        rotation,
                        ..transforms[joint_idx]
                    }
                }
                ChannelData::Scale(values) => {
                    let s0 = values[idx0];
                    let s1 = values[idx1];
                    let scale = match channel.interpolation {
                        Interpolation::Step => s0,
                        Interpolation::Linear => s0.lerp(s1, t),
                        Interpolation::CubicSpline => s0.lerp(s1, t), // Fallback to linear
                    };
                    Transform {
                        scale,
                        ..transforms[joint_idx]
                    }
                }
            };

            transforms[joint_idx] = transform;
        }

        transforms
    }

    /// Find keyframe indices and interpolation factor for a given time
    fn find_keyframes(times: &[f32], time: f32) -> (usize, usize, f32) {
        if times.is_empty() {
            return (0, 0, 0.0);
        }

        if times.len() == 1 || time <= times[0] {
            return (0, 0, 0.0);
        }

        if time >= *times.last().unwrap() {
            let last_idx = times.len() - 1;
            return (last_idx, last_idx, 0.0);
        }

        // Binary search for keyframe
        for i in 0..times.len() - 1 {
            if time >= times[i] && time < times[i + 1] {
                let t = (time - times[i]) / (times[i + 1] - times[i]);
                return (i, i + 1, t);
            }
        }

        (0, 0, 0.0)
    }
}

// ============================================================================
// Animation State & Playback
// ============================================================================

/// Animation playback state
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub clip_index: usize,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
    pub playing: bool,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            clip_index: 0,
            time: 0.0,
            speed: 1.0,
            looping: true,
            playing: false,
        }
    }
}

impl AnimationState {
    /// Advance animation time by delta
    pub fn update(&mut self, dt: f32, clip_duration: f32) {
        if !self.playing {
            return;
        }

        self.time += dt * self.speed;

        if self.looping {
            // Wrap around
            if self.time > clip_duration {
                self.time = self.time % clip_duration;
            }
            if self.time < 0.0 {
                self.time = clip_duration + (self.time % clip_duration);
            }
        } else {
            // Clamp
            self.time = self.time.clamp(0.0, clip_duration);
            if self.time >= clip_duration {
                self.playing = false;
            }
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.time = 0.0;
    }

    pub fn restart(&mut self) {
        self.time = 0.0;
        self.playing = true;
    }
}

// ============================================================================
// Pose Computation (Joint Matrices)
// ============================================================================

/// Compute world-space joint matrices from local transforms and skeleton hierarchy
pub fn compute_joint_matrices(skeleton: &Skeleton, local_transforms: &[Transform]) -> Vec<Mat4> {
    let mut world_matrices = vec![Mat4::IDENTITY; skeleton.joints.len()];

    // Compute world transforms via hierarchical traversal
    fn compute_recursive(
        skeleton: &Skeleton,
        local_transforms: &[Transform],
        world_matrices: &mut [Mat4],
        joint_idx: usize,
        parent_world: Mat4,
    ) {
        let local = local_transforms[joint_idx].to_matrix();
        let world = parent_world * local;
        world_matrices[joint_idx] = world;

        // Recurse to children
        for (child_idx, child_joint) in skeleton.joints.iter().enumerate() {
            if child_joint.parent_index == Some(joint_idx) {
                compute_recursive(skeleton, local_transforms, world_matrices, child_idx, world);
            }
        }
    }

    // Start from root joints
    for &root_idx in &skeleton.root_indices {
        compute_recursive(
            skeleton,
            local_transforms,
            &mut world_matrices,
            root_idx,
            Mat4::IDENTITY,
        );
    }

    // Apply inverse bind matrices to get final skinning matrices
    let mut skinning_matrices = Vec::with_capacity(skeleton.joints.len());
    for (i, joint) in skeleton.joints.iter().enumerate() {
        skinning_matrices.push(world_matrices[i] * joint.inverse_bind_matrix);
    }

    skinning_matrices
}

// ============================================================================
// GPU Skinning Data Structures
// ============================================================================

/// Joint matrix for GPU upload (64 bytes, aligned)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct JointMatrixGPU {
    pub matrix: [[f32; 4]; 4],
}

impl From<Mat4> for JointMatrixGPU {
    fn from(m: Mat4) -> Self {
        Self {
            matrix: m.to_cols_array_2d(),
        }
    }
}

/// Maximum joints supported (for buffer sizing)
pub const MAX_JOINTS: usize = 256;

/// Joint palette for GPU skinning (uploaded as storage buffer)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct JointPalette {
    pub joints: [JointMatrixGPU; MAX_JOINTS],
    pub joint_count: u32,
    pub _padding: [u32; 3],
}

impl Default for JointPalette {
    fn default() -> Self {
        Self {
            joints: [JointMatrixGPU {
                matrix: Mat4::IDENTITY.to_cols_array_2d(),
            }; MAX_JOINTS],
            joint_count: 0,
            _padding: [0; 3],
        }
    }
}

impl JointPalette {
    pub fn from_matrices(matrices: &[Mat4]) -> Self {
        let mut palette = Self::default();
        palette.joint_count = matrices.len().min(MAX_JOINTS) as u32;

        for (i, matrix) in matrices.iter().take(MAX_JOINTS).enumerate() {
            palette.joints[i] = JointMatrixGPU::from(*matrix);
        }

        palette
    }
}

// ============================================================================
// CPU Skinning (Default Path)
// ============================================================================

/// Apply CPU skinning to a vertex with joint indices and weights
pub fn skin_vertex_cpu(
    position: Vec3,
    normal: Vec3,
    joints: [u16; 4],
    weights: [f32; 4],
    joint_matrices: &[Mat4],
) -> (Vec3, Vec3) {
    let mut skinned_pos = Vec3::ZERO;
    let mut skinned_normal = Vec3::ZERO;

    for i in 0..4 {
        let weight = weights[i];
        if weight == 0.0 {
            continue;
        }

        let joint_idx = joints[i] as usize;
        if joint_idx >= joint_matrices.len() {
            continue;
        }

        let matrix = joint_matrices[joint_idx];
        skinned_pos += weight * matrix.transform_point3(position);

        // Normal uses upper-left 3x3 (no translation) - extract mat3
        let mat3 = glam::Mat3::from_cols(
            matrix.x_axis.truncate(),
            matrix.y_axis.truncate(),
            matrix.z_axis.truncate(),
        );
        skinned_normal += weight * (mat3 * normal);
    }

    (skinned_pos, skinned_normal.normalize_or_zero())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_default() {
        let t = Transform::default();
        assert_eq!(t.translation, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_to_matrix() {
        let t = Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let mat = t.to_matrix();
        assert_eq!(mat.w_axis.truncate(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_animation_state_update_looping() {
        let mut state = AnimationState {
            time: 0.5,
            speed: 1.0,
            looping: true,
            playing: true,
            ..Default::default()
        };

        state.update(0.6, 1.0); // time = 0.5 + 0.6 = 1.1 -> wraps to 0.1
        assert!((state.time - 0.1).abs() < 0.001);
        assert!(state.playing);
    }

    #[test]
    fn test_animation_state_update_clamping() {
        let mut state = AnimationState {
            time: 0.8,
            speed: 1.0,
            looping: false,
            playing: true,
            ..Default::default()
        };

        state.update(0.5, 1.0); // time = 0.8 + 0.5 = 1.3 -> clamps to 1.0
        assert_eq!(state.time, 1.0);
        assert!(!state.playing); // Should stop at end
    }

    #[test]
    fn test_find_keyframes() {
        let times = vec![0.0, 1.0, 2.0, 3.0];

        // Before first keyframe
        let (i0, i1, t) = AnimationClip::find_keyframes(&times, -0.5);
        assert_eq!(i0, 0);
        assert_eq!(i1, 0);
        assert_eq!(t, 0.0);

        // Between keyframes
        let (i0, i1, t) = AnimationClip::find_keyframes(&times, 1.5);
        assert_eq!(i0, 1);
        assert_eq!(i1, 2);
        assert!((t - 0.5).abs() < 0.001);

        // After last keyframe
        let (i0, i1, t) = AnimationClip::find_keyframes(&times, 5.0);
        assert_eq!(i0, 3);
        assert_eq!(i1, 3);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn test_joint_matrices_single_joint() {
        let skeleton = Skeleton {
            joints: vec![Joint {
                name: "root".to_string(),
                parent_index: None,
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform::default(),
            }],
            root_indices: vec![0],
        };

        let local_transforms = vec![Transform {
            translation: Vec3::new(1.0, 0.0, 0.0),
            ..Default::default()
        }];

        let matrices = compute_joint_matrices(&skeleton, &local_transforms);
        assert_eq!(matrices.len(), 1);

        // Should be translation matrix
        assert_eq!(matrices[0].w_axis.truncate(), Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_joint_matrices_hierarchy() {
        let skeleton = Skeleton {
            joints: vec![
                Joint {
                    name: "root".to_string(),
                    parent_index: None,
                    inverse_bind_matrix: Mat4::IDENTITY,
                    local_transform: Transform::default(),
                },
                Joint {
                    name: "child".to_string(),
                    parent_index: Some(0),
                    inverse_bind_matrix: Mat4::IDENTITY,
                    local_transform: Transform::default(),
                },
            ],
            root_indices: vec![0],
        };

        let local_transforms = vec![
            Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                ..Default::default()
            },
            Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                ..Default::default()
            },
        ];

        let matrices = compute_joint_matrices(&skeleton, &local_transforms);
        assert_eq!(matrices.len(), 2);

        // Root should be at (1, 0, 0)
        assert_eq!(matrices[0].w_axis.truncate(), Vec3::new(1.0, 0.0, 0.0));

        // Child should be at (2, 0, 0) (parent + local)
        assert_eq!(matrices[1].w_axis.truncate(), Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn test_cpu_skinning_single_joint() {
        let position = Vec3::new(1.0, 0.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let joints = [0, 0, 0, 0];
        let weights = [1.0, 0.0, 0.0, 0.0];

        // Translation matrix
        let matrix = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let joint_matrices = vec![matrix];

        let (skinned_pos, skinned_normal) =
            skin_vertex_cpu(position, normal, joints, weights, &joint_matrices);

        assert_eq!(skinned_pos, Vec3::new(2.0, 2.0, 3.0));
        assert_eq!(skinned_normal, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn test_cpu_skinning_blend() {
        let position = Vec3::new(1.0, 0.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let joints = [0, 1, 0, 0];
        let weights = [0.5, 0.5, 0.0, 0.0];

        let matrix0 = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let matrix1 = Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0));
        let joint_matrices = vec![matrix0, matrix1];

        let (skinned_pos, _) = skin_vertex_cpu(position, normal, joints, weights, &joint_matrices);

        // 0.5 * (1,0,0) + 0.5 * (3,0,0) = (2,0,0)
        assert_eq!(skinned_pos, Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn test_joint_palette_creation() {
        let matrices = vec![
            Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        ];

        let palette = JointPalette::from_matrices(&matrices);
        assert_eq!(palette.joint_count, 2);
        assert_eq!(
            palette.joints[0].matrix,
            Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)).to_cols_array_2d()
        );
    }
}
