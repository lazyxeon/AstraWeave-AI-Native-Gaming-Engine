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

    #[test]
    fn test_config_roundtrip() {
        let config = FluidEditorConfig::quality();
        let json = config.to_json().unwrap();
        let loaded = FluidEditorConfig::from_json(&json).unwrap();
        assert_eq!(config.iterations, loaded.iterations);
    }
}
