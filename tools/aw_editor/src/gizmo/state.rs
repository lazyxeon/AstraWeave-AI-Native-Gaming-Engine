//! Gizmo state machine implementation.
//!
//! Manages modal state transitions (Inactive → Translate → Rotate → Scale)
//! and constraint application (None → X → Y → Z → XY → XZ → YZ).

use crate::command::TransformTransaction;
use glam::{Quat, Vec2, Vec3};
use winit::keyboard::KeyCode;

/// Gizmo operation mode (modal, like Blender).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    /// No active transform.
    Inactive,
    /// Translation mode (G key).
    Translate { constraint: AxisConstraint },
    /// Rotation mode (R key).
    Rotate { constraint: AxisConstraint },
    /// Scale mode (S key).
    Scale {
        constraint: AxisConstraint,
        uniform: bool,
    },
}

/// Axis constraint for transform operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisConstraint {
    /// Free movement in all axes.
    None,
    /// Lock to X axis only.
    X,
    /// Lock to Y axis only.
    Y,
    /// Lock to Z axis only.
    Z,
    /// Planar movement (XY plane, Z locked).
    XY,
    /// Planar movement (XZ plane, Y locked).
    XZ,
    /// Planar movement (YZ plane, X locked).
    YZ,
}

impl AxisConstraint {
    /// Get the constraint axis vector(s).
    pub fn axis_vector(self) -> Vec3 {
        match self {
            AxisConstraint::None => Vec3::ONE,
            AxisConstraint::X => Vec3::X,
            AxisConstraint::Y => Vec3::Y,
            AxisConstraint::Z => Vec3::Z,
            AxisConstraint::XY => Vec3::new(1.0, 1.0, 0.0),
            AxisConstraint::XZ => Vec3::new(1.0, 0.0, 1.0),
            AxisConstraint::YZ => Vec3::new(0.0, 1.0, 1.0),
        }
    }

    /// Check if this is a planar constraint.
    pub fn is_planar(self) -> bool {
        matches!(
            self,
            AxisConstraint::XY | AxisConstraint::XZ | AxisConstraint::YZ
        )
    }

    /// Check if this is a single-axis constraint.
    pub fn is_single_axis(self) -> bool {
        matches!(
            self,
            AxisConstraint::X | AxisConstraint::Y | AxisConstraint::Z
        )
    }

    /// Cycle constraint: None → X → XX (plane) → None pattern.
    pub fn cycle(self, pressed_axis: AxisConstraint) -> Self {
        match (self, pressed_axis) {
            // First press: single axis
            (AxisConstraint::None, AxisConstraint::X) => AxisConstraint::X,
            (AxisConstraint::None, AxisConstraint::Y) => AxisConstraint::Y,
            (AxisConstraint::None, AxisConstraint::Z) => AxisConstraint::Z,

            // Second press: planar (exclude axis)
            (AxisConstraint::X, AxisConstraint::X) => AxisConstraint::YZ,
            (AxisConstraint::Y, AxisConstraint::Y) => AxisConstraint::XZ,
            (AxisConstraint::Z, AxisConstraint::Z) => AxisConstraint::XY,

            // Third press or different axis: back to that axis
            (AxisConstraint::YZ, AxisConstraint::X) => AxisConstraint::None,
            (AxisConstraint::XZ, AxisConstraint::Y) => AxisConstraint::None,
            (AxisConstraint::XY, AxisConstraint::Z) => AxisConstraint::None,

            // Different axis pressed: switch to that axis
            (_, new_axis) => new_axis,
        }
    }
}

/// Transform snapshot for undo/redo.
#[derive(Debug, Clone, Copy)]
pub struct TransformSnapshot {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for TransformSnapshot {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// Main gizmo state machine.
#[derive(Debug, Clone)]
pub struct GizmoState {
    /// Current operation mode.
    pub mode: GizmoMode,

    /// Last non-inactive operation mode (used for telemetry).
    pub last_operation: GizmoMode,

    /// Selected entity ID (if any).
    pub selected_entity: Option<u32>,

    /// Transform before operation started (for undo/cancel).
    pub start_transform: Option<TransformSnapshot>,

    /// Active transform transaction used for undo/redo aggregation.
    pub transform_transaction: Option<TransformTransaction>,

    /// Mouse position when operation started.
    pub start_mouse: Option<Vec2>,

    /// Current mouse position.
    pub current_mouse: Option<Vec2>,

    /// Numeric input buffer (e.g., user types "5.2").
    pub numeric_buffer: String,

