//! Editor Integration for Fluid System
//!
//! Provides inspector-friendly types and real-time parameter tweaking.

use serde::{Deserialize, Serialize};

/// Editor-friendly fluid system configuration
/// All fields have sensible defaults and clamped ranges for safe tweaking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FluidEditorConfig {
    // Physics
    /// Particle smoothing radius (0.5 - 5.0)
    pub smoothing_radius: f32,
    /// Target fluid density (1.0 - 50.0)
    pub target_density: f32,
    /// Pressure multiplier (10.0 - 1000.0)
    pub pressure_multiplier: f32,
    /// Viscosity coefficient (0.0 - 100.0)
    pub viscosity: f32,
    /// Surface tension strength (0.0 - 1.0)
    pub surface_tension: f32,
    /// Gravity strength (-30.0 - 30.0)
    pub gravity: f32,
    /// Solver iterations (1 - 20)
    pub iterations: u32,

    // Thermal
    /// Enable temperature simulation
    pub enable_temperature: bool,
    /// Thermal diffusivity (0.0 - 1.0)
    pub thermal_diffusivity: f32,
    /// Buoyancy strength (0.0 - 0.01)
    pub buoyancy_coefficient: f32,

    // Rendering
    /// Enable SSFR rendering
    pub enable_ssfr: bool,
    /// Fluid color (RGBA)
    pub fluid_color: [f32; 4],
    /// Absorption coefficients (RGB)
    pub absorption: [f32; 3],
    /// Scatter color (RGB)
    pub scatter_color: [f32; 3],
    /// Enable caustics
    pub enable_caustics: bool,
    /// Caustic intensity (0.0 - 5.0)
    pub caustic_intensity: f32,

    // Temporal
    /// Enable temporal reprojection
    pub enable_temporal: bool,
    /// Temporal blend factor (0.0 - 1.0, higher = more stable)
    pub temporal_blend: f32,

    // LOD
    /// LOD distance thresholds
    pub lod_distances: [f32; 4],
}

impl Default for FluidEditorConfig {
    fn default() -> Self {
        Self {
            // Physics
            smoothing_radius: 1.0,
            target_density: 12.0,
            pressure_multiplier: 300.0,
            viscosity: 10.0,
            surface_tension: 0.02,
            gravity: -9.8,
            iterations: 4,
            // Thermal
            enable_temperature: true,
            thermal_diffusivity: 0.1,
            buoyancy_coefficient: 0.0002,
            // Rendering
            enable_ssfr: true,
            fluid_color: [0.2, 0.5, 0.8, 1.0],
            absorption: [1.5, 0.5, 0.05],
            scatter_color: [0.0, 0.1, 0.2],
            enable_caustics: true,
            caustic_intensity: 1.0,
            // Temporal
            enable_temporal: true,
            temporal_blend: 0.9,
            // LOD
            lod_distances: [20.0, 50.0, 100.0, 200.0],
        }
    }
}

impl FluidEditorConfig {
    /// Create config optimized for performance
    pub fn performance() -> Self {
        Self {
            smoothing_radius: 1.2,
            target_density: 10.0,
            pressure_multiplier: 200.0,
            viscosity: 5.0,
            surface_tension: 0.01,
            gravity: -9.8,
            iterations: 2,
            enable_temperature: false,
            thermal_diffusivity: 0.0,
            buoyancy_coefficient: 0.0,
            enable_ssfr: true,
            fluid_color: [0.2, 0.5, 0.8, 1.0],
            absorption: [1.0, 0.3, 0.02],
            scatter_color: [0.0, 0.05, 0.1],
            enable_caustics: false,
            caustic_intensity: 0.0,
            enable_temporal: true,
            temporal_blend: 0.95,
            lod_distances: [15.0, 40.0, 80.0, 150.0],
        }
    }

