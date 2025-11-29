//! # Projectile System
//!
//! Lightweight projectile simulation optimized for games. Uses custom ballistic
//! solving instead of full rigid body simulation for performance.
//!
//! ## Features
//!
//! - **Hitscan**: Instant raycast projectiles (bullets, lasers)
//! - **Kinematic**: Physically simulated projectiles (grenades, arrows)
//! - **Ballistics**: Gravity, drag, wind effects
//! - **Collision**: Raycast/shapecast detection with penetration
//! - **Explosions**: Radial impulse with falloff curves
//!
//! ## Usage
//!
//! ```rust
//! use astraweave_physics::projectile::{ProjectileManager, ProjectileConfig, ProjectileKind};
//! use glam::Vec3;
//!
//! let mut manager = ProjectileManager::new();
//!
//! // Spawn a grenade
//! let config = ProjectileConfig {
//!     kind: ProjectileKind::Kinematic,
//!     position: Vec3::new(0.0, 1.0, 0.0),
//!     velocity: Vec3::new(10.0, 5.0, 0.0),
//!     gravity_scale: 1.0,
//!     drag: 0.01,
//!     radius: 0.1,
//!     max_lifetime: 10.0,
//!     ..Default::default()
//! };
//!
//! let id = manager.spawn(config);
//! ```

use glam::Vec3;
use std::collections::HashMap;

/// Unique identifier for a projectile
pub type ProjectileId = u64;

/// Type of projectile simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectileKind {
    /// Instant raycast (bullets, lasers) - no travel time
    Hitscan,
    /// Physically simulated with ballistics (grenades, arrows)
    #[default]
    Kinematic,
}

/// Configuration for spawning a projectile
#[derive(Debug, Clone)]
pub struct ProjectileConfig {
    /// Type of projectile
    pub kind: ProjectileKind,
    /// Initial world position
    pub position: Vec3,
    /// Initial velocity (direction * speed)
    pub velocity: Vec3,
    /// Gravity multiplier (0.0 = no gravity, 1.0 = normal, -1.0 = reverse)
    pub gravity_scale: f32,
    /// Air resistance coefficient (0.0 = none, higher = more drag)
    pub drag: f32,
    /// Collision radius for shapecast
    pub radius: f32,
    /// Maximum time before auto-despawn (seconds)
    pub max_lifetime: f32,
    /// Maximum bounces before despawn (0 = no bounce)
    pub max_bounces: u32,
    /// Bounciness factor (0.0 = no bounce, 1.0 = perfect elastic)
    pub restitution: f32,
    /// Penetration power (0.0 = no penetration)
    pub penetration: f32,
    /// Owner entity ID (for friendly fire detection)
    pub owner: Option<u64>,
    /// User data for game logic
    pub user_data: u64,
}

impl Default for ProjectileConfig {
    fn default() -> Self {
        Self {
            kind: ProjectileKind::Kinematic,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            gravity_scale: 1.0,
            drag: 0.0,
            radius: 0.05,
            max_lifetime: 10.0,
            max_bounces: 0,
            restitution: 0.5,
            penetration: 0.0,
            owner: None,
            user_data: 0,
        }
    }
}

/// Active projectile state
#[derive(Debug, Clone)]
pub struct Projectile {
    pub id: ProjectileId,
    pub config: ProjectileConfig,
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub bounces: u32,
    pub active: bool,
}

impl Projectile {
    fn new(id: ProjectileId, config: ProjectileConfig) -> Self {
        let position = config.position;
        let velocity = config.velocity;
        Self {
            id,
            config,
            position,
            velocity,
            lifetime: 0.0,
            bounces: 0,
            active: true,
        }
    }
}

/// Result of a projectile hit
#[derive(Debug, Clone)]
pub struct ProjectileHit {
    /// Projectile that hit
    pub projectile_id: ProjectileId,
    /// World position of impact
    pub position: Vec3,
    /// Surface normal at impact
    pub normal: Vec3,
    /// Body ID that was hit (if any)
    pub body_id: Option<u64>,
    /// Distance traveled to hit
    pub distance: f32,
    /// Whether the projectile penetrated
    pub penetrated: bool,
}

/// Falloff curve for explosion damage/force
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FalloffCurve {
    /// Force = max * (1 - distance/radius)
    #[default]
    Linear,
    /// Force = max * (1 - (distance/radius)^2)
    Quadratic,
    /// Force = max * e^(-distance/radius)
    Exponential,
    /// Force = max (no falloff)
    Constant,
}