    /// Whether transform has been confirmed.
    pub confirmed: bool,

    /// Whether transform has been cancelled.
    pub cancelled: bool,

    /// World space vs local space toggle.
    pub local_space: bool,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            mode: GizmoMode::Inactive,
            last_operation: GizmoMode::Inactive,
            selected_entity: None,
            start_transform: None,
            transform_transaction: None,
            start_mouse: None,
            current_mouse: None,
            numeric_buffer: String::new(),
            confirmed: false,
            cancelled: false,
            local_space: false,
        }
    }
}

impl GizmoState {
    /// Create a new gizmo state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a translate operation.
    pub fn start_translate(&mut self) {
        if self.selected_entity.is_some() {
            let mode = GizmoMode::Translate {
                constraint: AxisConstraint::None,
            };
            self.mode = mode;
            self.last_operation = mode;
            self.reset_operation_state();
        }
    }

    /// Start a rotate operation.
    pub fn start_rotate(&mut self) {
        if self.selected_entity.is_some() {
            let mode = GizmoMode::Rotate {
                constraint: AxisConstraint::None,
            };
            self.mode = mode;
            self.last_operation = mode;
            self.reset_operation_state();
            println!("🔄 Rotate mode started - constraint reset to None");
        }
    }

    /// Start a scale operation.
    pub fn start_scale(&mut self, uniform: bool) {
        if self.selected_entity.is_some() {
            let mode = GizmoMode::Scale {
                constraint: AxisConstraint::None,
                uniform,
            };
            self.mode = mode;
            self.last_operation = mode;
            self.reset_operation_state();
        }
    }

    /// Add or cycle axis constraint.
    pub fn add_constraint(&mut self, axis: AxisConstraint) {
        match &mut self.mode {
            GizmoMode::Translate { constraint } => {
                let old = *constraint;
                *constraint = constraint.cycle(axis);
                println!("🎯 Translate constraint: {:?} → {:?}", old, *constraint);
            }
            GizmoMode::Rotate { constraint } => {
                let old = *constraint;
                *constraint = constraint.cycle(axis);
                println!("🎯 Rotate constraint: {:?} → {:?}", old, *constraint);
            }
            GizmoMode::Scale { constraint, .. } => {
                let old = *constraint;
                *constraint = constraint.cycle(axis);
                println!("🎯 Scale constraint: {:?} → {:?}", old, *constraint);
            }
            GizmoMode::Inactive => {}
        }
    }

    /// Confirm the current transform.
    pub fn confirm_transform(&mut self) {
        if self.mode != GizmoMode::Inactive {
            self.confirmed = true;
            self.last_operation = self.mode;
            self.mode = GizmoMode::Inactive;
            self.numeric_buffer.clear();
        }
    }

    /// Cancel the current transform (revert to start).
    pub fn cancel_transform(&mut self) {
        if self.mode != GizmoMode::Inactive {
            self.cancelled = true;
            self.last_operation = self.mode;
            self.mode = GizmoMode::Inactive;
            self.numeric_buffer.clear();
        }
    }

