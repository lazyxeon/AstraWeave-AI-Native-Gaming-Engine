//! Destruction System
//!
//! Provides physics-based destruction for breakable objects:
//! - Pre-fractured mesh system (swap intact → debris)
//! - Structural integrity (health-based breaking)
//! - Debris generation and lifetime
//! - Force-based destruction triggers

use glam::Vec3;
use std::collections::HashMap;

/// Unique identifier for destructible objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DestructibleId(pub u64);

/// Unique identifier for debris pieces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebrisId(pub u64);

/// Shape of debris pieces
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebrisShape {
    /// Box-shaped debris
    Box { half_extents: Vec3 },
    /// Spherical debris
    Sphere { radius: f32 },
    /// Convex hull (represented as box for simplicity)
    ConvexHull { half_extents: Vec3 },
}

impl Default for DebrisShape {
    fn default() -> Self {
        Self::Box {
            half_extents: Vec3::splat(0.2),
        }
    }
}

/// Configuration for a debris piece
#[derive(Debug, Clone)]
pub struct DebrisConfig {
    /// Shape of the debris
    pub shape: DebrisShape,
    /// Relative position from object center
    pub local_position: Vec3,
    /// Mass of debris piece
    pub mass: f32,
    /// Initial velocity multiplier (from destruction force)
    pub velocity_factor: f32,
    /// Angular velocity multiplier
    pub angular_velocity_factor: f32,
    /// Lifetime in seconds (0 = permanent)
    pub lifetime: f32,
    /// Whether debris can cause secondary destruction
    pub can_damage: bool,
    /// Damage amount if hits another destructible
    pub damage_on_hit: f32,
}

impl Default for DebrisConfig {
    fn default() -> Self {
        Self {
            shape: DebrisShape::default(),
            local_position: Vec3::ZERO,
            mass: 1.0,
            velocity_factor: 1.0,
            angular_velocity_factor: 0.5,
            lifetime: 10.0,
            can_damage: false,
            damage_on_hit: 0.0,
        }
    }
}

/// Fracture pattern for pre-fractured meshes
#[derive(Debug, Clone)]
pub struct FracturePattern {
    /// Debris pieces that make up this object when broken
    pub debris: Vec<DebrisConfig>,
    /// Center of mass for the intact object
    pub center_of_mass: Vec3,
}

impl FracturePattern {
    /// Create a simple uniform fracture pattern
    pub fn uniform(piece_count: usize, object_half_extents: Vec3, mass: f32) -> Self {
        let mut debris = Vec::with_capacity(piece_count);
        let piece_mass = mass / piece_count as f32;

        // Create a simple grid of debris
        let pieces_per_axis = (piece_count as f32).cbrt().ceil() as i32;
        let piece_size = object_half_extents * 2.0 / pieces_per_axis as f32;

        for x in 0..pieces_per_axis {
            for y in 0..pieces_per_axis {
                for z in 0..pieces_per_axis {
                    if debris.len() >= piece_count {
                        break;
                    }

                    let local_pos = Vec3::new(
                        (x as f32 + 0.5) * piece_size.x - object_half_extents.x,
                        (y as f32 + 0.5) * piece_size.y - object_half_extents.y,
                        (z as f32 + 0.5) * piece_size.z - object_half_extents.z,
                    );

                    debris.push(DebrisConfig {
                        shape: DebrisShape::Box {
                            half_extents: piece_size * 0.4,
                        },
                        local_position: local_pos,
                        mass: piece_mass,
                        ..Default::default()
                    });
                }
            }
        }

        Self {
            debris,
            center_of_mass: Vec3::ZERO,
        }
    }

    /// Create a radial fracture pattern (for explosions)
    pub fn radial(piece_count: usize, radius: f32, mass: f32) -> Self {
        let mut debris = Vec::with_capacity(piece_count);
        let piece_mass = mass / piece_count as f32;

        // Golden angle distribution for even spacing
        let golden_angle = std::f32::consts::PI * (3.0 - (5.0_f32).sqrt());

        for i in 0..piece_count {
            let t = i as f32 / piece_count as f32;
            let inclination = (1.0 - 2.0 * t).acos();
            let azimuth = golden_angle * i as f32;

            let local_pos = Vec3::new(
                inclination.sin() * azimuth.cos() * radius * 0.8,
                inclination.cos() * radius * 0.8,
                inclination.sin() * azimuth.sin() * radius * 0.8,
            );

            debris.push(DebrisConfig {
                shape: DebrisShape::Sphere {
                    radius: radius * 0.15,
                },
                local_position: local_pos,
                mass: piece_mass,
                velocity_factor: 1.5, // Radial patterns fly outward faster
                ..Default::default()
            });
        }

        Self {
            debris,
            center_of_mass: Vec3::ZERO,
        }
    }

