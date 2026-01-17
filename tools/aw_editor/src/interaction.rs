use crate::command::{MoveEntityCommand, RotateEntityCommand, ScaleEntityCommand, UndoStack};
use crate::gizmo::state::{GizmoMode, GizmoState, TransformSnapshot};
use crate::prefab::PrefabManager;
use astraweave_core::{Entity, IVec2, World};
use glam::Quat;

/// High-level gizmo operation classification (used by telemetry/tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoOperationKind {
    Translate,
    Rotate,
    Scale,
}

/// Measurement payload captured when committing a gizmo operation.
#[derive(Debug, Clone, PartialEq)]
pub enum GizmoMeasurement {
    Translate {
        from: IVec2,
        to: IVec2,
    },
    Rotate {
        from: (f32, f32, f32),
        to: (f32, f32, f32),
    },
    Scale {
        from: f32,
        to: f32,
    },
}

/// Metadata emitted when a gizmo operation commits successfully.
#[derive(Debug, Clone)]
pub struct GizmoCommitMetadata {
    pub entity: Entity,
    pub operation: GizmoOperationKind,
    pub measurement: GizmoMeasurement,
    pub constraint: Option<String>,
}

/// Metadata emitted when a gizmo operation is cancelled.
#[derive(Debug, Clone)]
pub struct GizmoCancelMetadata {
    pub entity: Entity,
    pub operation: GizmoOperationKind,
    pub snapshot: TransformSnapshot,
}

fn active_or_last_mode(state: &GizmoState) -> GizmoMode {
    match state.mode {
        GizmoMode::Inactive => state.last_mode,
        mode => mode,
    }
}

fn constraint_label(mode: GizmoMode) -> Option<String> {
    match mode {
        GizmoMode::Translate { constraint } | GizmoMode::Rotate { constraint } => {
            Some(format!("{:?}", constraint))
        }
        _ => None,
    }
}

fn mode_to_kind(mode: GizmoMode) -> Option<GizmoOperationKind> {
    match mode {
        GizmoMode::Translate { .. } => Some(GizmoOperationKind::Translate),
        GizmoMode::Rotate { .. } => Some(GizmoOperationKind::Rotate),
        GizmoMode::Scale { .. } => Some(GizmoOperationKind::Scale),
        GizmoMode::Inactive => None,
    }
}

/// Attempt to commit the active gizmo operation, pushing an undo command if needed.
///
/// **NEW (Nov 17)**: Optionally accepts a `PrefabManager` to auto-track overrides when
/// transforming prefab instances. If `prefab_manager` is `Some`, any committed transform
/// will automatically mark the entity as having overrides.
pub fn commit_active_gizmo(
    state: &mut GizmoState,
    world: &mut World,
    undo_stack: &mut UndoStack,
) -> Option<GizmoCommitMetadata> {
    commit_active_gizmo_with_prefab_tracking(state, world, undo_stack, None)
}

/// Extended version of `commit_active_gizmo` with auto-tracking for prefab overrides.
///
/// When `prefab_manager` is provided and the entity is part of a prefab instance,
/// successfully committed transforms will automatically call `track_override()` to
/// mark the entity as modified.
pub fn commit_active_gizmo_with_prefab_tracking(
    state: &mut GizmoState,
    world: &mut World,
    undo_stack: &mut UndoStack,
    prefab_manager: Option<&mut PrefabManager>,
) -> Option<GizmoCommitMetadata> {
    let entity = state.selected_entity?;
    let snapshot = state.start_transform?;
    let mode = active_or_last_mode(state);
    let operation = mode_to_kind(mode)?;
    let constraint = constraint_label(mode);

    let metadata = match (operation, world.pose(entity)) {
        (GizmoOperationKind::Translate, Some(pose)) => {
            let old_pos = IVec2 {
                x: snapshot.position.x.round() as i32,
                y: snapshot.position.z.round() as i32,
            };
            let new_pos = pose.pos;
            if old_pos == new_pos {
                None
            } else {
                undo_stack.push_executed(MoveEntityCommand::new(entity, old_pos, new_pos));
                Some(GizmoCommitMetadata {
                    entity,
                    operation,
                    measurement: GizmoMeasurement::Translate {
                        from: old_pos,
                        to: new_pos,
                    },
                    constraint,
                })
            }
        }
        (GizmoOperationKind::Rotate, Some(pose)) => {
            let old_rot = snapshot.rotation.to_euler(glam::EulerRot::XYZ);
            let new_rot = (pose.rotation_x, pose.rotation, pose.rotation_z);
            let changed = (old_rot.0 - new_rot.0).abs() > 0.01
                || (old_rot.1 - new_rot.1).abs() > 0.01
                || (old_rot.2 - new_rot.2).abs() > 0.01;
            if changed {
                undo_stack.push_executed(RotateEntityCommand::new(entity, old_rot, new_rot));
                Some(GizmoCommitMetadata {
                    entity,
                    operation,
                    measurement: GizmoMeasurement::Rotate {
                        from: old_rot,
                        to: new_rot,
                    },
                    constraint,
                })
            } else {
                None
            }
        }
        (GizmoOperationKind::Scale, Some(pose)) => {
            let old_scale = snapshot.scale.x;
            let new_scale = pose.scale;
            if (old_scale - new_scale).abs() > 0.01 {
                undo_stack.push_executed(ScaleEntityCommand::new(entity, old_scale, new_scale));
                Some(GizmoCommitMetadata {
                    entity,
                    operation,
                    measurement: GizmoMeasurement::Scale {
                        from: old_scale,
                        to: new_scale,
                    },
                    constraint: None,
                })
            } else {
                None
            }
        }
        _ => None,
    };

    state.start_transform = None;

    // Auto-track prefab override if entity is part of a prefab instance
    if let (Some(mgr), Some(meta)) = (prefab_manager, &metadata) {
        if let Some(instance) = mgr.find_instance_mut(meta.entity) {
            instance.track_override(meta.entity, world);
        }
    }

    metadata
}

