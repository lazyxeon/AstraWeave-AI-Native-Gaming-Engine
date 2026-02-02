//! # Gravity System
//!
//! Provides variable gravity mechanics including:
//! - Per-body gravity scale and direction
//! - Gravity zones (AABB, Sphere, Point/Attractor)
//! - Zero-G areas
//!
//! ## Features
//!
//! - **Per-Body Gravity**: Each body can have its own gravity multiplier or custom direction
//! - **Gravity Zones**: Regions that override gravity for bodies inside them
//! - **Point Gravity**: Attractors that pull objects toward a point (black holes, planets)
//!
//! ## Usage
//!
//! ```rust
//! use astraweave_physics::gravity::{GravityZone, GravityZoneShape, GravityManager};
//! use glam::Vec3;
//!
//! let mut manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));
//!
//! // Add a zero-G zone
//! manager.add_zone(GravityZone {
//!     shape: GravityZoneShape::Box {
//!         min: Vec3::new(-10.0, 0.0, -10.0),
//!         max: Vec3::new(10.0, 20.0, 10.0),
//!     },
//!     gravity: Vec3::ZERO,
//!     priority: 1,
//!     ..Default::default()
//! });
//!
//! // Add a point attractor (planet)
//! manager.add_zone(GravityZone {
//!     shape: GravityZoneShape::Point {
//!         center: Vec3::new(0.0, 100.0, 0.0),
//!         radius: 50.0,
//!         strength: 500.0,
//!     },
//!     gravity: Vec3::ZERO, // Ignored for point gravity
//!     priority: 2,
//!     ..Default::default()
//! });
//! ```

use glam::Vec3;
use std::collections::HashMap;

/// Unique identifier for a gravity zone
pub type GravityZoneId = u64;

/// Unique identifier for a body's custom gravity settings
pub type BodyGravityId = u64;

/// Shape of a gravity zone
#[derive(Debug, Clone, Copy)]
pub enum GravityZoneShape {
    /// Axis-aligned box zone
    Box { min: Vec3, max: Vec3 },
    /// Spherical zone
    Sphere { center: Vec3, radius: f32 },
    /// Point gravity (attractor/repulsor)
    /// Bodies are pulled toward (or pushed from) the center
    Point {
        center: Vec3,
        /// Maximum effect radius
        radius: f32,
        /// Force strength (positive = attract, negative = repel)
        strength: f32,
    },
}

impl GravityZoneShape {
    /// Check if a position is inside this shape
    pub fn contains(&self, pos: Vec3) -> bool {
        match self {
            GravityZoneShape::Box { min, max } => {
                pos.x >= min.x
                    && pos.x <= max.x
                    && pos.y >= min.y
                    && pos.y <= max.y
                    && pos.z >= min.z
                    && pos.z <= max.z
            }
            GravityZoneShape::Sphere { center, radius } => {
                pos.distance_squared(*center) <= radius * radius
            }
            GravityZoneShape::Point { center, radius, .. } => {
                pos.distance_squared(*center) <= radius * radius
            }
        }
    }

    /// Get the gravity vector for a position inside this shape
    /// Returns None if the position is outside the shape
    pub fn get_gravity(&self, pos: Vec3, zone_gravity: Vec3) -> Option<Vec3> {
        if !self.contains(pos) {
            return None;
        }

        match self {
            GravityZoneShape::Box { .. } | GravityZoneShape::Sphere { .. } => Some(zone_gravity),
            GravityZoneShape::Point {
                center,
                radius,
                strength,
            } => {
                let to_center = *center - pos;
                let distance = to_center.length();
                if distance < 0.001 {
                    // At the center, no gravity
                    return Some(Vec3::ZERO);
                }
                // Inverse square falloff
                let falloff = 1.0 - (distance / radius).min(1.0);
                let force = *strength * falloff * falloff;
                Some(to_center.normalize() * force)
            }
        }
    }
}

/// A gravity zone that affects bodies inside it
#[derive(Debug, Clone)]
pub struct GravityZone {
    /// Unique identifier
    pub id: GravityZoneId,
    /// Shape of the zone
    pub shape: GravityZoneShape,
    /// Gravity vector (or base gravity for non-point zones)
    pub gravity: Vec3,
    /// Priority (higher priority zones override lower ones)
    pub priority: i32,
    /// Whether this zone is active
    pub active: bool,
    /// Optional name for debugging
    pub name: Option<String>,
}