    /// Create a layered fracture pattern (for walls)
    pub fn layered(layers: usize, pieces_per_layer: usize, half_extents: Vec3, mass: f32) -> Self {
        let total_pieces = layers * pieces_per_layer;
        let mut debris = Vec::with_capacity(total_pieces);
        let piece_mass = mass / total_pieces as f32;

        let layer_height = half_extents.y * 2.0 / layers as f32;

        for layer in 0..layers {
            let y = (layer as f32 + 0.5) * layer_height - half_extents.y;

            for piece in 0..pieces_per_layer {
                let angle =
                    piece as f32 * std::f32::consts::TAU / pieces_per_layer as f32;
                let x = angle.cos() * half_extents.x * 0.7;
                let z = angle.sin() * half_extents.z * 0.7;

                debris.push(DebrisConfig {
                    shape: DebrisShape::Box {
                        half_extents: Vec3::new(
                            half_extents.x * 0.3,
                            layer_height * 0.4,
                            half_extents.z * 0.3,
                        ),
                    },
                    local_position: Vec3::new(x, y, z),
                    mass: piece_mass,
                    ..Default::default()
                });
            }
        }

        Self {
            debris,
            center_of_mass: Vec3::ZERO,
        }
    }
}

/// Destruction trigger type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DestructionTrigger {
    /// Destroy when force exceeds threshold
    Force { threshold: f32 },
    /// Destroy when health reaches zero
    Health,
    /// Destroy on any collision
    Collision,
    /// Manual destruction only
    Manual,
}

impl Default for DestructionTrigger {
    fn default() -> Self {
        Self::Force { threshold: 1000.0 }
    }
}

/// Configuration for a destructible object
#[derive(Debug, Clone)]
pub struct DestructibleConfig {
    /// Fracture pattern (debris layout)
    pub fracture_pattern: FracturePattern,
    /// How destruction is triggered
    pub trigger: DestructionTrigger,
    /// Maximum health (for Health trigger)
    pub max_health: f32,
    /// Minimum force to deal damage
    pub damage_threshold: f32,
    /// Force-to-damage conversion factor
    pub force_to_damage: f32,
    /// Explosion force when destroyed
    pub destruction_force: f32,
    /// Sound effect ID (for audio integration)
    pub destruction_sound: Option<u32>,
    /// Particle effect ID (for VFX integration)
    pub destruction_particles: Option<u32>,
}

impl Default for DestructibleConfig {
    fn default() -> Self {
        Self {
            fracture_pattern: FracturePattern::uniform(8, Vec3::splat(0.5), 10.0),
            trigger: DestructionTrigger::default(),
            max_health: 100.0,
            damage_threshold: 10.0,
            force_to_damage: 0.1,
            destruction_force: 5.0,
            destruction_sound: None,
            destruction_particles: None,
        }
    }
}

/// State of a destructible object
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DestructibleState {
    /// Object is intact
    Intact,
    /// Object is damaged but functional
    Damaged,
    /// Object is being destroyed (playing animation/effects)
    Destroying,
    /// Object has been destroyed
    Destroyed,
}

/// Runtime data for a destructible object
#[derive(Debug, Clone)]
pub struct Destructible {
    pub id: DestructibleId,
    pub config: DestructibleConfig,
    pub state: DestructibleState,
    /// Current health
    pub health: f32,
    /// World position
    pub position: Vec3,
    /// Accumulated force this frame
    pub accumulated_force: f32,
    /// Body ID in physics world (if any)
    pub body_id: Option<u64>,
}

impl Destructible {
    /// Create a new destructible
    pub fn new(id: DestructibleId, config: DestructibleConfig, position: Vec3) -> Self {
        Self {
            id,
            health: config.max_health,
            config,
            state: DestructibleState::Intact,
            position,
            accumulated_force: 0.0,
            body_id: None,
        }
    }

    /// Apply damage to the object
    pub fn apply_damage(&mut self, damage: f32) {
        if self.state != DestructibleState::Intact && self.state != DestructibleState::Damaged {
            return;
        }

        self.health = (self.health - damage).max(0.0);

        if self.health <= 0.0 {
            self.state = DestructibleState::Destroying;
        } else if self.health < self.config.max_health * 0.5 {
            self.state = DestructibleState::Damaged;
        }
    }

    /// Apply force to the object
    pub fn apply_force(&mut self, force: f32) {
        self.accumulated_force += force;

        // Check force threshold trigger
        if let DestructionTrigger::Force { threshold } = self.config.trigger {
            if self.accumulated_force >= threshold {
                self.state = DestructibleState::Destroying;
            }
        }

        // Apply force-based damage for Health trigger
        if matches!(self.config.trigger, DestructionTrigger::Health)
            && force >= self.config.damage_threshold {
                let damage = (force - self.config.damage_threshold) * self.config.force_to_damage;
                self.apply_damage(damage);
            }
    }

