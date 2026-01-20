//! Environmental Physics Systems
//!
//! This module provides environmental effects that interact with rigid bodies:
//! - Wind zones (directional, vortex, turbulent)
//! - Gust system (noise-based variation)
//! - Buoyancy (water volumes)

use glam::Vec3;
use std::collections::HashMap;

/// Unique identifier for wind zones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindZoneId(pub u64);

/// Unique identifier for water volumes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WaterVolumeId(pub u64);

/// Shape of a wind zone
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindZoneShape {
    /// Infinite directional wind (global)
    Global,
    /// Box-shaped wind zone
    Box { half_extents: Vec3 },
    /// Spherical wind zone
    Sphere { radius: f32 },
    /// Cylindrical wind zone (useful for tornadoes)
    Cylinder { radius: f32, height: f32 },
}

impl Default for WindZoneShape {
    fn default() -> Self {
        Self::Global
    }
}

/// Type of wind behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindType {
    /// Constant directional wind
    Directional,
    /// Rotational wind around center (tornado, whirlpool)
    Vortex {
        /// Tangential speed at edge
        tangential_speed: f32,
        /// Inward pull strength
        inward_pull: f32,
        /// Upward lift
        updraft: f32,
    },
    /// Turbulent wind with noise
    Turbulent {
        /// Base turbulence intensity
        intensity: f32,
        /// Frequency of turbulence changes
        frequency: f32,
    },
}

impl Default for WindType {
    fn default() -> Self {
        Self::Directional
    }
}

/// Configuration for a wind zone
#[derive(Debug, Clone)]
pub struct WindZoneConfig {
    /// Position of the wind zone center
    pub position: Vec3,
    /// Shape of the zone
    pub shape: WindZoneShape,
    /// Type of wind behavior
    pub wind_type: WindType,
    /// Base wind direction (for directional/turbulent)
    pub direction: Vec3,
    /// Base wind strength (force per unit area)
    pub strength: f32,
    /// Falloff from center (0 = uniform, 1 = linear falloff to edge)
    pub falloff: f32,
    /// Whether this zone is active
    pub active: bool,
}

impl Default for WindZoneConfig {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            shape: WindZoneShape::Global,
            wind_type: WindType::Directional,
            direction: Vec3::new(1.0, 0.0, 0.0),
            strength: 10.0,
            falloff: 0.0,
            active: true,
        }
    }
}

/// Runtime state for a wind zone
#[derive(Debug, Clone)]
pub struct WindZone {
    pub id: WindZoneId,
    pub config: WindZoneConfig,
    /// Current gust offset (for turbulent wind)
    pub gust_offset: Vec3,
    /// Phase for noise-based variation
    pub noise_phase: f32,
}

impl WindZone {
    /// Create a new wind zone
    pub fn new(id: WindZoneId, config: WindZoneConfig) -> Self {
        Self {
            id,
            config,
            gust_offset: Vec3::ZERO,
            noise_phase: 0.0,
        }
    }

    /// Check if a point is inside this wind zone
    pub fn contains(&self, point: Vec3) -> bool {
        match self.config.shape {
            WindZoneShape::Global => true,
            WindZoneShape::Box { half_extents } => {
                let local = point - self.config.position;
                local.x.abs() <= half_extents.x
                    && local.y.abs() <= half_extents.y
                    && local.z.abs() <= half_extents.z
            }
            WindZoneShape::Sphere { radius } => {
                (point - self.config.position).length() <= radius
            }
            WindZoneShape::Cylinder { radius, height } => {
                let local = point - self.config.position;
                let horizontal_dist = Vec3::new(local.x, 0.0, local.z).length();
                horizontal_dist <= radius && local.y.abs() <= height / 2.0
            }
        }
    }