    /// Handle keyboard input.
    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::KeyG => self.start_translate(),
            KeyCode::KeyR => self.start_rotate(),
            KeyCode::KeyS => {
                // Toggle scale mode on/off
                if matches!(self.mode, GizmoMode::Scale { .. }) {
                    // Already in scale mode - exit it
                    self.mode = GizmoMode::Inactive;
                    self.numeric_buffer.clear();
                } else {
                    // Enter scale mode
                    self.start_scale(false);
                }
            }
            KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                // Shift+S = uniform scale
                if matches!(self.mode, GizmoMode::Scale { .. }) {
                    if let GizmoMode::Scale { constraint, .. } = &mut self.mode {
                        self.mode = GizmoMode::Scale {
                            constraint: *constraint,
                            uniform: true,
                        };
                    }
                }
            }
            KeyCode::KeyX => self.add_constraint(AxisConstraint::X),
            KeyCode::KeyY => self.add_constraint(AxisConstraint::Y),
            KeyCode::KeyZ => self.add_constraint(AxisConstraint::Z),
            KeyCode::Escape => self.cancel_transform(),
            KeyCode::Enter => self.confirm_transform(),
            KeyCode::Minus => self.numeric_buffer.push('-'),
            KeyCode::Period => self.numeric_buffer.push('.'),
            KeyCode::Digit0 => self.numeric_buffer.push('0'),
            KeyCode::Digit1 => self.numeric_buffer.push('1'),
            KeyCode::Digit2 => self.numeric_buffer.push('2'),
            KeyCode::Digit3 => self.numeric_buffer.push('3'),
            KeyCode::Digit4 => self.numeric_buffer.push('4'),
            KeyCode::Digit5 => self.numeric_buffer.push('5'),
            KeyCode::Digit6 => self.numeric_buffer.push('6'),
            KeyCode::Digit7 => self.numeric_buffer.push('7'),
            KeyCode::Digit8 => self.numeric_buffer.push('8'),
            KeyCode::Digit9 => self.numeric_buffer.push('9'),
            KeyCode::Backspace => {
                self.numeric_buffer.pop();
            }
            _ => {}
        }
    }

    /// Update mouse position.
    pub fn update_mouse(&mut self, pos: Vec2) {
        if self.start_mouse.is_none() && self.mode != GizmoMode::Inactive {
            self.start_mouse = Some(pos);
        }
        self.current_mouse = Some(pos);
    }

    /// Get mouse delta since operation started.
    pub fn mouse_delta(&self) -> Vec2 {
        match (self.start_mouse, self.current_mouse) {
            (Some(start), Some(current)) => current - start,
            _ => Vec2::ZERO,
        }
    }

    /// Parse numeric input buffer as f32.
    pub fn parse_numeric_input(&self) -> Option<f32> {
        if self.numeric_buffer.is_empty() {
            None
        } else {
            self.numeric_buffer.parse::<f32>().ok()
        }
    }

    /// Check if operation is active.
    pub fn is_active(&self) -> bool {
        self.mode != GizmoMode::Inactive
    }

    /// Reset operation state (for new operation).
    fn reset_operation_state(&mut self) {
        self.start_mouse = None;
        self.current_mouse = None;
        self.numeric_buffer.clear();
        self.confirmed = false;
        self.cancelled = false;
    }

    /// Get constraint description for UI display.
    pub fn constraint_text(&self) -> String {
        let constraint = match self.mode {
            GizmoMode::Translate { constraint } => constraint,
            GizmoMode::Rotate { constraint } => constraint,
            GizmoMode::Scale { constraint, .. } => constraint,
            GizmoMode::Inactive => return String::new(),
        };

        match constraint {
            AxisConstraint::None => String::new(),
            AxisConstraint::X => "X".to_string(),
            AxisConstraint::Y => "Y".to_string(),
            AxisConstraint::Z => "Z".to_string(),
            AxisConstraint::XY => "XY".to_string(),
            AxisConstraint::XZ => "XZ".to_string(),
            AxisConstraint::YZ => "YZ".to_string(),
        }
    }

    /// Get mode description for UI display.
    pub fn mode_text(&self) -> String {
        match self.mode {
            GizmoMode::Inactive => String::new(),
            GizmoMode::Translate { .. } => "Translate".to_string(),
            GizmoMode::Rotate { .. } => "Rotate".to_string(),
            GizmoMode::Scale { uniform, .. } => {
                if uniform {
                    "Scale (Uniform)".to_string()
                } else {
                    "Scale".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gizmo_state_default() {
        let state = GizmoState::default();
        assert_eq!(state.mode, GizmoMode::Inactive);
        assert!(state.selected_entity.is_none());
        assert!(!state.is_active());
    }

    #[test]
    fn test_start_translate() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_translate();

        assert!(matches!(state.mode, GizmoMode::Translate { .. }));
        assert!(state.is_active());
    }

    #[test]
    fn test_start_rotate() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_rotate();

        assert!(matches!(state.mode, GizmoMode::Rotate { .. }));
        assert!(state.is_active());
    }

    #[test]
    fn test_start_scale() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_scale(false);

        assert!(matches!(
            state.mode,
            GizmoMode::Scale { uniform: false, .. }
        ));
        assert!(state.is_active());
    }

    #[test]
    fn test_constraint_cycle_x() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_translate();

        // First X press: X axis
        state.add_constraint(AxisConstraint::X);
        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::X);
        } else {
            panic!("Expected Translate mode");
        }

        // Second X press: YZ plane
        state.add_constraint(AxisConstraint::X);
        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::YZ);
        } else {
            panic!("Expected Translate mode");
        }

        // Third X press: None
        state.add_constraint(AxisConstraint::X);
        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::None);
        } else {
            panic!("Expected Translate mode");
        }
    }

    #[test]
    fn test_constraint_switch_axis() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_translate();

        // X → Y should switch to Y
        state.add_constraint(AxisConstraint::X);
        state.add_constraint(AxisConstraint::Y);

        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::Y);
        } else {
            panic!("Expected Translate mode");
        }
    }

    #[test]
    fn test_keyboard_handling() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);

        // G key starts translate
        state.handle_key(KeyCode::KeyG);
        assert!(matches!(state.mode, GizmoMode::Translate { .. }));

        // X key adds constraint
        state.handle_key(KeyCode::KeyX);
        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::X);
        }

        // Escape cancels
        state.handle_key(KeyCode::Escape);
        assert_eq!(state.mode, GizmoMode::Inactive);
        assert!(state.cancelled);
    }

    #[test]
    fn test_numeric_input() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_translate();

        // Type "5.2"
        state.handle_key(KeyCode::Digit5);
        state.handle_key(KeyCode::Period);
        state.handle_key(KeyCode::Digit2);

        assert_eq!(state.numeric_buffer, "5.2");
        assert_eq!(state.parse_numeric_input(), Some(5.2));
    }

    #[test]
    fn test_numeric_input_negative() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_translate();

        // Type "-3.5"
        state.handle_key(KeyCode::Minus);
        state.handle_key(KeyCode::Digit3);
        state.handle_key(KeyCode::Period);
        state.handle_key(KeyCode::Digit5);

        assert_eq!(state.numeric_buffer, "-3.5");
        assert_eq!(state.parse_numeric_input(), Some(-3.5));
    }

    #[test]
    fn test_backspace_numeric_input() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_translate();

        state.handle_key(KeyCode::Digit5);
        state.handle_key(KeyCode::Digit2);
        assert_eq!(state.numeric_buffer, "52");

        state.handle_key(KeyCode::Backspace);
        assert_eq!(state.numeric_buffer, "5");
    }

    #[test]
    fn test_mouse_delta() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_translate();

        state.update_mouse(Vec2::new(100.0, 100.0));
        state.update_mouse(Vec2::new(150.0, 120.0));

        let delta = state.mouse_delta();
        assert_eq!(delta, Vec2::new(50.0, 20.0));
    }

    #[test]
    fn test_confirm_clears_numeric_buffer() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_translate();
        state.handle_key(KeyCode::Digit5);

        assert_eq!(state.numeric_buffer, "5");

        state.confirm_transform();
        assert_eq!(state.mode, GizmoMode::Inactive);
        assert!(state.numeric_buffer.is_empty());
        assert!(state.confirmed);
    }

    #[test]
    fn test_axis_constraint_vectors() {
        assert_eq!(AxisConstraint::X.axis_vector(), Vec3::X);
        assert_eq!(AxisConstraint::Y.axis_vector(), Vec3::Y);
        assert_eq!(AxisConstraint::Z.axis_vector(), Vec3::Z);
        assert_eq!(AxisConstraint::XY.axis_vector(), Vec3::new(1.0, 1.0, 0.0));
        assert_eq!(AxisConstraint::XZ.axis_vector(), Vec3::new(1.0, 0.0, 1.0));
        assert_eq!(AxisConstraint::YZ.axis_vector(), Vec3::new(0.0, 1.0, 1.0));
    }

    #[test]
    fn test_constraint_types() {
        assert!(AxisConstraint::X.is_single_axis());
        assert!(AxisConstraint::Y.is_single_axis());
        assert!(AxisConstraint::Z.is_single_axis());
        assert!(!AxisConstraint::XY.is_single_axis());

        assert!(AxisConstraint::XY.is_planar());
        assert!(AxisConstraint::XZ.is_planar());
        assert!(AxisConstraint::YZ.is_planar());
        assert!(!AxisConstraint::X.is_planar());
    }

    #[test]
    fn test_mode_text() {
        let mut state = GizmoState::new();
        assert_eq!(state.mode_text(), "");

        state.selected_entity = Some(42);
        state.start_translate();
        assert_eq!(state.mode_text(), "Translate");

        state.start_rotate();
        assert_eq!(state.mode_text(), "Rotate");

        state.start_scale(false);
        assert_eq!(state.mode_text(), "Scale");

        state.start_scale(true);
        assert_eq!(state.mode_text(), "Scale (Uniform)");
    }

    #[test]
    fn test_constraint_text() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(42);
        state.start_translate();

        assert_eq!(state.constraint_text(), "");

        state.add_constraint(AxisConstraint::X);
        assert_eq!(state.constraint_text(), "X");

        state.add_constraint(AxisConstraint::X);
        assert_eq!(state.constraint_text(), "YZ");
    }
}
