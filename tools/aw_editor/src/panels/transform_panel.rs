// tools/aw_editor/src/panels/transform_panel.rs - Transform gizmo panel

use super::Panel;
use crate::gizmo::{
    scene_viewport::{CameraController, Transform},
    state::{AxisConstraint, GizmoMode, GizmoState, TransformSnapshot},
};
use egui::{Color32, RichText, Ui};
use glam::{Quat, Vec3};
use std::collections::HashMap;

/// Scene entity (simple representation for MVP)
#[derive(Debug, Clone)]
pub struct SceneEntity {
    pub id: u32,
    pub name: String,
    pub transform: Transform,
}

/// Transform panel with gizmo controls and entity list
///
/// Provides:
/// - Entity list (scene hierarchy)
/// - Entity selection
/// - Object transform editing (position, rotation, scale)
/// - Gizmo mode selection (G/R/S)
/// - Axis constraint controls (X/Y/Z)
/// - Numeric input for precise values
/// - Local/world space toggle
/// - Snap settings
pub struct TransformPanel {
    /// Scene entities (for MVP - will connect to ECS later)
    entities: HashMap<u32, SceneEntity>,

    /// Next entity ID
    next_entity_id: u32,

    /// Selected entity ID (if any)
    selected_entity_id: Option<u32>,
    /// Current gizmo state
    gizmo: GizmoState,

    /// Selected object transform (cached from entities)
    selected_transform: Option<Transform>,

    /// Transform snapshot for undo
    snapshot: Option<TransformSnapshot>,

    /// Camera controller for viewport
    camera: CameraController,

    /// Local space mode
    local_space: bool,

    /// Snap enabled
    snap_enabled: bool,

    /// Numeric input buffer
    numeric_buffer: String,
}

impl Default for TransformPanel {
    fn default() -> Self {
        let mut entities = HashMap::new();
        let mut next_id = 0;

        // Create some sample entities for MVP
        for i in 0..5 {
            let entity = SceneEntity {
                id: next_id,
                name: format!("Entity_{}", i),
                transform: Transform {
                    position: Vec3::new(i as f32 * 2.0, 0.0, 0.0),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            };
            entities.insert(next_id, entity);
            next_id += 1;
        }

        Self {
            entities,
            next_entity_id: next_id,
            selected_entity_id: None,
            gizmo: GizmoState::new(),
            selected_transform: None,
            snapshot: None,
            camera: CameraController::default(),
            local_space: false,
            snap_enabled: false,
            numeric_buffer: String::new(),
        }
    }
}

impl TransformPanel {
    /// Create new transform panel
    pub fn new() -> Self {
        Self::default()
    }

    /// Add new entity to scene
    pub fn add_entity(&mut self, name: String, transform: Transform) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;

        let entity = SceneEntity {
            id,
            name,
            transform,
        };

        self.entities.insert(id, entity);
        id
    }

    /// Remove entity from scene
    pub fn remove_entity(&mut self, id: u32) {
        self.entities.remove(&id);
        if self.selected_entity_id == Some(id) {
            self.clear_selection();
        }
    }

    /// Select entity by ID
    pub fn select_entity(&mut self, id: u32) {
        if let Some(entity) = self.entities.get(&id) {
            self.selected_entity_id = Some(id);
            self.selected_transform = Some(entity.transform.clone());
            self.gizmo.selected_entity = Some(id);
        }
    }

    /// Set selected object transform
    pub fn set_selected(&mut self, transform: Transform) {
        self.selected_transform = Some(transform);
    }

    /// Get current transform (if any)
    pub fn get_transform(&self) -> Option<&Transform> {
        self.selected_transform.as_ref()
    }

    /// Get mutable transform (if any)
    pub fn get_transform_mut(&mut self) -> Option<&mut Transform> {
        self.selected_transform.as_mut()
    }

    /// Clear selected object
    pub fn clear_selection(&mut self) {
        self.selected_transform = None;
        self.gizmo.cancel_transform();
    }

    /// Start translation mode
    pub fn start_translate(&mut self) {
        if self.selected_transform.is_some() {
            self.snapshot = self.selected_transform.as_ref().map(|t| TransformSnapshot {
                position: t.position,
                rotation: t.rotation,
                scale: t.scale,
            });
            self.gizmo.start_translate();
        }
    }

    /// Start rotation mode
    pub fn start_rotate(&mut self) {
        if self.selected_transform.is_some() {
            self.snapshot = self.selected_transform.as_ref().map(|t| TransformSnapshot {
                position: t.position,
                rotation: t.rotation,
                scale: t.scale,
            });
            self.gizmo.start_rotate();
        }
    }

    /// Start scale mode
    pub fn start_scale(&mut self) {
        if self.selected_transform.is_some() {
            self.snapshot = self.selected_transform.as_ref().map(|t| TransformSnapshot {
                position: t.position,
                rotation: t.rotation,
                scale: t.scale,
            });
            self.gizmo.start_scale(false); // Non-uniform by default
        }
    }

