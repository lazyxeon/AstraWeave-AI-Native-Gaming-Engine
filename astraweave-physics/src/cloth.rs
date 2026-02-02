//! Cloth Simulation System
//!
//! Verlet integration-based cloth simulation:
//! - Particle system with distance constraints
//! - Collision with rigid bodies (spheres, capsules)
//! - Wind interaction
//! - Pinned particles for attachment points

use glam::Vec3;
use std::collections::HashMap;

/// Unique identifier for cloth instances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClothId(pub u64);

/// A particle in the cloth simulation
#[derive(Debug, Clone)]
pub struct ClothParticle {
    /// Current position
    pub position: Vec3,
    /// Previous position (for Verlet integration)
    pub prev_position: Vec3,
    /// Accumulated forces this frame
    pub acceleration: Vec3,
    /// Inverse mass (0 = pinned/infinite mass)
    pub inv_mass: f32,
    /// Whether this particle is pinned (fixed position)
    pub pinned: bool,
}

impl ClothParticle {
    /// Create a new particle
    pub fn new(position: Vec3, mass: f32) -> Self {
        Self {
            position,
            prev_position: position,
            acceleration: Vec3::ZERO,
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            pinned: false,
        }
    }

    /// Create a pinned particle
    pub fn pinned(position: Vec3) -> Self {
        Self {
            position,
            prev_position: position,
            acceleration: Vec3::ZERO,
            inv_mass: 0.0,
            pinned: true,
        }
    }

    /// Apply force to particle
    pub fn apply_force(&mut self, force: Vec3) {
        if !self.pinned {
            self.acceleration += force * self.inv_mass;
        }
    }

    /// Integrate using Verlet integration
    pub fn integrate(&mut self, dt: f32, damping: f32) {
        if self.pinned {
            return;
        }

        let velocity = self.position - self.prev_position;
        self.prev_position = self.position;
        self.position += velocity * damping + self.acceleration * dt * dt;
        self.acceleration = Vec3::ZERO;
    }

    /// Get velocity
    pub fn velocity(&self) -> Vec3 {
        self.position - self.prev_position
    }
}

/// A distance constraint between two particles
#[derive(Debug, Clone, Copy)]
pub struct DistanceConstraint {
    /// First particle index
    pub p1: usize,
    /// Second particle index
    pub p2: usize,
    /// Rest length
    pub rest_length: f32,
    /// Stiffness (0-1, higher = stiffer)
    pub stiffness: f32,
}

impl DistanceConstraint {
    /// Create a new constraint
    pub fn new(p1: usize, p2: usize, rest_length: f32) -> Self {
        Self {
            p1,
            p2,
            rest_length,
            stiffness: 1.0,
        }
    }

    /// Solve the constraint
    pub fn solve(&self, particles: &mut [ClothParticle]) {
        let p1 = &particles[self.p1];
        let p2 = &particles[self.p2];

        let delta = p2.position - p1.position;
        let current_length = delta.length();

        if current_length < 0.0001 {
            return;
        }

        let diff = (current_length - self.rest_length) / current_length;
        let correction = delta * diff * 0.5 * self.stiffness;

        let w1 = p1.inv_mass;
        let w2 = p2.inv_mass;
        let total_weight = w1 + w2;

        if total_weight > 0.0 {
            if !particles[self.p1].pinned {
                particles[self.p1].position += correction * (w1 / total_weight);
            }
            if !particles[self.p2].pinned {
                particles[self.p2].position -= correction * (w2 / total_weight);
            }
        }
    }
}

/// Collision shape for cloth collision
#[derive(Debug, Clone, Copy)]
pub enum ClothCollider {
    /// Sphere collider
    Sphere { center: Vec3, radius: f32 },
    /// Capsule collider
    Capsule {
        start: Vec3,
        end: Vec3,
        radius: f32,
    },
    /// Infinite plane
    Plane { point: Vec3, normal: Vec3 },
}

