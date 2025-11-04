//! Orbit Camera Controller
//!
//! Professional camera controller using spherical coordinates for smooth
//! orbit, pan, and zoom operations. Designed for 3D editing workflows.
//!
//! # Features
//!
//! - **Orbit**: Rotate around focal point (left mouse drag)
//! - **Pan**: Move focal point in screen space (middle mouse drag)
//! - **Zoom**: Change distance from focal point (scroll wheel)
//! - **Frame**: Center camera on selected entity (F key)
//! - **Constraints**: Min/max distance, pitch limits
//!
//! # Example
//!
//! ```
//! use aw_editor_lib::viewport::OrbitCamera;
//! use glam::Vec3;
//!
//! let mut camera = OrbitCamera::default();
//!
//! // Orbit camera
//! camera.orbit(10.0, 5.0);
//!
//! // Zoom in
//! camera.zoom(5.0);
//!
//! // Frame entity
//! camera.frame_entity(Vec3::new(5.0, 0.0, 5.0), 2.0);
//! ```

use glam::{Mat4, Vec3};

/// Professional orbit camera controller
///
/// Uses spherical coordinates (distance, yaw, pitch) around a focal point.
/// Provides smooth, predictable camera controls for 3D editing.
///
/// # Coordinate System
///
/// - **Yaw**: Rotation around Y axis (horizontal), in radians
/// - **Pitch**: Rotation around X axis (vertical), in radians, constrained to [-π/2, π/2]
/// - **Distance**: Radius from focal point, constrained to [min_distance, max_distance]
///
/// # Performance
///
/// Camera updates are O(1) and typically take <0.1ms per frame.
#[derive(Debug, Clone)]
pub struct OrbitCamera {
    /// Focal point (what camera orbits around)
    focal_point: Vec3,

    /// Distance from focal point (meters)
    distance: f32,

    /// Yaw angle (rotation around Y axis, radians)
    yaw: f32,

    /// Pitch angle (rotation around X axis, radians)
    pitch: f32,

    /// Field of view (degrees)
    fov: f32,

    /// Aspect ratio (width / height)
    aspect: f32,

    /// Near clip plane (meters)
    near: f32,

    /// Far clip plane (meters)
    far: f32,

    /// Minimum distance from focal point (meters)
    min_distance: f32,

    /// Maximum distance from focal point (meters)
    max_distance: f32,

    /// Minimum pitch angle (radians, slightly above -π/2 to prevent gimbal lock)
    min_pitch: f32,

    /// Maximum pitch angle (radians, slightly below π/2 to prevent gimbal lock)
    max_pitch: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focal_point: Vec3::ZERO,
            distance: 10.0,
            yaw: 0.0,
            pitch: std::f32::consts::PI / 4.0, // 45 degrees (nice default angle)
            fov: 60.0,
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            min_distance: 1.0,
            max_distance: 100.0,
            min_pitch: -std::f32::consts::PI / 2.0 + 0.01, // Prevent gimbal lock
            max_pitch: std::f32::consts::PI / 2.0 - 0.01,  // Prevent gimbal lock
        }
    }
}

impl OrbitCamera {
    /// Create camera with custom parameters
    ///
    /// # Arguments
    ///
    /// * `focal_point` - Initial focal point
    /// * `distance` - Initial distance from focal point
    /// * `yaw` - Initial yaw angle (radians)
    /// * `pitch` - Initial pitch angle (radians)
    pub fn new(focal_point: Vec3, distance: f32, yaw: f32, pitch: f32) -> Self {
        Self {
            focal_point,
            distance,
            yaw,
            pitch,
            ..Default::default()
        }
    }

    /// Orbit camera (rotate around focal point)
    ///
    /// # Arguments
    ///
    /// * `delta_x` - Horizontal mouse movement (pixels)
    /// * `delta_y` - Vertical mouse movement (pixels)
    ///
    /// # Performance
    ///
    /// O(1), typically <0.01ms
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        const SENSITIVITY: f32 = 0.005; // Radians per pixel

