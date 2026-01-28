//! Underwater Particle Effects
//!
//! Provides particle systems for underwater atmosphere:
//! - Air bubbles (player breathing, vents)
//! - Debris/sediment particles
//! - Light rays (volumetric god rays)
//! - Plankton/organic particles

use glam::Vec3;

/// Configuration for underwater particles
#[derive(Clone, Debug, PartialEq)]
pub struct UnderwaterParticleConfig {
    /// Maximum bubble particles
    pub max_bubbles: u32,
    /// Maximum debris particles  
    pub max_debris: u32,
    /// Bubble rise speed
    pub bubble_rise_speed: f32,
    /// Bubble wobble amplitude
    pub bubble_wobble: f32,
    /// Debris drift speed
    pub debris_drift_speed: f32,
    /// Ambient debris density (0-1)
    pub ambient_density: f32,
    /// Player bubble rate (bubbles per second when underwater)
    pub player_bubble_rate: f32,
}

impl Default for UnderwaterParticleConfig {
    fn default() -> Self {
        Self {
            max_bubbles: 2000,
            max_debris: 5000,
            bubble_rise_speed: 1.5,
            bubble_wobble: 0.3,
            debris_drift_speed: 0.1,
            ambient_density: 0.3,
            player_bubble_rate: 3.0,
        }
    }
}

impl UnderwaterParticleConfig {
    /// Create config for murky water (lots of debris)
    pub fn murky() -> Self {
        Self {
            max_debris: 10000,
            ambient_density: 0.8,
            debris_drift_speed: 0.05,
            ..Default::default()
        }
    }

    /// Create config for crystal clear water (minimal debris)
    pub fn crystal_clear() -> Self {
        Self {
            max_debris: 1000,
            ambient_density: 0.1,
            bubble_rise_speed: 2.0,
            ..Default::default()
        }
    }

    /// Create config for deep ocean (slow, sparse)
    pub fn deep_ocean() -> Self {
        Self {
            max_bubbles: 500,
            max_debris: 3000,
            bubble_rise_speed: 0.8,
            debris_drift_speed: 0.02,
            ambient_density: 0.2,
            player_bubble_rate: 2.0,
            ..Default::default()
        }
    }
}

/// Type of underwater particle
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum UnderwaterParticleType {
    /// Air bubble (rises to surface)
    Bubble = 0,
    /// Organic debris (drifts slowly)
    Debris = 1,
    /// Plankton (bioluminescent option)
    Plankton = 2,
    /// Sediment (sinks slowly)
    Sediment = 3,
}

/// A single underwater particle
#[derive(Clone, Copy, Debug)]
pub struct UnderwaterParticle {
    /// World position
    pub position: Vec3,
    /// Velocity
    pub velocity: Vec3,
    /// Size
    pub size: f32,
    /// Remaining lifetime
    pub lifetime: f32,
    /// Maximum lifetime
    pub max_lifetime: f32,
    /// Particle type
    pub particle_type: UnderwaterParticleType,
    /// Phase offset for wobble animation
    pub phase: f32,
}

impl UnderwaterParticle {
    /// Get opacity based on lifetime
    pub fn opacity(&self) -> f32 {
        let life_ratio = self.lifetime / self.max_lifetime;
        // Fade in at start, fade out at end
        if life_ratio > 0.9 {
            (1.0 - life_ratio) * 10.0
        } else if life_ratio < 0.1 {
            life_ratio * 10.0
        } else {
            1.0
        }
    }
}

/// GPU-compatible underwater particle
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GpuUnderwaterParticle {
    /// Position (xyz) + size (w)
    pub position_size: [f32; 4],
    /// Type (x) + opacity (y) + phase (z) + padding (w)
    pub type_opacity_phase: [f32; 4],
}

impl From<&UnderwaterParticle> for GpuUnderwaterParticle {
    fn from(p: &UnderwaterParticle) -> Self {
        Self {
            position_size: [p.position.x, p.position.y, p.position.z, p.size],
            type_opacity_phase: [p.particle_type as u8 as f32, p.opacity(), p.phase, 0.0],
        }
    }
}

/// Manages underwater particle effects
#[derive(Debug)]
pub struct UnderwaterParticleSystem {
    /// Bubble particles
    bubbles: Vec<UnderwaterParticle>,
    /// Debris particles
    debris: Vec<UnderwaterParticle>,
    /// Configuration
    config: UnderwaterParticleConfig,
    /// Bubble spawn accumulator
    bubble_spawn_accum: f32,
    /// Debris spawn accumulator
    debris_spawn_accum: f32,
    /// Random seed
    seed: u32,
    /// Current time (for animations)
    time: f32,
    /// Water surface height (bubbles pop above this)
    water_surface_y: f32,
}

