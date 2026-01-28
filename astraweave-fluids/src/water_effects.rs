//! Unified Water Effects Manager
//!
//! Coordinates all water visual effects systems (caustics, god rays, reflections,
//! foam, particles) into a single production-ready API with proper lifecycle
//! management, validation, and performance monitoring.

use glam::{Mat4, Vec3};
use std::time::Instant;

use crate::caustics::{CausticsConfig, CausticsProjector, CausticsSystem, CausticsUniforms};
use crate::foam::{FoamConfig, FoamSystem};
use crate::god_rays::{GodRaysConfig, GodRaysSystem, GodRaysUniforms};
use crate::underwater::{UnderwaterConfig, UnderwaterState};
use crate::underwater_particles::{UnderwaterParticleConfig, UnderwaterParticleSystem};
use crate::water_reflections::{ReflectionUniforms, WaterReflectionConfig, WaterReflectionSystem};
use crate::waterfall::{WaterfallConfig, WaterfallSystem};

/// Error types for water effects operations
#[derive(Debug, Clone, PartialEq)]
pub enum WaterEffectsError {
    /// Invalid configuration value
    InvalidConfig { field: String, reason: String },
    /// System not initialized
    NotInitialized { system: String },
    /// Resource limit exceeded
    ResourceLimitExceeded { resource: String, limit: usize, requested: usize },
    /// Invalid state transition
    InvalidStateTransition { from: String, to: String },
}

impl std::fmt::Display for WaterEffectsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig { field, reason } => {
                write!(f, "Invalid config for '{}': {}", field, reason)
            }
            Self::NotInitialized { system } => {
                write!(f, "System '{}' not initialized", system)
            }
            Self::ResourceLimitExceeded { resource, limit, requested } => {
                write!(f, "Resource '{}' limit exceeded: {} requested, {} max", resource, requested, limit)
            }
            Self::InvalidStateTransition { from, to } => {
                write!(f, "Invalid state transition from '{}' to '{}'", from, to)
            }
        }
    }
}

impl std::error::Error for WaterEffectsError {}

/// Result type for water effects operations
pub type WaterEffectsResult<T> = Result<T, WaterEffectsError>;

/// Quality preset for water effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WaterQualityPreset {
    /// Minimal effects for low-end hardware
    Low,
    /// Balanced quality and performance
    #[default]
    Medium,
    /// High quality effects
    High,
    /// Maximum quality (demanding)
    Ultra,
    /// Custom configuration (use individual settings)
    Custom,
}

impl WaterQualityPreset {
    /// Get description for UI display
    pub fn description(&self) -> &'static str {
        match self {
            Self::Low => "Low - Minimal effects, best performance",
            Self::Medium => "Medium - Balanced quality and performance",
            Self::High => "High - Enhanced visuals",
            Self::Ultra => "Ultra - Maximum quality (demanding)",
            Self::Custom => "Custom - User-defined settings",
        }
    }
}

/// Master configuration for all water effects
#[derive(Debug, Clone)]
pub struct WaterEffectsConfig {
    /// Quality preset
    pub quality: WaterQualityPreset,
    /// Enable caustics
    pub caustics_enabled: bool,
    /// Caustics configuration
    pub caustics: CausticsConfig,
    /// Enable god rays
    pub god_rays_enabled: bool,
    /// God rays configuration
    pub god_rays: GodRaysConfig,
    /// Enable reflections
    pub reflections_enabled: bool,
    /// Reflection configuration
    pub reflections: WaterReflectionConfig,
    /// Enable foam
    pub foam_enabled: bool,
    /// Foam configuration
    pub foam: FoamConfig,
    /// Enable underwater particles
    pub underwater_particles_enabled: bool,
    /// Underwater particle configuration
    pub underwater_particles: UnderwaterParticleConfig,
    /// Enable waterfalls
    pub waterfalls_enabled: bool,
    /// Waterfall configuration
    pub waterfalls: WaterfallConfig,
    /// Underwater visual configuration
    pub underwater: UnderwaterConfig,
    /// Maximum total particles across all systems
    pub max_total_particles: usize,
    /// Frame budget for effects (milliseconds)
    pub frame_budget_ms: f32,
}

impl Default for WaterEffectsConfig {
    fn default() -> Self {
        Self::from_preset(WaterQualityPreset::Medium)
    }
}

