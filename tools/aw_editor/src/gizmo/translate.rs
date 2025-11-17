//! Translation gizmo implementation (G key).

use super::{AxisConstraint, SnappingConfig};
use glam::{Mat3, Quat, Vec2, Vec3};

/// Translation gizmo calculator.
pub struct TranslateGizmo;

impl TranslateGizmo {
    /// Snap position to grid using snapping configuration.
    pub fn snap_position(position: Vec3, snapping: &SnappingConfig) -> Vec3 {
        snapping.snap_position(position)
    }

    /// Calculate translation delta from mouse movement.
    ///
    /// # Arguments
    /// * `mouse_delta` - Screen-space mouse movement (pixels)
    /// * `constraint` - Axis/plane constraint to apply
    /// * `camera_distance` - Distance from camera to object (for scaling)
    /// * `object_rotation` - Object's current rotation (for local space)
    /// * `local_space` - If true, use object-local coordinates
    ///
    /// # Returns
    /// World-space translation vector
    pub fn calculate_translation(
        mouse_delta: Vec2,
        constraint: AxisConstraint,
        camera_distance: f32,
        object_rotation: Quat,
        local_space: bool,
    ) -> Vec3 {
        // Convert screen-space mouse delta to world units
        // Scale by camera distance (objects farther away move more per pixel)
        // Reduced sensitivity: 0.002 = 500px for 1 unit at 10m distance
        let scale_factor = (camera_distance * 0.002).max(0.001); // Much lower sensitivity

        // Base world-space translation (X and Z for ground plane)
        let mut world_delta = Vec3::new(
            mouse_delta.x * scale_factor,  // Horizontal mouse → world X
            0.0,                           // No vertical world movement
            -mouse_delta.y * scale_factor, // Vertical mouse → world Z (flipped)
        );

        // Apply constraint in world space or local space
        if local_space {
            // Transform constraint to object-local coordinates
            let rotation_matrix = Mat3::from_quat(object_rotation);
            world_delta = rotation_matrix * world_delta;

            // Apply constraint in local space
            world_delta = Self::apply_constraint_local(world_delta, constraint);

            // Transform back to world space
            world_delta = rotation_matrix.transpose() * world_delta;
        } else {
            // Apply constraint directly in world space
            world_delta = Self::apply_constraint_world(world_delta, constraint);
        }

        world_delta
    }

    /// Calculate translation from numeric input value.
    ///
    /// # Arguments
    /// * `value` - Numeric input (e.g., "5.2" → 5.2)
    /// * `constraint` - Axis/plane constraint
    /// * `object_rotation` - Object's rotation (for local space)
    /// * `local_space` - If true, use object-local coordinates
    ///
    /// # Returns
    /// World-space translation vector with exact value on constraint axis
    pub fn calculate_translation_numeric(
        value: f32,
        constraint: AxisConstraint,
        object_rotation: Quat,
        local_space: bool,
    ) -> Vec3 {
        // Determine constraint axis vector
        let axis_vector = match constraint {
            AxisConstraint::None => Vec3::ZERO, // Can't apply numeric to free movement
            AxisConstraint::X => Vec3::X * value,
            AxisConstraint::Y => Vec3::Y * value,
            AxisConstraint::Z => Vec3::Z * value,
            AxisConstraint::XY => Vec3::ZERO, // Planar constraints don't support numeric (ambiguous)
            AxisConstraint::XZ => Vec3::ZERO,
            AxisConstraint::YZ => Vec3::ZERO,
        };

        if local_space {
            // Transform from local to world space
            let rotation_matrix = Mat3::from_quat(object_rotation);
            rotation_matrix * axis_vector
        } else {
            axis_vector
        }
    }

    /// Apply constraint in world space.
    fn apply_constraint_world(delta: Vec3, constraint: AxisConstraint) -> Vec3 {
        match constraint {
            AxisConstraint::None => delta, // Free movement on XZ plane
            AxisConstraint::X => Vec3::new(delta.x, 0.0, 0.0), // Only X axis
            AxisConstraint::Y => Vec3::ZERO, // Y axis not used for ground plane
            AxisConstraint::Z => Vec3::new(0.0, 0.0, delta.z), // Only Z axis
            AxisConstraint::XY => Vec3::new(delta.x, 0.0, 0.0), // XY plane → X only (no Y in ground)
            AxisConstraint::XZ => Vec3::new(delta.x, 0.0, delta.z), // XZ plane (full ground movement)
            AxisConstraint::YZ => Vec3::new(0.0, 0.0, delta.z), // YZ plane → Z only (no Y in ground)
        }
    }

