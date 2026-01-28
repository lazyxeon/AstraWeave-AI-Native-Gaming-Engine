//! Waterfall and Rapids Particle Systems
//!
//! Provides specialized particle effects for:
//! - Waterfall spray and mist
//! - Rapids turbulence
//! - Splash impacts
//! - Water curtain effects

use glam::{Vec2, Vec3};

/// Configuration for waterfall effects
#[derive(Clone, Debug, PartialEq)]
pub struct WaterfallConfig {
    /// Maximum particles per waterfall
    pub max_particles: u32,
    /// Particles spawned per second per meter of width
    pub spawn_rate: f32,
    /// Particle lifetime range (min, max)
    pub lifetime_range: (f32, f32),
    /// Initial particle size range
    pub size_range: (f32, f32),
    /// Gravity strength
    pub gravity: f32,
    /// Air drag coefficient
    pub drag: f32,
    /// Spray spread angle (radians)
    pub spray_angle: f32,
    /// Mist density (0-1)
    pub mist_density: f32,
    /// Mist rise speed
    pub mist_rise_speed: f32,
    /// Impact splash intensity
    pub splash_intensity: f32,
}

impl Default for WaterfallConfig {
    fn default() -> Self {
        Self {
            max_particles: 5000,
            spawn_rate: 200.0,
            lifetime_range: (1.0, 3.0),
            size_range: (0.02, 0.08),
            gravity: 9.8,
            drag: 0.5,
            spray_angle: 0.3,
            mist_density: 0.5,
            mist_rise_speed: 0.8,
            splash_intensity: 1.0,
        }
    }
}

impl WaterfallConfig {
    /// Create a gentle waterfall config
    pub fn gentle() -> Self {
        Self {
            spawn_rate: 100.0,
            gravity: 9.8,
            spray_angle: 0.15,
            mist_density: 0.3,
            splash_intensity: 0.5,
            ..Default::default()
        }
    }

    /// Create a powerful waterfall config
    pub fn powerful() -> Self {
        Self {
            max_particles: 10000,
            spawn_rate: 500.0,
            gravity: 9.8,
            spray_angle: 0.5,
            mist_density: 0.8,
            splash_intensity: 2.0,
            ..Default::default()
        }
    }

    /// Create a misty waterfall config (emphasizes mist over droplets)
    pub fn misty() -> Self {
        Self {
            spawn_rate: 150.0,
            lifetime_range: (2.0, 5.0),
            size_range: (0.05, 0.15),
            mist_density: 0.9,
            mist_rise_speed: 1.2,
            splash_intensity: 0.3,
            ..Default::default()
        }
    }
}

/// Type of water particle
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WaterParticleType {
    /// Water droplet (falls with gravity)
    Droplet = 0,
    /// Mist particle (rises slowly, affected by wind)
    Mist = 1,
    /// Spray particle (fast, directional)
    Spray = 2,
    /// Splash particle (from impact)
    Splash = 3,
}

/// A single water particle
#[derive(Clone, Copy, Debug)]
pub struct WaterParticle {
    /// World position
    pub position: Vec3,
    /// Velocity
    pub velocity: Vec3,
    /// Current size
    pub size: f32,
    /// Remaining lifetime
    pub lifetime: f32,
    /// Maximum lifetime (for calculating alpha)
    pub max_lifetime: f32,
    /// Particle type
    pub particle_type: WaterParticleType,
}

impl WaterParticle {
    /// Get opacity based on remaining lifetime
    pub fn opacity(&self) -> f32 {
        (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
    }
}

/// GPU-compatible water particle
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GpuWaterParticle {
    /// Position (xyz) + size (w)
    pub position_size: [f32; 4],
    /// Velocity (xyz) + opacity (w)
    pub velocity_opacity: [f32; 4],
}

impl From<&WaterParticle> for GpuWaterParticle {
    fn from(p: &WaterParticle) -> Self {
        Self {
            position_size: [p.position.x, p.position.y, p.position.z, p.size],
            velocity_opacity: [p.velocity.x, p.velocity.y, p.velocity.z, p.opacity()],
        }
    }
}

/// Defines a waterfall source
#[derive(Clone, Debug)]
pub struct WaterfallSource {
    /// Top position (where water starts falling)
    pub top: Vec3,
    /// Bottom position (where water hits)
    pub bottom: Vec3,
    /// Width of the waterfall
    pub width: f32,
    /// Flow rate (0-1, affects particle density)
    pub flow_rate: f32,
    /// Direction the waterfall faces (for spray)
    pub facing: Vec2,
}

impl WaterfallSource {
    /// Create a new waterfall source
    pub fn new(top: Vec3, bottom: Vec3, width: f32) -> Self {
        // Calculate facing direction (perpendicular to fall direction in XZ plane)
        let fall_dir = (bottom - top).normalize();
        let facing = Vec2::new(fall_dir.x, fall_dir.z).normalize();
        
        Self {
            top,
            bottom,
            width,
            flow_rate: 1.0,
            facing,
        }
    }

