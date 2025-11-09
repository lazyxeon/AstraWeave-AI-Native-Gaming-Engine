//! Scene viewport widget for 3D gizmo interaction.
//!
//! Provides a 3D viewport with:
//! - Orbit/pan/zoom camera controls
//! - Gizmo rendering and picking
//! - Keyboard-driven transform workflow (G/R/S, X/Y/Z, Esc/Enter)
//! - Mouse-based handle selection

use glam::{Mat4, Quat, Vec2, Vec3};
use winit::keyboard::KeyCode;

use super::{
    AxisConstraint, GizmoHandle, GizmoMode, GizmoPicker, GizmoRenderParams, GizmoState,
    RotateGizmo, ScaleGizmo, TranslateGizmo,
};

/// Camera controller for orbit/pan/zoom.
#[derive(Debug, Clone)]
pub struct CameraController {
    /// Camera position in world space.
    pub position: Vec3,
    /// Camera target (look-at point).
    pub target: Vec3,
    /// Up vector.
    pub up: Vec3,
    /// Field of view (radians).
    pub fov: f32,
    /// Aspect ratio (width / height).
    pub aspect: f32,
    /// Near plane distance.
    pub near: f32,
    /// Far plane distance.
    pub far: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            position: Vec3::new(5.0, 5.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: std::f32::consts::FRAC_PI_4, // 45 degrees
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl CameraController {
    /// Get view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get projection matrix.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    /// Get combined view-projection matrix.
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Get inverse view-projection matrix (for ray casting).
    pub fn inverse_view_projection_matrix(&self) -> Mat4 {
        self.view_projection_matrix().inverse()
    }

    /// Orbit camera around target.
    ///
    /// # Arguments
    /// - `delta`: Mouse delta in screen space (normalized -1..1)
    /// - `sensitivity`: Orbit sensitivity multiplier
    pub fn orbit(&mut self, delta: Vec2, sensitivity: f32) {
        let offset = self.position - self.target;
        let radius = offset.length();

        // Horizontal rotation (yaw)
        let yaw = -delta.x * sensitivity;
        let yaw_quat = Quat::from_axis_angle(Vec3::Y, yaw);

        // Vertical rotation (pitch)
        let right = offset.cross(self.up).normalize();
        let pitch = -delta.y * sensitivity;
        let pitch_quat = Quat::from_axis_angle(right, pitch);

        // Apply rotations
        let new_offset = pitch_quat * yaw_quat * offset;
        self.position = self.target + new_offset.normalize() * radius;
    }

    /// Pan camera in screen space.
    ///
    /// # Arguments
    /// - `delta`: Mouse delta in screen space (normalized -1..1)
    /// - `sensitivity`: Pan sensitivity multiplier
    pub fn pan(&mut self, delta: Vec2, sensitivity: f32) {
        let offset = self.position - self.target;
        let distance = offset.length();

        // Get camera right and up vectors
        let forward = offset.normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward);

        // Pan in screen space
        let pan_offset =
            right * (-delta.x * sensitivity * distance) + up * (delta.y * sensitivity * distance);

        self.position += pan_offset;
        self.target += pan_offset;
    }

    /// Zoom camera (dolly in/out).
    ///
    /// # Arguments
    /// - `delta`: Scroll delta (positive = zoom in, negative = zoom out)
    /// - `sensitivity`: Zoom sensitivity multiplier
    pub fn zoom(&mut self, delta: f32, sensitivity: f32) {
        let offset = self.position - self.target;
        let distance = offset.length();

        // Zoom by moving along forward vector (positive delta = zoom in = reduce distance)
        let zoom_amount = delta * sensitivity * distance;
        let new_distance = (distance - zoom_amount).max(0.1); // Clamp to prevent inverting

        self.position = self.target + offset.normalize() * new_distance;
    }

    /// Get camera distance from target.
    pub fn distance(&self) -> f32 {
        (self.position - self.target).length()
    }
}

/// Transform state for ECS entity.
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// Get transform matrix.
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

/// Scene viewport widget.
pub struct SceneViewport {
    /// Camera controller.
    pub camera: CameraController,
    /// Gizmo state machine.
    pub gizmo_state: GizmoState,
    /// Gizmo picker.
    pub gizmo_picker: GizmoPicker,
    /// Currently selected entity transform.
    pub selected_transform: Option<Transform>,
    /// Hovered gizmo handle.
    pub hovered_handle: Option<GizmoHandle>,
    /// Active drag state.
    pub is_dragging: bool,
    /// Mouse position in screen space (normalized -1..1).
    pub mouse_pos: Vec2,
    /// Previous mouse position (for delta calculation).
    pub prev_mouse_pos: Vec2,
    /// Snapping configuration.
    pub snapping_config: super::SnappingConfig,
}

impl Default for SceneViewport {
    fn default() -> Self {
        Self {
            camera: CameraController::default(),
            gizmo_state: GizmoState::new(),
            gizmo_picker: GizmoPicker::default(),
            selected_transform: None,
            hovered_handle: None,
            is_dragging: false,
            mouse_pos: Vec2::ZERO,
            prev_mouse_pos: Vec2::ZERO,
            snapping_config: super::SnappingConfig::default(),
        }
    }
}

impl SceneViewport {
    /// Create new scene viewport.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update camera aspect ratio.
    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) {
        self.camera.aspect = width / height;
    }