impl WaterEffectsConfig {
    /// Create configuration from quality preset
    pub fn from_preset(preset: WaterQualityPreset) -> Self {
        match preset {
            WaterQualityPreset::Low => Self {
                quality: preset,
                caustics_enabled: false,
                caustics: CausticsConfig::murky(),
                god_rays_enabled: false,
                god_rays: GodRaysConfig::low_quality(),
                reflections_enabled: true,
                reflections: WaterReflectionConfig::low_quality(),
                foam_enabled: true,
                foam: FoamConfig::calm(),
                underwater_particles_enabled: false,
                underwater_particles: UnderwaterParticleConfig::default(),
                waterfalls_enabled: true,
                waterfalls: WaterfallConfig::default(),
                underwater: UnderwaterConfig::default(),
                max_total_particles: 5_000,
                frame_budget_ms: 2.0,
            },
            WaterQualityPreset::Medium => Self {
                quality: preset,
                caustics_enabled: true,
                caustics: CausticsConfig::default(),
                god_rays_enabled: true,
                god_rays: GodRaysConfig::default(),
                reflections_enabled: true,
                reflections: WaterReflectionConfig::default(),
                foam_enabled: true,
                foam: FoamConfig::default(),
                underwater_particles_enabled: true,
                underwater_particles: UnderwaterParticleConfig::default(),
                waterfalls_enabled: true,
                waterfalls: WaterfallConfig::default(),
                underwater: UnderwaterConfig::default(),
                max_total_particles: 20_000,
                frame_budget_ms: 4.0,
            },
            WaterQualityPreset::High => Self {
                quality: preset,
                caustics_enabled: true,
                caustics: CausticsConfig::shallow(),
                god_rays_enabled: true,
                god_rays: GodRaysConfig::tropical(),
                reflections_enabled: true,
                reflections: WaterReflectionConfig::high_quality(),
                foam_enabled: true,
                foam: FoamConfig::stormy(),
                underwater_particles_enabled: true,
                underwater_particles: UnderwaterParticleConfig::crystal_clear(),
                waterfalls_enabled: true,
                waterfalls: WaterfallConfig::powerful(),
                underwater: UnderwaterConfig::default(),
                max_total_particles: 50_000,
                frame_budget_ms: 6.0,
            },
            WaterQualityPreset::Ultra => Self {
                quality: preset,
                caustics_enabled: true,
                caustics: CausticsConfig::shallow(),
                god_rays_enabled: true,
                god_rays: GodRaysConfig::cinematic(),
                reflections_enabled: true,
                reflections: WaterReflectionConfig::high_quality(),
                foam_enabled: true,
                foam: FoamConfig::stormy(),
                underwater_particles_enabled: true,
                underwater_particles: UnderwaterParticleConfig::crystal_clear(),
                waterfalls_enabled: true,
                waterfalls: WaterfallConfig::powerful(),
                underwater: UnderwaterConfig::default(),
                max_total_particles: 100_000,
                frame_budget_ms: 8.0,
            },
            WaterQualityPreset::Custom => Self {
                quality: preset,
                caustics_enabled: true,
                caustics: CausticsConfig::default(),
                god_rays_enabled: true,
                god_rays: GodRaysConfig::default(),
                reflections_enabled: true,
                reflections: WaterReflectionConfig::default(),
                foam_enabled: true,
                foam: FoamConfig::default(),
                underwater_particles_enabled: true,
                underwater_particles: UnderwaterParticleConfig::default(),
                waterfalls_enabled: true,
                waterfalls: WaterfallConfig::default(),
                underwater: UnderwaterConfig::default(),
                max_total_particles: 20_000,
                frame_budget_ms: 4.0,
            },
        }
    }

