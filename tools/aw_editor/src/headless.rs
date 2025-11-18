use crate::command::UndoStack;
use crate::gizmo::snapping::SnappingConfig;
use crate::gizmo::state::GizmoState;
use crate::interaction::{self, GizmoCancelMetadata, GizmoOperationKind};
use crate::telemetry::{self, EditorTelemetryEvent};
use anyhow::{Context, Result};
use astraweave_core::{Entity, IVec2, World};

/// Minimal harness that drives gizmo operations without a GPU/egui session.
///
/// This enables deterministic regression tests that mirror the viewport flow
/// (select → start gizmo → drag → confirm/cancel) entirely in headless mode.
pub struct GizmoHarness {
    world: World,
    undo_stack: UndoStack,
    gizmo: GizmoState,
    _snapping: SnappingConfig,
}

impl GizmoHarness {
    pub fn new(world: World) -> Self {
        Self {
            world,
            undo_stack: UndoStack::new(64),
            gizmo: GizmoState::new(),
            _snapping: SnappingConfig::default(),
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn into_world(self) -> World {
        self.world
    }

    pub fn undo_stack(&self) -> &UndoStack {
        &self.undo_stack
    }

    pub fn select(&mut self, entity: Entity) {
        self.gizmo.selected_entity = Some(entity);
        telemetry::record(EditorTelemetryEvent::SelectionChanged {
            primary: Some(entity as u32),
            count: 1,
        });
    }

    fn selected_entity(&self) -> Result<Entity> {
        self.gizmo
            .selected_entity
            .context("no entity selected in harness")
    }

    pub fn begin_translate(&mut self) -> Result<()> {
        let entity = self.selected_entity()?;
        self.gizmo.start_translate();
        interaction::ensure_world_snapshot(&mut self.gizmo, &self.world);
        telemetry::record(EditorTelemetryEvent::GizmoStarted {
            entity: entity as u32,
            operation: GizmoOperationKind::Translate,
        });
        Ok(())
    }

    pub fn drag_translate(&mut self, delta: IVec2) -> Result<()> {
        let entity = self.selected_entity()?;
        interaction::ensure_world_snapshot(&mut self.gizmo, &self.world)
            .context("failed to capture start snapshot for drag")?;

        if let Some(pose) = self.world.pose_mut(entity) {
            pose.pos.x += delta.x;
            pose.pos.y += delta.y;
        }
        Ok(())
    }

    pub fn confirm(&mut self) -> Result<()> {
        self.gizmo.confirm_transform();
        if let Some(metadata) =
            interaction::commit_active_gizmo(&mut self.gizmo, &mut self.world, &mut self.undo_stack)
        {
            telemetry::record(EditorTelemetryEvent::from(metadata));
        }
        self.gizmo.confirmed = false;
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<()> {
        self.gizmo.cancel_transform();
        if let Some(GizmoCancelMetadata {
            entity, operation, ..
        }) = interaction::cancel_active_gizmo(&mut self.gizmo, &mut self.world)
        {
            telemetry::record(EditorTelemetryEvent::GizmoCancelled {
                entity: entity as u32,
                operation,
            });
        }
        self.gizmo.cancelled = false;
        Ok(())
    }

    pub fn snapping_config_mut(&mut self) -> &mut SnappingConfig {
        &mut self._snapping
    }

    pub fn undo_depth(&self) -> usize {
        if self.undo_stack.can_undo() {
            self.undo_stack.len() - self.undo_stack.cursor()
        } else {
            0
        }
    }

    pub fn undo_last(&mut self) -> Result<(), String> {
        self.undo_stack
            .undo(&mut self.world)
            .map_err(|e| format!("Undo failed: {}", e))
    }

    pub fn redo_last(&mut self) -> Result<(), String> {
        self.undo_stack
            .redo(&mut self.world)
            .map_err(|e| format!("Redo failed: {}", e))
    }
}