    /// Handle keyboard input.
    pub fn handle_key(&mut self, key: KeyCode) {
        self.gizmo_state.handle_key(key);
    }

    /// Update mouse position (normalized -1..1).
    pub fn update_mouse(&mut self, pos: Vec2) {
        self.prev_mouse_pos = self.mouse_pos;
        self.mouse_pos = pos;
        self.gizmo_state.update_mouse(pos);
    }

    /// Get mouse delta.
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_pos - self.prev_mouse_pos
    }

    /// Handle mouse click (start gizmo drag).
    pub fn handle_mouse_down(&mut self) {
        if let Some(handle) = self.hovered_handle {
            // Start gizmo drag
            self.is_dragging = true;

            // Update gizmo mode from handle
            let mode = handle.mode();
            match mode {
                GizmoMode::Translate { .. } => self.gizmo_state.start_translate(),
                GizmoMode::Rotate { .. } => self.gizmo_state.start_rotate(),
                GizmoMode::Scale { .. } => self.gizmo_state.start_scale(false),
                GizmoMode::Inactive => {} // Shouldn't happen
            }

            // Apply constraint from handle
            let constraint = handle.to_constraint();
            if !matches!(constraint, AxisConstraint::None) {
                self.gizmo_state.add_constraint(constraint);
            }
        }
    }

    /// Handle mouse release (end gizmo drag).
    pub fn handle_mouse_up(&mut self) {
        if self.is_dragging {
            self.is_dragging = false;
            self.gizmo_state.confirm_transform();
        }
    }

    /// Update hovered handle (raycast against gizmo).
    pub fn update_hover(&mut self) {
        if let Some(transform) = &self.selected_transform {
            let inv_vp = self.camera.inverse_view_projection_matrix();
            let gizmo_pos = transform.position;

            self.hovered_handle = self.gizmo_picker.pick_handle(
                self.mouse_pos,
                inv_vp,
                gizmo_pos,
                self.gizmo_state.mode,
            );
        } else {
            self.hovered_handle = None;
        }
    }

    /// Apply gizmo transform to selected entity.
    pub fn apply_gizmo_transform(&mut self) {
        if !self.gizmo_state.is_active() {
            return;
        }

        let Some(transform) = &mut self.selected_transform else {
            return;
        };

        let mouse_delta = self.gizmo_state.mouse_delta();
        let camera_distance = self.camera.distance();

        match self.gizmo_state.mode {
            GizmoMode::Translate { constraint } => {
                // Check for numeric input first
                if let Some(value) = self.gizmo_state.parse_numeric_input() {
                    let translation = TranslateGizmo::calculate_translation_numeric(
                        value,
                        constraint,
                        transform.rotation,
                        false, // local_space
                    );
                    transform.position += translation;
                } else {
                    let translation = TranslateGizmo::calculate_translation(
                        mouse_delta,
                        constraint,
                        camera_distance,
                        transform.rotation,
                        false, // local_space
                    );
                    transform.position += translation;
                    
                    if self.snapping_config.grid_enabled {
                        transform.position = TranslateGizmo::snap_position(transform.position, &self.snapping_config);
                    }
                }
            }
            GizmoMode::Rotate { constraint } => {
                let snap_angle = self.snapping_config.angle_increment;
                if let Some(value) = self.gizmo_state.parse_numeric_input() {
                    let rotation = RotateGizmo::calculate_rotation_numeric(
                        value,
                        constraint,
                        transform.rotation,
                        false, // local_space
                    );
                    transform.rotation = rotation * transform.rotation;
                } else {
                    let rotation = RotateGizmo::calculate_rotation(
                        mouse_delta,
                        constraint,
                        snap_angle,
                        self.snapping_config.angle_enabled,
                        transform.rotation,
                        false, // local_space
                    );
                    transform.rotation = rotation * transform.rotation;
                }
            }
            GizmoMode::Scale {
                constraint,
                uniform,
            } => {
                if let Some(value) = self.gizmo_state.parse_numeric_input() {
                    let scale = ScaleGizmo::calculate_scale_numeric(value, constraint, uniform);
                    transform.scale *= scale;
                } else {
                    let scale = ScaleGizmo::calculate_scale(
                        mouse_delta,
                        constraint,
                        uniform,
                        1.0, // sensitivity
                        transform.rotation,
                        false, // local_space (scale is always local)
                    );
                    transform.scale *= scale;
                }
            }
            GizmoMode::Inactive => {} // No transform to apply
        }
    }

    /// Get gizmo render params for current state.
    pub fn gizmo_render_params(&self) -> Option<GizmoRenderParams> {
        let transform = self.selected_transform.as_ref()?;

        let constraint = match self.gizmo_state.mode {
            GizmoMode::Translate { constraint } => constraint,
            GizmoMode::Rotate { constraint } => constraint,
            GizmoMode::Scale { constraint, .. } => constraint,
            GizmoMode::Inactive => AxisConstraint::None,
        };

        let hovered_axis = self.hovered_handle.map(|h| h.to_constraint());

        Some(GizmoRenderParams {
            position: transform.position,
            rotation: transform.rotation,
            scale: 1.0, // Gizmo scale (not entity scale)
            camera_pos: self.camera.position,
            view_proj: self.camera.view_projection_matrix(),
            mode: self.gizmo_state.mode,
            constraint,
            hovered_axis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    // Camera Controller Tests

    #[test]
    fn test_camera_default() {
        let camera = CameraController::default();
        assert_eq!(camera.position, Vec3::new(5.0, 5.0, 5.0));
        assert_eq!(camera.target, Vec3::ZERO);
        assert_eq!(camera.up, Vec3::Y);
    }

    #[test]
    fn test_camera_view_matrix() {
        let camera = CameraController::default();
        let view = camera.view_matrix();

        // View matrix should transform camera position to origin
        let cam_in_view = view.transform_point3(camera.position);
        assert_relative_eq!(cam_in_view.z, 0.0, epsilon = 0.01);
    }

    #[test]
    fn test_camera_orbit() {
        let mut camera = CameraController::default();
        let initial_distance = camera.distance();

        camera.orbit(Vec2::new(0.1, 0.0), 1.0);

        // Distance should remain constant
        assert_relative_eq!(camera.distance(), initial_distance, epsilon = 0.01);

        // Position should have changed
        assert_ne!(camera.position, Vec3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_camera_pan() {
        let mut camera = CameraController::default();
        let initial_distance = camera.distance();

        camera.pan(Vec2::new(0.1, 0.1), 1.0);

        // Distance should remain constant
        assert_relative_eq!(camera.distance(), initial_distance, epsilon = 0.01);

        // Both position and target should have moved
        assert_ne!(camera.position, Vec3::new(5.0, 5.0, 5.0));
        assert_ne!(camera.target, Vec3::ZERO);
    }

    #[test]
    fn test_camera_zoom_in() {
        let mut camera = CameraController::default();
        let initial_distance = camera.distance();

        camera.zoom(1.0, 0.1); // Positive = zoom in

        // Distance should decrease
        assert!(camera.distance() < initial_distance);
    }

    #[test]
    fn test_camera_zoom_out() {
        let mut camera = CameraController::default();
        let initial_distance = camera.distance();

        camera.zoom(-1.0, 0.1); // Negative = zoom out

        // Distance should increase
        assert!(camera.distance() > initial_distance);
    }

    #[test]
    fn test_camera_zoom_clamp() {
        let mut camera = CameraController::default();
        let initial_distance = camera.distance();

        // Zoom in extremely far (positive delta should decrease distance)
        camera.zoom(1000.0, 1.0);

        // Should clamp to minimum distance (0.1, with floating point tolerance)
        let final_distance = camera.distance();
        assert!(
            final_distance >= 0.099, // Account for floating point precision
            "Expected distance >= 0.099, got {}. Initial was {}",
            final_distance,
            initial_distance
        );
        assert!(
            final_distance <= 0.11, // Upper bound
            "Expected distance to clamp near 0.1, got {}",
            final_distance
        );
    }

    // Transform Tests

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_matrix() {
        let transform = Transform {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        let matrix = transform.matrix();
        let transformed = matrix.transform_point3(Vec3::ZERO);

        // Compare components individually
        assert_relative_eq!(transformed.x, transform.position.x, epsilon = 0.01);
        assert_relative_eq!(transformed.y, transform.position.y, epsilon = 0.01);
        assert_relative_eq!(transformed.z, transform.position.z, epsilon = 0.01);
    }

    // SceneViewport Tests

    #[test]
    fn test_viewport_default() {
        let viewport = SceneViewport::default();
        assert!(viewport.selected_transform.is_none());
        assert!(viewport.hovered_handle.is_none());
        assert!(!viewport.is_dragging);
    }

    #[test]
    fn test_viewport_mouse_delta() {
        let mut viewport = SceneViewport::new();
        viewport.update_mouse(Vec2::new(0.5, 0.5));
        viewport.update_mouse(Vec2::new(0.6, 0.7));

        let delta = viewport.mouse_delta();
        assert_relative_eq!(delta.x, 0.1, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.2, epsilon = 0.001);
    }

    #[test]
    fn test_viewport_handle_mouse_down_no_hover() {
        let mut viewport = SceneViewport::new();
        viewport.hovered_handle = None;

        viewport.handle_mouse_down();

        // Should not start dragging if no handle is hovered
        assert!(!viewport.is_dragging);
    }

    #[test]
    fn test_viewport_handle_mouse_down_with_hover() {
        let mut viewport = SceneViewport::new();
        viewport.hovered_handle = Some(GizmoHandle::TranslateX);

        viewport.handle_mouse_down();

        // Should start dragging
        assert!(viewport.is_dragging);
    }

    #[test]
    fn test_viewport_handle_mouse_up() {
        let mut viewport = SceneViewport::new();
        viewport.is_dragging = true;
        viewport.gizmo_state.start_translate();

        viewport.handle_mouse_up();

        // Should stop dragging
        assert!(!viewport.is_dragging);
    }
}