    /// Calculate wind force at a given point
    pub fn wind_force_at(&self, point: Vec3, drag_coefficient: f32, cross_section: f32) -> Vec3 {
        if !self.config.active || !self.contains(point) {
            return Vec3::ZERO;
        }

        // Calculate distance factor for falloff
        let distance_factor = self.calculate_falloff(point);

        // Calculate base wind velocity at this point
        let wind_velocity = match self.config.wind_type {
            WindType::Directional => {
                self.config.direction.normalize_or_zero() * self.config.strength
            }
            WindType::Vortex {
                tangential_speed,
                inward_pull,
                updraft,
            } => {
                let to_center = self.config.position - point;
                let horizontal = Vec3::new(to_center.x, 0.0, to_center.z);
                let dist = horizontal.length();

                if dist < 0.1 {
                    Vec3::new(0.0, updraft, 0.0)
                } else {
                    // Tangential component (perpendicular to radius)
                    let tangent = Vec3::new(-horizontal.z, 0.0, horizontal.x).normalize();
                    let tangential = tangent * tangential_speed;

                    // Inward pull
                    let inward = horizontal.normalize() * inward_pull;

                    // Combine
                    tangential + inward + Vec3::new(0.0, updraft, 0.0)
                }
            }
            WindType::Turbulent { intensity, .. } => {
                let base = self.config.direction.normalize_or_zero() * self.config.strength;
                base + self.gust_offset * intensity
            }
        };

        // Apply falloff
        let effective_velocity = wind_velocity * distance_factor;

        // Wind force = 0.5 * air_density * velocity^2 * drag_coefficient * area
        // Simplified: F = k * v^2 * direction
        let speed = effective_velocity.length();
        if speed < 0.01 {
            return Vec3::ZERO;
        }

        let force_magnitude = 0.5 * 1.225 * speed * speed * drag_coefficient * cross_section;
        effective_velocity.normalize() * force_magnitude
    }

    /// Calculate falloff factor based on distance from center
    fn calculate_falloff(&self, point: Vec3) -> f32 {
        if self.config.falloff <= 0.0 {
            return 1.0;
        }

        let normalized_dist = match self.config.shape {
            WindZoneShape::Global => 0.0,
            WindZoneShape::Box { half_extents } => {
                let local = (point - self.config.position).abs();
                
                (local / half_extents).max_element()
            }
            WindZoneShape::Sphere { radius } => {
                (point - self.config.position).length() / radius
            }
            WindZoneShape::Cylinder { radius, height } => {
                let local = point - self.config.position;
                let horizontal_dist = Vec3::new(local.x, 0.0, local.z).length() / radius;
                let vertical_dist = local.y.abs() / (height / 2.0);
                horizontal_dist.max(vertical_dist)
            }
        };

        (1.0 - normalized_dist * self.config.falloff).max(0.0)
    }

    /// Update turbulence/gust state
    pub fn update(&mut self, dt: f32) {
        if let WindType::Turbulent { frequency, .. } = self.config.wind_type {
            self.noise_phase += dt * frequency;

            // Simple pseudo-random gust using sine waves at different frequencies
            self.gust_offset = Vec3::new(
                (self.noise_phase * 1.0).sin() * 0.5 + (self.noise_phase * 2.3).sin() * 0.3,
                (self.noise_phase * 0.7).sin() * 0.2 + (self.noise_phase * 1.9).sin() * 0.15,
                (self.noise_phase * 1.3).sin() * 0.5 + (self.noise_phase * 2.7).sin() * 0.3,
            );
        }
    }
}

/// Gust event for sudden wind changes
#[derive(Debug, Clone)]
pub struct GustEvent {
    /// Direction of the gust
    pub direction: Vec3,
    /// Peak strength
    pub strength: f32,
    /// Duration in seconds
    pub duration: f32,
    /// Time elapsed
    pub elapsed: f32,
    /// Shape of gust envelope (0 = instant, 1 = smooth)
    pub smoothness: f32,
}

impl GustEvent {
    /// Create a new gust event
    pub fn new(direction: Vec3, strength: f32, duration: f32) -> Self {
        Self {
            direction: direction.normalize_or_zero(),
            strength,
            duration,
            elapsed: 0.0,
            smoothness: 0.5,
        }
    }