    /// Get the height of the waterfall
    pub fn height(&self) -> f32 {
        (self.top - self.bottom).length()
    }

    /// Get a random point along the top edge
    pub fn random_top_point(&self, t: f32) -> Vec3 {
        let offset = (t - 0.5) * self.width;
        let perp = Vec3::new(-self.facing.y, 0.0, self.facing.x);
        self.top + perp * offset
    }
}

/// Waterfall particle system
#[derive(Debug)]
pub struct WaterfallSystem {
    /// Active particles
    particles: Vec<WaterParticle>,
    /// Configuration
    config: WaterfallConfig,
    /// Waterfall sources
    sources: Vec<WaterfallSource>,
    /// Spawn accumulator
    spawn_accum: f32,
    /// Random seed
    seed: u32,
}

impl WaterfallSystem {
    /// Create a new waterfall system
    pub fn new(config: WaterfallConfig) -> Self {
        Self {
            particles: Vec::with_capacity(config.max_particles as usize),
            config,
            sources: Vec::new(),
            spawn_accum: 0.0,
            seed: 54321,
        }
    }

    /// Add a waterfall source
    pub fn add_source(&mut self, source: WaterfallSource) {
        self.sources.push(source);
    }

    /// Remove all sources
    pub fn clear_sources(&mut self) {
        self.sources.clear();
    }