    /// Handle collision
    pub fn on_collision(&mut self, impact_force: f32) {
        if let DestructionTrigger::Collision = self.config.trigger {
            self.state = DestructibleState::Destroying;
        } else {
            self.apply_force(impact_force);
        }
    }

    /// Manually trigger destruction
    pub fn destroy(&mut self) {
        self.state = DestructibleState::Destroying;
    }

    /// Check if object should spawn debris
    pub fn should_spawn_debris(&self) -> bool {
        self.state == DestructibleState::Destroying
    }

    /// Mark destruction as complete
    pub fn complete_destruction(&mut self) {
        self.state = DestructibleState::Destroyed;
    }

    /// Reset accumulated force (call at end of frame)
    pub fn reset_frame(&mut self) {
        self.accumulated_force = 0.0;
    }

    /// Get health percentage
    pub fn health_percent(&self) -> f32 {
        self.health / self.config.max_health
    }

    /// Check if destroyed
    pub fn is_destroyed(&self) -> bool {
        self.state == DestructibleState::Destroyed
    }
}

/// Active debris piece in the world
#[derive(Debug, Clone)]
pub struct Debris {
    pub id: DebrisId,
    /// Source destructible ID
    pub source: DestructibleId,
    pub config: DebrisConfig,
    /// World position
    pub position: Vec3,
    /// Linear velocity
    pub velocity: Vec3,
    /// Angular velocity
    pub angular_velocity: Vec3,
    /// Time alive
    pub age: f32,
    /// Physics body ID (if any)
    pub body_id: Option<u64>,
}

impl Debris {
    /// Create new debris
    pub fn new(
        id: DebrisId,
        source: DestructibleId,
        config: DebrisConfig,
        position: Vec3,
        velocity: Vec3,
    ) -> Self {
        Self {
            id,
            source,
            config,
            position,
            velocity,
            angular_velocity: Vec3::ZERO,
            age: 0.0,
            body_id: None,
        }
    }

    /// Update debris (if not physics-driven)
    pub fn update(&mut self, dt: f32, gravity: Vec3) {
        self.age += dt;

        // Simple physics if not driven by physics engine
        if self.body_id.is_none() {
            self.velocity += gravity * dt;
            self.position += self.velocity * dt;
        }
    }

    /// Check if debris should be removed
    pub fn should_remove(&self) -> bool {
        self.config.lifetime > 0.0 && self.age >= self.config.lifetime
    }
}

/// Destruction event for callbacks
#[derive(Debug, Clone)]
pub struct DestructionEvent {
    /// ID of destroyed object
    pub destructible_id: DestructibleId,
    /// Position of destruction
    pub position: Vec3,
    /// Force that caused destruction
    pub force: f32,
    /// Direction of force
    pub force_direction: Vec3,
    /// Number of debris spawned
    pub debris_count: usize,
}

/// Manager for destruction system
#[derive(Debug, Default)]
pub struct DestructionManager {
    destructibles: HashMap<DestructibleId, Destructible>,
    debris: HashMap<DebrisId, Debris>,
    pending_events: Vec<DestructionEvent>,
    next_destructible_id: u64,
    next_debris_id: u64,
    /// Maximum active debris pieces
    pub max_debris: usize,
    /// Default debris lifetime
    pub default_debris_lifetime: f32,
}

impl DestructionManager {
    /// Create a new destruction manager
    pub fn new() -> Self {
        Self {
            destructibles: HashMap::new(),
            debris: HashMap::new(),
            pending_events: Vec::new(),
            next_destructible_id: 1,
            next_debris_id: 1,
            max_debris: 500,
            default_debris_lifetime: 10.0,
        }
    }

    // === Destructible Management ===

    /// Add a destructible object
    pub fn add_destructible(
        &mut self,
        config: DestructibleConfig,
        position: Vec3,
    ) -> DestructibleId {
        let id = DestructibleId(self.next_destructible_id);
        self.next_destructible_id += 1;
        self.destructibles
            .insert(id, Destructible::new(id, config, position));
        id
    }

    /// Remove a destructible (and its debris)
    pub fn remove_destructible(&mut self, id: DestructibleId) -> bool {
        // Remove associated debris
        self.debris.retain(|_, d| d.source != id);
        self.destructibles.remove(&id).is_some()
    }

    /// Get a destructible
    pub fn get(&self, id: DestructibleId) -> Option<&Destructible> {
        self.destructibles.get(&id)
    }

    /// Get a mutable destructible
    pub fn get_mut(&mut self, id: DestructibleId) -> Option<&mut Destructible> {
        self.destructibles.get_mut(&id)
    }

    /// Apply damage to a destructible
    pub fn apply_damage(&mut self, id: DestructibleId, damage: f32) {
        if let Some(dest) = self.destructibles.get_mut(&id) {
            dest.apply_damage(damage);
        }
    }

    /// Apply force to a destructible
    pub fn apply_force(&mut self, id: DestructibleId, force: f32) {
        if let Some(dest) = self.destructibles.get_mut(&id) {
            dest.apply_force(force);
        }
    }

