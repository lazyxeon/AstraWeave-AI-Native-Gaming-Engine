//! Underwater rendering effects
//!
//! This module provides underwater visual effects including:
//! - Caustics (light patterns on surfaces)
//! - Underwater fog with depth-based absorption
//! - Screen distortion effects
//! - God rays (volumetric light shafts)

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// Configuration for underwater visual effects
#[derive(Clone, Debug)]
pub struct UnderwaterConfig {
    /// Fog color when underwater
    pub fog_color: Vec3,
    /// Fog density (affects visibility distance)
    pub fog_density: f32,
    /// Caustics pattern intensity (0.0 = none, 1.0 = full)
    pub caustics_intensity: f32,
    /// Caustics pattern scale
    pub caustics_scale: f32,
    /// Screen distortion strength
    pub distortion_strength: f32,
    /// Color absorption rates (r, g, b) - higher = faster absorption
    pub absorption_rates: Vec3,
    /// God ray intensity
    pub god_ray_intensity: f32,
    /// God ray density (number of samples)
    pub god_ray_samples: u32,
}

impl Default for UnderwaterConfig {
    fn default() -> Self {
        Self {
            fog_color: Vec3::new(0.0, 0.15, 0.3),
            fog_density: 0.1,
            caustics_intensity: 0.3,
            caustics_scale: 10.0,
            distortion_strength: 0.01,
            absorption_rates: Vec3::new(0.3, 0.1, 0.05),
            god_ray_intensity: 0.5,
            god_ray_samples: 32,
        }
    }
}

impl UnderwaterConfig {
    /// Create a murky water configuration
    pub fn murky() -> Self {
        Self {
            fog_color: Vec3::new(0.1, 0.15, 0.1),
            fog_density: 0.3,
            caustics_intensity: 0.1,
            caustics_scale: 8.0,
            distortion_strength: 0.015,
            absorption_rates: Vec3::new(0.4, 0.2, 0.1),
            god_ray_intensity: 0.2,
            god_ray_samples: 16,
        }
    }

    /// Create a crystal clear water configuration
    pub fn crystal_clear() -> Self {
        Self {
            fog_color: Vec3::new(0.1, 0.3, 0.5),
            fog_density: 0.02,
            caustics_intensity: 0.6,
            caustics_scale: 15.0,
            distortion_strength: 0.005,
            absorption_rates: Vec3::new(0.1, 0.05, 0.02),
            god_ray_intensity: 0.8,
            god_ray_samples: 64,
        }
    }

    /// Create a deep ocean configuration
    pub fn deep_ocean() -> Self {
        Self {
            fog_color: Vec3::new(0.0, 0.05, 0.15),
            fog_density: 0.15,
            caustics_intensity: 0.05,
            caustics_scale: 5.0,
            distortion_strength: 0.008,
            absorption_rates: Vec3::new(0.5, 0.2, 0.08),
            god_ray_intensity: 0.3,
            god_ray_samples: 48,
        }
    }

    /// Create a swamp water configuration
    pub fn swamp() -> Self {
        Self {
            fog_color: Vec3::new(0.15, 0.12, 0.05),
            fog_density: 0.5,
            caustics_intensity: 0.0,
            caustics_scale: 5.0,
            distortion_strength: 0.02,
            absorption_rates: Vec3::new(0.5, 0.4, 0.2),
            god_ray_intensity: 0.0,
            god_ray_samples: 0,
        }
    }
}

/// GPU uniform buffer for underwater effects (64 bytes, 16-byte aligned)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct UnderwaterUniforms {
    /// Fog color (xyz) and density (w)
    pub fog_color_density: [f32; 4],
    /// Caustics intensity (x), scale (y), distortion (z), god ray intensity (w)
    pub caustics_params: [f32; 4],
    /// Absorption rates (xyz) and god ray samples (w as float)
    pub absorption_samples: [f32; 4],
    /// Time (x), water depth (y), padding (z, w)
    pub time_depth: [f32; 4],
}