impl ClothCollider {
    /// Resolve collision with a particle
    pub fn resolve_collision(&self, particle: &mut ClothParticle, friction: f32) {
        if particle.pinned {
            return;
        }

        match *self {
            ClothCollider::Sphere { center, radius } => {
                let to_particle = particle.position - center;
                let dist = to_particle.length();

                if dist < radius {
                    let normal = to_particle.normalize_or_zero();
                    let penetration = radius - dist;
                    particle.position += normal * penetration;

                    // Apply friction
                    let velocity = particle.velocity();
                    let normal_vel = velocity.dot(normal) * normal;
                    let tangent_vel = velocity - normal_vel;
                    particle.prev_position =
                        particle.position - (normal_vel + tangent_vel * (1.0 - friction));
                }
            }
            ClothCollider::Capsule { start, end, radius } => {
                let axis = end - start;
                let axis_length = axis.length();
                if axis_length < 0.0001 {
                    // Degenerate capsule = sphere
                    let sphere = ClothCollider::Sphere {
                        center: start,
                        radius,
                    };
                    sphere.resolve_collision(particle, friction);
                    return;
                }

                let axis_dir = axis / axis_length;
                let to_particle = particle.position - start;
                let t = to_particle.dot(axis_dir).clamp(0.0, axis_length);
                let closest = start + axis_dir * t;

                let to_particle = particle.position - closest;
                let dist = to_particle.length();

                if dist < radius {
                    let normal = to_particle.normalize_or_zero();
                    let penetration = radius - dist;
                    particle.position += normal * penetration;

                    let velocity = particle.velocity();
                    let normal_vel = velocity.dot(normal) * normal;
                    let tangent_vel = velocity - normal_vel;
                    particle.prev_position =
                        particle.position - (normal_vel + tangent_vel * (1.0 - friction));
                }
            }
            ClothCollider::Plane { point, normal } => {
                let to_particle = particle.position - point;
                let dist = to_particle.dot(normal);

                if dist < 0.0 {
                    particle.position -= normal * dist;

                    let velocity = particle.velocity();
                    let normal_vel = velocity.dot(normal) * normal;
                    let tangent_vel = velocity - normal_vel;
                    particle.prev_position =
                        particle.position - (normal_vel + tangent_vel * (1.0 - friction));
                }
            }
        }
    }
}

/// Configuration for cloth simulation
#[derive(Debug, Clone)]
pub struct ClothConfig {
    /// Width in particles
    pub width: usize,
    /// Height in particles
    pub height: usize,
    /// Spacing between particles
    pub spacing: f32,
    /// Mass per particle
    pub particle_mass: f32,
    /// Constraint stiffness (0-1)
    pub stiffness: f32,
    /// Velocity damping (0-1, lower = more damping)
    pub damping: f32,
    /// Constraint solver iterations
    pub solver_iterations: usize,
    /// Gravity
    pub gravity: Vec3,
    /// Wind force
    pub wind: Vec3,
    /// Air resistance
    pub air_resistance: f32,
}

impl Default for ClothConfig {
    fn default() -> Self {
        Self {
            width: 20,
            height: 20,
            spacing: 0.1,
            particle_mass: 0.1,
            stiffness: 0.8,
            damping: 0.98,
            solver_iterations: 3,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            wind: Vec3::ZERO,
            air_resistance: 0.01,
        }
    }
}

/// A cloth instance
#[derive(Debug, Clone)]
pub struct Cloth {
    pub id: ClothId,
    pub config: ClothConfig,
    pub particles: Vec<ClothParticle>,
    pub constraints: Vec<DistanceConstraint>,
    pub colliders: Vec<ClothCollider>,
    /// Collision friction
    pub collision_friction: f32,
}