    /// Create config optimized for quality
    pub fn quality() -> Self {
        Self {
            smoothing_radius: 0.8,
            target_density: 15.0,
            pressure_multiplier: 400.0,
            viscosity: 15.0,
            surface_tension: 0.03,
            gravity: -9.8,
            iterations: 8,
            enable_temperature: true,
            thermal_diffusivity: 0.15,
            buoyancy_coefficient: 0.0003,
            enable_ssfr: true,
            fluid_color: [0.15, 0.45, 0.75, 1.0],
            absorption: [2.0, 0.7, 0.08],
            scatter_color: [0.0, 0.15, 0.25],
            enable_caustics: true,
            caustic_intensity: 2.0,
            enable_temporal: true,
            temporal_blend: 0.85,
            lod_distances: [30.0, 70.0, 150.0, 300.0],
        }
    }

    /// Clamp all values to safe ranges
    pub fn clamp(&mut self) {
        self.smoothing_radius = self.smoothing_radius.clamp(0.5, 5.0);
        self.target_density = self.target_density.clamp(1.0, 50.0);
        self.pressure_multiplier = self.pressure_multiplier.clamp(10.0, 1000.0);
        self.viscosity = self.viscosity.clamp(0.0, 100.0);
        self.surface_tension = self.surface_tension.clamp(0.0, 1.0);
        self.gravity = self.gravity.clamp(-30.0, 30.0);
        self.iterations = self.iterations.clamp(1, 20);
        self.thermal_diffusivity = self.thermal_diffusivity.clamp(0.0, 1.0);
        self.buoyancy_coefficient = self.buoyancy_coefficient.clamp(0.0, 0.01);
        self.caustic_intensity = self.caustic_intensity.clamp(0.0, 5.0);
        self.temporal_blend = self.temporal_blend.clamp(0.0, 1.0);
    }

    /// Save config to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Load config from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut config: Self = serde_json::from_str(json)?;
        config.clamp();
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Default Config Tests ====================

    #[test]
    fn test_config_default() {
        let config = FluidEditorConfig::default();
        
        assert_eq!(config.smoothing_radius, 1.0);
        assert_eq!(config.target_density, 12.0);
        assert_eq!(config.pressure_multiplier, 300.0);
        assert_eq!(config.viscosity, 10.0);
        assert_eq!(config.iterations, 4);
    }

    #[test]
    fn test_config_default_thermal() {
        let config = FluidEditorConfig::default();
        
        assert!(config.enable_temperature);
        assert_eq!(config.thermal_diffusivity, 0.1);
        assert_eq!(config.buoyancy_coefficient, 0.0002);
    }

    #[test]
    fn test_config_default_rendering() {
        let config = FluidEditorConfig::default();
        
        assert!(config.enable_ssfr);
        assert!(config.enable_caustics);
        assert!(config.enable_temporal);
        assert_eq!(config.caustic_intensity, 1.0);
    }

    #[test]
    fn test_config_default_lod() {
        let config = FluidEditorConfig::default();
        
        assert_eq!(config.lod_distances.len(), 4);
        assert_eq!(config.lod_distances[0], 20.0);
        assert_eq!(config.lod_distances[3], 200.0);
    }

    // ==================== Preset Tests ====================

    #[test]
    fn test_config_performance_preset() {
        let config = FluidEditorConfig::performance();
        
        // Performance should have fewer iterations
        assert!(config.iterations < FluidEditorConfig::default().iterations);
        // And disabled expensive features
        assert!(!config.enable_caustics);
        assert!(!config.enable_temperature);
    }

    #[test]
    fn test_config_quality_preset() {
        let config = FluidEditorConfig::quality();
        
        // Quality should have more iterations
        assert!(config.iterations > FluidEditorConfig::default().iterations);
        // And enabled expensive features
        assert!(config.enable_caustics);
        assert!(config.enable_temperature);
        assert!(config.caustic_intensity > FluidEditorConfig::default().caustic_intensity);
    }

    #[test]
    fn test_presets_iterations_ordering() {
        let perf = FluidEditorConfig::performance();
        let default = FluidEditorConfig::default();
        let quality = FluidEditorConfig::quality();
        
        assert!(perf.iterations < default.iterations);
        assert!(default.iterations < quality.iterations);
    }