    /// Get current gust force multiplier
    pub fn current_strength(&self) -> f32 {
        if self.elapsed >= self.duration {
            return 0.0;
        }

        let t = self.elapsed / self.duration;

        // Smooth envelope: ramp up, hold, ramp down
        let envelope = if self.smoothness > 0.0 {
            let attack = (t * 4.0).min(1.0);
            let release = ((1.0 - t) * 4.0).min(1.0);
            attack * release
        } else {
            1.0
        };

        self.strength * envelope
    }

    /// Update gust timer
    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    /// Check if gust is finished
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Water volume for buoyancy calculations
#[derive(Debug, Clone)]
pub struct WaterVolume {
    pub id: WaterVolumeId,
    /// Center position of water surface
    pub position: Vec3,
    /// Half extents of the water volume
    pub half_extents: Vec3,
    /// Water surface height (Y coordinate)
    pub surface_height: f32,
    /// Water density (kg/m³, default 1000 for fresh water)
    pub density: f32,
    /// Linear drag coefficient in water
    pub linear_drag: f32,
    /// Angular drag coefficient in water
    pub angular_drag: f32,
    /// Current flow velocity
    pub current: Vec3,
    /// Wave amplitude
    pub wave_amplitude: f32,
    /// Wave frequency
    pub wave_frequency: f32,
    /// Wave phase
    pub wave_phase: f32,
}

impl WaterVolume {
    /// Create a new water volume
    pub fn new(id: WaterVolumeId, position: Vec3, half_extents: Vec3) -> Self {
        Self {
            id,
            position,
            half_extents,
            surface_height: position.y + half_extents.y,
            density: 1000.0,
            linear_drag: 0.5,
            angular_drag: 0.5,
            current: Vec3::ZERO,
            wave_amplitude: 0.0,
            wave_frequency: 1.0,
            wave_phase: 0.0,
        }
    }

    /// Check if a point is inside the water volume
    pub fn contains(&self, point: Vec3) -> bool {
        let local = point - self.position;
        local.x.abs() <= self.half_extents.x
            && local.y.abs() <= self.half_extents.y
            && local.z.abs() <= self.half_extents.z
    }

    /// Get water surface height at a given XZ position (includes waves)
    pub fn surface_height_at(&self, x: f32, z: f32) -> f32 {
        let base = self.surface_height;
        if self.wave_amplitude > 0.0 {
            let wave = self.wave_amplitude
                * (self.wave_phase + x * 0.1 + z * 0.15).sin()
                * (self.wave_phase * 0.7 + x * 0.08 - z * 0.12).cos();
            base + wave
        } else {
            base
        }
    }

    /// Calculate buoyancy force for a submerged body
    pub fn buoyancy_force(&self, _center: Vec3, volume: f32, submerged_fraction: f32) -> Vec3 {
        // Archimedes' principle: F = ρ * V * g
        let gravity = 9.81;
        let force = self.density * volume * submerged_fraction * gravity;
        Vec3::new(0.0, force, 0.0)
    }

    /// Calculate submerged fraction for a sphere
    pub fn sphere_submerged_fraction(&self, center: Vec3, radius: f32) -> f32 {
        let surface = self.surface_height_at(center.x, center.z);
        let depth = surface - center.y;

        if depth <= -radius {
            // Fully above water
            0.0
        } else if depth >= radius {
            // Fully submerged
            1.0
        } else {
            // Partially submerged - approximate
            let h = depth + radius; // Height of submerged cap
            let fraction = h / (2.0 * radius);
            fraction.clamp(0.0, 1.0)
        }
    }

    /// Update wave phase
    pub fn update(&mut self, dt: f32) {
        self.wave_phase += dt * self.wave_frequency * std::f32::consts::TAU;
    }
}

/// Manager for all environmental effects
#[derive(Debug, Default)]
pub struct EnvironmentManager {
    wind_zones: HashMap<WindZoneId, WindZone>,
    water_volumes: HashMap<WaterVolumeId, WaterVolume>,
    gusts: Vec<GustEvent>,
    next_wind_id: u64,
    next_water_id: u64,
    /// Global wind (affects everything)
    pub global_wind: Vec3,
    /// Global wind strength multiplier
    pub global_wind_strength: f32,
}

impl EnvironmentManager {
    /// Create a new environment manager
    pub fn new() -> Self {
        Self {
            wind_zones: HashMap::new(),
            water_volumes: HashMap::new(),
            gusts: Vec::new(),
            next_wind_id: 1,
            next_water_id: 1,
            global_wind: Vec3::ZERO,
            global_wind_strength: 1.0,
        }
    }