impl Cloth {
    /// Create a new cloth from config
    pub fn new(id: ClothId, config: ClothConfig, origin: Vec3) -> Self {
        let mut particles = Vec::with_capacity(config.width * config.height);
        let mut constraints = Vec::new();

        // Create particles in a grid
        for y in 0..config.height {
            for x in 0..config.width {
                let pos = origin
                    + Vec3::new(x as f32 * config.spacing, 0.0, y as f32 * config.spacing);
                particles.push(ClothParticle::new(pos, config.particle_mass));
            }
        }

        // Create structural constraints (horizontal and vertical)
        for y in 0..config.height {
            for x in 0..config.width {
                let idx = y * config.width + x;

                // Horizontal constraint
                if x < config.width - 1 {
                    let mut c = DistanceConstraint::new(idx, idx + 1, config.spacing);
                    c.stiffness = config.stiffness;
                    constraints.push(c);
                }

                // Vertical constraint
                if y < config.height - 1 {
                    let mut c = DistanceConstraint::new(idx, idx + config.width, config.spacing);
                    c.stiffness = config.stiffness;
                    constraints.push(c);
                }

                // Shear constraints (diagonals)
                if x < config.width - 1 && y < config.height - 1 {
                    let diag_len = config.spacing * std::f32::consts::SQRT_2;

                    let mut c1 = DistanceConstraint::new(idx, idx + config.width + 1, diag_len);
                    c1.stiffness = config.stiffness * 0.5;
                    constraints.push(c1);

                    let mut c2 = DistanceConstraint::new(idx + 1, idx + config.width, diag_len);
                    c2.stiffness = config.stiffness * 0.5;
                    constraints.push(c2);
                }

                // Bend constraints (skip one particle)
                if x < config.width - 2 {
                    let mut c =
                        DistanceConstraint::new(idx, idx + 2, config.spacing * 2.0);
                    c.stiffness = config.stiffness * 0.3;
                    constraints.push(c);
                }
                if y < config.height - 2 {
                    let mut c = DistanceConstraint::new(
                        idx,
                        idx + config.width * 2,
                        config.spacing * 2.0,
                    );
                    c.stiffness = config.stiffness * 0.3;
                    constraints.push(c);
                }
            }
        }

        Self {
            id,
            config,
            particles,
            constraints,
            colliders: Vec::new(),
            collision_friction: 0.5,
        }
    }

    /// Pin particles at the top edge
    pub fn pin_top_edge(&mut self) {
        for x in 0..self.config.width {
            self.particles[x].pinned = true;
            self.particles[x].inv_mass = 0.0;
        }
    }

    /// Pin specific corners
    pub fn pin_corners(&mut self) {
        let w = self.config.width;
        let _h = self.config.height;

        // Top-left
        self.particles[0].pinned = true;
        self.particles[0].inv_mass = 0.0;

        // Top-right
        self.particles[w - 1].pinned = true;
        self.particles[w - 1].inv_mass = 0.0;
    }

    /// Pin a specific particle by index
    pub fn pin_particle(&mut self, index: usize) {
        if index < self.particles.len() {
            self.particles[index].pinned = true;
            self.particles[index].inv_mass = 0.0;
        }
    }

    /// Unpin a particle
    pub fn unpin_particle(&mut self, index: usize) {
        if index < self.particles.len() {
            self.particles[index].pinned = false;
            self.particles[index].inv_mass = 1.0 / self.config.particle_mass;
        }
    }

    /// Move a pinned particle
    pub fn move_pinned(&mut self, index: usize, new_position: Vec3) {
        if index < self.particles.len() && self.particles[index].pinned {
            self.particles[index].position = new_position;
            self.particles[index].prev_position = new_position;
        }
    }

    /// Add a collider
    pub fn add_collider(&mut self, collider: ClothCollider) {
        self.colliders.push(collider);
    }

    /// Clear all colliders
    pub fn clear_colliders(&mut self) {
        self.colliders.clear();
    }

    /// Get particle index from grid position
    pub fn particle_index(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.config.width && y < self.config.height {
            Some(y * self.config.width + x)
        } else {
            None
        }
    }

    /// Get particle position
    pub fn particle_position(&self, index: usize) -> Option<Vec3> {
        self.particles.get(index).map(|p| p.position)
    }

    /// Calculate cloth normal at a particle (for wind)
    fn particle_normal(&self, x: usize, y: usize) -> Vec3 {
        let idx = y * self.config.width + x;
        let center = self.particles[idx].position;

        let mut normal = Vec3::ZERO;
        let mut count = 0;

        // Get neighboring particles for normal calculation
        let neighbors = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];

        for i in 0..4 {
            let (nx, ny) = neighbors[i];
            let (nx2, ny2) = neighbors[(i + 1) % 4];

            if nx < self.config.width
                && ny < self.config.height
                && nx2 < self.config.width
                && ny2 < self.config.height
            {
                let idx1 = ny * self.config.width + nx;
                let idx2 = ny2 * self.config.width + nx2;

                let v1 = self.particles[idx1].position - center;
                let v2 = self.particles[idx2].position - center;
                normal += v1.cross(v2);
                count += 1;
            }
        }