    /// Confirm transform (apply changes)
    pub fn confirm_transform(&mut self) {
        // Apply transform to entity if selected
        if let (Some(id), Some(transform)) = (self.selected_entity_id, &self.selected_transform) {
            if let Some(entity) = self.entities.get_mut(&id) {
                entity.transform = transform.clone();
            }
        }

        self.gizmo.confirm_transform();
        self.snapshot = None;
    }

    /// Cancel transform (revert to snapshot)
    pub fn cancel_transform(&mut self) {
        if let (Some(transform), Some(snapshot)) =
            (self.selected_transform.as_mut(), self.snapshot.as_ref())
        {
            transform.position = snapshot.position;
            transform.rotation = snapshot.rotation;
            transform.scale = snapshot.scale;
        }
        self.gizmo.cancel_transform();
        self.snapshot = None;
    }

    /// Apply numeric input
    fn apply_numeric_input(&mut self, value: f32) {
        if let Some(transform) = self.selected_transform.as_mut() {
            match &self.gizmo.mode {
                GizmoMode::Translate { constraint } => {
                    let delta = Self::calculate_translation_numeric(
                        value,
                        *constraint,
                        transform.rotation,
                        self.local_space,
                    );
                    transform.position += delta;
                }
                GizmoMode::Rotate { constraint } => {
                    let rotation = Self::calculate_rotation_numeric(
                        value,
                        *constraint,
                        transform.rotation,
                        self.local_space,
                    );
                    transform.rotation = rotation * transform.rotation;
                }
                GizmoMode::Scale {
                    constraint,
                    uniform,
                } => {
                    let scale = Self::calculate_scale_numeric(value, *constraint, *uniform);
                    transform.scale *= scale;
                }
                _ => {}
            }
        }
    }

    // Simplified transform calculations (stub implementations)
    // These would use the full TranslateGizmo, RotateGizmo, ScaleGizmo in production

    fn calculate_translation_numeric(
        value: f32,
        constraint: AxisConstraint,
        _rotation: Quat,
        _local_space: bool,
    ) -> Vec3 {
        match constraint {
            AxisConstraint::X => Vec3::new(value, 0.0, 0.0),
            AxisConstraint::Y => Vec3::new(0.0, value, 0.0),
            AxisConstraint::Z => Vec3::new(0.0, 0.0, value),
            _ => Vec3::ZERO,
        }
    }

    fn calculate_rotation_numeric(
        degrees: f32,
        constraint: AxisConstraint,
        _rotation: Quat,
        _local_space: bool,
    ) -> Quat {
        let radians = degrees.to_radians();
        match constraint {
            AxisConstraint::X => Quat::from_rotation_x(radians),
            AxisConstraint::Y => Quat::from_rotation_y(radians),
            AxisConstraint::Z => Quat::from_rotation_z(radians),
            _ => Quat::IDENTITY,
        }
    }

    fn calculate_scale_numeric(value: f32, constraint: AxisConstraint, uniform: bool) -> Vec3 {
        let clamped = value.clamp(0.01, 100.0);
        if uniform {
            Vec3::splat(clamped)
        } else {
            match constraint {
                AxisConstraint::X => Vec3::new(clamped, 1.0, 1.0),
                AxisConstraint::Y => Vec3::new(1.0, clamped, 1.0),
                AxisConstraint::Z => Vec3::new(1.0, 1.0, clamped),
                _ => Vec3::ONE,
            }
        }
    }
}

impl Panel for TransformPanel {
    fn name(&self) -> &str {
        "Transform"
    }