    // === Wind Zone Management ===

    /// Add a wind zone
    pub fn add_wind_zone(&mut self, config: WindZoneConfig) -> WindZoneId {
        let id = WindZoneId(self.next_wind_id);
        self.next_wind_id += 1;
        self.wind_zones.insert(id, WindZone::new(id, config));
        id
    }

    /// Remove a wind zone
    pub fn remove_wind_zone(&mut self, id: WindZoneId) -> bool {
        self.wind_zones.remove(&id).is_some()
    }

    /// Get a wind zone
    pub fn get_wind_zone(&self, id: WindZoneId) -> Option<&WindZone> {
        self.wind_zones.get(&id)
    }

    /// Get a mutable wind zone
    pub fn get_wind_zone_mut(&mut self, id: WindZoneId) -> Option<&mut WindZone> {
        self.wind_zones.get_mut(&id)
    }

    /// Set wind zone active state
    pub fn set_wind_zone_active(&mut self, id: WindZoneId, active: bool) {
        if let Some(zone) = self.wind_zones.get_mut(&id) {
            zone.config.active = active;
        }
    }

    // === Water Volume Management ===

    /// Add a water volume
    pub fn add_water_volume(&mut self, position: Vec3, half_extents: Vec3) -> WaterVolumeId {
        let id = WaterVolumeId(self.next_water_id);
        self.next_water_id += 1;
        self.water_volumes
            .insert(id, WaterVolume::new(id, position, half_extents));
        id
    }

    /// Remove a water volume
    pub fn remove_water_volume(&mut self, id: WaterVolumeId) -> bool {
        self.water_volumes.remove(&id).is_some()
    }

    /// Get a water volume
    pub fn get_water_volume(&self, id: WaterVolumeId) -> Option<&WaterVolume> {
        self.water_volumes.get(&id)
    }

    /// Get a mutable water volume
    pub fn get_water_volume_mut(&mut self, id: WaterVolumeId) -> Option<&mut WaterVolume> {
        self.water_volumes.get_mut(&id)
    }

    // === Gust Events ===

    /// Trigger a gust event
    pub fn trigger_gust(&mut self, direction: Vec3, strength: f32, duration: f32) {
        self.gusts.push(GustEvent::new(direction, strength, duration));
    }

    /// Get current gust force
    pub fn current_gust_force(&self) -> Vec3 {
        self.gusts
            .iter()
            .map(|g| g.direction * g.current_strength())
            .fold(Vec3::ZERO, |a, b| a + b)
    }

    // === Force Calculations ===

    /// Calculate total wind force at a point
    pub fn wind_force_at(&self, point: Vec3, drag_coefficient: f32, cross_section: f32) -> Vec3 {
        let mut total = Vec3::ZERO;

        // Global wind
        if self.global_wind.length_squared() > 0.001 {
            let speed = self.global_wind.length() * self.global_wind_strength;
            let force = 0.5 * 1.225 * speed * speed * drag_coefficient * cross_section;
            total += self.global_wind.normalize() * force;
        }

        // Wind zones
        for zone in self.wind_zones.values() {
            total += zone.wind_force_at(point, drag_coefficient, cross_section);
        }

        // Gusts
        let gust = self.current_gust_force();
        if gust.length_squared() > 0.001 {
            let speed = gust.length();
            let force = 0.5 * 1.225 * speed * speed * drag_coefficient * cross_section;
            total += gust.normalize() * force;
        }

        total
    }