        if count > 0 {
            normal.normalize_or_zero()
        } else {
            Vec3::Y
        }
    }

    /// Update cloth simulation
    pub fn update(&mut self, dt: f32) {
        // Pre-compute normals for wind calculation
        let mut normals = Vec::new();
        if self.config.wind.length_squared() > 0.001 {
            for y in 0..self.config.height {
                for x in 0..self.config.width {
                    normals.push(self.particle_normal(x, y));
                }
            }
        }

        // Apply forces
        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let idx = y * self.config.width + x;
                let inv_mass = self.particles[idx].inv_mass;

                // Gravity
                let gravity_force = self.config.gravity * (1.0 / inv_mass.max(0.001));
                self.particles[idx].apply_force(gravity_force);

                // Wind (affected by particle normal)
                if !normals.is_empty() {
                    let normal = normals[idx];
                    let wind_effect = self.config.wind.dot(normal).abs();
                    let wind_force = self.config.wind * wind_effect * (1.0 / inv_mass.max(0.001));
                    self.particles[idx].apply_force(wind_force);
                }

                // Air resistance
                let velocity = self.particles[idx].velocity();
                let drag = -velocity * self.config.air_resistance * (1.0 / inv_mass.max(0.001));
                self.particles[idx].apply_force(drag);
            }
        }

        // Integrate
        for particle in &mut self.particles {
            particle.integrate(dt, self.config.damping);
        }

        // Solve constraints
        for _ in 0..self.config.solver_iterations {
            for constraint in &self.constraints {
                constraint.solve(&mut self.particles);
            }
        }

        // Resolve collisions
        for particle in &mut self.particles {
            for collider in &self.colliders {
                collider.resolve_collision(particle, self.collision_friction);
            }
        }
    }

    /// Get all particle positions as a flat array
    pub fn get_positions(&self) -> Vec<Vec3> {
        self.particles.iter().map(|p| p.position).collect()
    }

    /// Get triangle indices for rendering
    pub fn get_indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();
        let w = self.config.width as u32;

        for y in 0..(self.config.height - 1) as u32 {
            for x in 0..(self.config.width - 1) as u32 {
                let idx = y * w + x;

                // First triangle
                indices.push(idx);
                indices.push(idx + 1);
                indices.push(idx + w);

                // Second triangle
                indices.push(idx + 1);
                indices.push(idx + w + 1);
                indices.push(idx + w);
            }
        }

        indices
    }

    /// Get particle count
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Get constraint count
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }
}

/// Manager for multiple cloth simulations
#[derive(Debug, Default)]
pub struct ClothManager {
    cloths: HashMap<ClothId, Cloth>,
    next_id: u64,
}