impl Default for UnderwaterUniforms {
    fn default() -> Self {
        let config = UnderwaterConfig::default();
        Self::from_config(&config, 0.0, 0.0)
    }
}

impl UnderwaterUniforms {
    /// Create uniforms from configuration
    pub fn from_config(config: &UnderwaterConfig, time: f32, water_depth: f32) -> Self {
        Self {
            fog_color_density: [
                config.fog_color.x,
                config.fog_color.y,
                config.fog_color.z,
                config.fog_density,
            ],
            caustics_params: [
                config.caustics_intensity,
                config.caustics_scale,
                config.distortion_strength,
                config.god_ray_intensity,
            ],
            absorption_samples: [
                config.absorption_rates.x,
                config.absorption_rates.y,
                config.absorption_rates.z,
                config.god_ray_samples as f32,
            ],
            time_depth: [time, water_depth, 0.0, 0.0],
        }
    }

    /// Update time and depth
    pub fn update(&mut self, time: f32, water_depth: f32) {
        self.time_depth[0] = time;
        self.time_depth[1] = water_depth;
    }
}

/// State tracking for underwater rendering
#[derive(Clone, Debug)]
pub struct UnderwaterState {
    /// Whether the camera is currently underwater
    pub is_underwater: bool,
    /// Current depth below water surface (0.0 if not underwater)
    pub depth: f32,
    /// Transition progress (0.0 = above water, 1.0 = fully underwater)
    pub transition: f32,
    /// Current configuration
    pub config: UnderwaterConfig,
    /// Time accumulator for animations
    pub time: f32,
}

impl Default for UnderwaterState {
    fn default() -> Self {
        Self {
            is_underwater: false,
            depth: 0.0,
            transition: 0.0,
            config: UnderwaterConfig::default(),
            time: 0.0,
        }
    }
}

impl UnderwaterState {
    /// Create a new underwater state with configuration
    pub fn new(config: UnderwaterConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Update the underwater state
    ///
    /// # Arguments
    /// * `camera_y` - Camera Y position in world space
    /// * `water_surface_y` - Water surface Y position at camera location
    /// * `dt` - Delta time in seconds
    pub fn update(&mut self, camera_y: f32, water_surface_y: f32, dt: f32) {
        let was_underwater = self.is_underwater;
        self.is_underwater = camera_y < water_surface_y;

        if self.is_underwater {
            self.depth = water_surface_y - camera_y;
            // Smoothly transition to underwater
            self.transition = (self.transition + dt * 5.0).min(1.0);
        } else {
            self.depth = 0.0;
            // Smoothly transition out of underwater
            self.transition = (self.transition - dt * 5.0).max(0.0);
        }

        // Handle water entry/exit events
        if !was_underwater && self.is_underwater {
            self.on_enter_water();
        } else if was_underwater && !self.is_underwater {
            self.on_exit_water();
        }

        // Accumulate time for animations
        self.time += dt;
    }

    /// Called when entering water
    fn on_enter_water(&mut self) {
        // Could trigger splash sound, screen effect, etc.
        // This is a hook for the gameplay system
    }

    /// Called when exiting water
    fn on_exit_water(&mut self) {
        // Could trigger water droplet effect on screen, etc.
    }

    /// Get uniforms for GPU rendering
    pub fn get_uniforms(&self) -> UnderwaterUniforms {
        UnderwaterUniforms::from_config(&self.config, self.time, self.depth)
    }

    /// Check if underwater effects should be rendered
    pub fn should_render_effects(&self) -> bool {
        self.transition > 0.01
    }

    /// Get the effective visibility distance based on fog
    pub fn visibility_distance(&self) -> f32 {
        if self.config.fog_density > 0.0 {
            // Distance at which fog reduces visibility to ~5%
            3.0 / self.config.fog_density
        } else {
            f32::MAX
        }
    }

    /// Set configuration
    pub fn set_config(&mut self, config: UnderwaterConfig) {
        self.config = config;
    }

    /// Blend between two configurations based on a factor
    pub fn blend_configs(a: &UnderwaterConfig, b: &UnderwaterConfig, t: f32) -> UnderwaterConfig {
        let t = t.clamp(0.0, 1.0);
        UnderwaterConfig {
            fog_color: a.fog_color.lerp(b.fog_color, t),
            fog_density: a.fog_density + (b.fog_density - a.fog_density) * t,
            caustics_intensity: a.caustics_intensity + (b.caustics_intensity - a.caustics_intensity) * t,
            caustics_scale: a.caustics_scale + (b.caustics_scale - a.caustics_scale) * t,
            distortion_strength: a.distortion_strength + (b.distortion_strength - a.distortion_strength) * t,
            absorption_rates: a.absorption_rates.lerp(b.absorption_rates, t),
            god_ray_intensity: a.god_ray_intensity + (b.god_ray_intensity - a.god_ray_intensity) * t,
            god_ray_samples: ((a.god_ray_samples as f32 + (b.god_ray_samples as f32 - a.god_ray_samples as f32) * t) as u32),
        }
    }
}

/// Depth zones for varying underwater effects
#[derive(Clone, Debug)]
pub struct DepthZone {
    /// Maximum depth for this zone
    pub max_depth: f32,
    /// Configuration for this depth zone
    pub config: UnderwaterConfig,
}

/// Manages depth-based underwater configuration changes
#[derive(Clone, Debug, Default)]
pub struct DepthZoneManager {
    zones: Vec<DepthZone>,
}

impl DepthZoneManager {
    /// Create a new depth zone manager
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    /// Create default ocean depth zones
    pub fn ocean_default() -> Self {
        let mut manager = Self::new();
        manager.add_zone(5.0, UnderwaterConfig::crystal_clear());
        manager.add_zone(20.0, UnderwaterConfig::default());
        manager.add_zone(100.0, UnderwaterConfig::deep_ocean());
        manager
    }