    /// Calculate buoyancy force at a point
    pub fn buoyancy_force_at(&self, center: Vec3, volume: f32, radius: f32) -> Vec3 {
        let mut total = Vec3::ZERO;

        for water in self.water_volumes.values() {
            let submerged = water.sphere_submerged_fraction(center, radius);
            if submerged > 0.0 {
                total += water.buoyancy_force(center, volume, submerged);
            }
        }

        total
    }

    /// Check if a point is underwater
    pub fn is_underwater(&self, point: Vec3) -> bool {
        for water in self.water_volumes.values() {
            if water.contains(point) {
                let surface = water.surface_height_at(point.x, point.z);
                if point.y < surface {
                    return true;
                }
            }
        }
        false
    }

    /// Get water drag at a point
    pub fn water_drag_at(&self, point: Vec3) -> (f32, f32) {
        for water in self.water_volumes.values() {
            if water.contains(point) {
                let surface = water.surface_height_at(point.x, point.z);
                if point.y < surface {
                    return (water.linear_drag, water.angular_drag);
                }
            }
        }
        (0.0, 0.0)
    }

    /// Get water current at a point
    pub fn water_current_at(&self, point: Vec3) -> Vec3 {
        for water in self.water_volumes.values() {
            if water.contains(point) {
                let surface = water.surface_height_at(point.x, point.z);
                if point.y < surface {
                    return water.current;
                }
            }
        }
        Vec3::ZERO
    }

    // === Update ===

    /// Update all environmental effects
    pub fn update(&mut self, dt: f32) {
        // Update wind zones
        for zone in self.wind_zones.values_mut() {
            zone.update(dt);
        }

        // Update water volumes
        for water in self.water_volumes.values_mut() {
            water.update(dt);
        }

        // Update gusts and remove finished ones
        for gust in &mut self.gusts {
            gust.update(dt);
        }
        self.gusts.retain(|g| !g.is_finished());
    }

    /// Get number of active wind zones
    pub fn wind_zone_count(&self) -> usize {
        self.wind_zones.len()
    }