impl ClothManager {
    /// Create a new cloth manager
    pub fn new() -> Self {
        Self {
            cloths: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new cloth
    pub fn create(&mut self, config: ClothConfig, origin: Vec3) -> ClothId {
        let id = ClothId(self.next_id);
        self.next_id += 1;
        self.cloths.insert(id, Cloth::new(id, config, origin));
        id
    }

    /// Remove a cloth
    pub fn remove(&mut self, id: ClothId) -> bool {
        self.cloths.remove(&id).is_some()
    }

    /// Get a cloth
    pub fn get(&self, id: ClothId) -> Option<&Cloth> {
        self.cloths.get(&id)
    }

    /// Get a mutable cloth
    pub fn get_mut(&mut self, id: ClothId) -> Option<&mut Cloth> {
        self.cloths.get_mut(&id)
    }

    /// Update all cloths
    pub fn update(&mut self, dt: f32) {
        for cloth in self.cloths.values_mut() {
            cloth.update(dt);
        }
    }

    /// Get cloth count
    pub fn count(&self) -> usize {
        self.cloths.len()
    }

    /// Iterate over all cloths
    pub fn iter(&self) -> impl Iterator<Item = &Cloth> {
        self.cloths.values()
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_creation() {
        let particle = ClothParticle::new(Vec3::new(1.0, 2.0, 3.0), 0.5);
        assert_eq!(particle.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(particle.inv_mass, 2.0); // 1/0.5
        assert!(!particle.pinned);
    }

    #[test]
    fn test_pinned_particle() {
        let particle = ClothParticle::pinned(Vec3::ZERO);
        assert!(particle.pinned);
        assert_eq!(particle.inv_mass, 0.0);
    }

    #[test]
    fn test_particle_force() {
        let mut particle = ClothParticle::new(Vec3::ZERO, 1.0);
        particle.apply_force(Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(particle.acceleration, Vec3::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_pinned_particle_no_force() {
        let mut particle = ClothParticle::pinned(Vec3::ZERO);
        particle.apply_force(Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(particle.acceleration, Vec3::ZERO);
    }

    #[test]
    fn test_particle_integrate() {
        let mut particle = ClothParticle::new(Vec3::ZERO, 1.0);
        particle.apply_force(Vec3::new(0.0, -10.0, 0.0)); // Gravity-like

        particle.integrate(0.016, 1.0);

        // Should have moved downward
        assert!(particle.position.y < 0.0);
    }

    #[test]
    fn test_distance_constraint() {
        let mut particles = vec![
            ClothParticle::new(Vec3::new(0.0, 0.0, 0.0), 1.0),
            ClothParticle::new(Vec3::new(2.0, 0.0, 0.0), 1.0), // Too far apart
        ];

        let constraint = DistanceConstraint::new(0, 1, 1.0); // Rest length = 1

        constraint.solve(&mut particles);

        // Particles should have moved closer
        let dist = (particles[1].position - particles[0].position).length();
        assert!(dist < 2.0, "Particles should be closer after constraint");
    }

    #[test]
    fn test_cloth_creation() {
        let config = ClothConfig {
            width: 5,
            height: 5,
            spacing: 0.1,
            ..Default::default()
        };

        let cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);

        assert_eq!(cloth.particle_count(), 25);
        assert!(cloth.constraint_count() > 0);
    }

    #[test]
    fn test_cloth_pin_top_edge() {
        let config = ClothConfig {
            width: 5,
            height: 5,
            ..Default::default()
        };

        let mut cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);
        cloth.pin_top_edge();

        // All particles in first row should be pinned
        for x in 0..5 {
            assert!(cloth.particles[x].pinned);
        }
    }

    #[test]
    fn test_cloth_pin_corners() {
        let config = ClothConfig {
            width: 5,
            height: 5,
            ..Default::default()
        };

        let mut cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);
        cloth.pin_corners();

        assert!(cloth.particles[0].pinned); // Top-left
        assert!(cloth.particles[4].pinned); // Top-right
    }

    #[test]
    fn test_cloth_move_pinned() {
        let config = ClothConfig::default();
        let mut cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);
        cloth.pin_particle(0);

        let new_pos = Vec3::new(5.0, 5.0, 5.0);
        cloth.move_pinned(0, new_pos);

        assert_eq!(cloth.particles[0].position, new_pos);
    }

    #[test]
    fn test_sphere_collider() {
        let collider = ClothCollider::Sphere {
            center: Vec3::ZERO,
            radius: 1.0,
        };

        let mut particle = ClothParticle::new(Vec3::new(0.5, 0.0, 0.0), 1.0);
        collider.resolve_collision(&mut particle, 0.5);

        // Particle should be pushed out of sphere
        let dist = particle.position.length();
        assert!(dist >= 0.99, "Particle should be at or outside sphere surface");
    }

    #[test]
    fn test_plane_collider() {
        let collider = ClothCollider::Plane {
            point: Vec3::ZERO,
            normal: Vec3::Y,
        };

        let mut particle = ClothParticle::new(Vec3::new(0.0, -0.5, 0.0), 1.0);
        collider.resolve_collision(&mut particle, 0.5);

        // Particle should be above plane
        assert!(particle.position.y >= 0.0);
    }

    #[test]
    fn test_cloth_update_gravity() {
        let config = ClothConfig {
            width: 3,
            height: 3,
            gravity: Vec3::new(0.0, -10.0, 0.0),
            ..Default::default()
        };

        let mut cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);

        // Get initial Y of unpinned particle
        let initial_y = cloth.particles[4].position.y; // Center particle

        // Update
        cloth.update(0.016);

        // Should have fallen
        assert!(cloth.particles[4].position.y < initial_y);
    }

    #[test]
    fn test_cloth_with_wind() {
        let config = ClothConfig {
            width: 5,
            height: 5,
            wind: Vec3::new(10.0, 0.0, 0.0),
            ..Default::default()
        };

        let mut cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);
        cloth.pin_top_edge();

        let initial_x = cloth.particles[12].position.x; // Middle particle

        // Multiple updates
        for _ in 0..10 {
            cloth.update(0.016);
        }

        // Should have moved in wind direction
        assert!(cloth.particles[12].position.x > initial_x);
    }

    #[test]
    fn test_cloth_indices() {
        let config = ClothConfig {
            width: 3,
            height: 3,
            ..Default::default()
        };

        let cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);
        let indices = cloth.get_indices();

        // 3x3 grid = 2x2 quads = 4 quads × 2 triangles × 3 indices = 24 indices
        assert_eq!(indices.len(), 24);
    }

    #[test]
    fn test_cloth_manager() {
        let mut manager = ClothManager::new();

        let id = manager.create(ClothConfig::default(), Vec3::ZERO);
        assert_eq!(manager.count(), 1);

        assert!(manager.get(id).is_some());
        assert!(manager.remove(id));
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_cloth_manager_update() {
        let mut manager = ClothManager::new();

        manager.create(
            ClothConfig {
                width: 3,
                height: 3,
                ..Default::default()
            },
            Vec3::ZERO,
        );

        // Should not panic
        manager.update(0.016);
    }

    #[test]
    fn test_capsule_collider() {
        let collider = ClothCollider::Capsule {
            start: Vec3::new(0.0, 0.0, 0.0),
            end: Vec3::new(0.0, 2.0, 0.0),
            radius: 0.5,
        };

        let mut particle = ClothParticle::new(Vec3::new(0.2, 1.0, 0.0), 1.0);
        collider.resolve_collision(&mut particle, 0.5);

        // Particle should be pushed out of capsule
        let dist_from_axis = Vec3::new(particle.position.x, 0.0, particle.position.z).length();
        assert!(
            dist_from_axis >= 0.49,
            "Particle should be at capsule surface"
        );
    }

    #[test]
    fn test_particle_index() {
        let config = ClothConfig {
            width: 5,
            height: 4,
            ..Default::default()
        };

        let cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);

        assert_eq!(cloth.particle_index(0, 0), Some(0));
        assert_eq!(cloth.particle_index(4, 0), Some(4));
        assert_eq!(cloth.particle_index(0, 1), Some(5));
        assert_eq!(cloth.particle_index(4, 3), Some(19));
        assert_eq!(cloth.particle_index(5, 0), None); // Out of bounds
    }

    #[test]
    fn test_cloth_damping() {
        let config = ClothConfig {
            width: 3,
            height: 3,
            damping: 0.5, // Heavy damping
            gravity: Vec3::ZERO,
            ..Default::default()
        };

        let mut cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);

        // Give a particle some velocity
        cloth.particles[4].prev_position = cloth.particles[4].position - Vec3::X;

        cloth.update(0.016);

        // Velocity should be reduced due to damping
        let velocity = cloth.particles[4].velocity();
        assert!(velocity.length() < 1.0);
    }