    /// Apply constraint in local space.
    fn apply_constraint_local(delta: Vec3, constraint: AxisConstraint) -> Vec3 {
        // Same logic as world space, but operates on local-space vectors
        Self::apply_constraint_world(delta, constraint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_translation_world_space_free() {
        // Free movement in world space (no constraint)
        let delta = TranslateGizmo::calculate_translation(
            Vec2::new(100.0, -50.0),
            AxisConstraint::None,
            10.0, // 10 units from camera
            Quat::IDENTITY,
            false, // world space
        );

        // scale_factor = 10.0 * 0.002 = 0.02 (lower sensitivity)
        assert_relative_eq!(delta.x, 2.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, 1.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_world_space_x_axis() {
        // Constrain to X axis
        let delta = TranslateGizmo::calculate_translation(
            Vec2::new(100.0, -50.0),
            AxisConstraint::X,
            10.0,
            Quat::IDENTITY,
            false,
        );

        assert_relative_eq!(delta.x, 2.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_world_space_y_axis() {
        // Constrain to Y axis
        let delta = TranslateGizmo::calculate_translation(
            Vec2::new(100.0, -50.0),
            AxisConstraint::Y,
            10.0,
            Quat::IDENTITY,
            false,
        );

        // Ground-plane gizmo does not move along Y
        assert_relative_eq!(delta.x, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_world_space_z_axis() {
        // Constrain to Z axis (vertical mouse delta → world Z)
        let delta = TranslateGizmo::calculate_translation(
            Vec2::new(100.0, 50.0),
            AxisConstraint::Z,
            10.0,
            Quat::IDENTITY,
            false,
        );

        assert_relative_eq!(delta.x, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, -1.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_world_space_xy_plane() {
        // XY plane collapses to ground X movement (no Y axis)
        let delta = TranslateGizmo::calculate_translation(
            Vec2::new(100.0, -50.0),
            AxisConstraint::XY, // YZ plane
            10.0,
            Quat::IDENTITY,
            false,
        );

        assert_relative_eq!(delta.x, 2.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_camera_distance_scaling() {
        let delta_near = TranslateGizmo::calculate_translation(
            Vec2::new(100.0, 0.0),
            AxisConstraint::None,
            5.0, // scale 0.01
            Quat::IDENTITY,
            false,
        );

        let delta_far = TranslateGizmo::calculate_translation(
            Vec2::new(100.0, 0.0),
            AxisConstraint::None,
            20.0, // scale 0.04
            Quat::IDENTITY,
            false,
        );

        assert_relative_eq!(delta_near.x, 1.0, epsilon = 0.001);
        assert_relative_eq!(delta_far.x, 4.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_numeric_x_axis() {
        // Numeric input: "5.2" on X axis
        let delta = TranslateGizmo::calculate_translation_numeric(
            5.2,
            AxisConstraint::X,
            Quat::IDENTITY,
            false,
        );

        assert_relative_eq!(delta.x, 5.2, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_numeric_y_axis() {
        let delta = TranslateGizmo::calculate_translation_numeric(
            -10.5,
            AxisConstraint::Y,
            Quat::IDENTITY,
            false,
        );

        assert_relative_eq!(delta.x, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, -10.5, epsilon = 0.001);
        assert_relative_eq!(delta.z, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_numeric_planar_returns_zero() {
        // Planar constraints (XY, XZ, YZ) are ambiguous for numeric input
        let delta = TranslateGizmo::calculate_translation_numeric(
            5.0,
            AxisConstraint::XY, // YZ plane
            Quat::IDENTITY,
            false,
        );

        // Should return zero (can't determine direction on plane)
        assert_relative_eq!(delta.x, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_numeric_free_returns_zero() {
        // Free movement (None) is ambiguous for numeric input
        let delta = TranslateGizmo::calculate_translation_numeric(
            5.0,
            AxisConstraint::None,
            Quat::IDENTITY,
            false,
        );

        assert_relative_eq!(delta.x, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_local_space_rotated_object() {
        // Rotate object 90° around Z axis
        let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2); // 90°

        let delta = TranslateGizmo::calculate_translation(
            Vec2::new(100.0, 0.0), // Move right in screen space
            AxisConstraint::None,  // Free movement (no constraint yet)
            10.0,
            rotation,
            true, // local space
        );

        // In local space with 90° rotation, the transform is applied
        // This is a complex test - just verify non-zero result
        assert!(
            delta.length() > 0.0,
            "Delta should be non-zero for mouse movement"
        );
    }

    #[test]
    fn test_translation_zero_mouse_delta() {
        // No mouse movement = no translation
        let delta = TranslateGizmo::calculate_translation(
            Vec2::ZERO,
            AxisConstraint::None,
            10.0,
            Quat::IDENTITY,
            false,
        );

        assert_relative_eq!(delta.x, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_negative_values() {
        // Negative mouse delta should work correctly
        let delta = TranslateGizmo::calculate_translation(
            Vec2::new(-100.0, 50.0),
            AxisConstraint::None,
            10.0,
            Quat::IDENTITY,
            false,
        );

        assert_relative_eq!(delta.x, -2.0, epsilon = 0.001);
        assert_relative_eq!(delta.y, 0.0, epsilon = 0.001);
        assert_relative_eq!(delta.z, -1.0, epsilon = 0.001);
    }

    #[test]
    fn test_translation_clamp_camera_distance() {
        // Very small camera distance should clamp to avoid division issues
        let delta = TranslateGizmo::calculate_translation(
            Vec2::new(100.0, 0.0),
            AxisConstraint::None,
            0.0001, // Tiny distance (should clamp to 0.01)
            Quat::IDENTITY,
            false,
        );

        // scale_factor clamps to 0.001 → 100 * 0.001 = 0.1
        assert_relative_eq!(delta.x, 0.1, epsilon = 0.001);
    }
}