impl FalloffCurve {
    /// Calculate falloff multiplier (0.0 to 1.0) given distance and radius
    pub fn calculate(&self, distance: f32, radius: f32) -> f32 {
        if distance >= radius {
            return 0.0;
        }
        if radius <= 0.0 {
            return 1.0;
        }
        let t = distance / radius;
        match self {
            FalloffCurve::Linear => 1.0 - t,
            FalloffCurve::Quadratic => 1.0 - t * t,
            FalloffCurve::Exponential => (-t * 3.0).exp(), // e^(-3t) gives ~5% at edge
            FalloffCurve::Constant => 1.0,
        }
    }
}

/// Configuration for an explosion
#[derive(Debug, Clone)]
pub struct ExplosionConfig {
    /// Center of explosion
    pub center: Vec3,
    /// Maximum radius of effect
    pub radius: f32,
    /// Maximum force at center
    pub force: f32,
    /// Force falloff curve
    pub falloff: FalloffCurve,
    /// Upward bias (0.0 = pure radial, 1.0 = pure upward)
    pub upward_bias: f32,
}

impl Default for ExplosionConfig {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 5.0,
            force: 1000.0,
            falloff: FalloffCurve::Linear,
            upward_bias: 0.3,
        }
    }
}

/// Result of explosion affecting a body
#[derive(Debug, Clone)]
pub struct ExplosionResult {
    /// Body that was affected
    pub body_id: u64,
    /// Impulse applied to body
    pub impulse: Vec3,
    /// Distance from explosion center
    pub distance: f32,
    /// Falloff multiplier applied
    pub falloff_multiplier: f32,
}

/// Manages all active projectiles
#[derive(Debug)]
pub struct ProjectileManager {
    projectiles: HashMap<ProjectileId, Projectile>,
    next_id: ProjectileId,
    /// Global wind vector affecting all projectiles
    pub wind: Vec3,
    /// Global gravity vector (default: -9.81 Y)
    pub gravity: Vec3,
    /// Pending hits from last update
    hits: Vec<ProjectileHit>,
}

impl Default for ProjectileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectileManager {
    /// Create a new projectile manager
    pub fn new() -> Self {
        Self {
            projectiles: HashMap::new(),
            next_id: 1,
            wind: Vec3::ZERO,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            hits: Vec::new(),
        }
    }

    /// Spawn a new projectile
    pub fn spawn(&mut self, config: ProjectileConfig) -> ProjectileId {
        let id = self.next_id;
        self.next_id += 1;
        
        let projectile = Projectile::new(id, config);
        self.projectiles.insert(id, projectile);
        id
    }

    /// Despawn a projectile
    pub fn despawn(&mut self, id: ProjectileId) -> bool {
        self.projectiles.remove(&id).is_some()
    }

    /// Get a projectile by ID
    pub fn get(&self, id: ProjectileId) -> Option<&Projectile> {
        self.projectiles.get(&id)
    }

    /// Get a mutable reference to a projectile
    pub fn get_mut(&mut self, id: ProjectileId) -> Option<&mut Projectile> {
        self.projectiles.get_mut(&id)
    }

    /// Get all active projectiles
    pub fn iter(&self) -> impl Iterator<Item = &Projectile> {
        self.projectiles.values().filter(|p| p.active)
    }

    /// Get number of active projectiles
    pub fn count(&self) -> usize {
        self.projectiles.len()
    }

    /// Get hits from last update (call after `update`)
    pub fn drain_hits(&mut self) -> Vec<ProjectileHit> {
        std::mem::take(&mut self.hits)
    }