    #[test]
    fn test_presets_lod_ordering() {
        let perf = FluidEditorConfig::performance();
        let quality = FluidEditorConfig::quality();
        
        // Quality should have farther LOD distances
        assert!(quality.lod_distances[0] > perf.lod_distances[0]);
    }

    // ==================== Clamp Tests ====================

    #[test]
    fn test_clamp_smoothing_radius() {
        let mut config = FluidEditorConfig::default();
        
        config.smoothing_radius = 0.1;
        config.clamp();
        assert_eq!(config.smoothing_radius, 0.5);
        
        config.smoothing_radius = 10.0;
        config.clamp();
        assert_eq!(config.smoothing_radius, 5.0);
    }

    #[test]
    fn test_clamp_target_density() {
        let mut config = FluidEditorConfig::default();
        
        config.target_density = 0.1;
        config.clamp();
        assert_eq!(config.target_density, 1.0);
        
        config.target_density = 100.0;
        config.clamp();
        assert_eq!(config.target_density, 50.0);
    }

    #[test]
    fn test_clamp_pressure_multiplier() {
        let mut config = FluidEditorConfig::default();
        
        config.pressure_multiplier = 1.0;
        config.clamp();
        assert_eq!(config.pressure_multiplier, 10.0);
        
        config.pressure_multiplier = 2000.0;
        config.clamp();
        assert_eq!(config.pressure_multiplier, 1000.0);
    }

    #[test]
    fn test_clamp_iterations() {
        let mut config = FluidEditorConfig::default();
        
        config.iterations = 0;
        config.clamp();
        assert_eq!(config.iterations, 1);
        
        config.iterations = 100;
        config.clamp();
        assert_eq!(config.iterations, 20);
    }

    #[test]
    fn test_clamp_gravity() {
        let mut config = FluidEditorConfig::default();
        
        config.gravity = -50.0;
        config.clamp();
        assert_eq!(config.gravity, -30.0);
        
        config.gravity = 50.0;
        config.clamp();
        assert_eq!(config.gravity, 30.0);
    }

    #[test]
    fn test_clamp_temporal_blend() {
        let mut config = FluidEditorConfig::default();
        
        config.temporal_blend = -0.5;
        config.clamp();
        assert_eq!(config.temporal_blend, 0.0);
        
        config.temporal_blend = 1.5;
        config.clamp();
        assert_eq!(config.temporal_blend, 1.0);
    }

    #[test]
    fn test_clamp_preserves_valid_values() {
        let config_before = FluidEditorConfig::default();
        let mut config = config_before.clone();
        config.clamp();
        
        // Default values should be unchanged after clamp
        assert_eq!(config.smoothing_radius, config_before.smoothing_radius);
        assert_eq!(config.iterations, config_before.iterations);
        assert_eq!(config.gravity, config_before.gravity);
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_config_roundtrip() {
        let config = FluidEditorConfig::quality();
        let json = config.to_json().unwrap();
        let loaded = FluidEditorConfig::from_json(&json).unwrap();
        assert_eq!(config.iterations, loaded.iterations);
    }

    #[test]
    fn test_config_to_json() {
        let config = FluidEditorConfig::default();
        let json = config.to_json().unwrap();
        
        assert!(json.contains("smoothing_radius"));
        assert!(json.contains("target_density"));
        assert!(json.contains("iterations"));
    }

    #[test]
    fn test_config_from_json_clamps() {
        let json = r#"{
            "smoothing_radius": 100.0,
            "target_density": 12.0,
            "pressure_multiplier": 300.0,
            "viscosity": 10.0,
            "surface_tension": 0.02,
            "gravity": -9.8,
            "iterations": 4,
            "enable_temperature": true,
            "thermal_diffusivity": 0.1,
            "buoyancy_coefficient": 0.0002,
            "enable_ssfr": true,
            "fluid_color": [0.2, 0.5, 0.8, 1.0],
            "absorption": [1.5, 0.5, 0.05],
            "scatter_color": [0.0, 0.1, 0.2],
            "enable_caustics": true,
            "caustic_intensity": 1.0,
            "enable_temporal": true,
            "temporal_blend": 0.9,
            "lod_distances": [20.0, 50.0, 100.0, 200.0]
        }"#;
        
        let config = FluidEditorConfig::from_json(json).unwrap();
        // Should be clamped to max
        assert_eq!(config.smoothing_radius, 5.0);
    }