    /// Handle collision for a destructible
    pub fn on_collision(&mut self, id: DestructibleId, impact_force: f32) {
        if let Some(dest) = self.destructibles.get_mut(&id) {
            dest.on_collision(impact_force);
        }
    }

    /// Manually destroy an object
    pub fn destroy(&mut self, id: DestructibleId) {
        if let Some(dest) = self.destructibles.get_mut(&id) {
            dest.destroy();
        }
    }

    // === Debris Management ===

    /// Get debris by ID
    pub fn get_debris(&self, id: DebrisId) -> Option<&Debris> {
        self.debris.get(&id)
    }

    /// Iterate over all debris
    pub fn debris_iter(&self) -> impl Iterator<Item = &Debris> {
        self.debris.values()
    }

    /// Spawn debris for a destructible
    fn spawn_debris(&mut self, dest: &Destructible, force_direction: Vec3) -> Vec<DebrisId> {
        let mut spawned = Vec::new();

        // Respect debris limit
        let available_slots = self.max_debris.saturating_sub(self.debris.len());
        let debris_to_spawn = dest.config.fracture_pattern.debris.len().min(available_slots);

        for debris_config in dest.config.fracture_pattern.debris.iter().take(debris_to_spawn) {
            let id = DebrisId(self.next_debris_id);
            self.next_debris_id += 1;

            // Calculate debris world position
            let position = dest.position + debris_config.local_position;

            // Calculate initial velocity (outward from center + force direction)
            let outward = debris_config.local_position.normalize_or_zero();
            let velocity = (outward * dest.config.destruction_force
                + force_direction * dest.config.destruction_force * 0.5)
                * debris_config.velocity_factor;

            let mut debris = Debris::new(id, dest.id, debris_config.clone(), position, velocity);

            // Random angular velocity
            debris.angular_velocity = Vec3::new(
                (id.0 as f32 * 1.234).sin() * 5.0,
                (id.0 as f32 * 2.345).sin() * 5.0,
                (id.0 as f32 * 3.456).sin() * 5.0,
            ) * debris_config.angular_velocity_factor;

            self.debris.insert(id, debris);
            spawned.push(id);
        }

        spawned
    }

    // === Update ===

    /// Update the destruction system
    pub fn update(&mut self, dt: f32, gravity: Vec3) {
        // Process destructibles that need debris spawning
        let mut to_process = Vec::new();
        for (id, dest) in &self.destructibles {
            if dest.should_spawn_debris() {
                to_process.push(*id);
            }
        }

        for id in to_process {
            if let Some(dest) = self.destructibles.get(&id) {
                // Clone needed data before mutable borrow
                let position = dest.position;
                let force = dest.accumulated_force;
                let dest_clone = dest.clone();

                // Spawn debris
                let debris_ids =
                    self.spawn_debris(&dest_clone, Vec3::Y); // Default upward force direction

                // Create event
                self.pending_events.push(DestructionEvent {
                    destructible_id: id,
                    position,
                    force,
                    force_direction: Vec3::Y,
                    debris_count: debris_ids.len(),
                });
            }

            // Mark as destroyed
            if let Some(dest) = self.destructibles.get_mut(&id) {
                dest.complete_destruction();
            }
        }

        // Update debris
        for debris in self.debris.values_mut() {
            debris.update(dt, gravity);
        }

        // Remove expired debris
        self.debris.retain(|_, d| !d.should_remove());

        // Reset frame state for destructibles
        for dest in self.destructibles.values_mut() {
            dest.reset_frame();
        }
    }

    /// Take pending destruction events
    pub fn take_events(&mut self) -> Vec<DestructionEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Get counts
    pub fn destructible_count(&self) -> usize {
        self.destructibles.len()
    }

    pub fn debris_count(&self) -> usize {
        self.debris.len()
    }

    pub fn active_debris_count(&self) -> usize {
        self.debris.values().filter(|d| !d.should_remove()).count()
    }

