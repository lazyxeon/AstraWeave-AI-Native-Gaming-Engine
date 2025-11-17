use std::f32::consts::FRAC_PI_4;

use astraweave_core::{Entity, IVec2, Team, World};
use aw_editor::scene_state::EditorSceneState;
use aw_editor::TransformableScene;
use glam::{Quat, Vec3};

type SceneAndEntity = (EditorSceneState, Entity);

fn build_scene_state() -> SceneAndEntity {
    let mut world = World::new();
    let entity = world.spawn(
        "EditorSceneEntity",
        IVec2 { x: 1, y: 2 },
        Team { id: 0 },
        100,
        20,
    );
    (EditorSceneState::new(world), entity)
}

#[test]
fn apply_transform_updates_world_and_cache() {
    let (mut scene_state, entity) = build_scene_state();

    let mut transform = scene_state
        .transform_for(entity)
        .expect("entity transform available");
    transform.position.x = 6.0;
    transform.position.z = -3.0;
    transform.rotation = Quat::from_rotation_y(FRAC_PI_4);
    transform.scale = Vec3::splat(2.5);

    scene_state.apply_transform(entity, &transform);

    let pose = scene_state.world().pose(entity).expect("pose exists");
    assert_eq!(pose.pos, IVec2 { x: 6, y: -3 });
    assert!((pose.scale - 2.5).abs() < f32::EPSILON);
    assert!((pose.rotation - FRAC_PI_4).abs() < f32::EPSILON);

    let cached = scene_state
        .transform_for(entity)
        .expect("cached transform available");
    assert!((cached.position.x - 6.0).abs() < f32::EPSILON);
    assert!((cached.position.z + 3.0).abs() < f32::EPSILON);
    assert!((cached.scale.x - 2.5).abs() < f32::EPSILON);
}

#[test]
fn sync_entity_reflects_direct_world_edits() {
    let (mut scene_state, entity) = build_scene_state();

    {
        let pose = scene_state
            .world_mut()
            .pose_mut(entity)
            .expect("pose available");
        pose.pos = IVec2 { x: -4, y: 9 };
        pose.scale = 0.75;
    }

    scene_state.sync_entity(entity);

    let transform = scene_state
        .transform_for(entity)
        .expect("transform exists after sync");
    assert!((transform.position.x + 4.0).abs() < f32::EPSILON);
    assert!((transform.position.z - 9.0).abs() < f32::EPSILON);
    assert!((transform.scale.x - 0.75).abs() < f32::EPSILON);
}

#[test]
fn snapshot_round_trip_restores_pose() {
    let (mut scene_state, entity) = build_scene_state();
    let snapshot = scene_state
        .snapshot_for(entity)
        .expect("snapshot should exist");

    {
        let pose = scene_state
            .world_mut()
            .pose_mut(entity)
            .expect("pose exists during mutation");
        pose.pos = IVec2 { x: 42, y: -18 };
        pose.rotation = 0.0;
        pose.scale = 0.5;
    }

    scene_state.apply_snapshot(entity, &snapshot);

    let pose = scene_state.world().pose(entity).expect("pose restored");
    let expected_pos = IVec2 {
        x: snapshot.position.x.round() as i32,
        y: snapshot.position.z.round() as i32,
    };
    assert_eq!(pose.pos, expected_pos);
    assert!((pose.scale - snapshot.scale.x).abs() < f32::EPSILON);
}