impl UnderwaterParticleSystem {
    /// Create a new underwater particle system
    pub fn new(config: UnderwaterParticleConfig) -> Self {
        Self {
            bubbles: Vec::with_capacity(config.max_bubbles as usize),
            debris: Vec::with_capacity(config.max_debris as usize),
            config,
            bubble_spawn_accum: 0.0,
            debris_spawn_accum: 0.0,
            seed: 13579,
            time: 0.0,
            water_surface_y: 0.0,
        }
    }

    /// Set water surface height
    pub fn set_water_surface(&mut self, y: f32) {
        self.water_surface_y = y;
    }

    /// Get bubble count
    pub fn bubble_count(&self) -> usize {
        self.bubbles.len()
    }

    /// Get debris count
    pub fn debris_count(&self) -> usize {
        self.debris.len()
    }

    /// Get total particle count
    pub fn particle_count(&self) -> usize {
        self.bubbles.len() + self.debris.len()
    }

    /// Simple random
    fn next_random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.seed >> 16) & 0x7FFF) as f32 / 32767.0
    }

    /// Spawn a bubble at position
    pub fn spawn_bubble(&mut self, position: Vec3, size_mult: f32) {
        if self.bubbles.len() >= self.config.max_bubbles as usize {
            return;
        }

        // Pre-compute random values to avoid borrow issues
        let size_rand = self.next_random();
        let lifetime_rand = self.next_random();
        let vel_x_rand = self.next_random();
        let vel_y_rand = self.next_random();
        let vel_z_rand = self.next_random();
        let phase_rand = self.next_random();

        let size = 0.02 + size_rand * 0.08 * size_mult;
        let lifetime = 5.0 + lifetime_rand * 5.0;

        self.bubbles.push(UnderwaterParticle {
            position,
            velocity: Vec3::new(
                (vel_x_rand - 0.5) * 0.1,
                self.config.bubble_rise_speed * (0.8 + vel_y_rand * 0.4),
                (vel_z_rand - 0.5) * 0.1,
            ),
            size,
            lifetime,
            max_lifetime: lifetime,
            particle_type: UnderwaterParticleType::Bubble,
            phase: phase_rand * std::f32::consts::TAU,
        });
    }

    /// Spawn bubbles from player (breathing)
    pub fn spawn_player_bubbles(&mut self, player_pos: Vec3, count: u32) {
        for _ in 0..count {
            let offset = Vec3::new(
                (self.next_random() - 0.5) * 0.2,
                0.3 + self.next_random() * 0.2,
                (self.next_random() - 0.5) * 0.2,
            );
            self.spawn_bubble(player_pos + offset, 0.5);
        }
    }

    /// Spawn debris particle
    pub fn spawn_debris(&mut self, position: Vec3, particle_type: UnderwaterParticleType) {
        if self.debris.len() >= self.config.max_debris as usize {
            return;
        }

        // Pre-compute random values to avoid borrow issues
        let size_rand = self.next_random();
        let y_vel_rand = self.next_random();
        let lifetime_rand = self.next_random();
        let vel_x_rand = self.next_random();
        let vel_z_rand = self.next_random();
        let phase_rand = self.next_random();

        let size = match particle_type {
            UnderwaterParticleType::Debris => 0.01 + size_rand * 0.03,
            UnderwaterParticleType::Plankton => 0.005 + size_rand * 0.01,
            UnderwaterParticleType::Sediment => 0.02 + size_rand * 0.05,
            _ => 0.02,
        };

        let y_vel = match particle_type {
            UnderwaterParticleType::Sediment => -0.05 - y_vel_rand * 0.1,
            UnderwaterParticleType::Plankton => (y_vel_rand - 0.5) * 0.02,
            _ => (y_vel_rand - 0.5) * 0.05,
        };

        let lifetime = 10.0 + lifetime_rand * 20.0;

        self.debris.push(UnderwaterParticle {
            position,
            velocity: Vec3::new(
                (vel_x_rand - 0.5) * self.config.debris_drift_speed,
                y_vel,
                (vel_z_rand - 0.5) * self.config.debris_drift_speed,
            ),
            size,
            lifetime,
            max_lifetime: lifetime,
            particle_type,
            phase: phase_rand * std::f32::consts::TAU,
        });
    }

    /// Spawn ambient debris in a volume around a point
    pub fn spawn_ambient_debris(&mut self, center: Vec3, radius: f32, count: u32) {
        for _ in 0..count {
            let offset = Vec3::new(
                (self.next_random() - 0.5) * 2.0 * radius,
                (self.next_random() - 0.5) * 2.0 * radius,
                (self.next_random() - 0.5) * 2.0 * radius,
            );

            // Choose particle type based on random
            let r = self.next_random();
            let ptype = if r < 0.6 {
                UnderwaterParticleType::Debris
            } else if r < 0.85 {
                UnderwaterParticleType::Plankton
            } else {
                UnderwaterParticleType::Sediment
            };

            self.spawn_debris(center + offset, ptype);
        }
    }

    /// Update the particle system
    pub fn update(&mut self, dt: f32, camera_pos: Vec3) {
        self.time += dt;
        let wobble = self.config.bubble_wobble;
        let water_y = self.water_surface_y;

        // Update bubbles
        self.bubbles.retain_mut(|b| {
            b.lifetime -= dt;
            if b.lifetime <= 0.0 {
                return false;
            }

            // Apply wobble motion
            let wobble_offset = (self.time * 3.0 + b.phase).sin() * wobble * dt;
            b.position.x += wobble_offset;
            b.position.z += (self.time * 2.5 + b.phase * 1.3).cos() * wobble * dt * 0.7;

            // Move upward
            b.position += b.velocity * dt;

            // Bubbles grow slightly as they rise (pressure decrease)
            b.size += dt * 0.01;

            // Pop at surface
            if b.position.y >= water_y {
                return false;
            }

            true
        });

        // Update debris
        self.debris.retain_mut(|d| {
            d.lifetime -= dt;
            if d.lifetime <= 0.0 {
                return false;
            }

            // Gentle drifting motion
            let drift_x = (self.time * 0.5 + d.phase).sin() * 0.02 * dt;
            let drift_z = (self.time * 0.3 + d.phase * 0.7).cos() * 0.02 * dt;
            d.position.x += drift_x;
            d.position.z += drift_z;
            d.position += d.velocity * dt;

            true
        });

        // Spawn ambient debris around camera if underwater
        if camera_pos.y < water_y {
            self.debris_spawn_accum += dt * self.config.ambient_density * 10.0;
            while self.debris_spawn_accum >= 1.0 {
                self.debris_spawn_accum -= 1.0;
                self.spawn_ambient_debris(camera_pos, 15.0, 1);
            }
        }
    }

    /// Update with player position for breathing bubbles
    pub fn update_with_player(&mut self, dt: f32, camera_pos: Vec3, player_pos: Vec3, player_underwater: bool) {
        self.update(dt, camera_pos);

        // Spawn player breathing bubbles
        if player_underwater {
            self.bubble_spawn_accum += dt * self.config.player_bubble_rate;
            while self.bubble_spawn_accum >= 1.0 {
                self.bubble_spawn_accum -= 1.0;
                self.spawn_player_bubbles(player_pos, 1);
            }
        }
    }

    /// Get all GPU particles
    pub fn get_gpu_particles(&self) -> Vec<GpuUnderwaterParticle> {
        let mut result = Vec::with_capacity(self.bubbles.len() + self.debris.len());
        result.extend(self.bubbles.iter().map(GpuUnderwaterParticle::from));
        result.extend(self.debris.iter().map(GpuUnderwaterParticle::from));
        result
    }

    /// Get only bubble GPU particles
    pub fn get_bubble_gpu_particles(&self) -> Vec<GpuUnderwaterParticle> {
        self.bubbles.iter().map(GpuUnderwaterParticle::from).collect()
    }

    /// Get only debris GPU particles
    pub fn get_debris_gpu_particles(&self) -> Vec<GpuUnderwaterParticle> {
        self.debris.iter().map(GpuUnderwaterParticle::from).collect()
    }

    /// Clear all particles
    pub fn clear(&mut self) {
        self.bubbles.clear();
        self.debris.clear();
    }

    /// Set configuration
    pub fn set_config(&mut self, config: UnderwaterParticleConfig) {
        self.config = config;
    }
}