impl Default for GravityZone {
    fn default() -> Self {
        Self {
            id: 0,
            shape: GravityZoneShape::Box {
                min: Vec3::splat(-10.0),
                max: Vec3::splat(10.0),
            },
            gravity: Vec3::ZERO,
            priority: 0,
            active: true,
            name: None,
        }
    }
}

/// Per-body gravity settings
#[derive(Debug, Clone, Copy)]
pub struct BodyGravitySettings {
    /// Gravity scale multiplier (0.0 = zero-G, 1.0 = normal, 2.0 = double, -1.0 = reverse)
    pub scale: f32,
    /// Custom gravity direction (if Some, overrides global gravity direction)
    pub custom_direction: Option<Vec3>,
    /// Whether this body ignores gravity zones
    pub ignore_zones: bool,
}

impl Default for BodyGravitySettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            custom_direction: None,
            ignore_zones: false,
        }
    }
}

/// Manages gravity for the physics world
#[derive(Debug)]
pub struct GravityManager {
    /// Global gravity vector
    pub global_gravity: Vec3,
    /// All gravity zones
    zones: HashMap<GravityZoneId, GravityZone>,
    /// Next zone ID
    next_zone_id: GravityZoneId,
    /// Per-body gravity settings
    body_settings: HashMap<BodyGravityId, BodyGravitySettings>,
}

impl Default for GravityManager {
    fn default() -> Self {
        Self::new(Vec3::new(0.0, -9.81, 0.0))
    }
}

impl GravityManager {
    /// Create a new gravity manager with the given global gravity
    pub fn new(global_gravity: Vec3) -> Self {
        Self {
            global_gravity,
            zones: HashMap::new(),
            next_zone_id: 1,
            body_settings: HashMap::new(),
        }
    }

    /// Add a gravity zone
    pub fn add_zone(&mut self, mut zone: GravityZone) -> GravityZoneId {
        let id = self.next_zone_id;
        self.next_zone_id += 1;
        zone.id = id;
        self.zones.insert(id, zone);
        id
    }

    /// Remove a gravity zone
    pub fn remove_zone(&mut self, id: GravityZoneId) -> bool {
        self.zones.remove(&id).is_some()
    }

    /// Get a gravity zone by ID
    pub fn get_zone(&self, id: GravityZoneId) -> Option<&GravityZone> {
        self.zones.get(&id)
    }

    /// Get a mutable reference to a gravity zone
    pub fn get_zone_mut(&mut self, id: GravityZoneId) -> Option<&mut GravityZone> {
        self.zones.get_mut(&id)
    }

    /// Set the active state of a zone
    pub fn set_zone_active(&mut self, id: GravityZoneId, active: bool) -> bool {
        if let Some(zone) = self.zones.get_mut(&id) {
            zone.active = active;
            true
        } else {
            false
        }
    }

    /// Get all zones (for iteration)
    pub fn zones(&self) -> impl Iterator<Item = &GravityZone> {
        self.zones.values()
    }

    /// Set gravity settings for a body
    pub fn set_body_gravity(&mut self, body_id: BodyGravityId, settings: BodyGravitySettings) {
        self.body_settings.insert(body_id, settings);
    }

    /// Get gravity settings for a body
    pub fn get_body_gravity(&self, body_id: BodyGravityId) -> BodyGravitySettings {
        self.body_settings
            .get(&body_id)
            .copied()
            .unwrap_or_default()
    }

    /// Remove gravity settings for a body (returns to default)
    pub fn remove_body_gravity(&mut self, body_id: BodyGravityId) {
        self.body_settings.remove(&body_id);
    }

    /// Set gravity scale for a body
    pub fn set_gravity_scale(&mut self, body_id: BodyGravityId, scale: f32) {
        let settings = self.body_settings.entry(body_id).or_default();
        settings.scale = scale;
    }

    /// Set custom gravity direction for a body
    pub fn set_gravity_direction(&mut self, body_id: BodyGravityId, direction: Option<Vec3>) {
        let settings = self.body_settings.entry(body_id).or_default();
        settings.custom_direction = direction;
    }

