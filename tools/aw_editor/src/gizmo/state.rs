//! Gizmo state machine implementation.
//!
//! Manages modal state transitions (Inactive → Translate → Rotate → Scale)
//! and constraint application (None → X → Y → Z → XY → XZ → YZ).

use glam::{Quat, Vec2, Vec3};
use tracing::debug;
use winit::keyboard::KeyCode;

/// Gizmo operation mode (modal, like Blender).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl std::fmt::Display for GizmoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GizmoMode::Inactive => write!(f, "Inactive"),
            GizmoMode::Translate { constraint } => write!(f, "Translate ({})", constraint),
            GizmoMode::Rotate { constraint } => write!(f, "Rotate ({})", constraint),
            GizmoMode::Scale { constraint, uniform } => {
                if *uniform {
                    write!(f, "Scale Uniform")
                } else {
                    write!(f, "Scale ({})", constraint)
                }
            }
        }
    }
}

impl Default for GizmoMode {
    fn default() -> Self {
        Self::Inactive
    }
}

impl GizmoMode {
    /// Returns all gizmo mode variants (using default constraints).
    pub fn all() -> &'static [Self] {
        &[
            GizmoMode::Inactive,
            GizmoMode::Translate { constraint: AxisConstraint::None },
            GizmoMode::Rotate { constraint: AxisConstraint::None },
            GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false },
        ]
    }

    /// Returns the display name of this mode.
    pub fn name(&self) -> &'static str {
        match self {
            GizmoMode::Inactive => "Inactive",
            GizmoMode::Translate { .. } => "Translate",
            GizmoMode::Rotate { .. } => "Rotate",
            GizmoMode::Scale { .. } => "Scale",
        }
    }

    /// Returns an icon for this mode.
    pub fn icon(&self) -> &'static str {
        match self {
            GizmoMode::Inactive => "⏸",
            GizmoMode::Translate { .. } => "↔",
            GizmoMode::Rotate { .. } => "↻",
            GizmoMode::Scale { .. } => "⇲",
        }
    }

    /// Returns the keyboard shortcut for this mode.
    pub fn shortcut(&self) -> Option<&'static str> {
        match self {
            GizmoMode::Inactive => None,
            GizmoMode::Translate { .. } => Some("G"),
            GizmoMode::Rotate { .. } => Some("R"),
            GizmoMode::Scale { .. } => Some("S"),
        }
    }

    /// Returns true if this mode is active (not Inactive).
    pub fn is_active(&self) -> bool {
        !matches!(self, GizmoMode::Inactive)
    }

    /// Returns true if this is a translation mode.
    pub fn is_translate(&self) -> bool {
        matches!(self, GizmoMode::Translate { .. })
    }

    /// Returns true if this is a rotation mode.
    pub fn is_rotate(&self) -> bool {
        matches!(self, GizmoMode::Rotate { .. })
    }

    /// Returns true if this is a scale mode.
    pub fn is_scale(&self) -> bool {
        matches!(self, GizmoMode::Scale { .. })
    }

    /// Returns the constraint for this mode, if any.
    pub fn constraint(&self) -> Option<AxisConstraint> {
        match self {
            GizmoMode::Inactive => None,
            GizmoMode::Translate { constraint } => Some(*constraint),
            GizmoMode::Rotate { constraint } => Some(*constraint),
            GizmoMode::Scale { constraint, .. } => Some(*constraint),
        }
    }
}

/// Axis constraint for transform operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AxisConstraint {
    /// Free movement in all axes.
    #[default]
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

impl std::fmt::Display for AxisConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AxisConstraint::None => write!(f, "Free"),
            AxisConstraint::X => write!(f, "X"),
            AxisConstraint::Y => write!(f, "Y"),
            AxisConstraint::Z => write!(f, "Z"),
            AxisConstraint::XY => write!(f, "XY Plane"),
            AxisConstraint::XZ => write!(f, "XZ Plane"),
            AxisConstraint::YZ => write!(f, "YZ Plane"),
        }
    }
}