    /// Validate configuration values
    pub fn validate(&self) -> WaterEffectsResult<()> {
        // Validate caustics
        if self.caustics.intensity < 0.0 || self.caustics.intensity > 2.0 {
            return Err(WaterEffectsError::InvalidConfig {
                field: "caustics.intensity".to_string(),
                reason: "Must be between 0.0 and 2.0".to_string(),
            });
        }
        if self.caustics.max_depth <= 0.0 {
            return Err(WaterEffectsError::InvalidConfig {
                field: "caustics.max_depth".to_string(),
                reason: "Must be positive".to_string(),
            });
        }

        // Validate god rays
        if self.god_rays.num_samples == 0 || self.god_rays.num_samples > 256 {
            return Err(WaterEffectsError::InvalidConfig {
                field: "god_rays.num_samples".to_string(),
                reason: "Must be between 1 and 256".to_string(),
            });
        }
        if self.god_rays.decay <= 0.0 || self.god_rays.decay >= 1.0 {
            return Err(WaterEffectsError::InvalidConfig {
                field: "god_rays.decay".to_string(),
                reason: "Must be between 0.0 and 1.0 (exclusive)".to_string(),
            });
        }

        // Validate reflections
        if self.reflections.resolution_scale <= 0.0 || self.reflections.resolution_scale > 2.0 {
            return Err(WaterEffectsError::InvalidConfig {
                field: "reflections.resolution_scale".to_string(),
                reason: "Must be between 0.0 (exclusive) and 2.0".to_string(),
            });
        }

        // Validate frame budget
        if self.frame_budget_ms <= 0.0 {
            return Err(WaterEffectsError::InvalidConfig {
                field: "frame_budget_ms".to_string(),
                reason: "Must be positive".to_string(),
            });
        }

        // Validate particle limit
        if self.max_total_particles == 0 {
            return Err(WaterEffectsError::InvalidConfig {
                field: "max_total_particles".to_string(),
                reason: "Must be at least 1".to_string(),
            });
        }

        Ok(())
    }
}

/// Performance statistics for water effects
#[derive(Debug, Clone, Default)]
pub struct WaterEffectsStats {
    /// Total update time in microseconds
    pub total_update_us: u64,
    /// Caustics update time
    pub caustics_us: u64,
    /// God rays update time
    pub god_rays_us: u64,
    /// Reflections update time
    pub reflections_us: u64,
    /// Foam update time
    pub foam_us: u64,
    /// Underwater particles update time
    pub underwater_particles_us: u64,
    /// Waterfalls update time
    pub waterfalls_us: u64,
    /// Total active particles
    pub total_particles: usize,
    /// Foam particle count
    pub foam_particles: usize,
    /// Underwater particle count
    pub underwater_particle_count: usize,
    /// Waterfall particle count
    pub waterfall_particles: usize,
    /// God ray shaft count
    pub god_ray_shafts: usize,
    /// Whether frame budget was exceeded
    pub budget_exceeded: bool,
    /// Frame number
    pub frame: u64,
}

impl WaterEffectsStats {
    /// Get total time as milliseconds
    pub fn total_ms(&self) -> f32 {
        self.total_update_us as f32 / 1000.0
    }

    /// Get breakdown as percentages
    pub fn breakdown_percentages(&self) -> [(&'static str, f32); 6] {
        let total = self.total_update_us.max(1) as f32;
        [
            ("Caustics", self.caustics_us as f32 / total * 100.0),
            ("God Rays", self.god_rays_us as f32 / total * 100.0),
            ("Reflections", self.reflections_us as f32 / total * 100.0),
            ("Foam", self.foam_us as f32 / total * 100.0),
            ("Underwater", self.underwater_particles_us as f32 / total * 100.0),
            ("Waterfalls", self.waterfalls_us as f32 / total * 100.0),
        ]
    }
}

/// Unified water effects manager
#[derive(Debug)]
pub struct WaterEffectsManager {
    /// Configuration
    config: WaterEffectsConfig,
    /// Caustics system
    caustics: CausticsSystem,
    /// God rays system
    god_rays: GodRaysSystem,
    /// Reflections system
    reflections: WaterReflectionSystem,
    /// Foam system
    foam: FoamSystem,
    /// Underwater particles
    underwater_particles: UnderwaterParticleSystem,
    /// Waterfall system
    waterfalls: WaterfallSystem,
    /// Underwater state
    underwater_state: UnderwaterState,
    /// Water surface height
    surface_height: f32,
    /// Current performance stats
    stats: WaterEffectsStats,
    /// Frame counter
    frame_count: u64,
    /// Initialized flag
    initialized: bool,
}

impl WaterEffectsManager {
    /// Create a new water effects manager with validation
    pub fn new(config: WaterEffectsConfig) -> WaterEffectsResult<Self> {
        config.validate()?;

        Ok(Self {
            caustics: CausticsSystem::new(config.caustics.clone()),
            god_rays: GodRaysSystem::new(config.god_rays.clone()),
            reflections: WaterReflectionSystem::new(config.reflections.clone()),
            foam: FoamSystem::new(config.foam.clone()),
            underwater_particles: UnderwaterParticleSystem::new(config.underwater_particles.clone()),
            waterfalls: WaterfallSystem::new(config.waterfalls.clone()),
            underwater_state: UnderwaterState::new(config.underwater.clone()),
            surface_height: 0.0,
            stats: WaterEffectsStats::default(),
            frame_count: 0,
            initialized: true,
            config,
        })
    }

