//! Biome Transition Effects
//!
//! Provides smooth visual transitions when the player moves between biomes.
//! Supports crossfade, color tint, and fog density interpolation.
//!
//! # Overview
//!
//! When a biome transition is detected (via `BiomeDetector`), this module
//! manages a timed crossfade of:
//! - Sky/HDRI environment lighting
//! - Terrain material tint (optional)
//! - Fog color and density
//! - Ambient light color
//!
//! # Usage
//!
//! ```rust,no_run
//! # use astraweave_render::biome_transition::{TransitionEffect, TransitionConfig};
//! # use astraweave_terrain::biome::BiomeType;
//! let mut effect = TransitionEffect::new(TransitionConfig::default());
//!
//! // When a biome transition is detected:
//! effect.start(Some(BiomeType::Forest), BiomeType::Desert);
//!
//! // Each frame, update and get blend params:
//! effect.update(delta_time);
//! if effect.is_active() {
//!     let t = effect.blend_factor(); // 0.0 -> 1.0
//!     // Use t to blend skybox, fog, materials, etc.
//! }
//! ```

use astraweave_terrain::biome::BiomeType;
use glam::Vec3;

/// Configuration for biome transition effects.
#[derive(Debug, Clone, Copy)]
pub struct TransitionConfig {
    /// Total transition duration in seconds.
    pub duration: f32,
    /// Easing function to use.
    pub easing: EasingFunction,
    /// Whether to blend fog parameters.
    pub blend_fog: bool,
    /// Whether to blend ambient light color.
    pub blend_ambient: bool,
    /// Whether to apply a color tint during transition.
    pub apply_tint: bool,
    /// Peak tint alpha (0-1) at midpoint of transition.
    pub tint_alpha: f32,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            duration: 2.0, // 2 seconds for smooth transition
            easing: EasingFunction::SmoothStep,
            blend_fog: true,
            blend_ambient: true,
            apply_tint: false,
            tint_alpha: 0.15,
        }
    }
}

/// Easing functions for smooth transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum EasingFunction {
    /// Linear interpolation (no easing).
    Linear,
    /// Smooth step (Hermite interpolation).
    SmoothStep,
    /// Smoother step (Ken Perlin's improved version).
    SmootherStep,
    /// Ease in (quadratic).
    EaseIn,
    /// Ease out (quadratic).
    EaseOut,
    /// Ease in-out (quadratic).
    EaseInOut,
}

impl EasingFunction {
    /// Apply the easing function to a normalized time (0-1).
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingFunction::Linear => t,
            EasingFunction::SmoothStep => t * t * (3.0 - 2.0 * t),
            EasingFunction::SmootherStep => t * t * t * (t * (t * 6.0 - 15.0) + 10.0),
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

/// Per-biome visual parameters for blending.
#[derive(Debug, Clone, Copy)]
pub struct BiomeVisuals {
    /// Fog color (RGB, 0-1).
    pub fog_color: Vec3,
    /// Fog density (exponential fog factor).
    pub fog_density: f32,
    /// Fog start distance.
    pub fog_start: f32,
    /// Fog end distance.
    pub fog_end: f32,
    /// Ambient light color (RGB, 0-1).
    pub ambient_color: Vec3,
    /// Ambient light intensity.
    pub ambient_intensity: f32,

    // ── Sky colors ───────────────────────────────────────────────────────
    /// Daytime sky zenith color.
    pub sky_day_top: Vec3,
    /// Daytime sky horizon color.
    pub sky_day_horizon: Vec3,
    /// Sunset sky zenith color.
    pub sky_sunset_top: Vec3,
    /// Sunset sky horizon color.
    pub sky_sunset_horizon: Vec3,
    /// Night sky zenith color.
    pub sky_night_top: Vec3,
    /// Night sky horizon color.
    pub sky_night_horizon: Vec3,

    // ── Water colors ─────────────────────────────────────────────────────
    /// Deep water tint (RGB, 0-1).
    pub water_deep: Vec3,
    /// Shallow water tint (RGB, 0-1).
    pub water_shallow: Vec3,
    /// Foam color (RGB, 0-1).
    pub water_foam: Vec3,