    /// Update all projectiles (call once per frame)
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds
    /// * `raycast_fn` - Function to perform raycasts: (origin, direction, max_dist) -> Option<(hit_pos, normal, body_id, dist)>
    pub fn update<F>(&mut self, dt: f32, mut raycast_fn: F)
    where
        F: FnMut(Vec3, Vec3, f32) -> Option<(Vec3, Vec3, Option<u64>, f32)>,
    {
        let gravity = self.gravity;
        let wind = self.wind;
        let mut to_despawn = Vec::new();

        for projectile in self.projectiles.values_mut() {
            if !projectile.active {
                continue;
            }

            // Update lifetime
            projectile.lifetime += dt;
            if projectile.lifetime >= projectile.config.max_lifetime {
                projectile.active = false;
                to_despawn.push(projectile.id);
                continue;
            }

            // Skip hitscan (they resolve instantly on spawn)
            if projectile.config.kind == ProjectileKind::Hitscan {
                continue;
            }

            // Store previous position for collision detection
            let prev_pos = projectile.position;

            // Apply gravity
            let grav_accel = gravity * projectile.config.gravity_scale;
            projectile.velocity += grav_accel * dt;

            // Apply drag: F_drag = -drag * v^2 * normalize(v)
            let speed = projectile.velocity.length();
            if speed > 0.001 && projectile.config.drag > 0.0 {
                let drag_force = projectile.config.drag * speed * speed;
                let drag_decel = (drag_force / 1.0) * dt; // Assume unit mass
                let decel = drag_decel.min(speed); // Don't reverse direction
                projectile.velocity -= projectile.velocity.normalize() * decel;
            }

            // Apply wind
            projectile.velocity += wind * dt;

            // Calculate new position
            let movement = projectile.velocity * dt;
            let new_pos = prev_pos + movement;

            // Raycast for collision
            let move_dist = movement.length();
            if move_dist > 0.001 {
                let dir = movement.normalize();
                if let Some((hit_pos, normal, body_id, dist)) =
                    raycast_fn(prev_pos, dir, move_dist + projectile.config.radius)
                {
                    // Record hit
                    self.hits.push(ProjectileHit {
                        projectile_id: projectile.id,
                        position: hit_pos,
                        normal,
                        body_id,
                        distance: dist,
                        penetrated: projectile.config.penetration > 0.0,
                    });

                    // Handle bounce
                    if projectile.bounces < projectile.config.max_bounces
                        && projectile.config.restitution > 0.0
                    {
                        // Reflect velocity
                        let reflect = projectile.velocity
                            - 2.0 * projectile.velocity.dot(normal) * normal;
                        projectile.velocity = reflect * projectile.config.restitution;
                        projectile.position = hit_pos + normal * 0.01; // Offset from surface
                        projectile.bounces += 1;
                    } else {
                        // Despawn on impact
                        projectile.active = false;
                        to_despawn.push(projectile.id);
                    }
                } else {
                    // No collision, update position
                    projectile.position = new_pos;
                }
            } else {
                projectile.position = new_pos;
            }
        }

        // Clean up despawned projectiles
        for id in to_despawn {
            self.projectiles.remove(&id);
        }
    }

    /// Perform hitscan (instant raycast projectile)
    ///
    /// Returns hit result if something was hit.
    pub fn hitscan<F>(
        &mut self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        mut raycast_fn: F,
    ) -> Option<ProjectileHit>
    where
        F: FnMut(Vec3, Vec3, f32) -> Option<(Vec3, Vec3, Option<u64>, f32)>,
    {
        let dir = direction.normalize();
        raycast_fn(origin, dir, max_distance).map(|(hit_pos, normal, body_id, dist)| {
            ProjectileHit {
                projectile_id: 0, // Hitscan doesn't create persistent projectile
                position: hit_pos,
                normal,
                body_id,
                distance: dist,
                penetrated: false,
            }
        })
    }

    /// Calculate explosion effects on nearby bodies
    ///
    /// # Arguments
    /// * `config` - Explosion configuration
    /// * `bodies` - Iterator of (body_id, position) pairs to check
    ///
    /// # Returns
    /// Vector of bodies affected with impulse to apply
    pub fn calculate_explosion<I>(
        &self,
        config: &ExplosionConfig,
        bodies: I,
    ) -> Vec<ExplosionResult>
    where
        I: IntoIterator<Item = (u64, Vec3)>,
    {
        let mut results = Vec::new();

        for (body_id, body_pos) in bodies {
            let to_body = body_pos - config.center;
            let distance = to_body.length();

            if distance >= config.radius {
                continue;
            }

            let falloff = config.falloff.calculate(distance, config.radius);
            let force_magnitude = config.force * falloff;

            // Calculate direction with upward bias
            let radial_dir = if distance > 0.001 {
                to_body.normalize()
            } else {
                Vec3::Y // Default to up if at center
            };

            let biased_dir =
                (radial_dir * (1.0 - config.upward_bias) + Vec3::Y * config.upward_bias).normalize();

            let impulse = biased_dir * force_magnitude;

            results.push(ExplosionResult {
                body_id,
                impulse,
                distance,
                falloff_multiplier: falloff,
            });
        }

        results
    }
}