    /// Create with default medium quality
    pub fn new_default() -> WaterEffectsResult<Self> {
        Self::new(WaterEffectsConfig::default())
    }

    /// Create from quality preset
    pub fn from_preset(preset: WaterQualityPreset) -> WaterEffectsResult<Self> {
        Self::new(WaterEffectsConfig::from_preset(preset))
    }

    /// Initialize systems for a water surface
    pub fn setup_water_surface(&mut self, surface_height: f32, bounds_min: glam::Vec2, bounds_max: glam::Vec2) {
        self.surface_height = surface_height;

        // Setup reflections
        self.reflections.setup_planar(surface_height);

        // Setup god rays
        self.god_rays.set_surface_height(surface_height);

        // Setup caustics projector
        self.caustics.clear_projectors();
        let projector = CausticsProjector::new(Vec3::new(0.0, -1.0, 0.2).normalize(), surface_height)
            .with_bounds(bounds_min, bounds_max);
        self.caustics.add_projector(projector);
    }

    /// Update all systems
    pub fn update(&mut self, dt: f32, camera_pos: Vec3, view_distance: f32) {
        let frame_start = Instant::now();
        self.frame_count += 1;

        let is_underwater = camera_pos.y < self.surface_height;

        // Update underwater state
        self.underwater_state.update(camera_pos.y, self.surface_height, dt);

        // Caustics
        let caustics_start = Instant::now();
        if self.config.caustics_enabled {
            self.caustics.update(dt);
        }
        let caustics_time = caustics_start.elapsed();

        // God rays
        let god_rays_start = Instant::now();
        if self.config.god_rays_enabled && is_underwater {
            self.god_rays.update(dt);
            self.god_rays.generate_shafts(camera_pos, view_distance);
        } else {
            self.god_rays.clear_shafts();
        }
        let god_rays_time = god_rays_start.elapsed();

        // Reflections (no per-frame update needed, just state tracking)
        let reflections_start = Instant::now();
        if self.config.reflections_enabled {
            self.reflections.set_surface_height(self.surface_height);
        }
        let reflections_time = reflections_start.elapsed();

        // Foam
        let foam_start = Instant::now();
        if self.config.foam_enabled {
            self.foam.update(dt);
        }
        let foam_time = foam_start.elapsed();

        // Underwater particles
        let underwater_start = Instant::now();
        if self.config.underwater_particles_enabled && is_underwater {
            self.underwater_particles.update(dt, camera_pos);
        }
        let underwater_time = underwater_start.elapsed();

        // Waterfalls
        let waterfalls_start = Instant::now();
        if self.config.waterfalls_enabled {
            self.waterfalls.update(dt);
        }
        let waterfalls_time = waterfalls_start.elapsed();

        // Update stats
        let total_time = frame_start.elapsed();
        self.stats = WaterEffectsStats {
            total_update_us: total_time.as_micros() as u64,
            caustics_us: caustics_time.as_micros() as u64,
            god_rays_us: god_rays_time.as_micros() as u64,
            reflections_us: reflections_time.as_micros() as u64,
            foam_us: foam_time.as_micros() as u64,
            underwater_particles_us: underwater_time.as_micros() as u64,
            waterfalls_us: waterfalls_time.as_micros() as u64,
            total_particles: self.total_particle_count(),
            foam_particles: self.foam.particle_count(),
            underwater_particle_count: self.underwater_particles.particle_count(),
            waterfall_particles: self.waterfalls.particle_count(),
            god_ray_shafts: self.god_rays.shaft_count(),
            budget_exceeded: total_time.as_secs_f32() * 1000.0 > self.config.frame_budget_ms,
            frame: self.frame_count,
        };
    }

    /// Get total particle count across all systems
    pub fn total_particle_count(&self) -> usize {
        self.foam.particle_count()
            + self.underwater_particles.particle_count()
            + self.waterfalls.particle_count()
    }