/// Bubble stream emitter (for vents, oxygen plants, etc.)
#[derive(Clone, Debug)]
pub struct BubbleStream {
    /// Position of the bubble source
    pub position: Vec3,
    /// Direction bubbles emit (usually up)
    pub direction: Vec3,
    /// Emission rate (bubbles per second)
    pub rate: f32,
    /// Bubble size multiplier
    pub size_mult: f32,
    /// Spread angle (radians)
    pub spread: f32,
    /// Whether the stream is active
    pub active: bool,
    /// Spawn accumulator
    spawn_accum: f32,
}

impl BubbleStream {
    /// Create a new bubble stream
    pub fn new(position: Vec3, rate: f32) -> Self {
        Self {
            position,
            direction: Vec3::Y,
            rate,
            size_mult: 1.0,
            spread: 0.2,
            active: true,
            spawn_accum: 0.0,
        }
    }

    /// Create a horizontal bubble stream
    pub fn horizontal(position: Vec3, direction: Vec3, rate: f32) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            rate,
            size_mult: 0.8,
            spread: 0.3,
            active: true,
            spawn_accum: 0.0,
        }
    }

    /// Update and spawn bubbles
    pub fn update(&mut self, dt: f32, system: &mut UnderwaterParticleSystem) {
        if !self.active {
            return;
        }

        self.spawn_accum += dt * self.rate;
        while self.spawn_accum >= 1.0 {
            self.spawn_accum -= 1.0;
            system.spawn_bubble(self.position, self.size_mult);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_underwater_config_default() {
        let config = UnderwaterParticleConfig::default();
        assert_eq!(config.max_bubbles, 2000);
        assert!(config.bubble_rise_speed > 0.0);
    }

    #[test]
    fn test_underwater_config_presets() {
        let murky = UnderwaterParticleConfig::murky();
        let clear = UnderwaterParticleConfig::crystal_clear();
        let deep = UnderwaterParticleConfig::deep_ocean();

        assert!(murky.ambient_density > clear.ambient_density);
        assert!(murky.max_debris > clear.max_debris);
        assert!(deep.bubble_rise_speed < clear.bubble_rise_speed);
    }

    #[test]
    fn test_system_creation() {
        let system = UnderwaterParticleSystem::new(UnderwaterParticleConfig::default());
        assert_eq!(system.particle_count(), 0);
        assert_eq!(system.bubble_count(), 0);
        assert_eq!(system.debris_count(), 0);
    }

    #[test]
    fn test_spawn_bubble() {
        let mut system = UnderwaterParticleSystem::new(UnderwaterParticleConfig::default());
        
        system.spawn_bubble(Vec3::ZERO, 1.0);
        assert_eq!(system.bubble_count(), 1);
        
        system.spawn_bubble(Vec3::ONE, 0.5);
        assert_eq!(system.bubble_count(), 2);
    }

    #[test]
    fn test_spawn_debris() {
        let mut system = UnderwaterParticleSystem::new(UnderwaterParticleConfig::default());
        
        system.spawn_debris(Vec3::ZERO, UnderwaterParticleType::Debris);
        system.spawn_debris(Vec3::ONE, UnderwaterParticleType::Plankton);
        system.spawn_debris(Vec3::NEG_ONE, UnderwaterParticleType::Sediment);
        
        assert_eq!(system.debris_count(), 3);
    }

    #[test]
    fn test_spawn_ambient() {
        let mut system = UnderwaterParticleSystem::new(UnderwaterParticleConfig::default());
        
        system.spawn_ambient_debris(Vec3::ZERO, 10.0, 50);
        assert_eq!(system.debris_count(), 50);
    }

    #[test]
    fn test_bubble_rises() {
        let config = UnderwaterParticleConfig {
            bubble_rise_speed: 2.0,
            ..Default::default()
        };
        let mut system = UnderwaterParticleSystem::new(config);
        system.set_water_surface(100.0); // High surface so bubbles don't pop
        
        system.spawn_bubble(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let initial_y = system.get_gpu_particles()[0].position_size[1];
        
        system.update(1.0, Vec3::ZERO);
        let final_y = system.get_gpu_particles()[0].position_size[1];
        
        assert!(final_y > initial_y, "Bubble should rise");
    }

    #[test]
    fn test_bubble_pops_at_surface() {
        let mut system = UnderwaterParticleSystem::new(UnderwaterParticleConfig::default());
        system.set_water_surface(0.0); // Surface at y=0
        
        // Spawn bubble below surface
        system.spawn_bubble(Vec3::new(0.0, -1.0, 0.0), 1.0);
        assert_eq!(system.bubble_count(), 1);
        
        // Update until bubble reaches surface
        for _ in 0..20 {
            system.update(0.2, Vec3::ZERO);
        }
        
        // Bubble should have popped
        assert_eq!(system.bubble_count(), 0);
    }

    #[test]
    fn test_player_bubbles() {
        let config = UnderwaterParticleConfig {
            player_bubble_rate: 10.0, // Fast rate for testing
            ..Default::default()
        };
        let mut system = UnderwaterParticleSystem::new(config);
        system.set_water_surface(10.0);
        
        system.update_with_player(1.0, Vec3::ZERO, Vec3::ZERO, true);
        
        assert!(system.bubble_count() > 0, "Player should generate bubbles");
    }

    #[test]
    fn test_no_player_bubbles_above_water() {
        let config = UnderwaterParticleConfig {
            player_bubble_rate: 10.0,
            ..Default::default()
        };
        let mut system = UnderwaterParticleSystem::new(config);
        system.set_water_surface(0.0);
        
        // Player not underwater
        system.update_with_player(1.0, Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, 5.0, 0.0), false);
        
        assert_eq!(system.bubble_count(), 0);
    }

    #[test]
    fn test_gpu_particle_conversion() {
        let particle = UnderwaterParticle {
            position: Vec3::new(1.0, 2.0, 3.0),
            velocity: Vec3::ZERO,
            size: 0.5,
            lifetime: 0.5,
            max_lifetime: 1.0,
            particle_type: UnderwaterParticleType::Bubble,
            phase: 0.0,
        };
        
        let gpu: GpuUnderwaterParticle = (&particle).into();
        
        assert_eq!(gpu.position_size[0], 1.0);
        assert_eq!(gpu.position_size[3], 0.5);
        assert_eq!(gpu.type_opacity_phase[0], 0.0); // Bubble type
    }

    #[test]
    fn test_particle_opacity_lifecycle() {
        let particle = UnderwaterParticle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            size: 1.0,
            lifetime: 0.05,
            max_lifetime: 1.0,
            particle_type: UnderwaterParticleType::Debris,
            phase: 0.0,
        };
        
        // Near end of life, should be fading
        assert!(particle.opacity() < 1.0);
    }

    #[test]
    fn test_bubble_stream() {
        let mut system = UnderwaterParticleSystem::new(UnderwaterParticleConfig::default());
        system.set_water_surface(100.0);
        
        let mut stream = BubbleStream::new(Vec3::ZERO, 10.0);
        
        stream.update(1.0, &mut system);
        
        assert!(system.bubble_count() > 0);
    }

    #[test]
    fn test_bubble_stream_inactive() {
        let mut system = UnderwaterParticleSystem::new(UnderwaterParticleConfig::default());
        
        let mut stream = BubbleStream::new(Vec3::ZERO, 10.0);
        stream.active = false;
        
        stream.update(1.0, &mut system);
        
        assert_eq!(system.bubble_count(), 0);
    }

    #[test]
    fn test_clear_particles() {
        let mut system = UnderwaterParticleSystem::new(UnderwaterParticleConfig::default());
        
        system.spawn_bubble(Vec3::ZERO, 1.0);
        system.spawn_debris(Vec3::ZERO, UnderwaterParticleType::Debris);
        assert!(system.particle_count() > 0);
        
        system.clear();
        assert_eq!(system.particle_count(), 0);
    }

    #[test]
    fn test_particle_type_values() {
        assert_eq!(UnderwaterParticleType::Bubble as u8, 0);
        assert_eq!(UnderwaterParticleType::Debris as u8, 1);
        assert_eq!(UnderwaterParticleType::Plankton as u8, 2);
        assert_eq!(UnderwaterParticleType::Sediment as u8, 3);
    }

    #[test]
    fn test_max_particles_limit() {
        let config = UnderwaterParticleConfig {
            max_bubbles: 5,
            max_debris: 5,
            ..Default::default()
        };
        let mut system = UnderwaterParticleSystem::new(config);
        
        for _ in 0..10 {
            system.spawn_bubble(Vec3::ZERO, 1.0);
            system.spawn_debris(Vec3::ZERO, UnderwaterParticleType::Debris);
        }
        
        assert_eq!(system.bubble_count(), 5);
        assert_eq!(system.debris_count(), 5);
    }

    #[test]
    fn test_gpu_underwater_particle_size() {
        // GpuUnderwaterParticle: 2 * [f32; 4] = 32 bytes (wgpu aligned)
        assert_eq!(std::mem::size_of::<GpuUnderwaterParticle>(), 32);
    }
}