        self.yaw -= delta_x * SENSITIVITY;
        self.pitch = (self.pitch - delta_y * SENSITIVITY).clamp(self.min_pitch, self.max_pitch);
    }

    /// Pan camera (move focal point in screen space)
    ///
    /// # Arguments
    ///
    /// * `delta_x` - Horizontal mouse movement (pixels)
    /// * `delta_y` - Vertical mouse movement (pixels)
    ///
    /// Pan speed scales with distance from focal point (further = faster pan).
    ///
    /// # Performance
    ///
    /// O(1), typically <0.05ms (involves cross products)
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        const SENSITIVITY: f32 = 0.005;

        // Calculate right and up vectors in world space
        let forward = self.forward();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward).normalize();

        // Pan speed scales with distance (more zoom = slower pan)
        let pan_speed = self.distance * SENSITIVITY;
        self.focal_point -= right * delta_x * pan_speed;
        self.focal_point += up * delta_y * pan_speed;
    }

    /// Zoom camera (change distance from focal point)
    ///
    /// # Arguments
    ///
    /// * `delta` - Scroll wheel delta (positive = zoom in, negative = zoom out)
    ///
    /// Zoom speed scales logarithmically with distance for smooth feel.
    ///
    /// # Performance
    ///
    /// O(1), typically <0.01ms
    pub fn zoom(&mut self, delta: f32) {
        const SENSITIVITY: f32 = 0.1;

        // Logarithmic zoom (feels more natural)
        let zoom_factor = 1.0 + delta * SENSITIVITY;
        self.distance = (self.distance / zoom_factor).clamp(self.min_distance, self.max_distance);
    }

    /// Frame entity (set focal point and distance to nicely view entity)
    ///
    /// # Arguments
    ///
    /// * `entity_pos` - Entity world position
    /// * `entity_radius` - Entity bounding radius (meters)
    ///
    /// Sets focal point to entity center and distance to 2.5× radius for nice framing.
    pub fn frame_entity(&mut self, entity_pos: Vec3, entity_radius: f32) {
        self.focal_point = entity_pos;
        self.distance = (entity_radius * 2.5).clamp(self.min_distance, self.max_distance);
    }

    /// Update aspect ratio (call when viewport resizes)
    pub fn set_aspect(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    /// Get camera position in world space
    ///
    /// Calculated from spherical coordinates (distance, yaw, pitch).
    ///
    /// # Performance
    ///
    /// O(1), involves trigonometry (~0.01ms)
    pub fn position(&self) -> Vec3 {
        // Convert spherical to Cartesian coordinates
        let x = self.distance * self.yaw.cos() * self.pitch.cos();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.yaw.sin() * self.pitch.cos();

        self.focal_point + Vec3::new(x, y, z)
    }

    /// Get focal point (what camera orbits around)
    pub fn target(&self) -> Vec3 {
        self.focal_point
    }

    /// Get distance from focal point (meters)
    pub fn distance(&self) -> f32 {
        self.distance
    }

    /// Get yaw angle (radians)
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Get pitch angle (radians)
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// Get camera forward vector (normalized)
    pub fn forward(&self) -> Vec3 {
        (self.focal_point - self.position()).normalize()
    }

    /// Get camera right vector (normalized)
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    /// Get camera up vector (normalized)
    pub fn up(&self) -> Vec3 {
        self.right().cross(self.forward()).normalize()
    }

    /// Get view matrix (world → camera space)
    ///
    /// Right-handed coordinate system (OpenGL/wgpu convention).
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.focal_point, Vec3::Y)
    }

    /// Get projection matrix (camera → clip space)
    ///
    /// Perspective projection with vertical FOV.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.far)
    }

    /// Get combined view-projection matrix
    ///
    /// Transforms vertices from world space directly to clip space.
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Create ray from screen position (for picking)
    ///
    /// # Arguments
    ///
    /// * `screen_pos` - Mouse position in viewport (top-left origin)
    /// * `viewport_size` - Viewport dimensions (width, height)
    ///
    /// # Returns
    ///
    /// Ray with origin at near plane and direction towards far plane.
    /// Suitable for ray-casting against scene geometry.
    pub fn ray_from_screen(&self, screen_pos: egui::Pos2, viewport_size: egui::Vec2) -> Ray {
        // Convert screen pos to NDC [-1, 1]
        let ndc_x = (screen_pos.x / viewport_size.x) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_pos.y / viewport_size.y) * 2.0; // Flip Y

        // Unproject to world space
        let inv_vp = self.view_projection_matrix().inverse();
        let near_point = inv_vp.project_point3(Vec3::new(ndc_x, ndc_y, -1.0));
        let far_point = inv_vp.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));

        Ray {
            origin: near_point,
            direction: (far_point - near_point).normalize(),
        }
    }
}