    /// Check if particle budget allows spawning more
    pub fn can_spawn_particles(&self, count: usize) -> bool {
        self.total_particle_count() + count <= self.config.max_total_particles
    }

    // === Accessors ===

    /// Get caustics system
    pub fn caustics(&self) -> &CausticsSystem {
        &self.caustics
    }

    /// Get mutable caustics system
    pub fn caustics_mut(&mut self) -> &mut CausticsSystem {
        &mut self.caustics
    }

    /// Get god rays system
    pub fn god_rays(&self) -> &GodRaysSystem {
        &self.god_rays
    }

    /// Get mutable god rays system
    pub fn god_rays_mut(&mut self) -> &mut GodRaysSystem {
        &mut self.god_rays
    }

    /// Get reflections system
    pub fn reflections(&self) -> &WaterReflectionSystem {
        &self.reflections
    }

    /// Get mutable reflections system
    pub fn reflections_mut(&mut self) -> &mut WaterReflectionSystem {
        &mut self.reflections
    }

    /// Get foam system
    pub fn foam(&self) -> &FoamSystem {
        &self.foam
    }

    /// Get mutable foam system
    pub fn foam_mut(&mut self) -> &mut FoamSystem {
        &mut self.foam
    }

    /// Get underwater particles
    pub fn underwater_particles(&self) -> &UnderwaterParticleSystem {
        &self.underwater_particles
    }

    /// Get mutable underwater particles
    pub fn underwater_particles_mut(&mut self) -> &mut UnderwaterParticleSystem {
        &mut self.underwater_particles
    }

    /// Get waterfall system
    pub fn waterfalls(&self) -> &WaterfallSystem {
        &self.waterfalls
    }

    /// Get mutable waterfall system
    pub fn waterfalls_mut(&mut self) -> &mut WaterfallSystem {
        &mut self.waterfalls
    }

    /// Get underwater state
    pub fn underwater_state(&self) -> &UnderwaterState {
        &self.underwater_state
    }

    /// Get current stats
    pub fn stats(&self) -> &WaterEffectsStats {
        &self.stats
    }

    /// Get configuration
    pub fn config(&self) -> &WaterEffectsConfig {
        &self.config
    }