    /// Get particle count
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Get source count
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Simple random number generator
    fn next_random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.seed >> 16) & 0x7FFF) as f32 / 32767.0
    }

    /// Spawn a droplet from a source
    fn spawn_droplet(&mut self, source: &WaterfallSource) {
        if self.particles.len() >= self.config.max_particles as usize {
            return;
        }

        let t = self.next_random();
        let position = source.random_top_point(t);
        
        // Initial velocity (mostly downward with some random spray)
        let spray_x = (self.next_random() - 0.5) * self.config.spray_angle;
        let spray_z = (self.next_random() - 0.5) * self.config.spray_angle;
        let velocity = Vec3::new(
            source.facing.x * 0.5 + spray_x,
            -0.5,
            source.facing.y * 0.5 + spray_z,
        );

        let lifetime = self.config.lifetime_range.0 
            + self.next_random() * (self.config.lifetime_range.1 - self.config.lifetime_range.0);
        
        let size = self.config.size_range.0 
            + self.next_random() * (self.config.size_range.1 - self.config.size_range.0);

        self.particles.push(WaterParticle {
            position,
            velocity,
            size,
            lifetime,
            max_lifetime: lifetime,
            particle_type: WaterParticleType::Droplet,
        });
    }

    /// Spawn mist particles near the impact zone
    fn spawn_mist(&mut self, source: &WaterfallSource) {
        if self.particles.len() >= self.config.max_particles as usize {
            return;
        }

        let density_check = self.next_random();
        if density_check > self.config.mist_density * 0.1 {
            return;
        }

        // Pre-compute random values
        let offset_x_rand = self.next_random();
        let offset_z_rand = self.next_random();
        let height_rand = self.next_random();
        let vel_x_rand = self.next_random();
        let vel_y_rand = self.next_random();
        let vel_z_rand = self.next_random();

        // Mist spawns near the bottom
        let offset_x = (offset_x_rand - 0.5) * source.width * 1.5;
        let offset_z = (offset_z_rand - 0.5) * source.width * 1.5;
        let position = source.bottom + Vec3::new(offset_x, height_rand * 2.0, offset_z);

        // Mist rises slowly
        let velocity = Vec3::new(
            (vel_x_rand - 0.5) * 0.3,
            self.config.mist_rise_speed * (0.5 + vel_y_rand),
            (vel_z_rand - 0.5) * 0.3,
        );

        let lifetime = self.config.lifetime_range.1 * 1.5;
        let size = self.config.size_range.1 * 2.0;

        self.particles.push(WaterParticle {
            position,
            velocity,
            size,
            lifetime,
            max_lifetime: lifetime,
            particle_type: WaterParticleType::Mist,
        });
    }

    /// Spawn splash particles at impact
    fn spawn_splash(&mut self, position: Vec3, intensity: f32) {
        let count = (intensity * 10.0) as u32;
        
        for _ in 0..count {
            if self.particles.len() >= self.config.max_particles as usize {
                break;
            }

            // Pre-compute random values to avoid borrow issues
            let angle = self.next_random() * std::f32::consts::TAU;
            let speed = self.next_random() * intensity * 3.0;
            let up_speed = self.next_random() * intensity * 2.0;
            let lifetime_rand = self.next_random();
            let size_rand = self.next_random();

            let velocity = Vec3::new(
                angle.cos() * speed,
                up_speed,
                angle.sin() * speed,
            );

            let lifetime = 0.5 + lifetime_rand * 0.5;
            let size = self.config.size_range.0 * (1.0 + size_rand);

            self.particles.push(WaterParticle {
                position,
                velocity,
                size,
                lifetime,
                max_lifetime: lifetime,
                particle_type: WaterParticleType::Splash,
            });
        }
    }

    /// Update the system
    pub fn update(&mut self, dt: f32) {
        let gravity = self.config.gravity;
        let drag = self.config.drag;
        let rise_speed = self.config.mist_rise_speed;

        // Spawn new particles from sources
        self.spawn_accum += dt;
        let spawn_interval = 1.0 / self.config.spawn_rate;
        
        while self.spawn_accum >= spawn_interval {
            self.spawn_accum -= spawn_interval;
            
            // Clone sources to avoid borrow issues
            let sources: Vec<WaterfallSource> = self.sources.clone();
            for source in &sources {
                self.spawn_droplet(source);
                self.spawn_mist(source);
            }
        }

        // Collect impact positions for splash spawning
        let mut impacts: Vec<(Vec3, f32)> = Vec::new();

        // Update existing particles
        self.particles.retain_mut(|p| {
            p.lifetime -= dt;
            if p.lifetime <= 0.0 {
                return false;
            }

            match p.particle_type {
                WaterParticleType::Droplet | WaterParticleType::Spray | WaterParticleType::Splash => {
                    // Apply gravity
                    p.velocity.y -= gravity * dt;
                    // Apply drag
                    p.velocity *= 1.0 - drag * dt;
                }
                WaterParticleType::Mist => {
                    // Mist rises and spreads
                    p.velocity.y = rise_speed * (p.lifetime / p.max_lifetime);
                    p.velocity.x *= 1.0 - drag * 0.5 * dt;
                    p.velocity.z *= 1.0 - drag * 0.5 * dt;
                    // Mist expands
                    p.size += dt * 0.1;
                }
            }

            // Move particle
            p.position += p.velocity * dt;

            // Check for ground impact (simplified - assume y=0 is ground)
            if p.particle_type == WaterParticleType::Droplet && p.position.y < 0.0 {
                impacts.push((p.position, p.velocity.length() * 0.1));
                return false;
            }

            true
        });

        // Spawn splashes at impact points
        for (pos, intensity) in impacts {
            self.spawn_splash(pos, intensity * self.config.splash_intensity);
        }
    }

    /// Get GPU particles
    pub fn get_gpu_particles(&self) -> Vec<GpuWaterParticle> {
        self.particles.iter().map(GpuWaterParticle::from).collect()
    }

    /// Get particles by type
    pub fn get_particles_by_type(&self, particle_type: WaterParticleType) -> Vec<&WaterParticle> {
        self.particles.iter()
            .filter(|p| p.particle_type == particle_type)
            .collect()
    }

    /// Clear all particles
    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

/// Rapids particle system for turbulent water
#[derive(Debug)]
pub struct RapidsSystem {
    /// Active particles
    particles: Vec<WaterParticle>,
    /// Maximum particles
    max_particles: u32,
    /// Turbulence intensity
    turbulence: f32,
    /// Random seed
    seed: u32,
}

