use crate::command::{MoveEntityCommand, RotateEntityCommand, ScaleEntityCommand, UndoStack};
use crate::gizmo::state::{GizmoMode, GizmoState, TransformSnapshot};
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
    if matches!(state.mode, GizmoMode::Inactive) {
        state.last_operation
    } else {
        state.mode
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
pub fn commit_active_gizmo(
    state: &mut GizmoState,
    world: &mut World,
    undo_stack: &mut UndoStack,
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
