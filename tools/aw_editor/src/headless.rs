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
    snapping: SnappingConfig,
}

impl GizmoHarness {
    pub fn new(world: World) -> Self {
        Self {
            world,
            undo_stack: UndoStack::new(64),
            gizmo: GizmoState::new(),
            snapping: SnappingConfig::default(),
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

    /// Begin a rotation operation on the selected entity
    pub fn begin_rotate(&mut self) -> Result<()> {
        let entity = self.selected_entity()?;
        self.gizmo.start_rotate();
        interaction::ensure_world_snapshot(&mut self.gizmo, &self.world);
        telemetry::record(EditorTelemetryEvent::GizmoStarted {
            entity: entity as u32,
            operation: GizmoOperationKind::Rotate,
        });
        Ok(())
    }

    /// Begin a scale operation on the selected entity
    pub fn begin_scale(&mut self) -> Result<()> {
        let entity = self.selected_entity()?;
        self.gizmo.start_scale(true); // Uniform scaling by default
        interaction::ensure_world_snapshot(&mut self.gizmo, &self.world);
        telemetry::record(EditorTelemetryEvent::GizmoStarted {
            entity: entity as u32,
            operation: GizmoOperationKind::Scale,
        });
        Ok(())
    }

    pub fn drag_translate(&mut self, delta: IVec2) -> Result<()> {
        let entity = self.selected_entity()?;
        interaction::ensure_world_snapshot(&mut self.gizmo, &self.world)
            .context("failed to capture start snapshot for drag")?;

        if let Some(pose) = self.world.pose_mut(entity) {
            // Apply delta to get new position
            let new_x = pose.pos.x + delta.x;
            let new_y = pose.pos.y + delta.y;

            // Apply grid snapping if enabled
            if self.snapping.grid_enabled && self.snapping.grid_size > 0.0 {
                let grid = self.snapping.grid_size;
                pose.pos.x = ((new_x as f32 / grid).round() * grid) as i32;
                pose.pos.y = ((new_y as f32 / grid).round() * grid) as i32;
            } else {
                pose.pos.x = new_x;
                pose.pos.y = new_y;
            }
        }
        Ok(())
    }

    /// Drag rotate by the given angle in radians
    pub fn drag_rotate(&mut self, angle_radians: f32) -> Result<()> {
        let entity = self.selected_entity()?;
        interaction::ensure_world_snapshot(&mut self.gizmo, &self.world)
            .context("failed to capture start snapshot for rotate")?;

        if let Some(pose) = self.world.pose_mut(entity) {
            let new_rotation = pose.rotation + angle_radians;

            // Apply angle snapping if enabled
            pose.rotation = self.snapping.snap_angle(new_rotation);
        }
        Ok(())
    }

    /// Drag scale by the given factor (1.0 = no change, 2.0 = double size)
    pub fn drag_scale(&mut self, factor: f32) -> Result<()> {
        let entity = self.selected_entity()?;
        interaction::ensure_world_snapshot(&mut self.gizmo, &self.world)
            .context("failed to capture start snapshot for scale")?;

        if let Some(pose) = self.world.pose_mut(entity) {
            pose.scale *= factor;
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
        &mut self.snapping
    }

    pub fn undo_depth(&self) -> usize {
        self.undo_stack.cursor()
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
