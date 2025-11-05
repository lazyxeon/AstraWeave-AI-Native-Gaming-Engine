// Phase 7: Additional Animation Tests
// Targeting edge cases and uncovered code paths in animation.rs

#[cfg(test)]
mod animation_extra_tests {
    use super::super::animation::*;
    use glam::{Mat4, Quat, Vec3};

    // ============================================================================
    // Transform Tests (Edge Cases)
    // ============================================================================

    #[test]
    fn test_transform_lerp_zero() {
        let t1 = Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let t2 = Transform {
            translation: Vec3::new(10.0, 10.0, 10.0),
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
            scale: Vec3::splat(2.0),
        };

        let result = t1.lerp(&t2, 0.0);
        assert_eq!(result.translation, t1.translation);
        assert_eq!(result.rotation, t1.rotation);
        assert_eq!(result.scale, t1.scale);
    }

    #[test]
    fn test_transform_lerp_one() {
        let t1 = Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let t2 = Transform {
            translation: Vec3::new(10.0, 10.0, 10.0),
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
            scale: Vec3::splat(2.0),
        };

        let result = t1.lerp(&t2, 1.0);
        assert_eq!(result.translation, t2.translation);
        assert_eq!(result.scale, t2.scale);
        // Quaternion slerp at t=1.0 should equal t2.rotation (within floating-point tolerance)
        // Use absolute difference of components since quaternions can have -q = q equivalence
        let dot = result.rotation.dot(t2.rotation).abs();
        assert!(
            dot > 0.999,
            "Expected quaternions to match, got dot product: {}",
            dot
        );
    }