    #[test]
    fn test_config_from_json_invalid() {
        let json = "not valid json";
        let result = FluidEditorConfig::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_json_preserves_colors() {
        let config = FluidEditorConfig::default();
        let json = config.to_json().unwrap();
        let loaded = FluidEditorConfig::from_json(&json).unwrap();
        
        assert_eq!(config.fluid_color, loaded.fluid_color);
        assert_eq!(config.absorption, loaded.absorption);
        assert_eq!(config.scatter_color, loaded.scatter_color);
    }

    // ==================== Clone/Debug Tests ====================

    #[test]
    fn test_config_clone() {
        let config = FluidEditorConfig::quality();
        let cloned = config.clone();
        
        assert_eq!(config.iterations, cloned.iterations);
        assert_eq!(config.smoothing_radius, cloned.smoothing_radius);
        assert_eq!(config.fluid_color, cloned.fluid_color);
    }

    #[test]
    fn test_config_debug() {
        let config = FluidEditorConfig::default();
        let debug_str = format!("{:?}", config);
        
        assert!(debug_str.contains("FluidEditorConfig"));
        assert!(debug_str.contains("smoothing_radius"));
    }

    // ==================== Mutation-Resistant Tests ====================

    #[test]
    fn test_config_field_independence() {
        let mut config = FluidEditorConfig::default();
        config.iterations = 10;
        
        // Other fields should be unchanged
        assert_eq!(config.smoothing_radius, 1.0);
        assert_eq!(config.gravity, -9.8);
    }

    #[test]
    fn test_config_lod_distances_monotonic() {
        let config = FluidEditorConfig::default();
        
        // LOD distances should be increasing
        assert!(config.lod_distances[0] < config.lod_distances[1]);
        assert!(config.lod_distances[1] < config.lod_distances[2]);
        assert!(config.lod_distances[2] < config.lod_distances[3]);
    }

    #[test]
    fn test_config_color_valid_range() {
        let config = FluidEditorConfig::default();
        
        for c in &config.fluid_color {
            assert!(*c >= 0.0 && *c <= 1.0);
        }
    }

    #[test]
    fn test_all_presets_valid_after_clamp() {
        let presets = [
            FluidEditorConfig::default(),
            FluidEditorConfig::performance(),
            FluidEditorConfig::quality(),
        ];
        
        for mut preset in presets {
            let original_iterations = preset.iterations;
            preset.clamp();
            // Presets should already be within valid ranges
            assert_eq!(preset.iterations, original_iterations);
        }
    }

    #[test]
    fn test_config_viscosity_range() {
        let mut config = FluidEditorConfig::default();
        
        // Test lower bound
        config.viscosity = -5.0;
        config.clamp();
        assert_eq!(config.viscosity, 0.0);
        
        // Test upper bound
        config.viscosity = 200.0;
        config.clamp();
        assert_eq!(config.viscosity, 100.0);
    }

    #[test]
    fn test_config_surface_tension_range() {
        let mut config = FluidEditorConfig::default();
        
        config.surface_tension = -0.5;
        config.clamp();
        assert_eq!(config.surface_tension, 0.0);
        
        config.surface_tension = 5.0;
        config.clamp();
        assert_eq!(config.surface_tension, 1.0);
    }

    #[test]
    fn test_config_caustic_intensity_range() {
        let mut config = FluidEditorConfig::default();
        
        config.caustic_intensity = -1.0;
        config.clamp();
        assert_eq!(config.caustic_intensity, 0.0);
        
        config.caustic_intensity = 10.0;
        config.clamp();
        assert_eq!(config.caustic_intensity, 5.0);
    }
}