    /// Get number of water volumes
    pub fn water_volume_count(&self) -> usize {
        self.water_volumes.len()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wind_zone_creation() {
        let config = WindZoneConfig::default();
        let zone = WindZone::new(WindZoneId(1), config);
        assert_eq!(zone.id, WindZoneId(1));
        assert!(zone.config.active);
    }

    #[test]
    fn test_global_wind_zone_contains() {
        let config = WindZoneConfig {
            shape: WindZoneShape::Global,
            ..Default::default()
        };
        let zone = WindZone::new(WindZoneId(1), config);

        // Global zone contains everything
        assert!(zone.contains(Vec3::ZERO));
        assert!(zone.contains(Vec3::new(1000.0, 1000.0, 1000.0)));
        assert!(zone.contains(Vec3::new(-500.0, 200.0, -300.0)));
    }

    #[test]
    fn test_box_wind_zone_contains() {
        let config = WindZoneConfig {
            position: Vec3::new(10.0, 5.0, 0.0),
            shape: WindZoneShape::Box {
                half_extents: Vec3::new(5.0, 5.0, 5.0),
            },
            ..Default::default()
        };
        let zone = WindZone::new(WindZoneId(1), config);

        // Inside
        assert!(zone.contains(Vec3::new(10.0, 5.0, 0.0)));
        assert!(zone.contains(Vec3::new(14.0, 5.0, 0.0)));

        // Outside
        assert!(!zone.contains(Vec3::new(16.0, 5.0, 0.0)));
        assert!(!zone.contains(Vec3::ZERO));
    }

    #[test]
    fn test_sphere_wind_zone_contains() {
        let config = WindZoneConfig {
            position: Vec3::new(0.0, 10.0, 0.0),
            shape: WindZoneShape::Sphere { radius: 5.0 },
            ..Default::default()
        };
        let zone = WindZone::new(WindZoneId(1), config);

        // Inside
        assert!(zone.contains(Vec3::new(0.0, 10.0, 0.0)));
        assert!(zone.contains(Vec3::new(0.0, 14.0, 0.0)));

        // Outside
        assert!(!zone.contains(Vec3::new(0.0, 16.0, 0.0)));
        assert!(!zone.contains(Vec3::ZERO));
    }

    #[test]
    fn test_cylinder_wind_zone_contains() {
        let config = WindZoneConfig {
            position: Vec3::new(0.0, 5.0, 0.0),
            shape: WindZoneShape::Cylinder {
                radius: 3.0,
                height: 10.0,
            },
            ..Default::default()
        };
        let zone = WindZone::new(WindZoneId(1), config);

        // Inside
        assert!(zone.contains(Vec3::new(0.0, 5.0, 0.0)));
        assert!(zone.contains(Vec3::new(2.0, 5.0, 0.0)));

        // Outside (beyond radius)
        assert!(!zone.contains(Vec3::new(4.0, 5.0, 0.0)));
        // Outside (beyond height)
        assert!(!zone.contains(Vec3::new(0.0, 15.0, 0.0)));
    }

    #[test]
    fn test_directional_wind_force() {
        let config = WindZoneConfig {
            shape: WindZoneShape::Global,
            wind_type: WindType::Directional,
            direction: Vec3::new(1.0, 0.0, 0.0),
            strength: 10.0,
            ..Default::default()
        };
        let zone = WindZone::new(WindZoneId(1), config);

        let force = zone.wind_force_at(Vec3::ZERO, 1.0, 1.0);
        assert!(force.x > 0.0, "Wind should push in +X direction");
        assert!(force.y.abs() < 0.01);
        assert!(force.z.abs() < 0.01);
    }

    #[test]
    fn test_vortex_wind_force() {
        let config = WindZoneConfig {
            position: Vec3::ZERO,
            shape: WindZoneShape::Sphere { radius: 100.0 },
            wind_type: WindType::Vortex {
                tangential_speed: 10.0,
                inward_pull: 5.0,
                updraft: 2.0,
            },
            ..Default::default()
        };
        let zone = WindZone::new(WindZoneId(1), config);

        // Test point to the right of center
        let force = zone.wind_force_at(Vec3::new(10.0, 0.0, 0.0), 1.0, 1.0);

        // Should have inward component (toward center, so -X)
        // Should have updraft (+ Y)
        // Should have tangential component
        assert!(force.length() > 0.0, "Vortex should produce force");
    }

    #[test]
    fn test_turbulent_wind_update() {
        let config = WindZoneConfig {
            wind_type: WindType::Turbulent {
                intensity: 1.0,
                frequency: 1.0,
            },
            ..Default::default()
        };
        let mut zone = WindZone::new(WindZoneId(1), config);

        let _initial_offset = zone.gust_offset;
        zone.update(0.5);

        // Gust offset should change after update
        assert!(zone.noise_phase > 0.0);
        // Note: gust_offset may still be zero at certain phases
    }

    #[test]
    fn test_wind_falloff() {
        let config = WindZoneConfig {
            position: Vec3::ZERO,
            shape: WindZoneShape::Sphere { radius: 10.0 },
            falloff: 1.0,
            strength: 10.0,
            ..Default::default()
        };
        let zone = WindZone::new(WindZoneId(1), config);

        let center_force = zone.wind_force_at(Vec3::ZERO, 1.0, 1.0);
        let edge_force = zone.wind_force_at(Vec3::new(9.0, 0.0, 0.0), 1.0, 1.0);

        // Force should be stronger at center
        assert!(
            center_force.length() > edge_force.length(),
            "Center force should be stronger with falloff"
        );
    }

    #[test]
    fn test_inactive_wind_zone() {
        let config = WindZoneConfig {
            active: false,
            strength: 100.0,
            ..Default::default()
        };
        let zone = WindZone::new(WindZoneId(1), config);

        let force = zone.wind_force_at(Vec3::ZERO, 1.0, 1.0);
        assert_eq!(force, Vec3::ZERO, "Inactive zone should produce no force");
    }

    #[test]
    fn test_gust_event_lifecycle() {
        let mut gust = GustEvent::new(Vec3::X, 10.0, 1.0);

        // Initial state (at t=0, envelope starts from 0 due to attack ramp)
        assert!(!gust.is_finished());

        // Small step forward - should have strength now
        gust.update(0.1);
        assert!(gust.current_strength() > 0.0, "Gust should have strength after starting");

        // Mid-gust
        gust.update(0.4);
        assert!(!gust.is_finished());

        // After duration
        gust.update(0.6);
        assert!(gust.is_finished());
        assert_eq!(gust.current_strength(), 0.0);
    }

    #[test]
    fn test_water_volume_creation() {
        let water = WaterVolume::new(
            WaterVolumeId(1),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 5.0, 10.0),
        );

        assert_eq!(water.surface_height, 5.0);
        assert_eq!(water.density, 1000.0);
    }