    fn show(&mut self, ui: &mut Ui) {
        // Entity list section
        ui.label(RichText::new("📐 Scene Entities").strong());
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("➕ Add Entity").clicked() {
                let id = self.add_entity(
                    format!("Entity_{}", self.next_entity_id),
                    Transform::default(),
                );
                self.select_entity(id);
            }

            if ui.button("🗑️ Delete Selected").clicked() {
                if let Some(id) = self.selected_entity_id {
                    self.remove_entity(id);
                }
            }
        });

        ui.add_space(5.0);

        // Entity list
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                let mut entity_ids: Vec<_> = self.entities.keys().copied().collect();
                entity_ids.sort();

                for id in entity_ids {
                    if let Some(entity) = self.entities.get(&id) {
                        let is_selected = self.selected_entity_id == Some(id);
                        let label = if is_selected {
                            RichText::new(&entity.name)
                                .strong()
                                .color(Color32::LIGHT_GREEN)
                        } else {
                            RichText::new(&entity.name)
                        };

                        if ui.selectable_label(is_selected, label).clicked() {
                            self.select_entity(id);
                        }
                    }
                }
            });

        ui.separator();

        // Header with selection status
        if let Some(id) = self.selected_entity_id {
            if let Some(entity) = self.entities.get(&id) {
                ui.label(
                    RichText::new(format!("✏️ Editing: {}", entity.name))
                        .strong()
                        .color(Color32::LIGHT_GREEN),
                );
            }
        } else {
            ui.label(RichText::new("No Selection").color(Color32::GRAY));
            ui.label("Select an entity to edit transforms");
            return;
        }

        ui.separator();

        // Mode selection buttons
        ui.horizontal(|ui| {
            ui.label("Mode:");

            let is_translating = matches!(self.gizmo.mode, GizmoMode::Translate { .. });
            if ui
                .selectable_label(is_translating, "Translate (G)")
                .clicked()
            {
                self.start_translate();
            }

            let is_rotating = matches!(self.gizmo.mode, GizmoMode::Rotate { .. });
            if ui.selectable_label(is_rotating, "Rotate (R)").clicked() {
                self.start_rotate();
            }

            let is_scaling = matches!(self.gizmo.mode, GizmoMode::Scale { .. });
            if ui.selectable_label(is_scaling, "Scale (S)").clicked() {
                self.start_scale();
            }
        });

        // Axis constraints (if active)
        let current_constraint = match &self.gizmo.mode {
            GizmoMode::Translate { constraint } => Some(*constraint),
            GizmoMode::Rotate { constraint } => Some(*constraint),
            GizmoMode::Scale { constraint, .. } => Some(*constraint),
            _ => None,
        };

        if let Some(constraint) = current_constraint {
            ui.horizontal(|ui| {
                ui.label("Axis:");

                if ui
                    .selectable_label(constraint == AxisConstraint::X, "X")
                    .clicked()
                {
                    self.gizmo.add_constraint(AxisConstraint::X);
                }
                if ui
                    .selectable_label(constraint == AxisConstraint::Y, "Y")
                    .clicked()
                {
                    self.gizmo.add_constraint(AxisConstraint::Y);
                }
                if ui
                    .selectable_label(constraint == AxisConstraint::Z, "Z")
                    .clicked()
                {
                    self.gizmo.add_constraint(AxisConstraint::Z);
                }
                if ui
                    .selectable_label(constraint == AxisConstraint::None, "All")
                    .clicked()
                {
                    self.gizmo.add_constraint(AxisConstraint::None);
                }
            });

            // Numeric input
            ui.horizontal(|ui| {
                ui.label("Value:");
                if ui.text_edit_singleline(&mut self.numeric_buffer).changed() {
                    // Validate numeric input
                    if let Ok(value) = self.numeric_buffer.parse::<f32>() {
                        self.apply_numeric_input(value);
                    }
                }
                if ui.button("Apply").clicked() {
                    if let Ok(value) = self.numeric_buffer.parse::<f32>() {
                        self.apply_numeric_input(value);
                        self.numeric_buffer.clear();
                    }
                }
            });

            // Confirm/Cancel buttons
            ui.horizontal(|ui| {
                if ui.button("✓ Confirm (Enter)").clicked() {
                    self.confirm_transform();
                }
                if ui.button("✗ Cancel (Esc)").clicked() {
                    self.cancel_transform();
                }
            });
        }

        ui.separator();

        // Settings
        ui.checkbox(&mut self.local_space, "Local Space");
        ui.checkbox(&mut self.snap_enabled, "Snap (15° for rotation)");

        ui.separator();

        // Transform values (read-only display)
        if let Some(transform) = &self.selected_transform {
            ui.label(RichText::new("Current Transform").strong());

            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.label(format!("X: {:.2}", transform.position.x));
                ui.label(format!("Y: {:.2}", transform.position.y));
                ui.label(format!("Z: {:.2}", transform.position.z));
            });

            // Convert quaternion to euler for display
            let (yaw, pitch, roll) = transform.rotation.to_euler(glam::EulerRot::YXZ);
            ui.horizontal(|ui| {
                ui.label("Rotation:");
                ui.label(format!("Yaw: {:.1}°", yaw.to_degrees()));
                ui.label(format!("Pitch: {:.1}°", pitch.to_degrees()));
                ui.label(format!("Roll: {:.1}°", roll.to_degrees()));
            });

            ui.horizontal(|ui| {
                ui.label("Scale:");
                ui.label(format!("X: {:.2}", transform.scale.x));
                ui.label(format!("Y: {:.2}", transform.scale.y));
                ui.label(format!("Z: {:.2}", transform.scale.z));
            });
        }

        ui.separator();

        // Keyboard shortcuts help
        ui.collapsing("Keyboard Shortcuts", |ui| {
            ui.label("G - Start Translation");
            ui.label("R - Start Rotation");
            ui.label("S - Start Scale");
            ui.label("X/Y/Z - Constrain to axis");
            ui.label("Enter - Confirm transform");
            ui.label("Esc - Cancel transform");
            ui.label("0-9, . - Numeric input");
        });
    }
}