    // ── Cloud & weather parameters ───────────────────────────────────────
    /// Cloud coverage (0.0 = clear, 1.0 = overcast).
    pub cloud_coverage: f32,
    /// Cloud movement speed.
    pub cloud_speed: f32,
    /// Weather particle density multiplier (0.0-1.0).
    pub weather_particle_density: f32,
}

impl Default for BiomeVisuals {
    fn default() -> Self {
        Self {
            fog_color: Vec3::new(0.7, 0.75, 0.8),
            fog_density: 0.001,
            fog_start: 50.0,
            fog_end: 500.0,
            ambient_color: Vec3::new(0.4, 0.45, 0.5),
            ambient_intensity: 0.3,
            // Neutral sky (same as SkyConfig::default)
            sky_day_top: Vec3::new(0.3, 0.6, 1.0),
            sky_day_horizon: Vec3::new(0.8, 0.9, 1.0),
            sky_sunset_top: Vec3::new(0.8, 0.4, 0.2),
            sky_sunset_horizon: Vec3::new(1.0, 0.6, 0.3),
            sky_night_top: Vec3::new(0.0, 0.0, 0.1),
            sky_night_horizon: Vec3::new(0.1, 0.1, 0.2),
            // Neutral water
            water_deep: Vec3::new(0.02, 0.08, 0.2),
            water_shallow: Vec3::new(0.1, 0.4, 0.5),
            water_foam: Vec3::new(0.95, 0.98, 1.0),
            // Neutral cloud/weather
            cloud_coverage: 0.3,
            cloud_speed: 0.02,
            weather_particle_density: 1.0,
        }
    }
}

impl BiomeVisuals {
    /// Get default visuals for a specific biome type.
    pub fn for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::Forest => Self {
                fog_color: Vec3::new(0.4, 0.5, 0.35),
                fog_density: 0.003,
                fog_start: 30.0,
                fog_end: 300.0,
                ambient_color: Vec3::new(0.3, 0.4, 0.25),
                ambient_intensity: 0.25,
                sky_day_top: Vec3::new(0.25, 0.55, 0.85),
                sky_day_horizon: Vec3::new(0.6, 0.8, 0.7),
                sky_sunset_top: Vec3::new(0.6, 0.35, 0.2),
                sky_sunset_horizon: Vec3::new(0.85, 0.55, 0.3),
                sky_night_top: Vec3::new(0.0, 0.02, 0.08),
                sky_night_horizon: Vec3::new(0.05, 0.08, 0.12),
                water_deep: Vec3::new(0.03, 0.1, 0.12),
                water_shallow: Vec3::new(0.08, 0.3, 0.25),
                water_foam: Vec3::new(0.85, 0.9, 0.8),
                cloud_coverage: 0.5, // Partial canopy cover mimics clouds
                cloud_speed: 0.01,
                weather_particle_density: 0.6, // Rain blocked by canopy
            },
            BiomeType::Desert => Self {
                fog_color: Vec3::new(0.9, 0.85, 0.7),
                fog_density: 0.0005,
                fog_start: 100.0,
                fog_end: 1000.0,
                ambient_color: Vec3::new(0.6, 0.55, 0.4),
                ambient_intensity: 0.4,
                sky_day_top: Vec3::new(0.35, 0.6, 0.95),
                sky_day_horizon: Vec3::new(0.95, 0.9, 0.8),
                sky_sunset_top: Vec3::new(0.9, 0.45, 0.15),
                sky_sunset_horizon: Vec3::new(1.0, 0.7, 0.35),
                sky_night_top: Vec3::new(0.02, 0.0, 0.12),
                sky_night_horizon: Vec3::new(0.12, 0.08, 0.18),
                water_deep: Vec3::new(0.05, 0.12, 0.18),
                water_shallow: Vec3::new(0.15, 0.35, 0.3),
                water_foam: Vec3::new(0.95, 0.92, 0.85),
                cloud_coverage: 0.1, // Rarely cloudy
                cloud_speed: 0.03,
                weather_particle_density: 1.5, // Sandstorms intense
            },
            BiomeType::Grassland => Self {
                fog_color: Vec3::new(0.7, 0.8, 0.85),
                fog_density: 0.001,
                fog_start: 80.0,
                fog_end: 600.0,
                ambient_color: Vec3::new(0.5, 0.55, 0.5),
                ambient_intensity: 0.35,
                sky_day_top: Vec3::new(0.3, 0.6, 1.0),
                sky_day_horizon: Vec3::new(0.8, 0.9, 1.0),
                sky_sunset_top: Vec3::new(0.8, 0.4, 0.2),
                sky_sunset_horizon: Vec3::new(1.0, 0.6, 0.3),
                sky_night_top: Vec3::new(0.0, 0.0, 0.1),
                sky_night_horizon: Vec3::new(0.1, 0.1, 0.2),
                water_deep: Vec3::new(0.02, 0.08, 0.2),
                water_shallow: Vec3::new(0.1, 0.4, 0.5),
                water_foam: Vec3::new(0.95, 0.98, 1.0),
                cloud_coverage: 0.4, // Variable
                cloud_speed: 0.025,
                weather_particle_density: 1.0,
            },
            BiomeType::Mountain => Self {
                fog_color: Vec3::new(0.75, 0.8, 0.9),
                fog_density: 0.002,
                fog_start: 50.0,
                fog_end: 400.0,
                ambient_color: Vec3::new(0.45, 0.5, 0.6),
                ambient_intensity: 0.3,
                sky_day_top: Vec3::new(0.2, 0.5, 0.95),
                sky_day_horizon: Vec3::new(0.7, 0.82, 0.95),
                sky_sunset_top: Vec3::new(0.7, 0.35, 0.25),
                sky_sunset_horizon: Vec3::new(0.95, 0.6, 0.4),
                sky_night_top: Vec3::new(0.0, 0.0, 0.12),
                sky_night_horizon: Vec3::new(0.08, 0.08, 0.2),
                water_deep: Vec3::new(0.01, 0.06, 0.2),
                water_shallow: Vec3::new(0.05, 0.3, 0.5),
                water_foam: Vec3::new(0.98, 1.0, 1.0),
                cloud_coverage: 0.6, // Often cloudy at altitude
                cloud_speed: 0.04, // Faster winds
                weather_particle_density: 1.2, // Snow more visible
            },
            BiomeType::Tundra => Self {
                fog_color: Vec3::new(0.85, 0.9, 0.95),
                fog_density: 0.002,
                fog_start: 40.0,
                fog_end: 350.0,
                ambient_color: Vec3::new(0.5, 0.55, 0.65),
                ambient_intensity: 0.35,
                sky_day_top: Vec3::new(0.4, 0.65, 0.95),
                sky_day_horizon: Vec3::new(0.85, 0.92, 1.0),
                sky_sunset_top: Vec3::new(0.75, 0.4, 0.3),
                sky_sunset_horizon: Vec3::new(1.0, 0.65, 0.45),
                sky_night_top: Vec3::new(0.0, 0.01, 0.1),
                sky_night_horizon: Vec3::new(0.1, 0.12, 0.22),
                water_deep: Vec3::new(0.02, 0.1, 0.25),
                water_shallow: Vec3::new(0.12, 0.45, 0.55),
                water_foam: Vec3::new(1.0, 1.0, 1.0),
                cloud_coverage: 0.55, // Often overcast
                cloud_speed: 0.03,
                weather_particle_density: 1.3, // Heavy snow
            },
            BiomeType::Swamp => Self {
                fog_color: Vec3::new(0.35, 0.4, 0.3),
                fog_density: 0.005,
                fog_start: 20.0,
                fog_end: 150.0,
                ambient_color: Vec3::new(0.25, 0.3, 0.2),
                ambient_intensity: 0.2,
                sky_day_top: Vec3::new(0.25, 0.45, 0.65),
                sky_day_horizon: Vec3::new(0.55, 0.6, 0.5),
                sky_sunset_top: Vec3::new(0.55, 0.3, 0.2),
                sky_sunset_horizon: Vec3::new(0.75, 0.45, 0.25),
                sky_night_top: Vec3::new(0.0, 0.02, 0.05),
                sky_night_horizon: Vec3::new(0.05, 0.06, 0.08),
                water_deep: Vec3::new(0.04, 0.08, 0.04),
                water_shallow: Vec3::new(0.12, 0.2, 0.1),
                water_foam: Vec3::new(0.6, 0.65, 0.5),
                cloud_coverage: 0.7, // Perpetually hazy/foggy
                cloud_speed: 0.01, // Stagnant air
                weather_particle_density: 0.8, // Mist more than rain
            },
            BiomeType::Beach => Self {
                fog_color: Vec3::new(0.75, 0.85, 0.9),
                fog_density: 0.0008,
                fog_start: 100.0,
                fog_end: 800.0,
                ambient_color: Vec3::new(0.55, 0.6, 0.65),
                ambient_intensity: 0.4,
                sky_day_top: Vec3::new(0.3, 0.65, 1.0),
                sky_day_horizon: Vec3::new(0.85, 0.93, 1.0),
                sky_sunset_top: Vec3::new(0.85, 0.45, 0.15),
                sky_sunset_horizon: Vec3::new(1.0, 0.7, 0.35),
                sky_night_top: Vec3::new(0.0, 0.01, 0.1),
                sky_night_horizon: Vec3::new(0.08, 0.1, 0.2),
                water_deep: Vec3::new(0.0, 0.05, 0.25),
                water_shallow: Vec3::new(0.05, 0.45, 0.6),
                water_foam: Vec3::new(1.0, 1.0, 1.0),
                cloud_coverage: 0.25, // Usually sunny
                cloud_speed: 0.02,
                weather_particle_density: 1.0,
            },
            BiomeType::River => Self {
                fog_color: Vec3::new(0.65, 0.75, 0.8),
                fog_density: 0.0015,
                fog_start: 60.0,
                fog_end: 400.0,
                ambient_color: Vec3::new(0.45, 0.5, 0.55),
                ambient_intensity: 0.3,
                sky_day_top: Vec3::new(0.3, 0.6, 0.95),
                sky_day_horizon: Vec3::new(0.75, 0.85, 0.95),
                sky_sunset_top: Vec3::new(0.75, 0.4, 0.2),
                sky_sunset_horizon: Vec3::new(0.95, 0.6, 0.3),
                sky_night_top: Vec3::new(0.0, 0.01, 0.08),
                sky_night_horizon: Vec3::new(0.08, 0.1, 0.18),
                water_deep: Vec3::new(0.02, 0.1, 0.18),
                water_shallow: Vec3::new(0.08, 0.38, 0.4),
                water_foam: Vec3::new(0.9, 0.95, 0.92),
                cloud_coverage: 0.35,
                cloud_speed: 0.02,
                weather_particle_density: 1.0,
            },
            // Fallback for any future biome types
            _ => Self::default(),
        }
    }

    /// Linearly interpolate between two BiomeVisuals.
    pub fn lerp(&self, other: &BiomeVisuals, t: f32) -> BiomeVisuals {
        BiomeVisuals {
            fog_color: self.fog_color.lerp(other.fog_color, t),
            fog_density: lerp(self.fog_density, other.fog_density, t),
            fog_start: lerp(self.fog_start, other.fog_start, t),
            fog_end: lerp(self.fog_end, other.fog_end, t),
            ambient_color: self.ambient_color.lerp(other.ambient_color, t),
            ambient_intensity: lerp(self.ambient_intensity, other.ambient_intensity, t),
            sky_day_top: self.sky_day_top.lerp(other.sky_day_top, t),
            sky_day_horizon: self.sky_day_horizon.lerp(other.sky_day_horizon, t),
            sky_sunset_top: self.sky_sunset_top.lerp(other.sky_sunset_top, t),
            sky_sunset_horizon: self.sky_sunset_horizon.lerp(other.sky_sunset_horizon, t),
            sky_night_top: self.sky_night_top.lerp(other.sky_night_top, t),
            sky_night_horizon: self.sky_night_horizon.lerp(other.sky_night_horizon, t),
            water_deep: self.water_deep.lerp(other.water_deep, t),
            water_shallow: self.water_shallow.lerp(other.water_shallow, t),
            water_foam: self.water_foam.lerp(other.water_foam, t),
            cloud_coverage: lerp(self.cloud_coverage, other.cloud_coverage, t),
            cloud_speed: lerp(self.cloud_speed, other.cloud_speed, t),
            weather_particle_density: lerp(self.weather_particle_density, other.weather_particle_density, t),
        }
    }

    /// Convert sky fields to a [`crate::environment::SkyConfig`].
    pub fn to_sky_config(&self) -> crate::environment::SkyConfig {
        crate::environment::SkyConfig {
            day_color_top: self.sky_day_top,
            day_color_horizon: self.sky_day_horizon,
            sunset_color_top: self.sky_sunset_top,
            sunset_color_horizon: self.sky_sunset_horizon,
            night_color_top: self.sky_night_top,
            night_color_horizon: self.sky_night_horizon,
            cloud_coverage: self.cloud_coverage,
            cloud_speed: self.cloud_speed,
            ..Default::default()
        }
    }
}