    #[test]
    fn test_water_volume_contains() {
        let water = WaterVolume::new(
            WaterVolumeId(1),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 5.0, 10.0),
        );

        assert!(water.contains(Vec3::ZERO));
        assert!(water.contains(Vec3::new(5.0, 2.0, 5.0)));
        assert!(!water.contains(Vec3::new(15.0, 0.0, 0.0)));
    }

    #[test]
    fn test_sphere_submerged_fraction() {
        let water = WaterVolume::new(
            WaterVolumeId(1),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(100.0, 10.0, 100.0),
        );

        // Fully above water
        let above = water.sphere_submerged_fraction(Vec3::new(0.0, 20.0, 0.0), 2.0);
        assert_eq!(above, 0.0);

        // Fully submerged
        let submerged = water.sphere_submerged_fraction(Vec3::new(0.0, 0.0, 0.0), 2.0);
        assert_eq!(submerged, 1.0);

        // Half submerged (center at surface)
        let half = water.sphere_submerged_fraction(Vec3::new(0.0, 10.0, 0.0), 2.0);
        assert!(half > 0.4 && half < 0.6, "Should be approximately half submerged");
    }

    #[test]
    fn test_buoyancy_force() {
        let water = WaterVolume::new(
            WaterVolumeId(1),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(100.0, 100.0, 100.0),
        );

        // 1 m³ sphere fully submerged
        let force = water.buoyancy_force(Vec3::ZERO, 1.0, 1.0);

        // F = ρVg = 1000 * 1 * 9.81 ≈ 9810 N
        assert!(force.y > 9000.0 && force.y < 10000.0);
    }

    #[test]
    fn test_environment_manager_wind_zones() {
        let mut manager = EnvironmentManager::new();

        let id = manager.add_wind_zone(WindZoneConfig::default());
        assert_eq!(manager.wind_zone_count(), 1);

        assert!(manager.get_wind_zone(id).is_some());

        manager.set_wind_zone_active(id, false);
        assert!(!manager.get_wind_zone(id).unwrap().config.active);

        assert!(manager.remove_wind_zone(id));
        assert_eq!(manager.wind_zone_count(), 0);
    }

    #[test]
    fn test_environment_manager_water_volumes() {
        let mut manager = EnvironmentManager::new();

        let id = manager.add_water_volume(Vec3::ZERO, Vec3::new(10.0, 5.0, 10.0));
        assert_eq!(manager.water_volume_count(), 1);

        assert!(manager.get_water_volume(id).is_some());
        assert!(manager.remove_water_volume(id));
        assert_eq!(manager.water_volume_count(), 0);
    }

    #[test]
    fn test_environment_manager_gusts() {
        let mut manager = EnvironmentManager::new();

        manager.trigger_gust(Vec3::X, 10.0, 1.0);

        // Advance time slightly for gust attack envelope to ramp up
        manager.update(0.1);
        let gust_force = manager.current_gust_force();
        assert!(gust_force.length() > 0.0, "Gust should produce force after ramp");

        // After duration, gust should be removed
        manager.update(1.1);
        let gust_force = manager.current_gust_force();
        assert_eq!(gust_force, Vec3::ZERO);
    }

    #[test]
    fn test_is_underwater() {
        let mut manager = EnvironmentManager::new();
        manager.add_water_volume(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 5.0, 10.0));

        // Below surface (surface at y=5)
        assert!(manager.is_underwater(Vec3::new(0.0, 3.0, 0.0)));