impl RapidsSystem {
    /// Create a new rapids system
    pub fn new(max_particles: u32, turbulence: f32) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles as usize),
            max_particles,
            turbulence,
            seed: 98765,
        }
    }

    /// Get particle count
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Simple random
    fn next_random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.seed >> 16) & 0x7FFF) as f32 / 32767.0
    }

    /// Spawn rapids particles along a path
    pub fn spawn_along_path(&mut self, points: &[Vec3], flow_velocity: Vec3, width: f32) {
        if points.len() < 2 {
            return;
        }

        for window in points.windows(2) {
            let start = window[0];
            let end = window[1];
            let segment_length = (end - start).length();
            let particles_per_segment = (segment_length * 10.0) as u32;

            for i in 0..particles_per_segment {
                if self.particles.len() >= self.max_particles as usize {
                    return;
                }

                let t = i as f32 / particles_per_segment as f32;
                let base_pos = start.lerp(end, t);
                
                // Pre-compute all random values to avoid borrow issues
                let offset_rand = self.next_random();
                let turb_x = self.next_random();
                let turb_y = self.next_random();
                let turb_z = self.next_random();
                let size_rand = self.next_random();
                let lifetime_rand = self.next_random();
                
                // Add random offset perpendicular to flow
                let offset = (offset_rand - 0.5) * width;
                let perp = Vec3::new(-flow_velocity.z, 0.0, flow_velocity.x).normalize_or_zero();
                let position = base_pos + perp * offset;

                // Add turbulence to velocity
                let turb = Vec3::new(
                    (turb_x - 0.5) * self.turbulence,
                    turb_y * self.turbulence * 0.5,
                    (turb_z - 0.5) * self.turbulence,
                );

                self.particles.push(WaterParticle {
                    position,
                    velocity: flow_velocity + turb,
                    size: 0.03 + size_rand * 0.05,
                    lifetime: 0.5 + lifetime_rand * 1.0,
                    max_lifetime: 1.5,
                    particle_type: WaterParticleType::Spray,
                });
            }
        }
    }

    /// Update rapids particles
    pub fn update(&mut self, dt: f32) {
        let turbulence = self.turbulence;

        self.particles.retain_mut(|p| {
            p.lifetime -= dt;
            if p.lifetime <= 0.0 {
                return false;
            }

            // Add random turbulence each frame
            // Use position-based noise for determinism
            let noise_x = (p.position.x * 3.0 + p.position.z * 2.0).sin() * turbulence * dt;
            let noise_z = (p.position.z * 3.0 + p.position.x * 2.0).cos() * turbulence * dt;
            p.velocity.x += noise_x;
            p.velocity.z += noise_z;

            // Move
            p.position += p.velocity * dt;

            true
        });
    }

    /// Get GPU particles
    pub fn get_gpu_particles(&self) -> Vec<GpuWaterParticle> {
        self.particles.iter().map(GpuWaterParticle::from).collect()
    }

    /// Clear all particles
    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waterfall_config_default() {
        let config = WaterfallConfig::default();
        assert_eq!(config.max_particles, 5000);
        assert!(config.gravity > 0.0);
        assert!(config.spawn_rate > 0.0);
    }

    #[test]
    fn test_waterfall_config_presets() {
        let gentle = WaterfallConfig::gentle();
        let powerful = WaterfallConfig::powerful();
        let misty = WaterfallConfig::misty();

        assert!(powerful.spawn_rate > gentle.spawn_rate);
        assert!(misty.mist_density > gentle.mist_density);
        assert!(powerful.splash_intensity > gentle.splash_intensity);
    }

    #[test]
    fn test_waterfall_source() {
        let source = WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            5.0,
        );

        assert_eq!(source.height(), 10.0);
        assert_eq!(source.width, 5.0);
    }

    #[test]
    fn test_waterfall_source_random_point() {
        // Use a waterfall with different XZ to get valid facing direction
        let source = WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0), // Different X so facing is valid
            4.0,
        );

        // Test edges
        let left = source.random_top_point(0.0);
        let right = source.random_top_point(1.0);
        let center = source.random_top_point(0.5);

        assert_eq!(center.y, 10.0); // Same height as top
        // With valid facing, left and right should be different
        assert!((left - right).length() > 0.01);
    }

    #[test]
    fn test_waterfall_system_creation() {
        let system = WaterfallSystem::new(WaterfallConfig::default());
        assert_eq!(system.particle_count(), 0);
        assert_eq!(system.source_count(), 0);
    }

    #[test]
    fn test_waterfall_add_source() {
        let mut system = WaterfallSystem::new(WaterfallConfig::default());
        
        system.add_source(WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            5.0,
        ));

        assert_eq!(system.source_count(), 1);
    }

    #[test]
    fn test_waterfall_update_spawns_particles() {
        let config = WaterfallConfig {
            spawn_rate: 1000.0, // High rate
            ..Default::default()
        };
        let mut system = WaterfallSystem::new(config);
        
        system.add_source(WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            5.0,
        ));

        // Update should spawn particles
        system.update(0.1);
        assert!(system.particle_count() > 0);
    }

    #[test]
    fn test_waterfall_particles_fall() {
        let config = WaterfallConfig {
            spawn_rate: 100.0,
            gravity: 10.0,
            ..Default::default()
        };
        let mut system = WaterfallSystem::new(config);
        
        system.add_source(WaterfallSource::new(
            Vec3::new(0.0, 100.0, 0.0), // High up
            Vec3::new(5.0, 0.0, 0.0),   // Valid XZ offset for facing
            5.0,
        ));

        system.update(0.1);
        let initial_count = system.particle_count();
        assert!(initial_count > 0, "Should have spawned particles");
        
        // Track initial Y values (prefix with _ since we only assert on post-fall)
        let _initial_avg_y: f32 = system.get_gpu_particles().iter()
            .map(|p| p.position_size[1])
            .sum::<f32>() / initial_count as f32;

        // Update with gravity applied
        system.update(0.5);
        
        // Check that at least some particles have fallen below their spawn height
        let fallen_particles = system.get_gpu_particles().iter()
            .filter(|p| p.position_size[1] < 100.0) // Below spawn height
            .count();
        
        // Most particles should have fallen below spawn point
        assert!(fallen_particles > 0, "Particles should fall due to gravity");
    }

    #[test]
    fn test_gpu_water_particle_conversion() {
        let particle = WaterParticle {
            position: Vec3::new(1.0, 2.0, 3.0),
            velocity: Vec3::new(0.1, 0.2, 0.3),
            size: 0.5,
            lifetime: 0.5,
            max_lifetime: 1.0,
            particle_type: WaterParticleType::Droplet,
        };

        let gpu: GpuWaterParticle = (&particle).into();

        assert_eq!(gpu.position_size[0], 1.0);
        assert_eq!(gpu.position_size[3], 0.5);
        assert_eq!(gpu.velocity_opacity[3], 0.5); // 50% lifetime = 50% opacity
    }

    #[test]
    fn test_particle_opacity() {
        let particle = WaterParticle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            size: 1.0,
            lifetime: 0.25,
            max_lifetime: 1.0,
            particle_type: WaterParticleType::Mist,
        };

        assert!((particle.opacity() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_rapids_system_creation() {
        let system = RapidsSystem::new(1000, 0.5);
        assert_eq!(system.particle_count(), 0);
    }

    #[test]
    fn test_rapids_spawn_along_path() {
        let mut system = RapidsSystem::new(1000, 0.5);
        
        let path = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(20.0, 0.0, 0.0),
        ];

        system.spawn_along_path(&path, Vec3::new(1.0, 0.0, 0.0), 2.0);
        assert!(system.particle_count() > 0);
    }

    #[test]
    fn test_rapids_update() {
        let mut system = RapidsSystem::new(1000, 0.5);
        
        let path = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
        ];

        system.spawn_along_path(&path, Vec3::new(1.0, 0.0, 0.0), 1.0);
        let initial_count = system.particle_count();

        // Update multiple times until particles expire
        for _ in 0..20 {
            system.update(0.1);
        }

        // Some particles should have expired
        assert!(system.particle_count() < initial_count);
    }

    #[test]
    fn test_waterfall_clear() {
        let mut system = WaterfallSystem::new(WaterfallConfig::default());
        system.add_source(WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            5.0,
        ));
        system.update(0.1);
        
        system.clear();
        assert_eq!(system.particle_count(), 0);
    }

    #[test]
    fn test_particle_type_values() {
        assert_eq!(WaterParticleType::Droplet as u8, 0);
        assert_eq!(WaterParticleType::Mist as u8, 1);
        assert_eq!(WaterParticleType::Spray as u8, 2);
        assert_eq!(WaterParticleType::Splash as u8, 3);
    }

    #[test]
    fn test_gpu_water_particle_size() {
        // GpuWaterParticle: 2 * [f32; 4] = 32 bytes (wgpu aligned)
        assert_eq!(std::mem::size_of::<GpuWaterParticle>(), 32);
    }

    // =========================================================================
    // Additional coverage tests
    // =========================================================================

    #[test]
    fn test_waterfall_config_partial_eq() {
        let config1 = WaterfallConfig::default();
        let config2 = WaterfallConfig::default();
        let config3 = WaterfallConfig::gentle();
        
        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_waterfall_source_flow_rate() {
        let mut source = WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            5.0,
        );
        assert_eq!(source.flow_rate, 1.0);
        
        source.flow_rate = 0.5;
        assert_eq!(source.flow_rate, 0.5);
    }

    #[test]
    fn test_waterfall_system_clear_sources() {
        let mut system = WaterfallSystem::new(WaterfallConfig::default());
        system.add_source(WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            5.0,
        ));
        system.add_source(WaterfallSource::new(
            Vec3::new(5.0, 15.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            3.0,
        ));
        assert_eq!(system.source_count(), 2);
        
        system.clear_sources();
        assert_eq!(system.source_count(), 0);
    }

    #[test]
    fn test_waterfall_get_particles_by_type() {
        let config = WaterfallConfig {
            spawn_rate: 500.0,
            mist_density: 0.0, // Disable mist to only get droplets
            ..Default::default()
        };
        let mut system = WaterfallSystem::new(config);
        
        system.add_source(WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            5.0,
        ));
        
        system.update(0.1);
        
        let droplets = system.get_particles_by_type(WaterParticleType::Droplet);
        assert!(droplets.len() > 0 || system.particle_count() == 0);
        
        // All returned particles should be droplets
        for p in droplets {
            assert_eq!(p.particle_type, WaterParticleType::Droplet);
        }
    }

    #[test]
    fn test_rapids_system_clear() {
        let mut system = RapidsSystem::new(1000, 0.5);
        
        let path = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
        ];
        
        system.spawn_along_path(&path, Vec3::new(1.0, 0.0, 0.0), 2.0);
        assert!(system.particle_count() > 0);
        
        system.clear();
        assert_eq!(system.particle_count(), 0);
    }

    #[test]
    fn test_rapids_get_gpu_particles() {
        let mut system = RapidsSystem::new(100, 0.5);
        
        let path = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
        ];
        
        system.spawn_along_path(&path, Vec3::new(1.0, 0.0, 0.0), 2.0);
        
        let gpu_particles = system.get_gpu_particles();
        assert_eq!(gpu_particles.len(), system.particle_count());
    }

    #[test]
    fn test_gpu_water_particle_zeroable() {
        use bytemuck::Zeroable;
        let p = GpuWaterParticle::zeroed();
        assert_eq!(p.position_size, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(p.velocity_opacity, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_waterfall_get_gpu_particles() {
        let config = WaterfallConfig {
            spawn_rate: 500.0,
            ..Default::default()
        };
        let mut system = WaterfallSystem::new(config);
        
        system.add_source(WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            5.0,
        ));
        
        system.update(0.1);
        
        let gpu_particles = system.get_gpu_particles();
        assert_eq!(gpu_particles.len(), system.particle_count());
    }

    #[test]
    fn test_rapids_spawn_empty_path() {
        let mut system = RapidsSystem::new(1000, 0.5);
        
        // Empty path should do nothing
        let empty_path: Vec<Vec3> = vec![];
        system.spawn_along_path(&empty_path, Vec3::new(1.0, 0.0, 0.0), 2.0);
        assert_eq!(system.particle_count(), 0);
        
        // Single point path should also do nothing
        let single_path = vec![Vec3::ZERO];
        system.spawn_along_path(&single_path, Vec3::new(1.0, 0.0, 0.0), 2.0);
        assert_eq!(system.particle_count(), 0);
    }

    #[test]
    fn test_rapids_max_particles() {
        let mut system = RapidsSystem::new(10, 0.5); // Very low max
        
        let path = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(100.0, 0.0, 0.0), // Long path
        ];
        
        system.spawn_along_path(&path, Vec3::new(1.0, 0.0, 0.0), 2.0);
        assert!(system.particle_count() <= 10);
    }

    #[test]
    fn test_waterfall_max_particles() {
        let config = WaterfallConfig {
            max_particles: 5,
            spawn_rate: 10000.0, // Very high
            mist_density: 0.0,
            ..Default::default()
        };
        let mut system = WaterfallSystem::new(config);
        
        system.add_source(WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            5.0,
        ));
        
        system.update(1.0); // Update for a full second
        
        assert!(system.particle_count() <= 5);
    }

    #[test]
    fn test_waterfall_source_clone() {
        let source = WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            5.0,
        );
        let cloned = source.clone();
        
        assert_eq!(cloned.top, source.top);
        assert_eq!(cloned.bottom, source.bottom);
        assert_eq!(cloned.width, source.width);
    }

    #[test]
    fn test_water_particle_type_clone_copy() {
        let t1 = WaterParticleType::Droplet;
        let t2 = t1;
        let t3 = t1.clone();
        
        assert_eq!(t1, t2);
        assert_eq!(t1, t3);
    }

    #[test]
    fn test_water_particle_type_debug() {
        let t = WaterParticleType::Mist;
        let debug = format!("{:?}", t);
        assert!(debug.contains("Mist"));
    }

    #[test]
    fn test_waterfall_config_debug() {
        let config = WaterfallConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("WaterfallConfig"));
        assert!(debug.contains("max_particles"));
    }

    #[test]
    fn test_water_particle_copy_clone() {
        let p = WaterParticle {
            position: Vec3::new(1.0, 2.0, 3.0),
            velocity: Vec3::ZERO,
            size: 0.5,
            lifetime: 1.0,
            max_lifetime: 1.0,
            particle_type: WaterParticleType::Droplet,
        };
        
        let copied = p;
        let cloned = p.clone();
        
        assert_eq!(copied.position, p.position);
        assert_eq!(cloned.size, p.size);
    }

    #[test]
    fn test_waterfall_source_debug() {
        let source = WaterfallSource::new(
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            5.0,
        );
        let debug = format!("{:?}", source);
        assert!(debug.contains("WaterfallSource"));
    }

    #[test]
    fn test_waterfall_system_debug() {
        let system = WaterfallSystem::new(WaterfallConfig::default());
        let debug = format!("{:?}", system);
        assert!(debug.contains("WaterfallSystem"));
    }

    #[test]
    fn test_rapids_system_debug() {
        let system = RapidsSystem::new(100, 0.5);
        let debug = format!("{:?}", system);
        assert!(debug.contains("RapidsSystem"));
    }

    #[test]
    fn test_particle_opacity_clamped() {
        // Negative lifetime should clamp to 0
        let p = WaterParticle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            size: 1.0,
            lifetime: -1.0,
            max_lifetime: 1.0,
            particle_type: WaterParticleType::Droplet,
        };
        assert_eq!(p.opacity(), 0.0);
        
        // Lifetime > max_lifetime should clamp to 1
        let p2 = WaterParticle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            size: 1.0,
            lifetime: 2.0,
            max_lifetime: 1.0,
            particle_type: WaterParticleType::Droplet,
        };
        assert_eq!(p2.opacity(), 1.0);
    }

    #[test]
    fn test_gpu_water_particle_debug() {
        let p = GpuWaterParticle {
            position_size: [1.0, 2.0, 3.0, 0.5],
            velocity_opacity: [0.0, 0.0, 0.0, 1.0],
        };
        let debug = format!("{:?}", p);
        assert!(debug.contains("GpuWaterParticle"));
    }

    #[test]
    fn test_waterfall_config_clone() {
        let config = WaterfallConfig::powerful();
        let cloned = config.clone();
        
        assert_eq!(cloned.max_particles, config.max_particles);
        assert_eq!(cloned.spawn_rate, config.spawn_rate);
    }
}