/// Calculate projectile trajectory points (for prediction/visualization)
///
/// # Arguments
/// * `start` - Starting position
/// * `velocity` - Initial velocity
/// * `gravity` - Gravity vector
/// * `drag` - Drag coefficient
/// * `dt` - Time step between points
/// * `num_points` - Number of trajectory points
pub fn predict_trajectory(
    start: Vec3,
    velocity: Vec3,
    gravity: Vec3,
    drag: f32,
    dt: f32,
    num_points: usize,
) -> Vec<Vec3> {
    let mut points = Vec::with_capacity(num_points);
    let mut pos = start;
    let mut vel = velocity;

    points.push(pos);

    for _ in 1..num_points {
        // Apply gravity
        vel += gravity * dt;

        // Apply drag
        let speed = vel.length();
        if speed > 0.001 && drag > 0.0 {
            let drag_force = drag * speed * speed;
            let drag_decel = (drag_force / 1.0) * dt;
            let decel = drag_decel.min(speed);
            vel -= vel.normalize() * decel;
        }

        pos += vel * dt;
        points.push(pos);
    }

    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projectile_spawn() {
        let mut manager = ProjectileManager::new();
        let id = manager.spawn(ProjectileConfig::default());
        assert!(manager.get(id).is_some());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_projectile_despawn() {
        let mut manager = ProjectileManager::new();
        let id = manager.spawn(ProjectileConfig::default());
        assert!(manager.despawn(id));
        assert!(manager.get(id).is_none());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_projectile_gravity() {
        let mut manager = ProjectileManager::new();
        let config = ProjectileConfig {
            position: Vec3::new(0.0, 10.0, 0.0),
            velocity: Vec3::new(10.0, 0.0, 0.0),
            gravity_scale: 1.0,
            ..Default::default()
        };
        let id = manager.spawn(config);

        // No-op raycast
        let raycast = |_: Vec3, _: Vec3, _: f32| -> Option<(Vec3, Vec3, Option<u64>, f32)> { None };

        // Simulate 1 second
        for _ in 0..60 {
            manager.update(1.0 / 60.0, raycast);
        }

        let proj = manager.get(id).unwrap();
        // Should have fallen ~4.9m (1/2 * 9.81 * 1^2)
        assert!(proj.position.y < 6.0, "Y should be < 6, got {}", proj.position.y);
        // Should have moved ~10m horizontally
        assert!(proj.position.x > 9.0, "X should be > 9, got {}", proj.position.x);
    }

    #[test]
    fn test_projectile_drag() {
        let mut manager = ProjectileManager::new();
        manager.gravity = Vec3::ZERO; // No gravity for this test

        let config = ProjectileConfig {
            position: Vec3::ZERO,
            velocity: Vec3::new(100.0, 0.0, 0.0),
            gravity_scale: 0.0,
            drag: 0.1,
            ..Default::default()
        };
        let id = manager.spawn(config);

        let raycast = |_: Vec3, _: Vec3, _: f32| -> Option<(Vec3, Vec3, Option<u64>, f32)> { None };

        // Simulate 1 second
        for _ in 0..60 {
            manager.update(1.0 / 60.0, raycast);
        }

        let proj = manager.get(id).unwrap();
        // With drag, should have slowed down significantly
        assert!(
            proj.velocity.x < 100.0,
            "Velocity should decrease with drag"
        );
    }

    #[test]
    fn test_projectile_bounce() {
        let mut manager = ProjectileManager::new();
        manager.gravity = Vec3::ZERO;

        let config = ProjectileConfig {
            position: Vec3::ZERO,
            velocity: Vec3::new(10.0, 0.0, 0.0),
            gravity_scale: 0.0,
            max_bounces: 3,
            restitution: 0.8,
            ..Default::default()
        };
        let id = manager.spawn(config);

        // Simulate hitting a wall at X=5
        let raycast = |origin: Vec3, dir: Vec3, max: f32| -> Option<(Vec3, Vec3, Option<u64>, f32)> {
            if origin.x < 5.0 && dir.x > 0.0 {
                let dist = 5.0 - origin.x;
                if dist < max {
                    return Some((Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0), Some(1), dist));
                }
            }
            None
        };

        // First update should hit the wall
        manager.update(1.0, raycast);

        let proj = manager.get(id).unwrap();
        assert_eq!(proj.bounces, 1, "Should have bounced once");
        assert!(proj.velocity.x < 0.0, "Velocity should be reversed");
    }

    #[test]
    fn test_projectile_lifetime() {
        let mut manager = ProjectileManager::new();
        let config = ProjectileConfig {
            max_lifetime: 0.5,
            ..Default::default()
        };
        let id = manager.spawn(config);

        let raycast = |_: Vec3, _: Vec3, _: f32| -> Option<(Vec3, Vec3, Option<u64>, f32)> { None };

        // Simulate 1 second (projectile should despawn at 0.5s)
        for _ in 0..60 {
            manager.update(1.0 / 60.0, raycast);
        }

        assert!(manager.get(id).is_none(), "Projectile should have despawned");
    }

    #[test]
    fn test_falloff_linear() {
        let curve = FalloffCurve::Linear;
        assert!((curve.calculate(0.0, 10.0) - 1.0).abs() < 0.001);
        assert!((curve.calculate(5.0, 10.0) - 0.5).abs() < 0.001);
        assert!((curve.calculate(10.0, 10.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_falloff_quadratic() {
        let curve = FalloffCurve::Quadratic;
        assert!((curve.calculate(0.0, 10.0) - 1.0).abs() < 0.001);
        assert!((curve.calculate(5.0, 10.0) - 0.75).abs() < 0.001); // 1 - 0.5^2 = 0.75
        assert!((curve.calculate(10.0, 10.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_explosion_radial() {
        let manager = ProjectileManager::new();
        let config = ExplosionConfig {
            center: Vec3::ZERO,
            radius: 10.0,
            force: 1000.0,
            falloff: FalloffCurve::Linear,
            upward_bias: 0.0,
        };

        let bodies = vec![
            (1, Vec3::new(5.0, 0.0, 0.0)),  // At half radius
            (2, Vec3::new(15.0, 0.0, 0.0)), // Outside radius
        ];

        let results = manager.calculate_explosion(&config, bodies);

        assert_eq!(results.len(), 1, "Only one body should be affected");
        assert_eq!(results[0].body_id, 1);
        assert!((results[0].falloff_multiplier - 0.5).abs() < 0.01);
        assert!(results[0].impulse.x > 0.0, "Impulse should push away from center");
    }

    #[test]
    fn test_explosion_upward_bias() {
        let manager = ProjectileManager::new();
        let config = ExplosionConfig {
            center: Vec3::ZERO,
            radius: 10.0,
            force: 1000.0,
            falloff: FalloffCurve::Constant,
            upward_bias: 1.0, // Full upward
        };

        let bodies = vec![(1, Vec3::new(5.0, 0.0, 0.0))];
        let results = manager.calculate_explosion(&config, bodies);

        assert_eq!(results.len(), 1);
        // With full upward bias, impulse should be purely vertical
        assert!(results[0].impulse.y > 900.0, "Impulse should be mostly upward");
    }

    #[test]
    fn test_predict_trajectory() {
        let points = predict_trajectory(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 10.0, 0.0),
            Vec3::new(0.0, -9.81, 0.0),
            0.0,
            0.1,
            10,
        );

        assert_eq!(points.len(), 10);
        assert_eq!(points[0], Vec3::ZERO);
        // Later points should show parabolic arc
        assert!(points[9].x > points[0].x, "Should move forward");
    }

    #[test]
    fn test_hitscan() {
        let mut manager = ProjectileManager::new();

        // Mock raycast that hits at distance 5
        let raycast = |_: Vec3, _: Vec3, _: f32| -> Option<(Vec3, Vec3, Option<u64>, f32)> {
            Some((Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0), Some(1), 5.0))
        };

        let hit = manager.hitscan(Vec3::ZERO, Vec3::X, 100.0, raycast);

        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.distance, 5.0);
        assert_eq!(hit.body_id, Some(1));
    }

    #[test]
    fn test_wind_effect() {
        let mut manager = ProjectileManager::new();
        manager.gravity = Vec3::ZERO;
        manager.wind = Vec3::new(5.0, 0.0, 0.0); // Wind blowing +X

        let config = ProjectileConfig {
            position: Vec3::ZERO,
            velocity: Vec3::new(0.0, 0.0, 10.0), // Moving +Z
            gravity_scale: 0.0,
            ..Default::default()
        };
        let id = manager.spawn(config);

        let raycast = |_: Vec3, _: Vec3, _: f32| -> Option<(Vec3, Vec3, Option<u64>, f32)> { None };

        for _ in 0..60 {
            manager.update(1.0 / 60.0, raycast);
        }

        let proj = manager.get(id).unwrap();
        // Wind should have pushed projectile in +X direction
        assert!(proj.position.x > 0.0, "Wind should affect trajectory");
    }
}