        // Above surface
        assert!(!manager.is_underwater(Vec3::new(0.0, 10.0, 0.0)));

        // Outside volume
        assert!(!manager.is_underwater(Vec3::new(20.0, 3.0, 0.0)));
    }

    #[test]
    fn test_combined_wind_force() {
        let mut manager = EnvironmentManager::new();

        // Global wind
        manager.global_wind = Vec3::new(5.0, 0.0, 0.0);
        manager.global_wind_strength = 1.0;

        // Add a wind zone
        manager.add_wind_zone(WindZoneConfig {
            direction: Vec3::new(0.0, 0.0, 5.0),
            strength: 5.0,
            ..Default::default()
        });

        let force = manager.wind_force_at(Vec3::ZERO, 1.0, 1.0);

        // Should have components from both sources
        assert!(force.x > 0.0, "Should have global wind X component");
        assert!(force.z > 0.0, "Should have zone wind Z component");
    }

    #[test]
    fn test_water_waves() {
        let mut water = WaterVolume::new(
            WaterVolumeId(1),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(100.0, 10.0, 100.0),
        );

        water.wave_amplitude = 1.0;
        water.wave_frequency = 1.0;

        let height1 = water.surface_height_at(0.0, 0.0);
        water.update(0.25); // Quarter wave period
        let height2 = water.surface_height_at(0.0, 0.0);

        // Heights should differ due to wave motion
        assert!(
            (height1 - height2).abs() > 0.01,
            "Wave should cause surface height variation"
        );
    }

    #[test]
    fn test_wind_zone_turbulent_force() {
        let config = WindZoneConfig {
            wind_type: WindType::Turbulent {
                intensity: 1.0,
                frequency: 1.0,
            },
            ..Default::default()
        };
        let mut zone = WindZone::new(WindZoneId(1), config);
        zone.gust_offset = Vec3::new(1.0, 1.0, 1.0);

        let force = zone.wind_force_at(Vec3::ZERO, 1.0, 1.0);
        assert!(force.length() > 0.0);
    }

    #[test]
    fn test_wind_zone_falloff_shapes() {
        // Box falloff
        let config_box = WindZoneConfig {
            shape: WindZoneShape::Box { half_extents: Vec3::ONE * 10.0 },
            falloff: 1.0,
            ..Default::default()
        };
        let zone_box = WindZone::new(WindZoneId(1), config_box);
        assert!(zone_box.calculate_falloff(Vec3::ZERO) == 1.0);
        assert!(zone_box.calculate_falloff(Vec3::ONE * 5.0) < 1.0);

        // Cylinder falloff
        let config_cyl = WindZoneConfig {
            shape: WindZoneShape::Cylinder { radius: 10.0, height: 10.0 },
            falloff: 1.0,
            ..Default::default()
        };
        let zone_cyl = WindZone::new(WindZoneId(2), config_cyl);
        assert!(zone_cyl.calculate_falloff(Vec3::ZERO) == 1.0);
        assert!(zone_cyl.calculate_falloff(Vec3::new(5.0, 0.0, 0.0)) < 1.0);
    }

    #[test]
    fn test_environment_manager_buoyancy_at() {
        let mut manager = EnvironmentManager::new();
        manager.add_water_volume(Vec3::new(0.0, -5.0, 0.0), Vec3::new(10.0, 5.0, 10.0));

        let force = manager.buoyancy_force_at(Vec3::new(0.0, -1.0, 0.0), 1.0, 1.0);
        assert!(force.y > 0.0);
    }

    #[test]
    fn test_environment_manager_mut_access() {
        let mut manager = EnvironmentManager::new();
        let w_id = manager.add_wind_zone(WindZoneConfig::default());
        let v_id = manager.add_water_volume(Vec3::ZERO, Vec3::ONE);

        assert!(manager.get_wind_zone_mut(w_id).is_some());
        assert!(manager.get_water_volume_mut(v_id).is_some());
    }

    #[test]
    fn test_wind_defaults() {
        let _ = WindZoneShape::default();
        let _ = WindType::default();
    }
}
