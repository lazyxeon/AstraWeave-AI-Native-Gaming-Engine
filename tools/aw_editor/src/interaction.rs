use crate::command::{TransformOperation, TransformTransaction, UndoStack};
use crate::gizmo::state::{GizmoMode, GizmoState, TransformSnapshot};
use astraweave_core::{Entity, IVec2, World};
use glam::Quat;
use tracing::{debug, field, info, info_span};

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

fn mode_to_operation(mode: GizmoMode) -> Option<TransformOperation> {
    match mode {
        GizmoMode::Translate { .. } => Some(TransformOperation::Translate),
        GizmoMode::Rotate { .. } => Some(TransformOperation::Rotate),
        GizmoMode::Scale { .. } => Some(TransformOperation::Scale),
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
    let mode = active_or_last_mode(state);
    let gizmo_kind = mode_to_kind(mode)?;
    let transaction_operation = mode_to_operation(mode)?;
    let constraint = constraint_label(mode);
    let constraint_text = constraint.as_deref().unwrap_or("none");
    let span = info_span!(
        "aw_editor.gizmo.commit",
        entity = entity as u32,
        mode = ?mode,
        operation = ?gizmo_kind,
        constraint = field::display(constraint_text)
    );
    let _guard = span.enter();

    if let Some(tx) = state.transform_transaction.as_mut() {
        if let Some(pose) = world.pose(entity) {
            tx.refresh_from_pose(&pose);
        }
    }

    let transaction = match state.transform_transaction.take() {
        Some(tx) => tx,
        None => {
            debug!("no transform transaction active; synthesizing from pose");
            let pose = world.pose(entity)?;
            let mut tx = TransformTransaction::begin(entity, transaction_operation, &pose);
            tx.refresh_from_pose(&pose);
            tx
        }
    };

    let metadata = build_metadata_from_transaction(&transaction, gizmo_kind, constraint);

    undo_stack.push_transaction(transaction);

    match &metadata {
        Some(meta) => {
            info!(
                target: "aw_editor::gizmo",
                entity = meta.entity,
                operation = ?meta.operation,
                measurement = ?meta.measurement,
                "gizmo_transaction_committed"
            );
        }
        None => {
            debug!(target: "aw_editor::gizmo", "commit skipped – no delta detected");
        }
    }

    state.start_transform = None;
    state.transform_transaction = None;
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
    let span = info_span!(
        "aw_editor.gizmo.cancel",
        entity = entity as u32,
        mode = ?mode,
        operation = ?operation
    );
    let _guard = span.enter();

    if let Some(transaction) = state.transform_transaction.take() {
        if let Err(err) = transaction.revert(world) {
            debug!("failed to revert transaction while cancelling: {err:?}");
        }
    } else if let Some(pose) = world.pose_mut(entity) {
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
    info!(
        target: "aw_editor::gizmo",
        entity,
        operation = ?operation,
        "gizmo_transaction_cancelled"
    );
    Some(GizmoCancelMetadata {
        entity,
        operation,
        snapshot,
    })
}

/// Ensure the gizmo has a starting snapshot sourced from the World.
pub fn ensure_world_snapshot(state: &mut GizmoState, world: &World) -> Option<TransformSnapshot> {
    if let Some(snapshot) = state.start_transform {
        return Some(snapshot);
    }
    let entity = state.selected_entity?;
    let pose = world.pose(entity)?;
    if state.transform_transaction.is_none() {
        if let Some(op) = mode_to_operation(state.mode) {
            state.transform_transaction = Some(TransformTransaction::begin(entity, op, &pose));
        }
    }
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

    if let Some(operation) = mode_to_kind(active_or_last_mode(state)) {
        let span = info_span!(
            "aw_editor.gizmo.start",
            entity = entity as u32,
            mode = ?state.mode,
            operation = ?operation
        );
        let _guard = span.enter();
        info!(
            target: "aw_editor::gizmo",
            entity,
            operation = ?operation,
            "gizmo_transaction_started"
        );
    }

    state.start_transform
}

/// Keep the active transaction's pending pose in sync with the world.
pub fn refresh_transaction_state(state: &mut GizmoState, world: &World) {
    let Some(entity) = state.selected_entity else {
        return;
    };
    let Some(transaction) = state.transform_transaction.as_mut() else {
        return;
    };
    if let Some(pose) = world.pose(entity) {
        transaction.refresh_from_pose(&pose);
    }
}

fn build_metadata_from_transaction(
    transaction: &TransformTransaction,
    kind: GizmoOperationKind,
    constraint: Option<String>,
) -> Option<GizmoCommitMetadata> {
    let start = transaction.start_pose();
    let end = transaction.pending_pose();
    match transaction.operation() {
        TransformOperation::Translate => {
            if start.translation() == end.translation() {
                None
            } else {
                let from = start.translation();
                let to = end.translation();
                Some(GizmoCommitMetadata {
                    entity: transaction.entity(),
                    operation: kind,
                    measurement: GizmoMeasurement::Translate { from, to },
                    constraint,
                })
            }
        }
        TransformOperation::Rotate => {
            let (start_x, start_y, start_z) = start.rotation();
            let (end_x, end_y, end_z) = end.rotation();
            let changed = (start_x - end_x).abs() > 0.01
                || (start_y - end_y).abs() > 0.01
                || (start_z - end_z).abs() > 0.01;
            if changed {
                Some(GizmoCommitMetadata {
                    entity: transaction.entity(),
                    operation: kind,
                    measurement: GizmoMeasurement::Rotate {
                        from: (start_x, start_y, start_z),
                        to: (end_x, end_y, end_z),
                    },
                    constraint,
                })
            } else {
                None
            }
        }
        TransformOperation::Scale => {
            let start_scale = start.scale_uniform();
            let end_scale = end.scale_uniform();
            if (start_scale - end_scale).abs() > 0.01 {
                Some(GizmoCommitMetadata {
                    entity: transaction.entity(),
                    operation: kind,
                    measurement: GizmoMeasurement::Scale {
                        from: start_scale,
                        to: end_scale,
                    },
                    constraint: None,
                })
            } else {
                None
            }
        }
    }
}