/// Active biome transition state.
pub struct TransitionEffect {
    /// Configuration.
    config: TransitionConfig,
    /// Source biome (where we're transitioning FROM).
    from_biome: Option<BiomeType>,
    /// Target biome (where we're transitioning TO).
    to_biome: Option<BiomeType>,
    /// Current progress (0.0 to 1.0).
    progress: f32,
    /// Whether a transition is currently active.
    active: bool,
    /// Cached source visuals.
    from_visuals: BiomeVisuals,
    /// Cached target visuals.
    to_visuals: BiomeVisuals,
}

impl TransitionEffect {
    /// Create a new transition effect with the given configuration.
    pub fn new(config: TransitionConfig) -> Self {
        Self {
            config,
            from_biome: None,
            to_biome: None,
            progress: 0.0,
            active: false,
            from_visuals: BiomeVisuals::default(),
            to_visuals: BiomeVisuals::default(),
        }
    }

    /// Start a new transition from one biome to another.
    ///
    /// If a transition is already in progress, this will override it
    /// (snapping from the current blend state to the new target).
    ///
    /// The `from` parameter is optional — when `None`, defaults to the
    /// target biome's visuals (instant transition).
    pub fn start(&mut self, from: Option<BiomeType>, to: BiomeType) {
        let from_biome = from.unwrap_or(to);
        if from_biome == to && from.is_some() {
            // No-op: same biome (but allow None→to for initial state)
            return;
        }

        self.from_biome = Some(from_biome);
        self.to_biome = Some(to);
        self.from_visuals = BiomeVisuals::for_biome(from_biome);
        self.to_visuals = BiomeVisuals::for_biome(to);
        self.progress = 0.0;
        self.active = true;
    }