    /// Clean up destroyed destructibles
    pub fn cleanup_destroyed(&mut self) {
        self.destructibles.retain(|_, d| !d.is_destroyed());
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destructible_creation() {
        let config = DestructibleConfig::default();
        let dest = Destructible::new(DestructibleId(1), config.clone(), Vec3::ZERO);

        assert_eq!(dest.state, DestructibleState::Intact);
        assert_eq!(dest.health, config.max_health);
    }

    #[test]
    fn test_damage_application() {
        let config = DestructibleConfig {
            max_health: 100.0,
            ..Default::default()
        };
        let mut dest = Destructible::new(DestructibleId(1), config, Vec3::ZERO);

        dest.apply_damage(30.0);
        assert_eq!(dest.health, 70.0);
        assert_eq!(dest.state, DestructibleState::Intact);

        dest.apply_damage(30.0);
        assert_eq!(dest.health, 40.0);
        assert_eq!(dest.state, DestructibleState::Damaged);

        dest.apply_damage(50.0);
        assert_eq!(dest.health, 0.0);
        assert_eq!(dest.state, DestructibleState::Destroying);
    }

    #[test]
    fn test_force_trigger() {
        let config = DestructibleConfig {
            trigger: DestructionTrigger::Force { threshold: 100.0 },
            ..Default::default()
        };
        let mut dest = Destructible::new(DestructibleId(1), config, Vec3::ZERO);

        dest.apply_force(50.0);
        assert_eq!(dest.state, DestructibleState::Intact);

        dest.apply_force(60.0);
        assert_eq!(dest.state, DestructibleState::Destroying);
    }

    #[test]
    fn test_collision_trigger() {
        let config = DestructibleConfig {
            trigger: DestructionTrigger::Collision,
            ..Default::default()
        };
        let mut dest = Destructible::new(DestructibleId(1), config, Vec3::ZERO);

        dest.on_collision(1.0);
        assert_eq!(dest.state, DestructibleState::Destroying);
    }

    #[test]
    fn test_manual_destruction() {
        let mut dest = Destructible::new(DestructibleId(1), DestructibleConfig::default(), Vec3::ZERO);

        dest.destroy();
        assert_eq!(dest.state, DestructibleState::Destroying);
    }

    #[test]
    fn test_health_percent() {
        let config = DestructibleConfig {
            max_health: 200.0,
            ..Default::default()
        };
        let mut dest = Destructible::new(DestructibleId(1), config, Vec3::ZERO);

        assert_eq!(dest.health_percent(), 1.0);
        dest.apply_damage(100.0);
        assert_eq!(dest.health_percent(), 0.5);
    }

    #[test]
    fn test_uniform_fracture_pattern() {
        let pattern = FracturePattern::uniform(8, Vec3::splat(1.0), 10.0);

        assert_eq!(pattern.debris.len(), 8);

        let total_mass: f32 = pattern.debris.iter().map(|d| d.mass).sum();
        assert!((total_mass - 10.0).abs() < 0.01, "Total mass should be preserved");
    }

    #[test]
    fn test_radial_fracture_pattern() {
        let pattern = FracturePattern::radial(20, 2.0, 5.0);

        assert_eq!(pattern.debris.len(), 20);
    }

    #[test]
    fn test_layered_fracture_pattern() {
        let pattern = FracturePattern::layered(3, 4, Vec3::new(2.0, 3.0, 2.0), 12.0);

        assert_eq!(pattern.debris.len(), 12); // 3 layers × 4 pieces
    }

    #[test]
    fn test_debris_lifetime() {
        let config = DebrisConfig {
            lifetime: 5.0,
            ..Default::default()
        };
        let mut debris = Debris::new(
            DebrisId(1),
            DestructibleId(1),
            config,
            Vec3::ZERO,
            Vec3::ZERO,
        );

        assert!(!debris.should_remove());

        debris.update(3.0, Vec3::ZERO);
        assert!(!debris.should_remove());

        debris.update(3.0, Vec3::ZERO);
        assert!(debris.should_remove());
    }

    #[test]
    fn test_permanent_debris() {
        let config = DebrisConfig {
            lifetime: 0.0, // Permanent
            ..Default::default()
        };
        let mut debris = Debris::new(
            DebrisId(1),
            DestructibleId(1),
            config,
            Vec3::ZERO,
            Vec3::ZERO,
        );

        debris.update(100.0, Vec3::ZERO);
        assert!(!debris.should_remove());
    }

    #[test]
    fn test_destruction_manager_add_remove() {
        let mut manager = DestructionManager::new();

        let id = manager.add_destructible(DestructibleConfig::default(), Vec3::ZERO);
        assert_eq!(manager.destructible_count(), 1);

        assert!(manager.get(id).is_some());
        assert!(manager.remove_destructible(id));
        assert_eq!(manager.destructible_count(), 0);
    }

    #[test]
    fn test_destruction_spawns_debris() {
        let mut manager = DestructionManager::new();

        let config = DestructibleConfig {
            fracture_pattern: FracturePattern::uniform(5, Vec3::splat(0.5), 5.0),
            trigger: DestructionTrigger::Manual,
            ..Default::default()
        };

        let id = manager.add_destructible(config, Vec3::ZERO);
        manager.destroy(id);

        // Update to process destruction
        manager.update(0.016, Vec3::new(0.0, -9.81, 0.0));

        assert_eq!(manager.debris_count(), 5);

        // Check events were generated
        let events = manager.take_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].debris_count, 5);
    }

    #[test]
    fn test_debris_limit() {
        let mut manager = DestructionManager::new();
        manager.max_debris = 3;

        let config = DestructibleConfig {
            fracture_pattern: FracturePattern::uniform(10, Vec3::splat(0.5), 10.0),
            trigger: DestructionTrigger::Manual,
            ..Default::default()
        };

        let id = manager.add_destructible(config, Vec3::ZERO);
        manager.destroy(id);
        manager.update(0.016, Vec3::ZERO);

        // Should only spawn up to max_debris
        assert_eq!(manager.debris_count(), 3);
    }

    #[test]
    fn test_debris_cleanup() {
        let mut manager = DestructionManager::new();

        let config = DestructibleConfig {
            fracture_pattern: FracturePattern {
                debris: vec![DebrisConfig {
                    lifetime: 0.5,
                    ..Default::default()
                }],
                center_of_mass: Vec3::ZERO,
            },
            trigger: DestructionTrigger::Manual,
            ..Default::default()
        };

        let id = manager.add_destructible(config, Vec3::ZERO);
        manager.destroy(id);
        manager.update(0.016, Vec3::ZERO);

        assert_eq!(manager.debris_count(), 1);

        // Age debris past lifetime
        manager.update(1.0, Vec3::ZERO);
        assert_eq!(manager.debris_count(), 0);
    }

    #[test]
    fn test_cleanup_destroyed() {
        let mut manager = DestructionManager::new();

        let id = manager.add_destructible(
            DestructibleConfig {
                trigger: DestructionTrigger::Manual,
                fracture_pattern: FracturePattern {
                    debris: vec![],
                    center_of_mass: Vec3::ZERO,
                },
                ..Default::default()
            },
            Vec3::ZERO,
        );

        manager.destroy(id);
        manager.update(0.016, Vec3::ZERO);

        assert_eq!(manager.destructible_count(), 1);
        manager.cleanup_destroyed();
        assert_eq!(manager.destructible_count(), 0);
    }

    #[test]
    fn test_debris_gravity() {
        let config = DebrisConfig::default();
        let mut debris = Debris::new(
            DebrisId(1),
            DestructibleId(1),
            config,
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::ZERO,
        );

        let gravity = Vec3::new(0.0, -9.81, 0.0);
        debris.update(1.0, gravity);

        // Should have fallen
        assert!(debris.position.y < 10.0);
        assert!(debris.velocity.y < 0.0);
    }

    // ============================================================================
    // CHAIN REACTION TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_damaging_debris_flag() {
        let config = DebrisConfig {
            can_damage: true,
            damage_on_hit: 25.0,
            ..Default::default()
        };
        
        assert!(config.can_damage);
        assert_eq!(config.damage_on_hit, 25.0);
    }

    #[test]
    fn test_multiple_destructibles_independent() {
        let mut manager = DestructionManager::new();

        let id1 = manager.add_destructible(DestructibleConfig::default(), Vec3::ZERO);
        let id2 = manager.add_destructible(DestructibleConfig::default(), Vec3::new(10.0, 0.0, 0.0));

        manager.apply_damage(id1, 1000.0);
        manager.update(0.016, Vec3::ZERO);

        // id1 should be destroyed, id2 should be intact
        let d1 = manager.get(id1).unwrap();
        let d2 = manager.get(id2).unwrap();
        
        assert_eq!(d1.state, DestructibleState::Destroyed);
        assert_eq!(d2.state, DestructibleState::Intact);
    }

    #[test]
    fn test_sequential_destruction() {
        let mut manager = DestructionManager::new();

        let ids: Vec<_> = (0..5)
            .map(|i| {
                manager.add_destructible(
                    DestructibleConfig {
                        trigger: DestructionTrigger::Manual,
                        fracture_pattern: FracturePattern::uniform(2, Vec3::splat(0.5), 1.0),
                        ..Default::default()
                    },
                    Vec3::new(i as f32 * 2.0, 0.0, 0.0),
                )
            })
            .collect();

        // Destroy all in sequence
        for id in &ids {
            manager.destroy(*id);
        }

        manager.update(0.016, Vec3::ZERO);

        assert_eq!(manager.debris_count(), 10); // 5 objects × 2 debris each
        
        let events = manager.take_events();
        assert_eq!(events.len(), 5);
    }

    // ============================================================================
    // STRESS PROPAGATION TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_incremental_force_accumulation() {
        let config = DestructibleConfig {
            trigger: DestructionTrigger::Force { threshold: 100.0 },
            ..Default::default()
        };
        let mut dest = Destructible::new(DestructibleId(1), config, Vec3::ZERO);

        // Apply force in small increments
        for _ in 0..10 {
            dest.apply_force(10.0);
        }

        assert_eq!(dest.accumulated_force, 100.0);
        assert_eq!(dest.state, DestructibleState::Destroying);
    }

    #[test]
    fn test_force_reset_between_frames() {
        let config = DestructibleConfig {
            trigger: DestructionTrigger::Force { threshold: 100.0 },
            ..Default::default()
        };
        let mut dest = Destructible::new(DestructibleId(1), config, Vec3::ZERO);

        dest.apply_force(50.0);
        dest.reset_frame();

        assert_eq!(dest.accumulated_force, 0.0);
        
        // Object should still be intact after reset
        assert_eq!(dest.state, DestructibleState::Intact);
    }

    #[test]
    fn test_health_trigger_force_conversion() {
        let config = DestructibleConfig {
            trigger: DestructionTrigger::Health,
            max_health: 100.0,
            damage_threshold: 10.0,
            force_to_damage: 0.5, // 50% of excess force becomes damage
            ..Default::default()
        };
        let mut dest = Destructible::new(DestructibleId(1), config, Vec3::ZERO);

        // Force of 30, threshold 10, so 20 excess × 0.5 = 10 damage
        dest.apply_force(30.0);
        assert_eq!(dest.health, 90.0);
    }

    #[test]
    fn test_sub_threshold_force_no_damage() {
        let config = DestructibleConfig {
            trigger: DestructionTrigger::Health,
            max_health: 100.0,
            damage_threshold: 50.0,
            force_to_damage: 1.0,
            ..Default::default()
        };
        let mut dest = Destructible::new(DestructibleId(1), config, Vec3::ZERO);

        dest.apply_force(49.0);
        assert_eq!(dest.health, 100.0); // No damage, below threshold
    }

    // ============================================================================
    // DEBRIS PHYSICS TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_debris_initial_velocity() {
        let mut manager = DestructionManager::new();

        let config = DestructibleConfig {
            fracture_pattern: FracturePattern::radial(8, 1.0, 4.0),
            trigger: DestructionTrigger::Manual,
            destruction_force: 10.0,
            ..Default::default()
        };

        let id = manager.add_destructible(config, Vec3::ZERO);
        manager.destroy(id);
        manager.update(0.001, Vec3::ZERO);

        // Check debris has non-zero velocity
        for debris in manager.debris_iter() {
            assert!(
                debris.velocity.length() > 0.0,
                "Debris should have initial velocity"
            );
        }
    }

    #[test]
    fn test_debris_angular_velocity() {
        let mut manager = DestructionManager::new();

        let debris_config = DebrisConfig {
            angular_velocity_factor: 2.0,
            ..Default::default()
        };

        let config = DestructibleConfig {
            fracture_pattern: FracturePattern {
                debris: vec![debris_config],
                center_of_mass: Vec3::ZERO,
            },
            trigger: DestructionTrigger::Manual,
            ..Default::default()
        };

        let id = manager.add_destructible(config, Vec3::ZERO);
        manager.destroy(id);
        manager.update(0.001, Vec3::ZERO);

        let debris = manager.debris_iter().next().unwrap();
        assert!(
            debris.angular_velocity.length() > 0.0,
            "Debris should have angular velocity"
        );
    }

    #[test]
    fn test_debris_position_from_local_offset() {
        let mut manager = DestructionManager::new();

        let debris_config = DebrisConfig {
            local_position: Vec3::new(5.0, 0.0, 0.0),
            ..Default::default()
        };

        let config = DestructibleConfig {
            fracture_pattern: FracturePattern {
                debris: vec![debris_config],
                center_of_mass: Vec3::ZERO,
            },
            trigger: DestructionTrigger::Manual,
            ..Default::default()
        };

        let id = manager.add_destructible(config, Vec3::new(10.0, 0.0, 0.0));
        manager.destroy(id);
        manager.update(0.001, Vec3::ZERO);

        let debris = manager.debris_iter().next().unwrap();
        // Position should be destructible position + local offset
        assert!((debris.position.x - 15.0).abs() < 0.01);
    }

    // ============================================================================
    // EDGE CASE TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_zero_health_destructible() {
        let config = DestructibleConfig {
            max_health: 0.0,
            ..Default::default()
        };
        let dest = Destructible::new(DestructibleId(1), config, Vec3::ZERO);

        // health_percent with max_health 0 should handle gracefully
        // (NaN or 0 depending on implementation)
        let pct = dest.health_percent();
        assert!(pct.is_nan() || pct == 0.0 || pct == 1.0);
    }

    #[test]
    fn test_empty_fracture_pattern() {
        let pattern = FracturePattern {
            debris: vec![],
            center_of_mass: Vec3::ZERO,
        };

        let mut manager = DestructionManager::new();
        let config = DestructibleConfig {
            fracture_pattern: pattern,
            trigger: DestructionTrigger::Manual,
            ..Default::default()
        };

        let id = manager.add_destructible(config, Vec3::ZERO);
        manager.destroy(id);
        manager.update(0.016, Vec3::ZERO);

        assert_eq!(manager.debris_count(), 0);
    }

    #[test]
    fn test_negative_damage() {
        let mut dest = Destructible::new(
            DestructibleId(1),
            DestructibleConfig {
                max_health: 100.0,
                ..Default::default()
            },
            Vec3::ZERO,
        );

        dest.apply_damage(-50.0); // Negative damage (healing?)
        
        // Implementation should allow this (health goes up)
        // or clamp to max health
        assert!(dest.health >= 100.0 && dest.health <= 150.0);
    }

    #[test]
    fn test_destroyed_ignores_damage() {
        let mut dest = Destructible::new(
            DestructibleId(1),
            DestructibleConfig::default(),
            Vec3::ZERO,
        );

        dest.destroy();
        dest.complete_destruction();
        
        let health_before = dest.health;
        dest.apply_damage(50.0);
        
        // Destroyed objects should ignore damage
        assert_eq!(dest.health, health_before);
    }

    #[test]
    fn test_destroying_state_ignores_damage() {
        let mut dest = Destructible::new(
            DestructibleId(1),
            DestructibleConfig::default(),
            Vec3::ZERO,
        );

        dest.destroy(); // Now in Destroying state
        
        let health_before = dest.health;
        dest.apply_damage(50.0);
        
        // Destroying objects should also ignore damage
        assert_eq!(dest.health, health_before);
    }

    // ============================================================================
    // DEFAULT CONFIG TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_debris_shape_default() {
        let shape = DebrisShape::default();
        if let DebrisShape::Box { half_extents } = shape {
            assert_eq!(half_extents, Vec3::splat(0.2));
        } else {
            panic!("Default debris shape should be Box");
        }
    }

    #[test]
    fn test_debris_config_default() {
        let config = DebrisConfig::default();
        
        assert_eq!(config.mass, 1.0);
        assert_eq!(config.velocity_factor, 1.0);
        assert_eq!(config.angular_velocity_factor, 0.5);
        assert_eq!(config.lifetime, 10.0);
        assert!(!config.can_damage);
        assert_eq!(config.damage_on_hit, 0.0);
    }

    #[test]
    fn test_destructible_config_default() {
        let config = DestructibleConfig::default();
        
        assert_eq!(config.max_health, 100.0);
        assert_eq!(config.damage_threshold, 10.0);
        assert_eq!(config.force_to_damage, 0.1);
        assert_eq!(config.destruction_force, 5.0);
        assert!(config.destruction_sound.is_none());
        assert!(config.destruction_particles.is_none());
    }

    #[test]
    fn test_destruction_trigger_default() {
        let trigger = DestructionTrigger::default();
        if let DestructionTrigger::Force { threshold } = trigger {
            assert_eq!(threshold, 1000.0);
        } else {
            panic!("Default trigger should be Force");
        }
    }

    // ============================================================================
    // MANAGER TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_manager_get_nonexistent() {
        let manager = DestructionManager::new();
        assert!(manager.get(DestructibleId(999)).is_none());
    }

    #[test]
    fn test_manager_remove_nonexistent() {
        let mut manager = DestructionManager::new();
        assert!(!manager.remove_destructible(DestructibleId(999)));
    }

    #[test]
    fn test_manager_apply_damage_nonexistent() {
        let mut manager = DestructionManager::new();
        // Should not panic
        manager.apply_damage(DestructibleId(999), 50.0);
    }

    #[test]
    fn test_manager_apply_force_nonexistent() {
        let mut manager = DestructionManager::new();
        // Should not panic
        manager.apply_force(DestructibleId(999), 100.0);
    }

    #[test]
    fn test_manager_on_collision_nonexistent() {
        let mut manager = DestructionManager::new();
        // Should not panic
        manager.on_collision(DestructibleId(999), 50.0);
    }

    #[test]
    fn test_manager_destroy_nonexistent() {
        let mut manager = DestructionManager::new();
        // Should not panic
        manager.destroy(DestructibleId(999));
    }

    #[test]
    fn test_manager_get_debris_nonexistent() {
        let manager = DestructionManager::new();
        assert!(manager.get_debris(DebrisId(999)).is_none());
    }

    #[test]
    fn test_manager_default_limits() {
        let manager = DestructionManager::new();
        assert_eq!(manager.max_debris, 500);
        assert_eq!(manager.default_debris_lifetime, 10.0);
    }

    #[test]
    fn test_active_debris_count() {
        let mut manager = DestructionManager::new();

        let config = DestructibleConfig {
            fracture_pattern: FracturePattern {
                debris: vec![
                    DebrisConfig { lifetime: 1.0, ..Default::default() },
                    DebrisConfig { lifetime: 10.0, ..Default::default() },
                ],
                center_of_mass: Vec3::ZERO,
            },
            trigger: DestructionTrigger::Manual,
            ..Default::default()
        };

        let id = manager.add_destructible(config, Vec3::ZERO);
        manager.destroy(id);
        manager.update(0.016, Vec3::ZERO);

        assert_eq!(manager.active_debris_count(), 2);

        // Age past first debris lifetime
        manager.update(2.0, Vec3::ZERO);
        assert_eq!(manager.debris_count(), 1); // One removed
    }
}