    #[test]
    fn test_constraint_stiffness() {
        // Low stiffness constraint
        let mut particles_soft = vec![
            ClothParticle::new(Vec3::ZERO, 1.0),
            ClothParticle::new(Vec3::new(2.0, 0.0, 0.0), 1.0),
        ];
        let mut constraint_soft = DistanceConstraint::new(0, 1, 1.0);
        constraint_soft.stiffness = 0.1;
        constraint_soft.solve(&mut particles_soft);

        // High stiffness constraint
        let mut particles_stiff = vec![
            ClothParticle::new(Vec3::ZERO, 1.0),
            ClothParticle::new(Vec3::new(2.0, 0.0, 0.0), 1.0),
        ];
        let mut constraint_stiff = DistanceConstraint::new(0, 1, 1.0);
        constraint_stiff.stiffness = 1.0;
        constraint_stiff.solve(&mut particles_stiff);

        let dist_soft = (particles_soft[1].position - particles_soft[0].position).length();
        let dist_stiff = (particles_stiff[1].position - particles_stiff[0].position).length();

        // Stiff constraint should correct more
        assert!(dist_stiff < dist_soft);
    }

    // ============================================================================
    // TEARING TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_constraint_zero_rest_length() {
        let mut particles = vec![
            ClothParticle::new(Vec3::ZERO, 1.0),
            ClothParticle::new(Vec3::new(1.0, 0.0, 0.0), 1.0),
        ];
        
        // Zero rest length constraint
        let constraint = DistanceConstraint::new(0, 1, 0.0);
        constraint.solve(&mut particles);