    /// Start a transition with custom visuals (for overrides).
    pub fn start_with_visuals(
        &mut self,
        from: Option<BiomeType>,
        to: BiomeType,
        from_visuals: BiomeVisuals,
        to_visuals: BiomeVisuals,
    ) {
        let from_biome = from.unwrap_or(to);
        if from_biome == to && from.is_some() {
            return;
        }

        self.from_biome = Some(from_biome);
        self.to_biome = Some(to);
        self.from_visuals = from_visuals;
        self.to_visuals = to_visuals;
        self.progress = 0.0;
        self.active = true;
    }

    /// Update the transition state for this frame.
    ///
    /// Call this every frame with the delta time in seconds.
    pub fn update(&mut self, delta_time: f32) {
        if !self.active {
            return;
        }

        let rate = 1.0 / self.config.duration.max(0.001);
        self.progress += delta_time * rate;

        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.active = false;
            // Transition complete — from becomes the new current
            self.from_biome = self.to_biome;
            self.from_visuals = self.to_visuals;
        }
    }

    /// Check if a transition is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get the raw progress (0.0 to 1.0, linear).
    pub fn raw_progress(&self) -> f32 {
        self.progress
    }

    /// Get the eased blend factor (0.0 to 1.0).
    pub fn blend_factor(&self) -> f32 {
        self.config.easing.apply(self.progress)
    }

    /// Get the current interpolated visuals.
    pub fn current_visuals(&self) -> BiomeVisuals {
        let t = self.blend_factor();
        self.from_visuals.lerp(&self.to_visuals, t)
    }

    /// Get the tint alpha for overlay effects.
    ///
    /// Returns 0.0 when not transitioning or at start/end,
    /// peaks at `config.tint_alpha` at the midpoint.
    pub fn tint_alpha(&self) -> f32 {
        if !self.config.apply_tint || !self.active {
            return 0.0;
        }

        // Bell curve: peaks at t=0.5
        let t = self.progress;
        let peak = self.config.tint_alpha;
        // 4 * t * (1 - t) gives a parabola that peaks at 0.5 with value 1
        peak * 4.0 * t * (1.0 - t)
    }

    /// Get the tint color for overlay effects.
    ///
    /// Uses a blend of source and target ambient colors.
    pub fn tint_color(&self) -> Vec3 {
        // Average of the two ambient colors for a neutral tint
        (self.from_visuals.ambient_color + self.to_visuals.ambient_color) * 0.5
    }

    /// Get the source biome (if transitioning).
    pub fn from_biome(&self) -> Option<BiomeType> {
        self.from_biome
    }

    /// Get the target biome (if transitioning).
    pub fn to_biome(&self) -> Option<BiomeType> {
        self.to_biome
    }

    /// Get the configuration.
    pub fn config(&self) -> &TransitionConfig {
        &self.config
    }

    /// Modify the configuration.
    pub fn config_mut(&mut self) -> &mut TransitionConfig {
        &mut self.config
    }

    /// Force-complete the current transition immediately.
    pub fn complete(&mut self) {
        if self.active {
            self.progress = 1.0;
            self.active = false;
            self.from_biome = self.to_biome;
            self.from_visuals = self.to_visuals;
        }
    }

    /// Cancel the current transition, snapping back to the source biome.
    pub fn cancel(&mut self) {
        self.active = false;
        self.progress = 0.0;
        self.to_biome = self.from_biome;
        self.to_visuals = self.from_visuals;
    }
}