impl AxisConstraint {
    /// Returns all axis constraint variants.
    pub fn all() -> &'static [Self] {
        &[
            AxisConstraint::None,
            AxisConstraint::X,
            AxisConstraint::Y,
            AxisConstraint::Z,
            AxisConstraint::XY,
            AxisConstraint::XZ,
            AxisConstraint::YZ,
        ]
    }

    /// Returns the display name of this constraint.
    pub fn name(&self) -> &'static str {
        match self {
            AxisConstraint::None => "Free",
            AxisConstraint::X => "X Axis",
            AxisConstraint::Y => "Y Axis",
            AxisConstraint::Z => "Z Axis",
            AxisConstraint::XY => "XY Plane",
            AxisConstraint::XZ => "XZ Plane",
            AxisConstraint::YZ => "YZ Plane",
        }
    }

    /// Returns the keyboard key for this constraint.
    pub fn key(&self) -> Option<&'static str> {
        match self {
            AxisConstraint::None => None,
            AxisConstraint::X => Some("X"),
            AxisConstraint::Y => Some("Y"),
            AxisConstraint::Z => Some("Z"),
            AxisConstraint::XY | AxisConstraint::XZ | AxisConstraint::YZ => None,
        }
    }

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

    /// Returns the color for this constraint (for gizmo rendering).
    pub fn color(&self) -> [f32; 3] {
        match self {
            AxisConstraint::None => [1.0, 1.0, 1.0],  // White
            AxisConstraint::X => [1.0, 0.2, 0.2],     // Red
            AxisConstraint::Y => [0.2, 1.0, 0.2],     // Green
            AxisConstraint::Z => [0.3, 0.3, 1.0],     // Blue
            AxisConstraint::XY => [1.0, 1.0, 0.2],    // Yellow
            AxisConstraint::XZ => [1.0, 0.2, 1.0],    // Magenta
            AxisConstraint::YZ => [0.2, 1.0, 1.0],    // Cyan
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
    /// Last non-inactive mode (needed for deferred commits).
    pub last_mode: GizmoMode,

    /// Selected entity ID (if any).
    pub selected_entity: Option<u32>,

    /// Transform before operation started (for undo/cancel).
    pub start_transform: Option<TransformSnapshot>,

    /// Position when axis constraint was first applied (for locking).
    /// This captures the entity's position at the moment the user presses X/Y/Z,
    /// not the start position of the operation. This ensures that if the user
    /// moves freely and THEN applies a constraint, the locked axis stays at
    /// its current value, not the original position.
    pub constraint_position: Option<Vec3>,

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
            last_mode: GizmoMode::Inactive,
            selected_entity: None,
            start_transform: None,
            constraint_position: None,
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
            self.mode = GizmoMode::Translate {
                constraint: AxisConstraint::None,
            };
            self.last_mode = self.mode;
            self.reset_operation_state();
        }
    }

    /// Start a rotate operation.
    pub fn start_rotate(&mut self) {
        if self.selected_entity.is_some() {
            self.mode = GizmoMode::Rotate {
                constraint: AxisConstraint::None,
            };
            self.last_mode = self.mode;
            self.reset_operation_state();
            debug!("🔄 Rotate mode started - constraint reset to None");
        }
    }

    /// Start a scale operation.
    pub fn start_scale(&mut self, uniform: bool) {
        if self.selected_entity.is_some() {
            self.mode = GizmoMode::Scale {
                constraint: AxisConstraint::None,
                uniform,
            };
            self.last_mode = self.mode;
            self.reset_operation_state();
        }
    }

    /// Add or cycle axis constraint.
    pub fn add_constraint(&mut self, axis: AxisConstraint) {
        match &mut self.mode {
            GizmoMode::Translate { constraint } => {
                let old = *constraint;
                *constraint = constraint.cycle(axis);
                debug!("🎯 Translate constraint: {:?} → {:?}", old, *constraint);
            }
            GizmoMode::Rotate { constraint } => {
                let old = *constraint;
                *constraint = constraint.cycle(axis);
                debug!("🎯 Rotate constraint: {:?} → {:?}", old, *constraint);
            }
            GizmoMode::Scale { constraint, .. } => {
                let old = *constraint;
                *constraint = constraint.cycle(axis);
                debug!("🎯 Scale constraint: {:?} → {:?}", old, *constraint);
            }
            GizmoMode::Inactive => {}
        }
    }

    /// Confirm the current transform.
    pub fn confirm_transform(&mut self) {
        if self.mode != GizmoMode::Inactive {
            self.confirmed = true;
            self.last_mode = self.mode;
            self.mode = GizmoMode::Inactive;
            self.numeric_buffer.clear();
        }
    }

    /// Cancel the current transform (revert to start).
    pub fn cancel_transform(&mut self) {
        if self.mode != GizmoMode::Inactive {
            self.cancelled = true;
            self.last_mode = self.mode;
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
        self.constraint_position = None; // Clear constraint position for fresh operation
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

    #[test]
    fn test_start_transform_requires_selection() {
        let mut state = GizmoState::new();
        state.selected_entity = None; // Explicitly None

        state.start_translate();
        assert_eq!(state.mode, GizmoMode::Inactive);

        state.start_rotate();
        assert_eq!(state.mode, GizmoMode::Inactive);

        state.start_scale(false);
        assert_eq!(state.mode, GizmoMode::Inactive);

        // Now select and verify it works
        state.selected_entity = Some(123);
        state.start_translate();
        assert!(matches!(state.mode, GizmoMode::Translate { .. }));
    }

    #[test]
    fn test_cycle_logic_detailed() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(1);
        state.start_translate();

        // Initial: None
        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::None);
        } else {
            panic!("Wrong mode");
        }

        // Press X -> X constraint
        state.add_constraint(AxisConstraint::X);
        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::X);
        } else {
            panic!("Wrong mode");
        }

        // Press X again -> YZ plane (exclude X)
        state.add_constraint(AxisConstraint::X);
        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::YZ);
        } else {
            panic!("Wrong mode");
        }

        // Press X again -> None (cycle complete)
        state.add_constraint(AxisConstraint::X);
        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::None);
        } else {
            panic!("Wrong mode");
        }

        // Press Y (switch axis)
        state.add_constraint(AxisConstraint::Y);
        if let GizmoMode::Translate { constraint } = state.mode {
            assert_eq!(constraint, AxisConstraint::Y);
        }
    }

    #[test]
    fn test_decimal_input() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(1);
        state.start_translate();

        state.handle_key(KeyCode::Digit1);
        state.handle_key(KeyCode::Period);
        state.handle_key(KeyCode::Digit5);

        assert_eq!(state.parse_numeric_input(), Some(1.5));
    }

    #[test]
    fn test_backspace_on_empty() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(1);
        state.start_translate();

        // Should not panic or error
        state.handle_key(KeyCode::Backspace);
        assert!(state.numeric_buffer.is_empty());

        state.handle_key(KeyCode::Digit1);
        assert_eq!(state.numeric_buffer, "1");
        state.handle_key(KeyCode::Backspace);
        assert!(state.numeric_buffer.is_empty());
    }

    #[test]
    fn test_reset_operation_state() {
        let mut state = GizmoState::new();
        state.selected_entity = Some(1);
        state.start_translate();
        
        state.update_mouse(Vec2::new(10.0, 10.0));
        state.handle_key(KeyCode::Digit5);
        
        // Internal helper check
        state.reset_operation_state();
        
        assert!(state.start_mouse.is_none());
        assert!(state.current_mouse.is_none());
        assert!(state.numeric_buffer.is_empty());
        assert!(state.constraint_position.is_none());
    }

    // ==================== GizmoMode Tests ====================

    #[test]
    fn test_gizmo_mode_display() {
        assert_eq!(GizmoMode::Inactive.to_string(), "Inactive");
        assert!(GizmoMode::Translate { constraint: AxisConstraint::X }.to_string().contains("Translate"));
        assert!(GizmoMode::Rotate { constraint: AxisConstraint::Y }.to_string().contains("Rotate"));
        assert!(GizmoMode::Scale { constraint: AxisConstraint::Z, uniform: false }.to_string().contains("Scale"));
        assert!(GizmoMode::Scale { constraint: AxisConstraint::None, uniform: true }.to_string().contains("Uniform"));
    }

    #[test]
    fn test_gizmo_mode_all_and_name() {
        let modes = GizmoMode::all();
        assert_eq!(modes.len(), 4);
        for mode in modes {
            assert!(!mode.name().is_empty());
            assert!(!mode.icon().is_empty());
        }
    }

    #[test]
    fn test_gizmo_mode_helpers() {
        assert!(!GizmoMode::Inactive.is_active());
        assert!(GizmoMode::Translate { constraint: AxisConstraint::None }.is_active());
        assert!(GizmoMode::Translate { constraint: AxisConstraint::X }.is_translate());
        assert!(GizmoMode::Rotate { constraint: AxisConstraint::Y }.is_rotate());
        assert!(GizmoMode::Scale { constraint: AxisConstraint::Z, uniform: false }.is_scale());
    }

    #[test]
    fn test_gizmo_mode_constraint() {
        assert_eq!(GizmoMode::Inactive.constraint(), None);
        assert_eq!(GizmoMode::Translate { constraint: AxisConstraint::X }.constraint(), Some(AxisConstraint::X));
        assert_eq!(GizmoMode::Rotate { constraint: AxisConstraint::Y }.constraint(), Some(AxisConstraint::Y));
    }

    #[test]
    fn test_gizmo_mode_shortcut() {
        assert_eq!(GizmoMode::Inactive.shortcut(), None);
        assert_eq!(GizmoMode::Translate { constraint: AxisConstraint::None }.shortcut(), Some("G"));
        assert_eq!(GizmoMode::Rotate { constraint: AxisConstraint::None }.shortcut(), Some("R"));
        assert_eq!(GizmoMode::Scale { constraint: AxisConstraint::None, uniform: false }.shortcut(), Some("S"));
    }

    // ==================== AxisConstraint Tests ====================

    #[test]
    fn test_axis_constraint_display() {
        assert_eq!(AxisConstraint::None.to_string(), "Free");
        assert_eq!(AxisConstraint::X.to_string(), "X");
        assert_eq!(AxisConstraint::XY.to_string(), "XY Plane");
    }

    #[test]
    fn test_axis_constraint_all_and_name() {
        let constraints = AxisConstraint::all();
        assert_eq!(constraints.len(), 7);
        for constraint in constraints {
            assert!(!constraint.name().is_empty());
        }
    }

    #[test]
    fn test_axis_constraint_key() {
        assert_eq!(AxisConstraint::None.key(), None);
        assert_eq!(AxisConstraint::X.key(), Some("X"));
        assert_eq!(AxisConstraint::Y.key(), Some("Y"));
        assert_eq!(AxisConstraint::Z.key(), Some("Z"));
        assert_eq!(AxisConstraint::XY.key(), None);
    }

    #[test]
    fn test_axis_constraint_color() {
        // X is red-ish
        let x_color = AxisConstraint::X.color();
        assert!(x_color[0] > 0.5);
        // Y is green-ish
        let y_color = AxisConstraint::Y.color();
        assert!(y_color[1] > 0.5);
        // Z is blue-ish
        let z_color = AxisConstraint::Z.color();
        assert!(z_color[2] > 0.5);
    }

    #[test]
    fn test_axis_constraint_default() {
        assert_eq!(AxisConstraint::default(), AxisConstraint::None);
    }
}