/// Cancel the active gizmo operation and revert to the snapshot if available.
pub fn cancel_active_gizmo(
    state: &mut GizmoState,
    world: &mut World,
) -> Option<GizmoCancelMetadata> {
    let entity = state.selected_entity?;
    let snapshot = state.start_transform?;
    let mode = active_or_last_mode(state);
    let operation = mode_to_kind(mode)?;

    if let Some(pose) = world.pose_mut(entity) {
        pose.pos = IVec2 {
            x: snapshot.position.x.round() as i32,
            y: snapshot.position.z.round() as i32,
        };
        let (rx, ry, rz) = snapshot.rotation.to_euler(glam::EulerRot::XYZ);
        pose.rotation_x = rx;
        pose.rotation = ry;
        pose.rotation_z = rz;
        pose.scale = snapshot.scale.x;
    }

    state.start_transform = None;
    Some(GizmoCancelMetadata {
        entity,
        operation,
        snapshot,
    })
}

/// Ensure the gizmo has a starting snapshot sourced from the World.
pub fn ensure_world_snapshot(state: &mut GizmoState, world: &World) -> Option<TransformSnapshot> {
    if state.start_transform.is_some() {
        return state.start_transform;
    }
    let entity = state.selected_entity?;
    let pose = world.pose(entity)?;
    let snapshot = TransformSnapshot {
        position: glam::Vec3::new(pose.pos.x as f32, 1.0, pose.pos.y as f32),
        rotation: Quat::from_euler(
            glam::EulerRot::XYZ,
            pose.rotation_x,
            pose.rotation,
            pose.rotation_z,
        ),
        scale: glam::Vec3::splat(pose.scale),
    };
    state.start_transform = Some(snapshot);
    Some(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::UndoStack;
    use crate::gizmo::state::{GizmoMode, TransformSnapshot, AxisConstraint};
    use astraweave_core::{World, Entity, IVec2, Team};
    use glam::{Vec3, Quat};

    fn create_test_world() -> (World, Entity) {
        let mut world = World::new();
        let entity = world.spawn("TestEntity", IVec2::new(0, 0), Team { id: 0 }, 100, 10);
        (world, entity)
    }

    #[test]
    fn test_commit_translate_generates_command() {
        let (mut world, entity) = create_test_world();
        let mut undo_stack = UndoStack::new(100);
        
        let start_pos = Vec3::new(10.0, 0.0, 10.0);
        let snapshot = TransformSnapshot {
            position: start_pos,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        
        let mut state = GizmoState {
            selected_entity: Some(entity),
            start_transform: Some(snapshot),
            last_mode: GizmoMode::Translate { constraint: AxisConstraint::X },
            mode: GizmoMode::Inactive,
            ..Default::default()
        };
        
        if let Some(pose) = world.pose_mut(entity) {
            pose.pos = IVec2::new(20, 20);
        }
        
        let meta = commit_active_gizmo(&mut state, &mut world, &mut undo_stack);
        
        assert!(meta.is_some());
        assert!(undo_stack.can_undo());
        
        let meta = meta.unwrap();
        match meta.measurement {
            GizmoMeasurement::Translate { from, to } => {
                assert_eq!(from, IVec2::new(10, 10));
                assert_eq!(to, IVec2::new(20, 20));
            }
            _ => panic!("Expected Translate measurement"),
        }
        
        assert!(state.start_transform.is_none());
    }

    #[test]
    fn test_commit_no_change_does_not_generate_command() {
        let (mut world, entity) = create_test_world();
        let mut undo_stack = UndoStack::new(100);
        
        let start_pos = Vec3::ZERO;
        let snapshot = TransformSnapshot {
            position: start_pos,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        
        let mut state = GizmoState {
            selected_entity: Some(entity),
            start_transform: Some(snapshot),
            last_mode: GizmoMode::Translate { constraint: AxisConstraint::X },
            mode: GizmoMode::Inactive,
            ..Default::default()
        };

        let meta = commit_active_gizmo(&mut state, &mut world, &mut undo_stack);
        
        assert!(meta.is_none());
        assert!(!undo_stack.can_undo());
        assert!(state.start_transform.is_none());
    }

    #[test]
    fn test_cancel_gizmo_reverts_transform() {
        let (mut world, entity) = create_test_world();
        
        let start_pos = Vec3::new(5.0, 0.0, 5.0);
        let snapshot = TransformSnapshot {
            position: start_pos,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        
        let mut state = GizmoState {
            selected_entity: Some(entity),
            start_transform: Some(snapshot),
            last_mode: GizmoMode::Translate { constraint: AxisConstraint::X },
            mode: GizmoMode::Translate { constraint: AxisConstraint::X },
            ..Default::default()
        }; 
        
        if let Some(pose) = world.pose_mut(entity) {
            pose.pos = IVec2::new(100, 100);
        }
        
        let meta = cancel_active_gizmo(&mut state, &mut world);
        
        assert!(meta.is_some());
        
        if let Some(pose) = world.pose(entity) {
            assert_eq!(pose.pos, IVec2::new(5, 5));
        }
    }
}