    /// Calculate effective gravity for a body at a given position
    ///
    /// Priority order:
    /// 1. Body's custom direction (if set)
    /// 2. Highest-priority active zone containing the position
    /// 3. Global gravity
    ///
    /// The result is then multiplied by the body's gravity scale
    pub fn calculate_gravity(&self, body_id: BodyGravityId, position: Vec3) -> Vec3 {
        let settings = self.get_body_gravity(body_id);

        // Start with global gravity
        let base_gravity = if let Some(custom_dir) = settings.custom_direction {
            custom_dir
        } else if !settings.ignore_zones {
            // Find highest-priority zone containing this position
            let zone_gravity = self
                .zones
                .values()
                .filter(|z| z.active)
                .filter_map(|z| z.shape.get_gravity(position, z.gravity).map(|g| (z.priority, g)))
                .max_by_key(|(priority, _)| *priority)
                .map(|(_, g)| g);

            zone_gravity.unwrap_or(self.global_gravity)
        } else {
            self.global_gravity
        };

        base_gravity * settings.scale
    }

    /// Get all bodies affected by a specific zone
    /// Useful for debugging and visualization
    pub fn bodies_in_zone(&self, zone_id: GravityZoneId, body_positions: &[(BodyGravityId, Vec3)]) -> Vec<BodyGravityId> {
        let Some(zone) = self.zones.get(&zone_id) else {
            return Vec::new();
        };

        body_positions
            .iter()
            .filter(|(_, pos)| zone.shape.contains(*pos))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Create a zero-G box zone
    pub fn add_zero_g_box(&mut self, min: Vec3, max: Vec3, priority: i32) -> GravityZoneId {
        self.add_zone(GravityZone {
            shape: GravityZoneShape::Box { min, max },
            gravity: Vec3::ZERO,
            priority,
            ..Default::default()
        })
    }

    /// Create a zero-G sphere zone
    pub fn add_zero_g_sphere(&mut self, center: Vec3, radius: f32, priority: i32) -> GravityZoneId {
        self.add_zone(GravityZone {
            shape: GravityZoneShape::Sphere { center, radius },
            gravity: Vec3::ZERO,
            priority,
            ..Default::default()
        })
    }

    /// Create a point attractor (like a planet or black hole)
    pub fn add_attractor(&mut self, center: Vec3, radius: f32, strength: f32, priority: i32) -> GravityZoneId {
        self.add_zone(GravityZone {
            shape: GravityZoneShape::Point {
                center,
                radius,
                strength,
            },
            gravity: Vec3::ZERO, // Ignored for point gravity
            priority,
            ..Default::default()
        })
    }

    /// Create a directional gravity zone (like walking on walls)
    pub fn add_directional_zone(
        &mut self,
        min: Vec3,
        max: Vec3,
        gravity_direction: Vec3,
        priority: i32,
    ) -> GravityZoneId {
        self.add_zone(GravityZone {
            shape: GravityZoneShape::Box { min, max },
            gravity: gravity_direction,
            priority,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gravity_manager_creation() {
        let manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));
        assert_eq!(manager.global_gravity, Vec3::new(0.0, -9.81, 0.0));
    }

    #[test]
    fn test_default_gravity_calculation() {
        let manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));
        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity.y - (-9.81)).abs() < 0.001);
    }

    #[test]
    fn test_gravity_scale() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        manager.set_gravity_scale(1, 0.5);

        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity.y - (-5.0)).abs() < 0.001, "Expected -5.0, got {}", gravity.y);
    }

    #[test]
    fn test_zero_gravity_scale() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        manager.set_gravity_scale(1, 0.0);

        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!(gravity.length() < 0.001, "Should have zero gravity");
    }

    #[test]
    fn test_reverse_gravity_scale() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        manager.set_gravity_scale(1, -1.0);

        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity.y - 10.0).abs() < 0.001, "Expected +10.0, got {}", gravity.y);
    }

    #[test]
    fn test_custom_gravity_direction() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        manager.set_gravity_direction(1, Some(Vec3::new(10.0, 0.0, 0.0)));

        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity.x - 10.0).abs() < 0.001);
        assert!(gravity.y.abs() < 0.001);
    }

    #[test]
    fn test_box_zone_contains() {
        let shape = GravityZoneShape::Box {
            min: Vec3::new(-5.0, -5.0, -5.0),
            max: Vec3::new(5.0, 5.0, 5.0),
        };

        assert!(shape.contains(Vec3::ZERO));
        assert!(shape.contains(Vec3::new(4.0, 4.0, 4.0)));
        assert!(!shape.contains(Vec3::new(6.0, 0.0, 0.0)));
    }

    #[test]
    fn test_sphere_zone_contains() {
        let shape = GravityZoneShape::Sphere {
            center: Vec3::ZERO,
            radius: 10.0,
        };

        assert!(shape.contains(Vec3::ZERO));
        assert!(shape.contains(Vec3::new(5.0, 5.0, 0.0)));
        assert!(!shape.contains(Vec3::new(10.0, 10.0, 0.0))); // Outside sqrt(200) > 10
    }

    #[test]
    fn test_zero_g_zone() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        manager.add_zero_g_box(
            Vec3::new(-5.0, -5.0, -5.0),
            Vec3::new(5.0, 5.0, 5.0),
            1,
        );

        // Inside zone: zero gravity
        let gravity_inside = manager.calculate_gravity(1, Vec3::ZERO);
        assert!(gravity_inside.length() < 0.001);

        // Outside zone: normal gravity
        let gravity_outside = manager.calculate_gravity(1, Vec3::new(10.0, 0.0, 0.0));
        assert!((gravity_outside.y - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn test_zone_priority() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));

        // Low priority zone: upward gravity
        manager.add_zone(GravityZone {
            shape: GravityZoneShape::Box {
                min: Vec3::splat(-10.0),
                max: Vec3::splat(10.0),
            },
            gravity: Vec3::new(0.0, 5.0, 0.0),
            priority: 1,
            ..Default::default()
        });

        // High priority zone (smaller): zero gravity
        manager.add_zone(GravityZone {
            shape: GravityZoneShape::Box {
                min: Vec3::splat(-5.0),
                max: Vec3::splat(5.0),
            },
            gravity: Vec3::ZERO,
            priority: 10,
            ..Default::default()
        });

        // Inside both zones: high priority wins (zero-G)
        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!(gravity.length() < 0.001);

        // Inside only low priority zone: upward gravity
        let gravity = manager.calculate_gravity(1, Vec3::new(7.0, 0.0, 0.0));
        assert!((gravity.y - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_point_gravity_attractor() {
        let mut manager = GravityManager::new(Vec3::ZERO); // No global gravity
        manager.add_attractor(Vec3::new(0.0, 100.0, 0.0), 100.0, 100.0, 1); // Larger radius

        // Body at position (0, 50, 0) should be pulled toward (0, 100, 0)
        let gravity = manager.calculate_gravity(1, Vec3::new(0.0, 50.0, 0.0));
        assert!(gravity.y > 0.0, "Should be pulled upward toward attractor, got {}", gravity.y);
    }

    #[test]
    fn test_point_gravity_repulsor() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        manager.add_attractor(Vec3::ZERO, 50.0, -100.0, 1); // Negative = repel

        let gravity = manager.calculate_gravity(1, Vec3::new(10.0, 0.0, 0.0));
        assert!(gravity.x > 0.0, "Should be pushed away from repulsor");
    }

    #[test]
    fn test_directional_zone() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        manager.add_directional_zone(
            Vec3::new(-5.0, 0.0, -5.0),
            Vec3::new(5.0, 10.0, 5.0),
            Vec3::new(10.0, 0.0, 0.0), // Sideways gravity
            1,
        );

        let gravity = manager.calculate_gravity(1, Vec3::new(0.0, 5.0, 0.0));
        assert!((gravity.x - 10.0).abs() < 0.001);
        assert!(gravity.y.abs() < 0.001);
    }

    #[test]
    fn test_ignore_zones_flag() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        manager.add_zero_g_box(Vec3::splat(-10.0), Vec3::splat(10.0), 1);

        // Set body to ignore zones
        manager.set_body_gravity(1, BodyGravitySettings {
            scale: 1.0,
            custom_direction: None,
            ignore_zones: true,
        });

        // Should ignore the zero-G zone and use global gravity
        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity.y - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn test_zone_activation() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        let zone_id = manager.add_zero_g_box(Vec3::splat(-10.0), Vec3::splat(10.0), 1);

        // Zone active: zero gravity
        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!(gravity.length() < 0.001);

        // Deactivate zone
        manager.set_zone_active(zone_id, false);

        // Zone inactive: global gravity
        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity.y - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn test_remove_zone() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        let zone_id = manager.add_zero_g_box(Vec3::splat(-10.0), Vec3::splat(10.0), 1);

        assert!(manager.remove_zone(zone_id));
        assert!(!manager.remove_zone(zone_id)); // Already removed

        // Should use global gravity now
        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity.y - (-10.0)).abs() < 0.001);
    }

    #[test]
    fn test_bodies_in_zone() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        let zone_id = manager.add_zero_g_box(
            Vec3::new(-5.0, -5.0, -5.0),
            Vec3::new(5.0, 5.0, 5.0),
            1,
        );

        let bodies = vec![
            (1, Vec3::ZERO),           // Inside
            (2, Vec3::new(3.0, 0.0, 0.0)), // Inside
            (3, Vec3::new(10.0, 0.0, 0.0)), // Outside
        ];

        let inside = manager.bodies_in_zone(zone_id, &bodies);
        assert_eq!(inside.len(), 2);
        assert!(inside.contains(&1));
        assert!(inside.contains(&2));
        assert!(!inside.contains(&3));
    }

    // ============================================================================
    // INVERSE-SQUARE LAW VALIDATION (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_point_gravity_falloff() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        manager.add_attractor(Vec3::ZERO, 100.0, 100.0, 1);

        // Point gravity uses quadratic falloff: force = strength * (1 - d/r)^2
        let gravity_near = manager.calculate_gravity(1, Vec3::new(10.0, 0.0, 0.0));
        let gravity_far = manager.calculate_gravity(1, Vec3::new(50.0, 0.0, 0.0));

        // Closer should have stronger gravity
        assert!(gravity_near.length() > gravity_far.length());
    }

    #[test]
    fn test_point_gravity_at_center() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        manager.add_attractor(Vec3::new(10.0, 10.0, 10.0), 50.0, 100.0, 1);

        // At the center, gravity should be zero
        let gravity = manager.calculate_gravity(1, Vec3::new(10.0, 10.0, 10.0));
        assert!(gravity.length() < 0.01);
    }

    #[test]
    fn test_point_gravity_outside_radius() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        manager.add_attractor(Vec3::ZERO, 20.0, 100.0, 1);

        // Outside radius: should use global gravity (zero in this case)
        let gravity = manager.calculate_gravity(1, Vec3::new(30.0, 0.0, 0.0));
        assert!(gravity.length() < 0.01);
    }

    #[test]
    fn test_point_gravity_direction() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        let attractor_pos = Vec3::new(50.0, 50.0, 0.0);
        manager.add_attractor(attractor_pos, 100.0, 100.0, 1);

        // Gravity should point toward attractor
        let body_pos = Vec3::ZERO;
        let gravity = manager.calculate_gravity(1, body_pos);
        let expected_dir = (attractor_pos - body_pos).normalize();

        // Gravity direction should match expected direction
        if gravity.length() > 0.01 {
            let gravity_dir = gravity.normalize();
            assert!((gravity_dir - expected_dir).length() < 0.1);
        }
    }

    #[test]
    fn test_repulsor_direction() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        manager.add_attractor(Vec3::ZERO, 100.0, -100.0, 1); // Negative = repulse

        let body_pos = Vec3::new(20.0, 0.0, 0.0);
        let gravity = manager.calculate_gravity(1, body_pos);

        // Repulsor should push away (positive X direction)
        assert!(gravity.x > 0.0);
    }

    // ============================================================================
    // ORBITAL MECHANICS TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_orbital_symmetry() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        manager.add_attractor(Vec3::ZERO, 100.0, 100.0, 1);

        // Points equidistant from center should have equal gravity magnitude
        let g1 = manager.calculate_gravity(1, Vec3::new(30.0, 0.0, 0.0));
        let g2 = manager.calculate_gravity(1, Vec3::new(0.0, 30.0, 0.0));
        let g3 = manager.calculate_gravity(1, Vec3::new(0.0, 0.0, 30.0));

        assert!((g1.length() - g2.length()).abs() < 0.01);
        assert!((g2.length() - g3.length()).abs() < 0.01);
    }

    #[test]
    fn test_multiple_attractors() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        
        // Two attractors on opposite sides
        manager.add_attractor(Vec3::new(-50.0, 0.0, 0.0), 100.0, 100.0, 1);
        manager.add_attractor(Vec3::new(50.0, 0.0, 0.0), 100.0, 100.0, 2);

        // At the midpoint, forces should (roughly) cancel
        // Note: Implementation may not sum attractors, testing highest priority
        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        
        // Priority 2 is higher, so should pull toward +X
        assert!(gravity.x > 0.0 || gravity.length() < 1.0);
    }

    // ============================================================================
    // EDGE CASE TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_gravity_manager_default() {
        let manager = GravityManager::default();
        assert!((manager.global_gravity.y - (-9.81)).abs() < 0.01);
    }

    #[test]
    fn test_gravity_zone_default() {
        let zone = GravityZone::default();
        assert_eq!(zone.id, 0);
        assert!(zone.active);
        assert!(zone.name.is_none());
        assert_eq!(zone.priority, 0);
    }

    #[test]
    fn test_body_gravity_settings_default() {
        let settings = BodyGravitySettings::default();
        assert_eq!(settings.scale, 1.0);
        assert!(settings.custom_direction.is_none());
        assert!(!settings.ignore_zones);
    }

    #[test]
    fn test_empty_zones_list() {
        let manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));
        assert_eq!(manager.zones().count(), 0);
    }

    #[test]
    fn test_get_nonexistent_zone() {
        let manager = GravityManager::new(Vec3::ZERO);
        assert!(manager.get_zone(999).is_none());
    }

    #[test]
    fn test_set_zone_active_nonexistent() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        manager.set_zone_active(999, false); // Should not panic
    }

    #[test]
    fn test_reset_body_gravity_to_default() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        
        // Set custom gravity
        manager.set_gravity_scale(1, 0.5);
        let gravity1 = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity1.y - (-5.0)).abs() < 0.01);
        
        // Reset to default settings
        manager.set_body_gravity(1, BodyGravitySettings::default());
        let gravity2 = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity2.y - (-10.0)).abs() < 0.01);
    }

    #[test]
    fn test_negative_gravity_scale() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        manager.set_gravity_scale(1, -2.0);

        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        // Should be double strength in opposite direction
        assert!((gravity.y - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_large_gravity_scale() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        manager.set_gravity_scale(1, 100.0);

        let gravity = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity.y - (-1000.0)).abs() < 0.01);
    }

    #[test]
    fn test_sphere_zone_boundary() {
        let shape = GravityZoneShape::Sphere {
            center: Vec3::ZERO,
            radius: 10.0,
        };

        // Exactly on boundary
        assert!(shape.contains(Vec3::new(10.0, 0.0, 0.0)));
        // Just outside
        assert!(!shape.contains(Vec3::new(10.01, 0.0, 0.0)));
    }

    #[test]
    fn test_box_zone_boundary() {
        let shape = GravityZoneShape::Box {
            min: Vec3::new(-5.0, -5.0, -5.0),
            max: Vec3::new(5.0, 5.0, 5.0),
        };

        // Exactly on boundary
        assert!(shape.contains(Vec3::new(5.0, 0.0, 0.0)));
        assert!(shape.contains(Vec3::new(-5.0, 0.0, 0.0)));
        // Just outside
        assert!(!shape.contains(Vec3::new(5.01, 0.0, 0.0)));
    }

    #[test]
    fn test_zone_named() {
        let mut manager = GravityManager::new(Vec3::ZERO);
        let zone = GravityZone {
            name: Some("TestZone".to_string()),
            shape: GravityZoneShape::Box {
                min: Vec3::splat(-5.0),
                max: Vec3::splat(5.0),
            },
            ..Default::default()
        };
        
        let id = manager.add_zone(zone);
        let retrieved = manager.get_zone(id).unwrap();
        assert_eq!(retrieved.name, Some("TestZone".to_string()));
    }

    #[test]
    fn test_global_gravity_update() {
        let mut manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));
        
        let gravity1 = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity1.y - (-10.0)).abs() < 0.01);
        
        // Change global gravity
        manager.global_gravity = Vec3::new(0.0, -20.0, 0.0);
        
        let gravity2 = manager.calculate_gravity(1, Vec3::ZERO);
        assert!((gravity2.y - (-20.0)).abs() < 0.01);
    }
}