    /// Add a depth zone
    pub fn add_zone(&mut self, max_depth: f32, config: UnderwaterConfig) {
        self.zones.push(DepthZone { max_depth, config });
        // Keep sorted by depth
        self.zones.sort_by(|a, b| a.max_depth.partial_cmp(&b.max_depth).unwrap());
    }

    /// Get the blended configuration for a given depth
    pub fn get_config_at_depth(&self, depth: f32) -> UnderwaterConfig {
        if self.zones.is_empty() {
            return UnderwaterConfig::default();
        }

        // Find the two zones we're between
        let mut prev_zone: Option<&DepthZone> = None;
        for zone in &self.zones {
            if depth <= zone.max_depth {
                if let Some(prev) = prev_zone {
                    // Blend between prev and current zone
                    let prev_depth = prev.max_depth;
                    let t = (depth - prev_depth) / (zone.max_depth - prev_depth);
                    return UnderwaterState::blend_configs(&prev.config, &zone.config, t);
                } else {
                    // Before first zone, use first config
                    return zone.config.clone();
                }
            }
            prev_zone = Some(zone);
        }

        // Deeper than all zones, use last config
        self.zones.last().unwrap().config.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_underwater_config_default() {
        let config = UnderwaterConfig::default();
        assert!(config.fog_density > 0.0);
        assert!(config.caustics_intensity >= 0.0);
        assert!(config.caustics_intensity <= 1.0);
    }

    #[test]
    fn test_underwater_state_update() {
        let mut state = UnderwaterState::default();
        
        // Above water
        state.update(10.0, 5.0, 0.016);
        assert!(!state.is_underwater);
        assert_eq!(state.depth, 0.0);
        
        // Underwater
        state.update(3.0, 5.0, 0.016);
        assert!(state.is_underwater);
        assert!((state.depth - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_underwater_transition() {
        let mut state = UnderwaterState::default();
        
        // Start above water
        state.update(10.0, 5.0, 0.016);
        assert!(state.transition < 0.01);
        
        // Go underwater
        for _ in 0..20 {
            state.update(3.0, 5.0, 0.1);
        }
        assert!((state.transition - 1.0).abs() < 0.01);
        
        // Go above water
        for _ in 0..20 {
            state.update(10.0, 5.0, 0.1);
        }
        assert!(state.transition < 0.01);
    }

    #[test]
    fn test_uniforms_size() {
        // Ensure proper alignment for GPU
        let size = std::mem::size_of::<UnderwaterUniforms>();
        assert_eq!(size, 64); // 4 vec4s * 16 bytes
        assert!(size % 16 == 0, "Uniforms must be 16-byte aligned");
    }

    #[test]
    fn test_depth_zone_manager() {
        let manager = DepthZoneManager::ocean_default();
        
        // Surface (crystal clear at depth <= 5.0)
        let config = manager.get_config_at_depth(2.0);
        assert!(config.caustics_intensity > 0.5, 
            "Surface caustics should be high, got {}", config.caustics_intensity);
        
        // Very deep (past max zone, uses deep_ocean preset = 0.05)
        let config = manager.get_config_at_depth(150.0);
        assert!(config.caustics_intensity < 0.1, 
            "Very deep caustics should be < 0.1, got {}", config.caustics_intensity);
        
        // Mid-depth blending check (between zones)
        let config_mid = manager.get_config_at_depth(60.0);
        assert!(config_mid.caustics_intensity < config.caustics_intensity + 0.3,
            "Mid-depth should blend caustics, got {}", config_mid.caustics_intensity);
    }

    #[test]
    fn test_config_presets() {
        let murky = UnderwaterConfig::murky();
        let crystal = UnderwaterConfig::crystal_clear();
        let deep = UnderwaterConfig::deep_ocean();
        let swamp = UnderwaterConfig::swamp();
        
        // Murky should have lower visibility
        assert!(murky.fog_density > crystal.fog_density);
        
        // Swamp should have no caustics
        assert_eq!(swamp.caustics_intensity, 0.0);
        
        // Deep ocean should have less god rays than crystal
        assert!(deep.god_ray_intensity < crystal.god_ray_intensity);
    }

    #[test]
    fn test_visibility_distance() {
        let mut state = UnderwaterState::new(UnderwaterConfig::crystal_clear());
        state.is_underwater = true;
        state.depth = 5.0;
        
        let vis = state.visibility_distance();
        assert!(vis > 100.0); // Crystal clear should have good visibility
        
        state.set_config(UnderwaterConfig::murky());
        let vis = state.visibility_distance();
        assert!(vis < 20.0); // Murky should have poor visibility
    }

    // =========================================================================
    // Additional coverage tests
    // =========================================================================

    #[test]
    fn test_underwater_uniforms_from_config() {
        let config = UnderwaterConfig {
            fog_color: Vec3::new(0.1, 0.2, 0.3),
            fog_density: 0.15,
            caustics_intensity: 0.4,
            caustics_scale: 12.0,
            distortion_strength: 0.02,
            absorption_rates: Vec3::new(0.3, 0.2, 0.1),
            god_ray_intensity: 0.6,
            god_ray_samples: 48,
        };
        
        let uniforms = UnderwaterUniforms::from_config(&config, 1.5, 10.0);
        
        assert_eq!(uniforms.fog_color_density[0], 0.1);
        assert_eq!(uniforms.fog_color_density[1], 0.2);
        assert_eq!(uniforms.fog_color_density[2], 0.3);
        assert_eq!(uniforms.fog_color_density[3], 0.15);
        
        assert_eq!(uniforms.caustics_params[0], 0.4);
        assert_eq!(uniforms.caustics_params[1], 12.0);
        assert_eq!(uniforms.caustics_params[2], 0.02);
        assert_eq!(uniforms.caustics_params[3], 0.6);
        
        assert_eq!(uniforms.absorption_samples[0], 0.3);
        assert_eq!(uniforms.absorption_samples[1], 0.2);
        assert_eq!(uniforms.absorption_samples[2], 0.1);
        assert_eq!(uniforms.absorption_samples[3], 48.0);
        
        assert_eq!(uniforms.time_depth[0], 1.5);
        assert_eq!(uniforms.time_depth[1], 10.0);
    }

    #[test]
    fn test_underwater_uniforms_update() {
        let mut uniforms = UnderwaterUniforms::default();
        
        assert_eq!(uniforms.time_depth[0], 0.0);
        assert_eq!(uniforms.time_depth[1], 0.0);
        
        uniforms.update(5.0, 15.0);
        
        assert_eq!(uniforms.time_depth[0], 5.0);
        assert_eq!(uniforms.time_depth[1], 15.0);
    }

    #[test]
    fn test_underwater_state_new() {
        let config = UnderwaterConfig::murky();
        let state = UnderwaterState::new(config.clone());
        
        assert!(!state.is_underwater);
        assert_eq!(state.depth, 0.0);
        assert_eq!(state.transition, 0.0);
        assert_eq!(state.time, 0.0);
        assert_eq!(state.config.fog_density, config.fog_density);
    }

    #[test]
    fn test_underwater_state_should_render_effects() {
        let mut state = UnderwaterState::default();
        
        // Above water, no transition
        assert!(!state.should_render_effects());
        
        // Slightly transitioned
        state.transition = 0.02;
        assert!(state.should_render_effects());
        
        // Just at threshold
        state.transition = 0.011;
        assert!(state.should_render_effects());
        
        // Below threshold
        state.transition = 0.005;
        assert!(!state.should_render_effects());
    }

    #[test]
    fn test_underwater_state_get_uniforms() {
        let mut state = UnderwaterState::new(UnderwaterConfig::deep_ocean());
        state.time = 2.5;
        state.depth = 50.0;
        
        let uniforms = state.get_uniforms();
        
        assert_eq!(uniforms.time_depth[0], 2.5);
        assert_eq!(uniforms.time_depth[1], 50.0);
        assert_eq!(uniforms.caustics_params[0], state.config.caustics_intensity);
    }

    #[test]
    fn test_underwater_state_blend_configs() {
        let a = UnderwaterConfig::crystal_clear();
        let b = UnderwaterConfig::murky();
        
        // t=0 should be config a
        let blended_0 = UnderwaterState::blend_configs(&a, &b, 0.0);
        assert_eq!(blended_0.fog_density, a.fog_density);
        
        // t=1 should be config b
        let blended_1 = UnderwaterState::blend_configs(&a, &b, 1.0);
        assert_eq!(blended_1.fog_density, b.fog_density);
        
        // t=0.5 should be midpoint
        let blended_half = UnderwaterState::blend_configs(&a, &b, 0.5);
        let expected_density = (a.fog_density + b.fog_density) / 2.0;
        assert!((blended_half.fog_density - expected_density).abs() < 0.001);
        
        // t<0 should clamp to 0
        let blended_neg = UnderwaterState::blend_configs(&a, &b, -1.0);
        assert_eq!(blended_neg.fog_density, a.fog_density);
        
        // t>1 should clamp to 1
        let blended_over = UnderwaterState::blend_configs(&a, &b, 2.0);
        assert_eq!(blended_over.fog_density, b.fog_density);
    }

    #[test]
    fn test_depth_zone_manager_new_and_add() {
        let mut manager = DepthZoneManager::new();
        assert!(manager.zones.is_empty());
        
        manager.add_zone(10.0, UnderwaterConfig::crystal_clear());
        assert_eq!(manager.zones.len(), 1);
        
        // Add out of order - should sort
        manager.add_zone(5.0, UnderwaterConfig::default());
        assert_eq!(manager.zones.len(), 2);
        assert_eq!(manager.zones[0].max_depth, 5.0);
        assert_eq!(manager.zones[1].max_depth, 10.0);
    }

    #[test]
    fn test_depth_zone_manager_empty() {
        let manager = DepthZoneManager::new();
        
        // Empty zones should return default config
        let config = manager.get_config_at_depth(50.0);
        let default_config = UnderwaterConfig::default();
        assert_eq!(config.fog_density, default_config.fog_density);
    }

    #[test]
    fn test_visibility_distance_zero_density() {
        let config = UnderwaterConfig {
            fog_density: 0.0,
            ..Default::default()
        };
        let state = UnderwaterState::new(config);
        
        let vis = state.visibility_distance();
        assert_eq!(vis, f32::MAX);
    }

    #[test]
    fn test_underwater_config_clone() {
        let config = UnderwaterConfig::swamp();
        let cloned = config.clone();
        
        assert_eq!(cloned.fog_density, config.fog_density);
        assert_eq!(cloned.caustics_intensity, config.caustics_intensity);
        assert_eq!(cloned.god_ray_samples, config.god_ray_samples);
    }

    #[test]
    fn test_underwater_config_debug() {
        let config = UnderwaterConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("fog_color"));
        assert!(debug.contains("fog_density"));
    }

    #[test]
    fn test_underwater_state_debug() {
        let state = UnderwaterState::default();
        let debug = format!("{:?}", state);
        assert!(debug.contains("UnderwaterState"));
        assert!(debug.contains("is_underwater"));
    }

    #[test]
    fn test_underwater_uniforms_debug() {
        let uniforms = UnderwaterUniforms::default();
        let debug = format!("{:?}", uniforms);
        assert!(debug.contains("UnderwaterUniforms"));
    }

    #[test]
    fn test_underwater_uniforms_copy_clone() {
        let uniforms = UnderwaterUniforms::default();
        let copied = uniforms;
        let cloned = uniforms.clone();
        
        assert_eq!(copied.time_depth, uniforms.time_depth);
        assert_eq!(cloned.fog_color_density, uniforms.fog_color_density);
    }

    #[test]
    fn test_underwater_state_clone() {
        let state = UnderwaterState {
            is_underwater: true,
            depth: 25.0,
            transition: 0.75,
            config: UnderwaterConfig::murky(),
            time: 10.0,
        };
        let cloned = state.clone();
        
        assert_eq!(cloned.is_underwater, state.is_underwater);
        assert_eq!(cloned.depth, state.depth);
        assert_eq!(cloned.transition, state.transition);
        assert_eq!(cloned.time, state.time);
    }

    #[test]
    fn test_depth_zone_clone_debug() {
        let zone = DepthZone {
            max_depth: 50.0,
            config: UnderwaterConfig::deep_ocean(),
        };
        
        let cloned = zone.clone();
        assert_eq!(cloned.max_depth, zone.max_depth);
        
        let debug = format!("{:?}", zone);
        assert!(debug.contains("DepthZone"));
        assert!(debug.contains("max_depth"));
    }

    #[test]
    fn test_depth_zone_manager_clone_debug() {
        let manager = DepthZoneManager::ocean_default();
        let cloned = manager.clone();
        
        assert_eq!(cloned.zones.len(), manager.zones.len());
        
        let debug = format!("{:?}", manager);
        assert!(debug.contains("DepthZoneManager"));
    }

    #[test]
    fn test_underwater_state_time_accumulation() {
        let mut state = UnderwaterState::default();
        
        state.update(3.0, 5.0, 0.5);
        assert!((state.time - 0.5).abs() < 0.001);
        
        state.update(3.0, 5.0, 0.5);
        assert!((state.time - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_depth_zone_blending_between_zones() {
        let mut manager = DepthZoneManager::new();
        
        // Zone 1: depth 0-10, density 0.1
        let config1 = UnderwaterConfig {
            fog_density: 0.1,
            ..Default::default()
        };
        manager.add_zone(10.0, config1);
        
        // Zone 2: depth 10-20, density 0.5
        let config2 = UnderwaterConfig {
            fog_density: 0.5,
            ..Default::default()
        };
        manager.add_zone(20.0, config2);
        
        // At depth 15, should be halfway between 0.1 and 0.5 = 0.3
        let blended = manager.get_config_at_depth(15.0);
        assert!((blended.fog_density - 0.3).abs() < 0.01);
    }
}
