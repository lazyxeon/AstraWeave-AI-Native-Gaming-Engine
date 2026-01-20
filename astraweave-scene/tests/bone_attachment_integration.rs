//! Integration Test: Bone Attachments with Scene Graph
//!
//! Phase 2 Task 5 (Phase E): Validates that bone attachment system (CParentBone)
//! correctly propagates joint transforms to attached entities across animation frames.

#[cfg(feature = "ecs")]
mod tests {
    use astraweave_ecs::World;
    use astraweave_scene::ecs::*;
    use astraweave_scene::Transform;
    use glam::{Mat4, Quat, Vec3};

    /// Helper to create simple animated skeleton in ECS
    fn setup_animated_skeleton(world: &mut World) -> astraweave_ecs::Entity {
        let entity = world.spawn();

        // Skeleton with 3 joints
        world.insert(
            entity,
            CSkeleton {
                joint_count: 3,
                root_indices: vec![0],
                parent_indices: vec![None, Some(0), Some(1)],
                inverse_bind_matrices: vec![
                    Mat4::IDENTITY,
                    Mat4::from_translation(Vec3::new(0.0, -1.0, 0.0)),
                    Mat4::from_translation(Vec3::new(0.0, -2.0, 0.0)),
                ],
                local_transforms: vec![
                    Transform::default(),
                    Transform {
                        translation: Vec3::new(0.0, 1.0, 0.0),
                        ..Default::default()
                    },
                    Transform {
                        translation: Vec3::new(0.0, 1.0, 0.0),
                        ..Default::default()
                    },
                ],
            },
        );

        // Joint matrices (initially identity for rest pose)
        world.insert(entity, CJointMatrices::default());

        // Animator (will be used to drive animation)
        world.insert(
            entity,
            CAnimator {
                clip_index: 0,
                time: 0.0,
                speed: 1.0,
                state: PlaybackState::Playing,
                looping: true,
            },
        );

        entity
    }

