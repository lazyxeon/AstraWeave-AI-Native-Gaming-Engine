use glam::{Quat, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnappingConfig {
    pub grid_size: f32,
    pub angle_increment: f32,
    pub grid_enabled: bool,
    pub angle_enabled: bool,
}

impl Default for SnappingConfig {
    fn default() -> Self {
        Self {
            grid_size: 1.0,
            angle_increment: 15.0,
            grid_enabled: true,
            angle_enabled: true,
        }
    }
}

impl SnappingConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_grid_size(mut self, size: f32) -> Self {
        self.grid_size = size;
        self
    }

    pub fn with_angle_increment(mut self, degrees: f32) -> Self {
        self.angle_increment = degrees;
        self
    }

    /// Returns true when grid snapping should be applied/rendered.
    pub fn grid_active(&self) -> bool {
        self.grid_enabled && self.grid_size > f32::EPSILON
    }

    /// Returns a positive grid size that can be fed directly into shaders/UI.
    pub fn resolved_grid_size(&self) -> f32 {
        if self.grid_size > f32::EPSILON {
            self.grid_size
        } else {
            1.0
        }
    }

    pub fn snap_position(&self, position: Vec3) -> Vec3 {
        if !self.grid_enabled || self.grid_size <= 0.0 {
            return position;
        }

        Vec3::new(
            (position.x / self.grid_size).round() * self.grid_size,
            (position.y / self.grid_size).round() * self.grid_size,
            (position.z / self.grid_size).round() * self.grid_size,
        )
    }

    pub fn snap_angle(&self, angle_radians: f32) -> f32 {
        if !self.angle_enabled || self.angle_increment <= 0.0 {
            return angle_radians;
        }

        let increment_radians = self.angle_increment.to_radians();
        (angle_radians / increment_radians).round() * increment_radians
    }

    pub fn snap_rotation(&self, rotation: Quat) -> Quat {
        if !self.angle_enabled {
            return rotation;
        }

        let (axis, angle) = rotation.to_axis_angle();
        let snapped_angle = self.snap_angle(angle);
        Quat::from_axis_angle(axis, snapped_angle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_position_default() {
        let config = SnappingConfig::default();
        let pos = Vec3::new(1.7, 2.3, -0.4);
        let snapped = config.snap_position(pos);
        assert_eq!(snapped, Vec3::new(2.0, 2.0, 0.0));
    }

    #[test]
    fn test_snap_position_half_grid() {
        let config = SnappingConfig::default().with_grid_size(0.5);
        let pos = Vec3::new(1.7, 2.3, -0.4);
        let snapped = config.snap_position(pos);
        assert_eq!(snapped, Vec3::new(1.5, 2.5, -0.5));
    }

    #[test]
    fn test_snap_position_disabled() {
        let mut config = SnappingConfig::default();
        config.grid_enabled = false;
        let pos = Vec3::new(1.7, 2.3, -0.4);
        let snapped = config.snap_position(pos);
        assert_eq!(snapped, pos);
    }

    #[test]
    fn test_grid_active_toggle() {
        let config = SnappingConfig::default();
        assert!(config.grid_active());

        let mut disabled = config;
        disabled.grid_enabled = false;
        assert!(!disabled.grid_active());

        let mut zero_size = SnappingConfig::default();
        zero_size.grid_size = 0.0;
        assert!(!zero_size.grid_active());
    }

    #[test]
    fn test_resolved_grid_size_defaults_positive() {
        let mut config = SnappingConfig::default();
        config.grid_size = 0.0;
        assert_eq!(config.resolved_grid_size(), 1.0);

        let mut custom = SnappingConfig::default();
        custom.grid_size = 2.5;
        assert_eq!(custom.resolved_grid_size(), 2.5);
    }

    #[test]
    fn test_snap_angle_default() {
        let config = SnappingConfig::default();
        let angle = 23.0_f32.to_radians();
        let snapped = config.snap_angle(angle);
        assert!((snapped - 30.0_f32.to_radians()).abs() < 0.001);
    }

    #[test]
    fn test_snap_angle_disabled() {
        let mut config = SnappingConfig::default();
        config.angle_enabled = false;
        let angle = 23.0_f32.to_radians();
        let snapped = config.snap_angle(angle);
        assert_eq!(snapped, angle);
    }

    #[test]
    fn test_snap_rotation() {
        let config = SnappingConfig::default();
        let rotation = Quat::from_rotation_z(23.0_f32.to_radians());
        let snapped = config.snap_rotation(rotation);
        let (_, snapped_angle) = snapped.to_axis_angle();
        let expected_angle = 30.0_f32.to_radians();
        assert!((snapped_angle - expected_angle).abs() < 0.001);
    }
}