        // Should pull particles together
        let dist = (particles[1].position - particles[0].position).length();
        assert!(dist < 1.0, "Particles should be closer");
    }

    #[test]
    fn test_constraint_stretched_beyond_limit() {
        let mut particles = vec![
            ClothParticle::new(Vec3::ZERO, 1.0),
            ClothParticle::new(Vec3::new(10.0, 0.0, 0.0), 1.0), // Very stretched
        ];
        
        let constraint = DistanceConstraint::new(0, 1, 1.0);
        
        // Multiple iterations should converge
        for _ in 0..10 {
            constraint.solve(&mut particles);
        }

        let dist = (particles[1].position - particles[0].position).length();
        assert!(dist < 2.0, "Should converge toward rest length");
    }

    // ============================================================================
    // WIND INTERACTION TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_wind_direction_affects_movement() {
        // Test X wind
        let config_x = ClothConfig {
            width: 3,
            height: 3,
            wind: Vec3::new(20.0, 0.0, 0.0),
            gravity: Vec3::ZERO,
            ..Default::default()
        };
        let mut cloth_x = Cloth::new(ClothId(1), config_x, Vec3::ZERO);
        cloth_x.pin_top_edge();
        
        // Test Z wind
        let config_z = ClothConfig {
            width: 3,
            height: 3,
            wind: Vec3::new(0.0, 0.0, 20.0),
            gravity: Vec3::ZERO,
            ..Default::default()
        };
        let mut cloth_z = Cloth::new(ClothId(2), config_z, Vec3::ZERO);
        cloth_z.pin_top_edge();
        
        for _ in 0..20 {
            cloth_x.update(0.016);
            cloth_z.update(0.016);
        }

        // Bottom center particle
        let bottom = cloth_x.config.width * (cloth_x.config.height - 1) + 1;
        
        // X wind should push X, Z wind should push Z
        assert!(cloth_x.particles[bottom].position.x > 0.0);
        assert!(cloth_z.particles[bottom].position.z > 0.0);
    }

    #[test]
    fn test_air_resistance_slows_movement() {
        let config_low_drag = ClothConfig {
            width: 3,
            height: 3,
            air_resistance: 0.0,
            gravity: Vec3::new(0.0, -10.0, 0.0),
            ..Default::default()
        };
        
        let config_high_drag = ClothConfig {
            width: 3,
            height: 3,
            air_resistance: 0.5,
            gravity: Vec3::new(0.0, -10.0, 0.0),
            ..Default::default()
        };
        
        let mut cloth_low = Cloth::new(ClothId(1), config_low_drag, Vec3::ZERO);
        let mut cloth_high = Cloth::new(ClothId(2), config_high_drag, Vec3::ZERO);
        
        for _ in 0..30 {
            cloth_low.update(0.016);
            cloth_high.update(0.016);
        }

        // High drag should fall slower
        let vel_low = cloth_low.particles[4].velocity().length();
        let vel_high = cloth_high.particles[4].velocity().length();
        
        assert!(vel_high < vel_low, "High drag should have lower velocity");
    }

    // ============================================================================
    // SELF-COLLISION TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_particles_same_position() {
        let mut particles = vec![
            ClothParticle::new(Vec3::ZERO, 1.0),
            ClothParticle::new(Vec3::ZERO, 1.0), // Same position
        ];
        
        let constraint = DistanceConstraint::new(0, 1, 1.0);
        
        // Should not panic or produce NaN
        constraint.solve(&mut particles);
        
        assert!(!particles[0].position.x.is_nan());
        assert!(!particles[1].position.x.is_nan());
    }

    // ============================================================================
    // EDGE CASE TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_single_particle_cloth() {
        // Note: Implementation requires at least 2x2 grid due to constraint generation
        // This test documents the minimum viable cloth size
        let config = ClothConfig {
            width: 2,
            height: 2,
            ..Default::default()
        };
        
        let cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);
        assert_eq!(cloth.particle_count(), 4);
        assert!(cloth.constraint_count() > 0);
    }

    #[test]
    fn test_cloth_1xN() {
        // Note: Implementation requires at least 2 in each dimension for proper constraint setup
        let config = ClothConfig {
            width: 5,
            height: 2,
            spacing: 0.2,
            ..Default::default()
        };
        
        let cloth = Cloth::new(ClothId(1), config, Vec3::ZERO);
        assert_eq!(cloth.particle_count(), 10);
        // Should have constraints
        assert!(cloth.constraint_count() > 0);
    }

    #[test]
    fn test_particle_velocity() {
        let mut particle = ClothParticle::new(Vec3::ZERO, 1.0);
        particle.prev_position = Vec3::new(-1.0, 0.0, 0.0);
        
        let velocity = particle.velocity();
        assert_eq!(velocity, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_unpin_particle() {
        let mut cloth = Cloth::new(ClothId(1), ClothConfig::default(), Vec3::ZERO);
        cloth.pin_particle(0);
        assert!(cloth.particles[0].pinned);
        
        cloth.unpin_particle(0);
        assert!(!cloth.particles[0].pinned);
        assert!(cloth.particles[0].inv_mass > 0.0);
    }

    #[test]
    fn test_clear_colliders() {
        let mut cloth = Cloth::new(ClothId(1), ClothConfig::default(), Vec3::ZERO);
        cloth.add_collider(ClothCollider::Sphere { center: Vec3::ZERO, radius: 1.0 });
        cloth.add_collider(ClothCollider::Plane { point: Vec3::ZERO, normal: Vec3::Y });
        
        assert_eq!(cloth.colliders.len(), 2);
        cloth.clear_colliders();
        assert_eq!(cloth.colliders.len(), 0);
    }

    #[test]
    fn test_particle_position_getter() {
        let cloth = Cloth::new(ClothId(1), ClothConfig::default(), Vec3::new(5.0, 5.0, 5.0));
        
        let pos = cloth.particle_position(0);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap(), Vec3::new(5.0, 5.0, 5.0));
        
        // Out of bounds
        assert!(cloth.particle_position(10000).is_none());
    }

    #[test]
    fn test_config_default() {
        let config = ClothConfig::default();
        
        assert_eq!(config.width, 20);
        assert_eq!(config.height, 20);
        assert_eq!(config.spacing, 0.1);
        assert_eq!(config.particle_mass, 0.1);
        assert_eq!(config.stiffness, 0.8);
        assert_eq!(config.damping, 0.98);
        assert_eq!(config.solver_iterations, 3);
    }

    #[test]
    fn test_degenerate_capsule() {
        // Capsule with start == end should act like sphere
        let collider = ClothCollider::Capsule {
            start: Vec3::ZERO,
            end: Vec3::ZERO,
            radius: 1.0,
        };

        let mut particle = ClothParticle::new(Vec3::new(0.5, 0.0, 0.0), 1.0);
        collider.resolve_collision(&mut particle, 0.5);

        // Should still push out
        let dist = particle.position.length();
        assert!(dist >= 0.99);
    }

    #[test]
    fn test_manager_get_mut() {
        let mut manager = ClothManager::new();
        let id = manager.create(ClothConfig::default(), Vec3::ZERO);
        
        {
            let cloth = manager.get_mut(id).unwrap();
            cloth.pin_top_edge();
        }
        
        let cloth = manager.get(id).unwrap();
        assert!(cloth.particles[0].pinned);
    }

    #[test]
    fn test_manager_iter() {
        let mut manager = ClothManager::new();
        manager.create(ClothConfig::default(), Vec3::ZERO);
        manager.create(ClothConfig::default(), Vec3::new(5.0, 0.0, 0.0));
        manager.create(ClothConfig::default(), Vec3::new(10.0, 0.0, 0.0));
        
        assert_eq!(manager.iter().count(), 3);
    }

    #[test]
    fn test_manager_remove_nonexistent() {
        let mut manager = ClothManager::new();
        assert!(!manager.remove(ClothId(999)));
    }

    #[test]
    fn test_manager_get_nonexistent() {
        let manager = ClothManager::new();
        assert!(manager.get(ClothId(999)).is_none());
    }

    #[test]
    fn test_pinned_particle_no_integrate() {
        let mut particle = ClothParticle::pinned(Vec3::new(1.0, 1.0, 1.0));
        particle.acceleration = Vec3::new(100.0, 100.0, 100.0);
        
        particle.integrate(1.0, 1.0);
        
        // Position should not change
        assert_eq!(particle.position, Vec3::new(1.0, 1.0, 1.0));
    }
}