    #[test]
    fn test_transform_lerp_midpoint() {
        let t1 = Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let t2 = Transform {
            translation: Vec3::new(10.0, 20.0, 30.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(3.0),
        };

        let result = t1.lerp(&t2, 0.5);
        assert_eq!(result.translation, Vec3::new(5.0, 10.0, 15.0));
        assert_eq!(result.scale, Vec3::splat(2.0));
    }

    #[test]
    fn test_transform_to_matrix_with_rotation() {
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let t = Transform {
            translation: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        };

        let mat = t.to_matrix();

        // Rotating 90° around Y should transform (1,0,0) to approximately (0,0,-1)
        let vec = Vec3::new(1.0, 0.0, 0.0);
        let transformed = mat.transform_point3(vec);
        assert!((transformed.x - 0.0).abs() < 0.001);
        assert!((transformed.z - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_transform_to_matrix_with_scale() {
        let t = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(2.0, 3.0, 4.0),
        };

        let mat = t.to_matrix();
        let vec = Vec3::new(1.0, 1.0, 1.0);
        let transformed = mat.transform_point3(vec);

        assert_eq!(transformed, Vec3::new(2.0, 3.0, 4.0));
    }

    // ============================================================================
    // AnimationState Tests (Edge Cases)
    // ============================================================================

    #[test]
    fn test_animation_state_update_not_playing() {
        let mut state = AnimationState {
            time: 0.5,
            speed: 1.0,
            looping: true,
            playing: false,
            ..Default::default()
        };

        state.update(1.0, 2.0);

        // Time should not advance when not playing
        assert_eq!(state.time, 0.5);
    }

    #[test]
    fn test_animation_state_update_negative_time_looping() {
        let mut state = AnimationState {
            time: 0.2,
            speed: -1.0, // Negative speed (reverse playback)
            looping: true,
            playing: true,
            ..Default::default()
        };

        state.update(0.5, 1.0); // time = 0.2 + (-1.0 * 0.5) = -0.3

        // Should wrap around: 1.0 + (-0.3 % 1.0) = 1.0 - 0.3 = 0.7
        assert!((state.time - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_animation_state_update_multiple_wraps() {
        let mut state = AnimationState {
            time: 0.5,
            speed: 1.0,
            looping: true,
            playing: true,
            ..Default::default()
        };

        state.update(3.6, 1.0); // time = 0.5 + 3.6 = 4.1 -> wraps to 0.1

        assert!((state.time - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_animation_state_update_speed_zero() {
        let mut state = AnimationState {
            time: 0.5,
            speed: 0.0,
            looping: true,
            playing: true,
            ..Default::default()
        };

        state.update(1.0, 2.0);

        // Time should not change with zero speed
        assert_eq!(state.time, 0.5);
    }

    #[test]
    fn test_animation_state_play_pause_stop() {
        let mut state = AnimationState::default();

        assert!(!state.playing);

        state.play();
        assert!(state.playing);

        state.pause();
        assert!(!state.playing);

        state.time = 1.5;
        state.stop();
        assert!(!state.playing);
        assert_eq!(state.time, 0.0);
    }

    #[test]
    fn test_animation_state_restart() {
        let mut state = AnimationState {
            time: 2.5,
            playing: false,
            ..Default::default()
        };

        state.restart();

        assert_eq!(state.time, 0.0);
        assert!(state.playing);
    }

    // ============================================================================
    // AnimationClip::sample Tests (Edge Cases)
    // Note: find_keyframes is private, so we test it indirectly via sample()
    // ============================================================================

    #[test]
    fn test_animation_sample_empty_channels() {
        let skeleton = Skeleton {
            joints: vec![Joint {
                name: "root".to_string(),
                parent_index: None,
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform::default(),
            }],
            root_indices: vec![0],
        };

        let clip = AnimationClip {
            name: "test".to_string(),
            duration: 1.0,
            channels: vec![], // No channels
        };

        let transforms = clip.sample(0.5, &skeleton);

        // Should return bind pose (default transform)
        assert_eq!(transforms.len(), 1);
        assert_eq!(transforms[0].translation, Vec3::ZERO);
        assert_eq!(transforms[0].rotation, Quat::IDENTITY);
        assert_eq!(transforms[0].scale, Vec3::ONE);
    }

    #[test]
    fn test_animation_sample_invalid_joint_index() {
        let skeleton = Skeleton {
            joints: vec![Joint {
                name: "root".to_string(),
                parent_index: None,
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform::default(),
            }],
            root_indices: vec![0],
        };

        let clip = AnimationClip {
            name: "test".to_string(),
            duration: 1.0,
            channels: vec![AnimationChannel {
                target_joint_index: 99, // Invalid index
                times: vec![0.0, 1.0],
                data: ChannelData::Translation(vec![Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)]),
                interpolation: Interpolation::Linear,
            }],
        };

        let transforms = clip.sample(0.5, &skeleton);

        // Should return bind pose (invalid channel ignored)
        assert_eq!(transforms.len(), 1);
        assert_eq!(transforms[0].translation, Vec3::ZERO);
    }

    #[test]
    fn test_animation_sample_step_interpolation() {
        let skeleton = Skeleton {
            joints: vec![Joint {
                name: "root".to_string(),
                parent_index: None,
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform::default(),
            }],
            root_indices: vec![0],
        };

        let clip = AnimationClip {
            name: "test".to_string(),
            duration: 1.0,
            channels: vec![AnimationChannel {
                target_joint_index: 0,
                times: vec![0.0, 1.0],
                data: ChannelData::Translation(vec![
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(10.0, 0.0, 0.0),
                ]),
                interpolation: Interpolation::Step,
            }],
        };

        // Sample at midpoint - step should use first value
        let transforms = clip.sample(0.5, &skeleton);
        assert_eq!(transforms[0].translation, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_animation_sample_rotation_channel() {
        let skeleton = Skeleton {
            joints: vec![Joint {
                name: "root".to_string(),
                parent_index: None,
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform::default(),
            }],
            root_indices: vec![0],
        };

        let q1 = Quat::IDENTITY;
        let q2 = Quat::from_rotation_y(std::f32::consts::PI);

        let clip = AnimationClip {
            name: "test".to_string(),
            duration: 1.0,
            channels: vec![AnimationChannel {
                target_joint_index: 0,
                times: vec![0.0, 1.0],
                data: ChannelData::Rotation(vec![q1, q2]),
                interpolation: Interpolation::Linear,
            }],
        };

        let transforms = clip.sample(0.5, &skeleton);

        // Rotation should be interpolated (slerp)
        // At t=0.5, should be roughly halfway between identity and 180° rotation
        let result_angle = transforms[0].rotation.to_axis_angle().1;
        assert!((result_angle - std::f32::consts::FRAC_PI_2).abs() < 0.1);
    }

    #[test]
    fn test_animation_sample_scale_channel() {
        let skeleton = Skeleton {
            joints: vec![Joint {
                name: "root".to_string(),
                parent_index: None,
                inverse_bind_matrix: Mat4::IDENTITY,
                local_transform: Transform::default(),
            }],
            root_indices: vec![0],
        };

        let clip = AnimationClip {
            name: "test".to_string(),
            duration: 1.0,
            channels: vec![AnimationChannel {
                target_joint_index: 0,
                times: vec![0.0, 1.0],
                data: ChannelData::Scale(vec![Vec3::ONE, Vec3::splat(3.0)]),
                interpolation: Interpolation::Linear,
            }],
        };

        let transforms = clip.sample(0.5, &skeleton);

        // Scale should be lerped: (1,1,1) -> (3,3,3) at t=0.5 = (2,2,2)
        assert_eq!(transforms[0].scale, Vec3::splat(2.0));
    }

    // ============================================================================
    // Pose Computation Tests (Hierarchy)
    // ============================================================================

    #[test]
    fn test_compute_joint_matrices_multiple_roots() {
        let skeleton = Skeleton {
            joints: vec![
                Joint {
                    name: "root1".to_string(),
                    parent_index: None,
                    inverse_bind_matrix: Mat4::IDENTITY,
                    local_transform: Transform::default(),
                },
                Joint {
                    name: "root2".to_string(),
                    parent_index: None,
                    inverse_bind_matrix: Mat4::IDENTITY,
                    local_transform: Transform::default(),
                },
            ],
            root_indices: vec![0, 1],
        };

        let local_transforms = vec![
            Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                ..Default::default()
            },
            Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                ..Default::default()
            },
        ];

        let matrices = compute_joint_matrices(&skeleton, &local_transforms).expect("Failed to compute joint matrices");

        assert_eq!(matrices.len(), 2);
        assert_eq!(matrices[0].w_axis.truncate(), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(matrices[1].w_axis.truncate(), Vec3::new(0.0, 2.0, 0.0));
    }

    #[test]
    fn test_compute_joint_matrices_deep_hierarchy() {
        let skeleton = Skeleton {
            joints: vec![
                Joint {
                    name: "root".to_string(),
                    parent_index: None,
                    inverse_bind_matrix: Mat4::IDENTITY,
                    local_transform: Transform::default(),
                },
                Joint {
                    name: "child1".to_string(),
                    parent_index: Some(0),
                    inverse_bind_matrix: Mat4::IDENTITY,
                    local_transform: Transform::default(),
                },
                Joint {
                    name: "child2".to_string(),
                    parent_index: Some(1),
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
            Transform {
                translation: Vec3::new(1.0, 0.0, 0.0),
                ..Default::default()
            },
        ];

        let matrices = compute_joint_matrices(&skeleton, &local_transforms).expect("Failed to compute joint matrices");

        // Cumulative positions: (1,0,0), (2,0,0), (3,0,0)
        assert_eq!(matrices[0].w_axis.truncate(), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(matrices[1].w_axis.truncate(), Vec3::new(2.0, 0.0, 0.0));
        assert_eq!(matrices[2].w_axis.truncate(), Vec3::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn test_compute_joint_matrices_with_inverse_bind() {
        let skeleton = Skeleton {
            joints: vec![Joint {
                name: "root".to_string(),
                parent_index: None,
                inverse_bind_matrix: Mat4::from_translation(Vec3::new(-1.0, 0.0, 0.0)),
                local_transform: Transform::default(),
            }],
            root_indices: vec![0],
        };

        let local_transforms = vec![Transform {
            translation: Vec3::new(2.0, 0.0, 0.0),
            ..Default::default()
        }];

        let matrices = compute_joint_matrices(&skeleton, &local_transforms).expect("Failed to compute joint matrices");

        // World = (2,0,0), then multiply by inverse bind (-1,0,0)
        // Result should be (1,0,0)
        assert_eq!(matrices[0].w_axis.truncate(), Vec3::new(1.0, 0.0, 0.0));
    }

    // ============================================================================
    // CPU Skinning Tests (Edge Cases)
    // ============================================================================

    #[test]
    fn test_cpu_skinning_zero_weights() {
        let position = Vec3::new(1.0, 2.0, 3.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let joints = [0, 1, 2, 3];
        let weights = [0.0, 0.0, 0.0, 0.0]; // All zero

        let matrices = vec![Mat4::from_translation(Vec3::new(10.0, 10.0, 10.0))];

        let (skinned_pos, skinned_normal) =
            skin_vertex_cpu(position, normal, joints, weights, &matrices);

        // All zero weights -> result should be zero
        assert_eq!(skinned_pos, Vec3::ZERO);
        assert_eq!(skinned_normal, Vec3::ZERO);
    }

    #[test]
    fn test_cpu_skinning_invalid_joint_index() {
        let position = Vec3::new(1.0, 0.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let joints = [99, 0, 0, 0]; // First joint index out of bounds
        let weights = [1.0, 0.0, 0.0, 0.0];

        let matrices = vec![Mat4::IDENTITY];

        let (skinned_pos, skinned_normal) =
            skin_vertex_cpu(position, normal, joints, weights, &matrices);

        // Invalid joint should be skipped, result is zero (no valid joints)
        assert_eq!(skinned_pos, Vec3::ZERO);
        assert_eq!(skinned_normal, Vec3::ZERO);
    }

    #[test]
    fn test_cpu_skinning_partial_weights() {
        let position = Vec3::new(1.0, 0.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let joints = [0, 1, 0, 0];
        let weights = [0.3, 0.7, 0.0, 0.0]; // Partial weights

        let matrix0 = Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0));
        let matrix1 = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        let matrices = vec![matrix0, matrix1];

        let (skinned_pos, _) = skin_vertex_cpu(position, normal, joints, weights, &matrices);

        // 0.3 * (1,0,0) + 0.7 * (11,0,0) = (0.3 + 7.7, 0, 0) = (8.0, 0, 0)
        assert!((skinned_pos.x - 8.0).abs() < 0.001);
    }

    #[test]
    fn test_cpu_skinning_normal_transformation() {
        let position = Vec3::new(1.0, 0.0, 0.0);
        let normal = Vec3::new(1.0, 0.0, 0.0);
        let joints = [0, 0, 0, 0];
        let weights = [1.0, 0.0, 0.0, 0.0];

        // Rotation matrix (90° around Y)
        let rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let matrix = Mat4::from_rotation_translation(rotation, Vec3::ZERO);
        let matrices = vec![matrix];

        let (_, skinned_normal) = skin_vertex_cpu(position, normal, joints, weights, &matrices);

        // Normal (1,0,0) rotated 90° around Y should become approximately (0,0,-1)
        assert!((skinned_normal.x - 0.0).abs() < 0.001);
        assert!((skinned_normal.z - (-1.0)).abs() < 0.001);
    }

    // ============================================================================
    // JointPalette Tests (GPU Data Structure)
    // ============================================================================

    #[test]
    fn test_joint_palette_default() {
        let palette = JointPalette::default();

        assert_eq!(palette.joint_count, 0);
        assert_eq!(palette.joints[0].matrix, Mat4::IDENTITY.to_cols_array_2d());
    }

    #[test]
    fn test_joint_palette_from_matrices_empty() {
        let matrices: Vec<Mat4> = vec![];
        let palette = JointPalette::from_matrices(&matrices);

        assert_eq!(palette.joint_count, 0);
    }

    #[test]
    fn test_joint_palette_from_matrices_max_overflow() {
        // More than MAX_JOINTS (256)
        let matrices: Vec<Mat4> = (0..300)
            .map(|i| Mat4::from_translation(Vec3::new(i as f32, 0.0, 0.0)))
            .collect();

        let palette = JointPalette::from_matrices(&matrices);

        // Should clamp to MAX_JOINTS
        assert_eq!(palette.joint_count, MAX_JOINTS as u32);
    }

    #[test]
    fn test_joint_palette_matrix_conversion() {
        let matrix = Mat4::from_translation(Vec3::new(5.0, 10.0, 15.0));
        let gpu_matrix = JointMatrixGPU::from(matrix);

        assert_eq!(gpu_matrix.matrix, matrix.to_cols_array_2d());

        // Check translation component (last column)
        assert_eq!(gpu_matrix.matrix[3][0], 5.0);
        assert_eq!(gpu_matrix.matrix[3][1], 10.0);
        assert_eq!(gpu_matrix.matrix[3][2], 15.0);
    }

    #[test]
    fn test_joint_matrix_gpu_size() {
        // Should be 64 bytes (4x4 floats)
        assert_eq!(std::mem::size_of::<JointMatrixGPU>(), 64);
    }

    #[test]
    fn test_joint_palette_size() {
        // 256 joints * 64 bytes + 4 bytes (count) + 12 bytes (padding) = 16,400 bytes
        assert_eq!(std::mem::size_of::<JointPalette>(), 256 * 64 + 16);
    }
}