    /// Get surface height
    pub fn surface_height(&self) -> f32 {
        self.surface_height
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    // === GPU Uniforms ===

    /// Get caustics uniforms for rendering
    pub fn get_caustics_uniforms(&self) -> CausticsUniforms {
        self.caustics.get_uniforms()
    }

    /// Get god rays uniforms for rendering
    pub fn get_god_rays_uniforms(&self, camera_underwater: bool) -> GodRaysUniforms {
        self.god_rays.get_uniforms(camera_underwater)
    }

    /// Get reflection uniforms for rendering
    pub fn get_reflection_uniforms(&self, view_proj: Mat4, camera_pos: Vec3) -> ReflectionUniforms {
        self.reflections.get_uniforms(view_proj, camera_pos)
    }

    // === Configuration Updates ===

    /// Update configuration with validation
    pub fn set_config(&mut self, config: WaterEffectsConfig) -> WaterEffectsResult<()> {
        config.validate()?;

        // Update individual systems
        self.caustics.set_config(config.caustics.clone());
        self.god_rays.set_config(config.god_rays.clone());
        self.reflections.set_config(config.reflections.clone());
        self.foam.set_config(config.foam.clone());
        self.underwater_particles.set_config(config.underwater_particles.clone());
        // Note: WaterfallSystem does not have set_config - reconfigure by recreating if needed

        self.config = config;
        Ok(())
    }

    /// Apply quality preset
    pub fn apply_preset(&mut self, preset: WaterQualityPreset) -> WaterEffectsResult<()> {
        self.set_config(WaterEffectsConfig::from_preset(preset))
    }

    /// Enable/disable caustics
    pub fn set_caustics_enabled(&mut self, enabled: bool) {
        self.config.caustics_enabled = enabled;
    }

    /// Enable/disable god rays
    pub fn set_god_rays_enabled(&mut self, enabled: bool) {
        self.config.god_rays_enabled = enabled;
    }

    /// Enable/disable reflections
    pub fn set_reflections_enabled(&mut self, enabled: bool) {
        self.config.reflections_enabled = enabled;
    }

    /// Enable/disable foam
    pub fn set_foam_enabled(&mut self, enabled: bool) {
        self.config.foam_enabled = enabled;
    }

    /// Enable/disable underwater particles
    pub fn set_underwater_particles_enabled(&mut self, enabled: bool) {
        self.config.underwater_particles_enabled = enabled;
    }

    /// Enable/disable waterfalls
    pub fn set_waterfalls_enabled(&mut self, enabled: bool) {
        self.config.waterfalls_enabled = enabled;
    }

    // === Cleanup ===

    /// Clear all particle systems
    pub fn clear_particles(&mut self) {
        self.foam.clear();
        self.underwater_particles.clear();
        self.waterfalls.clear();
    }

    /// Reset all systems to initial state
    pub fn reset(&mut self) {
        self.clear_particles();
        self.god_rays.clear_shafts();
        self.caustics.clear_projectors();
        self.frame_count = 0;
        self.stats = WaterEffectsStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = WaterEffectsError::InvalidConfig {
            field: "test".to_string(),
            reason: "bad value".to_string(),
        };
        assert!(err.to_string().contains("test"));
        assert!(err.to_string().contains("bad value"));
    }

    #[test]
    fn test_quality_preset_description() {
        assert!(!WaterQualityPreset::Low.description().is_empty());
        assert!(!WaterQualityPreset::Medium.description().is_empty());
        assert!(!WaterQualityPreset::High.description().is_empty());
        assert!(!WaterQualityPreset::Ultra.description().is_empty());
        assert!(!WaterQualityPreset::Custom.description().is_empty());
    }

    #[test]
    fn test_config_from_preset() {
        let low = WaterEffectsConfig::from_preset(WaterQualityPreset::Low);
        let high = WaterEffectsConfig::from_preset(WaterQualityPreset::High);

        // Low should have fewer particles
        assert!(low.max_total_particles < high.max_total_particles);

        // Low should have caustics disabled
        assert!(!low.caustics_enabled);
        assert!(high.caustics_enabled);
    }

    #[test]
    fn test_config_validation_valid() {
        let config = WaterEffectsConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_caustics() {
        let mut config = WaterEffectsConfig::default();
        config.caustics.intensity = -1.0;
        assert!(config.validate().is_err());

        config.caustics.intensity = 0.5;
        config.caustics.max_depth = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_god_rays() {
        let mut config = WaterEffectsConfig::default();
        config.god_rays.num_samples = 0;
        assert!(config.validate().is_err());

        config.god_rays.num_samples = 64;
        config.god_rays.decay = 1.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_reflections() {
        let mut config = WaterEffectsConfig::default();
        config.reflections.resolution_scale = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_budget() {
        let mut config = WaterEffectsConfig::default();
        config.frame_budget_ms = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_manager_creation() {
        let manager = WaterEffectsManager::new_default();
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert!(manager.is_initialized());
        assert_eq!(manager.total_particle_count(), 0);
    }

    #[test]
    fn test_manager_from_preset() {
        let low = WaterEffectsManager::from_preset(WaterQualityPreset::Low);
        let high = WaterEffectsManager::from_preset(WaterQualityPreset::High);

        assert!(low.is_ok());
        assert!(high.is_ok());
    }

    #[test]
    fn test_manager_invalid_config() {
        let mut config = WaterEffectsConfig::default();
        config.caustics.intensity = -1.0;

        let manager = WaterEffectsManager::new(config);
        assert!(manager.is_err());
    }

    #[test]
    fn test_setup_water_surface() {
        let mut manager = WaterEffectsManager::new_default().unwrap();
        
        manager.setup_water_surface(10.0, glam::Vec2::new(-50.0, -50.0), glam::Vec2::new(50.0, 50.0));
        
        assert_eq!(manager.surface_height(), 10.0);
        assert!(manager.reflections().planar().is_some());
        assert_eq!(manager.caustics().projector_count(), 1);
    }

    #[test]
    fn test_manager_update() {
        let mut manager = WaterEffectsManager::new_default().unwrap();
        manager.setup_water_surface(10.0, glam::Vec2::splat(-50.0), glam::Vec2::splat(50.0));

        // Update above water
        manager.update(0.016, Vec3::new(0.0, 20.0, 0.0), 100.0);
        assert_eq!(manager.god_rays().shaft_count(), 0);

        // Update underwater
        manager.update(0.016, Vec3::new(0.0, 5.0, 0.0), 100.0);
        assert!(manager.god_rays().shaft_count() > 0);
    }

    #[test]
    fn test_stats_collection() {
        let mut manager = WaterEffectsManager::new_default().unwrap();
        manager.setup_water_surface(10.0, glam::Vec2::splat(-50.0), glam::Vec2::splat(50.0));

        manager.update(0.016, Vec3::new(0.0, 5.0, 0.0), 100.0);

        let stats = manager.stats();
        assert!(stats.total_update_us > 0);
        assert_eq!(stats.frame, 1);
    }

    #[test]
    fn test_stats_breakdown() {
        let stats = WaterEffectsStats {
            total_update_us: 1000,
            caustics_us: 200,
            god_rays_us: 300,
            reflections_us: 100,
            foam_us: 150,
            underwater_particles_us: 150,
            waterfalls_us: 100,
            ..Default::default()
        };

        let breakdown = stats.breakdown_percentages();
        
        // Check percentages sum to ~100%
        let sum: f32 = breakdown.iter().map(|(_, p)| p).sum();
        assert!((sum - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_can_spawn_particles() {
        let manager = WaterEffectsManager::new_default().unwrap();

        // Should be able to spawn within limit
        assert!(manager.can_spawn_particles(100));
        
        // Should not be able to spawn more than limit
        assert!(!manager.can_spawn_particles(100_000_000));
    }

    #[test]
    fn test_enable_disable_systems() {
        let mut manager = WaterEffectsManager::new_default().unwrap();

        manager.set_caustics_enabled(false);
        assert!(!manager.config().caustics_enabled);

        manager.set_god_rays_enabled(false);
        assert!(!manager.config().god_rays_enabled);

        manager.set_reflections_enabled(false);
        assert!(!manager.config().reflections_enabled);
    }

    #[test]
    fn test_apply_preset() {
        let mut manager = WaterEffectsManager::new_default().unwrap();

        assert!(manager.apply_preset(WaterQualityPreset::Low).is_ok());
        assert!(!manager.config().caustics_enabled);

        assert!(manager.apply_preset(WaterQualityPreset::High).is_ok());
        assert!(manager.config().caustics_enabled);
    }

    #[test]
    fn test_clear_particles() {
        let mut manager = WaterEffectsManager::new_default().unwrap();
        manager.setup_water_surface(10.0, glam::Vec2::splat(-50.0), glam::Vec2::splat(50.0));

        // Spawn some particles using correct API
        use crate::foam::FoamSource;
        manager.foam_mut().spawn_foam(glam::Vec3::ZERO, glam::Vec2::ONE, FoamSource::Whitecap);

        assert!(manager.total_particle_count() > 0);

        manager.clear_particles();
        assert_eq!(manager.total_particle_count(), 0);
    }

    #[test]
    fn test_reset() {
        let mut manager = WaterEffectsManager::new_default().unwrap();
        manager.setup_water_surface(10.0, glam::Vec2::splat(-50.0), glam::Vec2::splat(50.0));
        
        manager.update(0.016, Vec3::new(0.0, 5.0, 0.0), 100.0);
        assert!(manager.stats().frame > 0);

        manager.reset();
        assert_eq!(manager.stats().frame, 0);
        assert_eq!(manager.total_particle_count(), 0);
    }

    #[test]
    fn test_get_uniforms() {
        let manager = WaterEffectsManager::new_default().unwrap();

        let caustics = manager.get_caustics_uniforms();
        assert!(caustics.intensity > 0.0);

        let god_rays = manager.get_god_rays_uniforms(true);
        assert!(god_rays.light_dir_intensity[3] > 0.0);

        let reflections = manager.get_reflection_uniforms(Mat4::IDENTITY, Vec3::ZERO);
        assert!(reflections.params[0] > 0.0); // max_steps
    }

    #[test]
    fn test_accessors() {
        let mut manager = WaterEffectsManager::new_default().unwrap();

        // Test immutable accessors
        let _ = manager.caustics();
        let _ = manager.god_rays();
        let _ = manager.reflections();
        let _ = manager.foam();
        let _ = manager.underwater_particles();
        let _ = manager.waterfalls();
        let _ = manager.underwater_state();

        // Test mutable accessors
        let _ = manager.caustics_mut();
        let _ = manager.god_rays_mut();
        let _ = manager.reflections_mut();
        let _ = manager.foam_mut();
        let _ = manager.underwater_particles_mut();
        let _ = manager.waterfalls_mut();
    }
}