/// Ray for picking (origin + direction)
///
/// Used for ray-casting to select entities in 3D viewport.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Ray origin (world space)
    pub origin: Vec3,

    /// Ray direction (world space, normalized)
    pub direction: Vec3,
}

impl Ray {
    /// Create new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get point along ray at distance t
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_orbit_camera_default() {
        let camera = OrbitCamera::default();
        assert_eq!(camera.focal_point, Vec3::ZERO);
        assert_eq!(camera.distance, 10.0);
        assert_relative_eq!(camera.pitch, std::f32::consts::PI / 4.0);
    }

    #[test]
    fn test_orbit_camera_position() {
        let camera = OrbitCamera::default();
        let pos = camera.position();

        // Position should be ~10 units from focal point
        let dist = (pos - camera.focal_point).length();
        assert_relative_eq!(dist, 10.0, epsilon = 0.01);
    }

    #[test]
    fn test_orbit_camera_zoom() {
        let mut camera = OrbitCamera::default();
        let initial_dist = camera.distance;

        // Zoom in
        camera.zoom(10.0);
        assert!(camera.distance < initial_dist);

        // Zoom out
        camera.zoom(-20.0);
        assert!(camera.distance > initial_dist * 0.9);
    }

    #[test]
    fn test_orbit_camera_zoom_clamp() {
        let mut camera = OrbitCamera::default();

        // Try to zoom beyond max distance
        camera.zoom(-1000.0);
        assert_eq!(camera.distance, camera.max_distance);

        // Try to zoom below min distance
        camera.zoom(1000.0);
        assert_eq!(camera.distance, camera.min_distance);
    }

    #[test]
    fn test_frame_entity() {
        let mut camera = OrbitCamera::default();
        let entity_pos = Vec3::new(5.0, 2.0, 5.0);
        let entity_radius = 2.0;

        camera.frame_entity(entity_pos, entity_radius);

        assert_eq!(camera.focal_point, entity_pos);
        assert_eq!(camera.distance, 5.0); // 2.0 * 2.5
    }

    #[test]
    fn test_orbit_pitch_clamp() {
        let mut camera = OrbitCamera::default();

        // Try to orbit beyond max pitch
        camera.orbit(0.0, -10000.0);
        assert_relative_eq!(camera.pitch, camera.max_pitch, epsilon = 0.01);

        // Try to orbit below min pitch
        camera.orbit(0.0, 10000.0);
        assert_relative_eq!(camera.pitch, camera.min_pitch, epsilon = 0.01);
    }

    #[test]
    fn test_camera_vectors() {
        let camera = OrbitCamera::default();

        let forward = camera.forward();
        let right = camera.right();
        let up = camera.up();

        // Vectors should be normalized
        assert_relative_eq!(forward.length(), 1.0, epsilon = 0.01);
        assert_relative_eq!(right.length(), 1.0, epsilon = 0.01);
        assert_relative_eq!(up.length(), 1.0, epsilon = 0.01);

        // Vectors should be orthogonal
        assert_relative_eq!(forward.dot(right), 0.0, epsilon = 0.01);
        assert_relative_eq!(forward.dot(up), 0.0, epsilon = 0.01);
        assert_relative_eq!(right.dot(up), 0.0, epsilon = 0.01);
    }

    #[test]
    fn test_ray_at() {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        assert_eq!(ray.at(0.0), Vec3::ZERO);
        assert_eq!(ray.at(5.0), Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn test_ray_direction_normalized() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(3.0, 4.0, 0.0));
        assert_relative_eq!(ray.direction.length(), 1.0, epsilon = 0.01);
    }
}