impl Default for TransitionEffect {
    fn default() -> Self {
        Self::new(TransitionConfig::default())
    }
}

/// Simple linear interpolation helper.
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_linear() {
        let e = EasingFunction::Linear;
        assert!((e.apply(0.0) - 0.0).abs() < 0.001);
        assert!((e.apply(0.5) - 0.5).abs() < 0.001);
        assert!((e.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_smoothstep_endpoints() {
        let e = EasingFunction::SmoothStep;
        assert!((e.apply(0.0) - 0.0).abs() < 0.001);
        assert!((e.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_smoothstep_midpoint() {
        let e = EasingFunction::SmoothStep;
        // SmoothStep at 0.5 should be 0.5
        assert!((e.apply(0.5) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_easing_clamps_input() {
        let e = EasingFunction::Linear;
        assert!((e.apply(-0.5) - 0.0).abs() < 0.001);
        assert!((e.apply(1.5) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_transition_start() {
        let mut effect = TransitionEffect::default();
        assert!(!effect.is_active());

        effect.start(Some(BiomeType::Forest), BiomeType::Desert);
        assert!(effect.is_active());
        assert_eq!(effect.from_biome(), Some(BiomeType::Forest));
        assert_eq!(effect.to_biome(), Some(BiomeType::Desert));
        assert!((effect.raw_progress() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_transition_same_biome_noop() {
        let mut effect = TransitionEffect::default();
        effect.start(Some(BiomeType::Forest), BiomeType::Forest);
        assert!(!effect.is_active()); // Same biome = no transition
    }

    #[test]
    fn test_transition_update_progress() {
        let config = TransitionConfig {
            duration: 1.0,
            easing: EasingFunction::Linear,
            ..Default::default()
        };
        let mut effect = TransitionEffect::new(config);
        effect.start(Some(BiomeType::Grassland), BiomeType::Tundra);

        effect.update(0.25); // 25% of 1 second
        assert!((effect.raw_progress() - 0.25).abs() < 0.01);
        assert!(effect.is_active());

        effect.update(0.5); // Now at 75%
        assert!((effect.raw_progress() - 0.75).abs() < 0.01);
        assert!(effect.is_active());

        effect.update(0.5); // Now at 125% -> clamped to 1.0, complete
        assert!((effect.raw_progress() - 1.0).abs() < 0.01);
        assert!(!effect.is_active()); // Transition finished
    }

    #[test]
    fn test_blend_factor_with_easing() {
        let config = TransitionConfig {
            duration: 1.0,
            easing: EasingFunction::EaseIn,
            ..Default::default()
        };
        let mut effect = TransitionEffect::new(config);
        effect.start(Some(BiomeType::Beach), BiomeType::River);

        effect.update(0.5); // 50% progress
        let raw = effect.raw_progress();
        let eased = effect.blend_factor();

        // EaseIn at 0.5 should be 0.25 (t * t)
        assert!((raw - 0.5).abs() < 0.01);
        assert!((eased - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_current_visuals_interpolation() {
        let config = TransitionConfig {
            duration: 1.0,
            easing: EasingFunction::Linear,
            ..Default::default()
        };
        let mut effect = TransitionEffect::new(config);
        effect.start(Some(BiomeType::Desert), BiomeType::Swamp);

        effect.update(0.5); // 50%

        let visuals = effect.current_visuals();
        let desert = BiomeVisuals::for_biome(BiomeType::Desert);
        let swamp = BiomeVisuals::for_biome(BiomeType::Swamp);

        // Fog density should be midpoint
        let expected_density = (desert.fog_density + swamp.fog_density) / 2.0;
        assert!((visuals.fog_density - expected_density).abs() < 0.0001);
    }

    #[test]
    fn test_tint_alpha_peaks_at_midpoint() {
        let config = TransitionConfig {
            duration: 1.0,
            easing: EasingFunction::Linear,
            apply_tint: true,
            tint_alpha: 0.2,
            ..Default::default()
        };
        let mut effect = TransitionEffect::new(config);
        effect.start(Some(BiomeType::Forest), BiomeType::Mountain);

        // At t=0
        assert!((effect.tint_alpha() - 0.0).abs() < 0.01);

        // At t=0.5 (midpoint)
        effect.update(0.5);
        // Peak should be tint_alpha * 4 * 0.5 * 0.5 = tint_alpha
        assert!((effect.tint_alpha() - 0.2).abs() < 0.01);

        // At t=1.0
        effect.update(0.5);
        assert!((effect.tint_alpha() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_complete_snaps_to_end() {
        let mut effect = TransitionEffect::default();
        effect.start(Some(BiomeType::Swamp), BiomeType::Beach);
        effect.update(0.25);

        assert!(effect.is_active());
        effect.complete();

        assert!(!effect.is_active());
        assert!((effect.raw_progress() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cancel_reverts_to_source() {
        let mut effect = TransitionEffect::default();
        effect.start(Some(BiomeType::River), BiomeType::Tundra);
        effect.update(0.5);

        assert!(effect.is_active());
        effect.cancel();

        assert!(!effect.is_active());
        assert!((effect.raw_progress() - 0.0).abs() < 0.001);
        assert_eq!(effect.to_biome(), effect.from_biome());
    }

    #[test]
    fn test_biome_visuals_for_all_biomes() {
        // Ensure no panics and reasonable values
        for bt in BiomeType::all() {
            let v = BiomeVisuals::for_biome(*bt);
            assert!(v.fog_density > 0.0);
            assert!(v.fog_end > v.fog_start);
            assert!(v.ambient_intensity > 0.0);
        }
    }

    #[test]
    fn test_visuals_lerp() {
        let a = BiomeVisuals {
            fog_color: Vec3::new(0.0, 0.0, 0.0),
            fog_density: 0.001,
            fog_start: 10.0,
            fog_end: 100.0,
            ambient_color: Vec3::new(0.0, 0.0, 0.0),
            ambient_intensity: 0.1,
            ..Default::default()
        };
        let b = BiomeVisuals {
            fog_color: Vec3::new(1.0, 1.0, 1.0),
            fog_density: 0.003,
            fog_start: 50.0,
            fog_end: 500.0,
            ambient_color: Vec3::new(1.0, 1.0, 1.0),
            ambient_intensity: 0.5,
            ..Default::default()
        };

        let mid = a.lerp(&b, 0.5);
        assert!((mid.fog_density - 0.002).abs() < 0.0001);
        assert!((mid.fog_start - 30.0).abs() < 0.1);
        assert!((mid.fog_end - 300.0).abs() < 0.1);
        assert!((mid.ambient_intensity - 0.3).abs() < 0.01);
        assert!((mid.fog_color.x - 0.5).abs() < 0.01);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Integration tests — BiomeDetector + TransitionEffect pipeline
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_integration_detector_fires_transition() {
        use crate::biome_detector::{BiomeDetector, BiomeDetectorConfig};
        use astraweave_terrain::climate::{ClimateConfig, ClimateMap};

        // Setup climate and detector
        let climate = ClimateMap::new(&ClimateConfig::default(), 42);
        let cfg = BiomeDetectorConfig {
            sample_distance_threshold: 1.0,
            hysteresis_count: 1, // Quick trigger for test
        };
        let mut detector = BiomeDetector::new(cfg);
        let mut effect = TransitionEffect::new(TransitionConfig::default());

        // First sample — should fire initial biome transition
        let height = climate.estimate_height(0.0, 0.0);
        if let Some(transition) = detector.update(&climate, 0.0, 0.0, height) {
            effect.start(transition.old_biome, transition.new_biome);
        }

        // Effect should be active with first biome
        assert!(effect.is_active() || detector.current_biome().is_some());
    }

    #[test]
    fn test_integration_full_walk_simulation() {
        use crate::biome_detector::{BiomeDetector, BiomeDetectorConfig};
        use astraweave_terrain::climate::{ClimateConfig, ClimateMap};

        let climate = ClimateMap::new(&ClimateConfig::default(), 12345);
        let cfg = BiomeDetectorConfig {
            sample_distance_threshold: 5.0,
            hysteresis_count: 2,
        };
        let mut detector = BiomeDetector::new(cfg);
        let mut effect = TransitionEffect::new(TransitionConfig {
            duration: 1.0,
            easing: EasingFunction::SmoothStep,
            ..Default::default()
        });

        let mut transitions_fired = 0;
        let mut transition_visuals: Vec<BiomeVisuals> = Vec::new();

        // Walk 200 steps
        for i in 0..200 {
            let x = i as f64 * 10.0;
            let height = climate.estimate_height(x, 0.0);

            if let Some(t) = detector.update(&climate, x, 0.0, height) {
                effect.start(t.old_biome, t.new_biome);
                transitions_fired += 1;
            }

            // Simulate 50ms per step
            if effect.is_active() {
                effect.update(0.05);
                transition_visuals.push(effect.current_visuals());
            }
        }

        // Should have detected at least one transition in 2000 world units
        assert!(transitions_fired > 0, "Expected at least one transition");
        // Should have collected some transition visuals
        assert!(transition_visuals.len() >= transitions_fired);
    }

    #[test]
    fn test_integration_biome_visuals_match_biome() {
        // Verify BiomeVisuals::for_biome returns distinct values per biome
        let grassland = BiomeVisuals::for_biome(BiomeType::Grassland);
        let desert = BiomeVisuals::for_biome(BiomeType::Desert);
        let forest = BiomeVisuals::for_biome(BiomeType::Forest);
        let tundra = BiomeVisuals::for_biome(BiomeType::Tundra);

        // Fog densities should differ meaningfully
        assert!((grassland.fog_density - desert.fog_density).abs() > 0.0001
            || (grassland.fog_color - desert.fog_color).length() > 0.1);
        assert!((forest.fog_density - tundra.fog_density).abs() > 0.0001
            || (forest.fog_color - tundra.fog_color).length() > 0.1);

        // Ambient should be distinct
        assert!((grassland.ambient_intensity - forest.ambient_intensity).abs() > 0.01
            || (grassland.ambient_color - forest.ambient_color).length() > 0.1);
    }

    #[test]
    fn test_integration_transition_produces_smooth_curve() {
        let mut effect = TransitionEffect::new(TransitionConfig {
            duration: 1.0,
            easing: EasingFunction::SmoothStep,
            blend_fog: true,
            blend_ambient: true,
            ..Default::default()
        });

        effect.start(Some(BiomeType::Grassland), BiomeType::Forest);

        let mut blend_factors = Vec::new();
        for _ in 0..20 {
            effect.update(0.05);
            blend_factors.push(effect.blend_factor());
        }

        // Verify monotonic increase (smoothstep should never decrease)
        for i in 1..blend_factors.len() {
            assert!(
                blend_factors[i] >= blend_factors[i - 1] - 0.001,
                "Blend factor should be monotonically increasing"
            );
        }

        // Final should be at or near 1.0
        assert!(blend_factors.last().unwrap() > &0.99);
    }

    #[test]
    fn test_integration_rapid_biome_changes() {
        // Simulate rapid biome changes (player running through border)
        let mut effect = TransitionEffect::new(TransitionConfig {
            duration: 2.0,
            ..Default::default()
        });

        // Start first transition
        effect.start(Some(BiomeType::Grassland), BiomeType::Forest);
        effect.update(0.5); // 25% through
        let mid_blend = effect.blend_factor();
        assert!(mid_blend > 0.0 && mid_blend < 1.0);

        // Immediately start new transition (interrupts)
        effect.start(Some(BiomeType::Forest), BiomeType::Desert);
        assert!(effect.is_active());
        assert_eq!(effect.to_biome(), Some(BiomeType::Desert));

        // Progress should reset
        assert!(effect.blend_factor() < 0.1);
    }

    #[test]
    fn test_integration_none_source_biome() {
        // Test behavior when old_biome is None (initial world entry)
        let mut effect = TransitionEffect::new(TransitionConfig::default());

        effect.start(None, BiomeType::Mountain);
        assert!(effect.is_active());
        assert_eq!(effect.to_biome(), Some(BiomeType::Mountain));

        // Should use Mountain as "from" when None provided
        let visuals = effect.current_visuals();
        let mountain = BiomeVisuals::for_biome(BiomeType::Mountain);
        assert!((visuals.fog_density - mountain.fog_density).abs() < 0.001);
    }

    // ── Sky / Water field tests ─────────────────────────────────

    #[test]
    fn sky_colors_differ_across_biomes() {
        let forest = BiomeVisuals::for_biome(BiomeType::Forest);
        let desert = BiomeVisuals::for_biome(BiomeType::Desert);
        // Day horizons should be noticeably different (forest=green tint, desert=warm)
        let diff = (forest.sky_day_horizon - desert.sky_day_horizon).length();
        assert!(diff > 0.05, "sky_day_horizon too similar: {diff}");
    }

    #[test]
    fn water_colors_differ_across_biomes() {
        let beach = BiomeVisuals::for_biome(BiomeType::Beach);
        let swamp = BiomeVisuals::for_biome(BiomeType::Swamp);
        let diff_deep = (beach.water_deep - swamp.water_deep).length();
        let diff_foam = (beach.water_foam - swamp.water_foam).length();
        assert!(diff_deep > 0.05, "water_deep too similar: {diff_deep}");
        assert!(diff_foam > 0.05, "water_foam too similar: {diff_foam}");
    }

    #[test]
    fn lerp_interpolates_sky_and_water() {
        let a = BiomeVisuals::for_biome(BiomeType::Forest);
        let b = BiomeVisuals::for_biome(BiomeType::Desert);
        let mid = a.lerp(&b, 0.5);

        // Mid-point should be between the two endpoints for every field
        for (va, vm, vb) in [
            (a.sky_day_top, mid.sky_day_top, b.sky_day_top),
            (a.sky_night_horizon, mid.sky_night_horizon, b.sky_night_horizon),
            (a.water_deep, mid.water_deep, b.water_deep),
            (a.water_shallow, mid.water_shallow, b.water_shallow),
            (a.water_foam, mid.water_foam, b.water_foam),
        ] {
            for i in 0..3 {
                let lo = va[i].min(vb[i]);
                let hi = va[i].max(vb[i]);
                assert!(
                    vm[i] >= lo - 0.001 && vm[i] <= hi + 0.001,
                    "lerp component out of range"
                );
            }
        }
    }

    #[test]
    fn lerp_at_zero_returns_source() {
        let a = BiomeVisuals::for_biome(BiomeType::Tundra);
        let b = BiomeVisuals::for_biome(BiomeType::Beach);
        let result = a.lerp(&b, 0.0);
        assert!((result.sky_day_top - a.sky_day_top).length() < 0.001);
        assert!((result.water_deep - a.water_deep).length() < 0.001);
    }

    #[test]
    fn lerp_at_one_returns_target() {
        let a = BiomeVisuals::for_biome(BiomeType::Mountain);
        let b = BiomeVisuals::for_biome(BiomeType::Swamp);
        let result = a.lerp(&b, 1.0);
        assert!((result.sky_sunset_top - b.sky_sunset_top).length() < 0.001);
        assert!((result.water_foam - b.water_foam).length() < 0.001);
    }

    #[test]
    fn to_sky_config_roundtrip() {
        let vis = BiomeVisuals::for_biome(BiomeType::River);
        let cfg = vis.to_sky_config();
        assert_eq!(cfg.day_color_top, vis.sky_day_top);
        assert_eq!(cfg.day_color_horizon, vis.sky_day_horizon);
        assert_eq!(cfg.sunset_color_top, vis.sky_sunset_top);
        assert_eq!(cfg.sunset_color_horizon, vis.sky_sunset_horizon);
        assert_eq!(cfg.night_color_top, vis.sky_night_top);
        assert_eq!(cfg.night_color_horizon, vis.sky_night_horizon);
    }

    #[test]
    fn all_biomes_have_valid_sky_ranges() {
        // Sky colours should be in [0, 1] for all biomes
        for biome in [
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Grassland,
            BiomeType::Mountain,
            BiomeType::Tundra,
            BiomeType::Swamp,
            BiomeType::Beach,
            BiomeType::River,
        ] {
            let v = BiomeVisuals::for_biome(biome);
            for sky in [
                v.sky_day_top,
                v.sky_day_horizon,
                v.sky_sunset_top,
                v.sky_sunset_horizon,
                v.sky_night_top,
                v.sky_night_horizon,
            ] {
                for c in [sky.x, sky.y, sky.z] {
                    assert!(
                        (0.0..=1.0).contains(&c),
                        "sky colour out of range for {:?}: {c}",
                        biome
                    );
                }
            }
        }
    }

    #[test]
    fn all_biomes_have_valid_water_ranges() {
        for biome in [
            BiomeType::Forest,
            BiomeType::Desert,
            BiomeType::Grassland,
            BiomeType::Mountain,
            BiomeType::Tundra,
            BiomeType::Swamp,
            BiomeType::Beach,
            BiomeType::River,
        ] {
            let v = BiomeVisuals::for_biome(biome);
            for w in [v.water_deep, v.water_shallow, v.water_foam] {
                for c in [w.x, w.y, w.z] {
                    assert!(
                        (0.0..=1.0).contains(&c),
                        "water colour out of range for {:?}: {c}",
                        biome
                    );
                }
            }
        }
    }

    #[test]
    fn transition_effect_carries_sky_water_through() {
        let mut effect = TransitionEffect::new(TransitionConfig {
            duration: 2.0,
            easing: EasingFunction::Linear,
            ..Default::default()
        });
        effect.start(Some(BiomeType::Beach), BiomeType::Swamp);

        // At t=0 should be Beach
        let v0 = effect.current_visuals();
        let beach = BiomeVisuals::for_biome(BiomeType::Beach);
        assert!((v0.water_deep - beach.water_deep).length() < 0.01);

        // Advance to 50%
        effect.update(1.0);
        let v_mid = effect.current_visuals();
        let swamp = BiomeVisuals::for_biome(BiomeType::Swamp);
        // Mid should be between beach and swamp
        let mid_deep_expected = beach.water_deep.lerp(swamp.water_deep, 0.5);
        assert!(
            (v_mid.water_deep - mid_deep_expected).length() < 0.05,
            "transition mid-point water_deep off"
        );

        // Advance to 100%
        effect.update(1.0);
        let v_end = effect.current_visuals();
        assert!((v_end.sky_day_top - swamp.sky_day_top).length() < 0.01);
    }
}