    /// Test: Bone attachment follows joint in rest pose
    #[test]
    fn test_bone_attachment_rest_pose() {
        let mut world = astraweave_ecs::World::new();

        // Create skeleton entity
        let skeleton_entity = setup_animated_skeleton(&mut world);

        // Set joint matrices to known values (rest pose)
        let matrices = CJointMatrices {
            matrices: vec![
            Mat4::IDENTITY,                                   // Joint 0 at origin
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)), // Joint 1 at Y=1
            Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)), // Joint 2 at Y=2
            ],
            ..Default::default()
        };
        world.insert(skeleton_entity, matrices);

        // Create attached entity (weapon)
        let weapon_entity = world.spawn();
        world.insert(
            weapon_entity,
            CParentBone {
                skeleton_entity,
                joint_index: 1, // Attach to joint 1 (hand)
            },
        );

        // Run bone attachment sync
        sync_bone_attachments(&mut world);

        // Verify weapon is at joint 1 position
        let weapon_transform = world.get::<CTransformWorld>(weapon_entity).unwrap();
        let weapon_pos = weapon_transform.0.w_axis.truncate();

        assert!(
            (weapon_pos - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-4,
            "Weapon should be at joint 1 position: {:?}",
            weapon_pos
        );
    }

    /// Test: Bone attachment follows joint across animation frames
    #[test]
    fn test_bone_attachment_animation_follow() {
        let mut world = astraweave_ecs::World::new();

        let skeleton_entity = setup_animated_skeleton(&mut world);

        // Create weapon attached to joint 2 (end effector)
        let weapon_entity = world.spawn();
        world.insert(
            weapon_entity,
            CParentBone {
                skeleton_entity,
                joint_index: 2,
            },
        );

        // Frame 1: Joint 2 at Y=2
        let matrices1 = CJointMatrices {
            matrices: vec![
            Mat4::IDENTITY,
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)),
            ],
            ..Default::default()
        };
        world.insert(skeleton_entity, matrices1);
        sync_bone_attachments(&mut world);

        let weapon_pos1 = world
            .get::<CTransformWorld>(weapon_entity)
            .unwrap()
            .0
            .w_axis
            .truncate();

        // Frame 2: Joint 2 moved to Y=3 (animation)
        let matrices2 = CJointMatrices {
            matrices: vec![
            Mat4::IDENTITY,
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            Mat4::from_translation(Vec3::new(0.0, 3.0, 0.0)), // Moved up
            ],
            ..Default::default()
        };
        world.insert(skeleton_entity, matrices2);
        sync_bone_attachments(&mut world);

        let weapon_pos2 = world
            .get::<CTransformWorld>(weapon_entity)
            .unwrap()
            .0
            .w_axis
            .truncate();

        // Verify weapon moved with joint
        assert!(
            (weapon_pos1 - Vec3::new(0.0, 2.0, 0.0)).length() < 1e-4,
            "Frame 1: weapon at Y=2, got {:?}",
            weapon_pos1
        );
        assert!(
            (weapon_pos2 - Vec3::new(0.0, 3.0, 0.0)).length() < 1e-4,
            "Frame 2: weapon at Y=3, got {:?}",
            weapon_pos2
        );
    }

    /// Test: Multiple attachments to different joints
    #[test]
    fn test_multiple_bone_attachments() {
        let mut world = astraweave_ecs::World::new();

        let skeleton_entity = setup_animated_skeleton(&mut world);

        // Attach weapon to joint 1 (hand)
        let weapon_entity = world.spawn();
        world.insert(
            weapon_entity,
            CParentBone {
                skeleton_entity,
                joint_index: 1,
            },
        );

        // Attach shield to joint 0 (root)
        let shield_entity = world.spawn();
        world.insert(
            shield_entity,
            CParentBone {
                skeleton_entity,
                joint_index: 0,
            },
        );

        // Set joint matrices
        let matrices = CJointMatrices {
            matrices: vec![
            Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)), // Joint 0
            Mat4::from_translation(Vec3::new(2.0, 1.0, 0.0)), // Joint 1
            Mat4::from_translation(Vec3::new(3.0, 2.0, 0.0)), // Joint 2
            ],
            ..Default::default()
        };
        world.insert(skeleton_entity, matrices);

        sync_bone_attachments(&mut world);

        // Verify both attachments
        let weapon_pos = world
            .get::<CTransformWorld>(weapon_entity)
            .unwrap()
            .0
            .w_axis
            .truncate();
        let shield_pos = world
            .get::<CTransformWorld>(shield_entity)
            .unwrap()
            .0
            .w_axis
            .truncate();

        assert!(
            (weapon_pos - Vec3::new(2.0, 1.0, 0.0)).length() < 1e-4,
            "Weapon at joint 1"
        );
        assert!(
            (shield_pos - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-4,
            "Shield at joint 0"
        );
    }

    /// Test: Bone attachment with rotation
    #[test]
    fn test_bone_attachment_rotation() {
        let mut world = astraweave_ecs::World::new();

        let skeleton_entity = setup_animated_skeleton(&mut world);
        let weapon_entity = world.spawn();
        world.insert(
            weapon_entity,
            CParentBone {
                skeleton_entity,
                joint_index: 1,
            },
        );

        // Joint 1 with 90 degree rotation around Z
        let rotation = Quat::from_rotation_z(std::f32::consts::PI / 2.0);
        let transform =
            Mat4::from_scale_rotation_translation(Vec3::ONE, rotation, Vec3::new(0.0, 1.0, 0.0));

        let matrices = CJointMatrices {
            matrices: vec![Mat4::IDENTITY, transform, Mat4::IDENTITY],
            ..Default::default()
        };
        world.insert(skeleton_entity, matrices);

        sync_bone_attachments(&mut world);

        let weapon_transform = world.get::<CTransformWorld>(weapon_entity).unwrap().0;

        // Verify translation
        let weapon_pos = weapon_transform.w_axis.truncate();
        assert!(
            (weapon_pos - Vec3::new(0.0, 1.0, 0.0)).length() < 0.1,
            "Position preserved"
        );

        // Verify rotation is applied (check matrix is not identity)
        let diff = (weapon_transform - Mat4::IDENTITY).abs();
        let max_diff = diff
            .to_cols_array()
            .iter()
            .fold(0.0f32, |acc, &x| acc.max(x));
        assert!(max_diff > 0.5, "Transform should include rotation");
    }

    /// Test: Invalid joint index (out of bounds)
    #[test]
    fn test_bone_attachment_invalid_joint() {
        let mut world = astraweave_ecs::World::new();

        let skeleton_entity = setup_animated_skeleton(&mut world);
        let weapon_entity = world.spawn();
        world.insert(
            weapon_entity,
            CParentBone {
                skeleton_entity,
                joint_index: 99, // Invalid index
            },
        );

        let matrices = CJointMatrices {
            matrices: vec![Mat4::IDENTITY, Mat4::IDENTITY, Mat4::IDENTITY],
            ..Default::default()
        };
        world.insert(skeleton_entity, matrices);

        // Should not panic, just skip this attachment
        sync_bone_attachments(&mut world);

        // Weapon should not have transform set (or have default)
        // This test documents behavior: invalid indices are silently skipped
    }

    /// Test: Attachment persists across multiple frames
    #[test]
    fn test_bone_attachment_persistence() {
        let mut world = astraweave_ecs::World::new();

        let skeleton_entity = setup_animated_skeleton(&mut world);
        let weapon_entity = world.spawn();
        world.insert(
            weapon_entity,
            CParentBone {
                skeleton_entity,
                joint_index: 1,
            },
        );

        // Run multiple frames
        for frame in 0..10 {
            let y_offset = frame as f32 * 0.1;
            let matrices = CJointMatrices {
                matrices: vec![
                Mat4::IDENTITY,
                Mat4::from_translation(Vec3::new(0.0, 1.0 + y_offset, 0.0)),
                Mat4::IDENTITY,
                ],
                ..Default::default()
            };
            world.insert(skeleton_entity, matrices);

            sync_bone_attachments(&mut world);

            let weapon_pos = world
                .get::<CTransformWorld>(weapon_entity)
                .unwrap()
                .0
                .w_axis
                .truncate();
            let expected_y = 1.0 + y_offset;

            assert!(
                (weapon_pos.y - expected_y).abs() < 1e-4,
                "Frame {}: weapon Y should be {}, got {}",
                frame,
                expected_y,
                weapon_pos.y
            );
        }
    }

    /// Test: Bone attachment with scene graph parent
    #[test]
    fn test_bone_attachment_with_scene_parent() {
        let mut world = astraweave_ecs::World::new();

        let skeleton_entity = setup_animated_skeleton(&mut world);

        // Create parent node
        let parent_entity = world.spawn();
        world.insert(
            parent_entity,
            CTransformWorld(Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0))),
        );

        // Create weapon attached to joint AND parented to another entity
        let weapon_entity = world.spawn();
        world.insert(weapon_entity, CParent(parent_entity));
        world.insert(
            weapon_entity,
            CParentBone {
                skeleton_entity,
                joint_index: 1,
            },
        );

        let matrices = CJointMatrices {
            matrices: vec![
            Mat4::IDENTITY,
            Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            Mat4::IDENTITY,
            ],
            ..Default::default()
        };
        world.insert(skeleton_entity, matrices);

        sync_bone_attachments(&mut world);

        // Weapon world transform should be joint transform
        // Local transform should be computed relative to parent
        let weapon_world = world.get::<CTransformWorld>(weapon_entity).unwrap().0;
        let weapon_pos = weapon_world.w_axis.truncate();

        // World position should match joint (bone attachment overrides parent)
        assert!(
            (weapon_pos - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-4,
            "World pos should match joint, got {:?}",
            weapon_pos
        );

        // Local transform should be computed
        let weapon_local = world.get::<CTransformLocal>(weapon_entity);
        assert!(weapon_local.is_some(), "Local transform should be computed");
    }
}
