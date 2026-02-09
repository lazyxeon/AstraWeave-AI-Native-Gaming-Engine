//! Weather System — Biome-aware weather transitions and probabilities.
//!
//! Provides:
//! - **`WeatherTransition`**: Smooth crossfade of fog/ambient multipliers and particle
//!   density when weather changes (e.g., Rain → Snow during a biome transition).
//! - **`BiomeWeatherMap`**: Per-biome weighted probability tables for weather selection.
//! - **`BiomeWindProfile`**: Characteristic wind parameters per biome.
//!
//! # Usage
//!
//! ```rust,no_run
//! # use astraweave_render::weather_system::{WeatherTransition, BiomeWeatherMap, BiomeWindProfile};
//! # use astraweave_render::effects::WeatherKind;
//! # use astraweave_terrain::biome::BiomeType;
//! // Select weather from biome probability table
//! let weather = BiomeWeatherMap::pick(BiomeType::Tundra, 0.3);
//! assert_eq!(weather, WeatherKind::Snow);
//!
//! // Smooth weather transition
//! let mut transition = WeatherTransition::new(3.0); // 3-second crossfade
//! transition.start(WeatherKind::Rain, WeatherKind::Snow);
//! transition.update(1.5); // half-way
//! let (fog, ambient) = transition.current_multipliers();
//!
//! // Wind profile for biome
//! let wind = BiomeWindProfile::for_biome(BiomeType::Mountain);
//! assert!(wind.base_strength > 1.0); // mountains are windy
//! ```

use crate::effects::WeatherKind;
use astraweave_terrain::biome::BiomeType;
use glam::Vec3;

// ═════════════════════════════════════════════════════════════════════════
// WeatherTransition — smooth crossfade between weather states
// ═════════════════════════════════════════════════════════════════════════

/// Fog/ambient multiplier pair for a weather type.
#[derive(Debug, Clone, Copy)]
struct WeatherMultipliers {
    fog: f32,
    ambient: f32,
    particle_density: f32,
}

impl WeatherMultipliers {
    /// Get the multiplier set for a given weather kind.
    fn for_kind(kind: WeatherKind) -> Self {
        match kind {
            WeatherKind::None => Self {
                fog: 1.0,
                ambient: 1.0,
                particle_density: 0.0,
            },
            WeatherKind::Rain => Self {
                fog: 2.5,
                ambient: 0.6,
                particle_density: 1.0,
            },
            WeatherKind::Snow => Self {
                fog: 1.8,
                ambient: 0.75,
                particle_density: 1.0,
            },
            WeatherKind::Sandstorm => Self {
                fog: 4.0,
                ambient: 0.4,
                particle_density: 1.0,
            },
            WeatherKind::WindTrails => Self {
                fog: 1.4,
                ambient: 0.9,
                particle_density: 0.6,
            },
        }
    }

    /// Linearly interpolate between two multiplier sets.
    fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            fog: a.fog + (b.fog - a.fog) * t,
            ambient: a.ambient + (b.ambient - a.ambient) * t,
            particle_density: a.particle_density + (b.particle_density - a.particle_density) * t,
        }
    }
}

/// Smooth crossfade between two weather states.
///
/// When weather changes (e.g. transitioning from a biome with Rain to one
/// with Snow), this struct provides eased interpolation of:
/// - Fog multiplier
/// - Ambient multiplier
/// - Particle density (for crossfading particle systems)
///
/// The particle crossfade allows both the outgoing and incoming weather
/// particle effects to coexist briefly, preventing jarring visual pops.
#[derive(Debug, Clone)]
pub struct WeatherTransition {
    /// Duration of the crossfade in seconds.
    duration: f32,
    /// Source weather.
    from: WeatherKind,
    /// Target weather.
    to: WeatherKind,
    /// Progress (0.0 … 1.0).
    progress: f32,
    /// Whether a transition is currently active.
    active: bool,
    /// Cached multipliers for source.
    from_mul: WeatherMultipliers,
    /// Cached multipliers for target.
    to_mul: WeatherMultipliers,
}

impl WeatherTransition {
    /// Create a new weather transition handler with a given crossfade
    /// duration (seconds).
    pub fn new(duration: f32) -> Self {
        Self {
            duration: duration.max(0.01),
            from: WeatherKind::None,
            to: WeatherKind::None,
            progress: 1.0,
            active: false,
            from_mul: WeatherMultipliers::for_kind(WeatherKind::None),
            to_mul: WeatherMultipliers::for_kind(WeatherKind::None),
        }
    }

    /// Start a crossfade from one weather kind to another.
    ///
    /// If the same kind is requested, this is a no-op.
    pub fn start(&mut self, from: WeatherKind, to: WeatherKind) {
        if from == to {
            return;
        }
        self.from = from;
        self.to = to;
        self.from_mul = WeatherMultipliers::for_kind(from);
        self.to_mul = WeatherMultipliers::for_kind(to);
        self.progress = 0.0;
        self.active = true;
    }

    /// Advance the crossfade by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }
        let rate = 1.0 / self.duration;
        self.progress += dt * rate;
        if self.progress >= 1.0 {
            self.progress = 1.0;
            self.active = false;
            self.from = self.to;
            self.from_mul = self.to_mul;
        }
    }

    /// Whether a transition is in progress.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Raw linear progress (0.0 … 1.0).
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Eased progress using SmoothStep for perceptual smoothness.
    pub fn eased_progress(&self) -> f32 {
        let t = self.progress.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t) // smoothstep
    }

    /// Current interpolated fog and ambient multipliers.
    ///
    /// Returns `(fog_multiplier, ambient_multiplier)`.
    pub fn current_multipliers(&self) -> (f32, f32) {
        let t = self.eased_progress();
        let m = WeatherMultipliers::lerp(&self.from_mul, &self.to_mul, t);
        (m.fog, m.ambient)
    }

    /// Current interpolated particle density (0.0 … 1.0).
    ///
    /// Use this to scale the incoming weather particle count while
    /// fading out the outgoing particles.
    pub fn current_particle_density(&self) -> f32 {
        let t = self.eased_progress();
        WeatherMultipliers::lerp(&self.from_mul, &self.to_mul, t).particle_density
    }

    /// Outgoing weather particle fade-out factor (1.0 → 0.0).
    ///
    /// Multiply the outgoing particle system's density by this value
    /// during the crossfade to smoothly retire it.
    pub fn outgoing_particle_fade(&self) -> f32 {
        if !self.active {
            return 0.0;
        }
        1.0 - self.eased_progress()
    }

    /// Incoming weather particle fade-in factor (0.0 → 1.0).
    ///
    /// Multiply the incoming particle system's density by this value
    /// during the crossfade.
    pub fn incoming_particle_fade(&self) -> f32 {
        if !self.active {
            return 1.0;
        }
        self.eased_progress()
    }

    /// Get the source weather kind.
    pub fn from_kind(&self) -> WeatherKind {
        self.from
    }

    /// Get the target weather kind.
    pub fn to_kind(&self) -> WeatherKind {
        self.to
    }

    /// Current effective weather (the target once completed).
    pub fn current_kind(&self) -> WeatherKind {
        self.to
    }

    /// Set the crossfade duration.
    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration.max(0.01);
    }

    /// Get the crossfade duration.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Force-complete the current transition.
    pub fn complete(&mut self) {
        self.progress = 1.0;
        self.active = false;
        self.from = self.to;
        self.from_mul = self.to_mul;
    }
}

impl Default for WeatherTransition {
    fn default() -> Self {
        Self::new(3.0)
    }
}

// ═════════════════════════════════════════════════════════════════════════
// BiomeWeatherMap — probabilities for weather per biome
// ═════════════════════════════════════════════════════════════════════════

/// A weighted entry in the weather probability table.
#[derive(Debug, Clone, Copy)]
pub struct WeatherWeight {
    pub kind: WeatherKind,
    pub weight: f32,
}

/// Maps each biome to a weighted set of likely weather types.
///
/// Use [`BiomeWeatherMap::pick`] with a random `roll` value (0.0 … 1.0)
/// to sample a weather type for a biome. The weights are normalized
/// internally so they need not sum to 1.0.
pub struct BiomeWeatherMap;

impl BiomeWeatherMap {
    /// Get the raw weighted table for a biome.
    pub fn weights(biome: BiomeType) -> &'static [WeatherWeight] {
        match biome {
            BiomeType::Forest => &[
                WeatherWeight {
                    kind: WeatherKind::None,
                    weight: 0.45,
                },
                WeatherWeight {
                    kind: WeatherKind::Rain,
                    weight: 0.35,
                },
                WeatherWeight {
                    kind: WeatherKind::WindTrails,
                    weight: 0.15,
                },
                WeatherWeight {
                    kind: WeatherKind::Snow,
                    weight: 0.05,
                },
            ],
            BiomeType::Desert => &[
                WeatherWeight {
                    kind: WeatherKind::None,
                    weight: 0.60,
                },
                WeatherWeight {
                    kind: WeatherKind::Sandstorm,
                    weight: 0.25,
                },
                WeatherWeight {
                    kind: WeatherKind::WindTrails,
                    weight: 0.12,
                },
                WeatherWeight {
                    kind: WeatherKind::Rain,
                    weight: 0.03,
                },
            ],
            BiomeType::Grassland => &[
                WeatherWeight {
                    kind: WeatherKind::None,
                    weight: 0.50,
                },
                WeatherWeight {
                    kind: WeatherKind::Rain,
                    weight: 0.25,
                },
                WeatherWeight {
                    kind: WeatherKind::WindTrails,
                    weight: 0.20,
                },
                WeatherWeight {
                    kind: WeatherKind::Snow,
                    weight: 0.05,
                },
            ],
            BiomeType::Mountain => &[
                WeatherWeight {
                    kind: WeatherKind::None,
                    weight: 0.30,
                },
                WeatherWeight {
                    kind: WeatherKind::Snow,
                    weight: 0.30,
                },
                WeatherWeight {
                    kind: WeatherKind::WindTrails,
                    weight: 0.25,
                },
                WeatherWeight {
                    kind: WeatherKind::Rain,
                    weight: 0.15,
                },
            ],
            BiomeType::Tundra => &[
                WeatherWeight {
                    kind: WeatherKind::None,
                    weight: 0.25,
                },
                WeatherWeight {
                    kind: WeatherKind::Snow,
                    weight: 0.50,
                },
                WeatherWeight {
                    kind: WeatherKind::WindTrails,
                    weight: 0.20,
                },
                WeatherWeight {
                    kind: WeatherKind::Rain,
                    weight: 0.05,
                },
            ],
            BiomeType::Swamp => &[
                WeatherWeight {
                    kind: WeatherKind::None,
                    weight: 0.20,
                },
                WeatherWeight {
                    kind: WeatherKind::Rain,
                    weight: 0.55,
                },
                WeatherWeight {
                    kind: WeatherKind::WindTrails,
                    weight: 0.15,
                },
                WeatherWeight {
                    kind: WeatherKind::Snow,
                    weight: 0.10,
                },
            ],
            BiomeType::Beach => &[
                WeatherWeight {
                    kind: WeatherKind::None,
                    weight: 0.55,
                },
                WeatherWeight {
                    kind: WeatherKind::Rain,
                    weight: 0.20,
                },
                WeatherWeight {
                    kind: WeatherKind::WindTrails,
                    weight: 0.20,
                },
                WeatherWeight {
                    kind: WeatherKind::Snow,
                    weight: 0.05,
                },
            ],
            BiomeType::River => &[
                WeatherWeight {
                    kind: WeatherKind::None,
                    weight: 0.45,
                },
                WeatherWeight {
                    kind: WeatherKind::Rain,
                    weight: 0.30,
                },
                WeatherWeight {
                    kind: WeatherKind::WindTrails,
                    weight: 0.20,
                },
                WeatherWeight {
                    kind: WeatherKind::Snow,
                    weight: 0.05,
                },
            ],
            // Fallback for future biome types
            _ => &[WeatherWeight {
                kind: WeatherKind::None,
                weight: 1.0,
            }],
        }
    }

    /// Sample a weather type for a biome given a random roll in [0.0, 1.0).
    ///
    /// The weights are normalized internally, so they need not sum to 1.0.
    /// If `roll` is out of range it is clamped.
    pub fn pick(biome: BiomeType, roll: f32) -> WeatherKind {
        let table = Self::weights(biome);
        let total: f32 = table.iter().map(|w| w.weight).sum();
        if total <= 0.0 {
            return WeatherKind::None;
        }
        let roll = roll.clamp(0.0, 0.9999) * total;
        let mut acc = 0.0;
        for entry in table {
            acc += entry.weight;
            if roll < acc {
                return entry.kind;
            }
        }
        // Fallback (shouldn't reach here)
        table.last().map_or(WeatherKind::None, |e| e.kind)
    }

    /// Get the probability (0.0 … 1.0) of a specific weather kind in a biome.
    pub fn probability(biome: BiomeType, kind: WeatherKind) -> f32 {
        let table = Self::weights(biome);
        let total: f32 = table.iter().map(|w| w.weight).sum();
        if total <= 0.0 {
            return 0.0;
        }
        table
            .iter()
            .find(|w| w.kind == kind)
            .map_or(0.0, |w| w.weight / total)
    }

    /// Get the most likely weather for a biome.
    pub fn most_likely(biome: BiomeType) -> WeatherKind {
        let table = Self::weights(biome);
        table
            .iter()
            .max_by(|a, b| {
                a.weight
                    .partial_cmp(&b.weight)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map_or(WeatherKind::None, |w| w.kind)
    }
}

// ═════════════════════════════════════════════════════════════════════════
// BiomeWindProfile — characteristic wind per biome
// ═════════════════════════════════════════════════════════════════════════

/// Per-biome wind characteristics fed into `WeatherFx::set_wind()`.
#[derive(Debug, Clone, Copy)]
pub struct BiomeWindProfile {
    /// Base wind strength multiplier (1.0 = default).
    pub base_strength: f32,
    /// Whether wind direction shifts over time (gusty).
    pub gusty: bool,
    /// Gust frequency in Hz (how often direction shifts).
    pub gust_frequency: f32,
    /// Gust strength variance (added to base_strength during gusts).
    pub gust_variance: f32,
    /// Dominant wind direction (normalized XZ plane).
    pub dominant_direction: Vec3,
}

impl Default for BiomeWindProfile {
    fn default() -> Self {
        Self {
            base_strength: 1.0,
            gusty: false,
            gust_frequency: 0.0,
            gust_variance: 0.0,
            dominant_direction: Vec3::new(1.0, 0.0, 0.0),
        }
    }
}

impl BiomeWindProfile {
    /// Get the wind profile for a specific biome.
    pub fn for_biome(biome: BiomeType) -> Self {
        match biome {
            BiomeType::Forest => Self {
                base_strength: 0.4, // Canopy blocks wind
                gusty: true,
                gust_frequency: 0.15, // Occasional gusts through clearings
                gust_variance: 0.6,
                dominant_direction: Vec3::new(0.8, 0.0, 0.6),
            },
            BiomeType::Desert => Self {
                base_strength: 1.8, // Open terrain, strong winds
                gusty: true,
                gust_frequency: 0.3, // Frequent gusts
                gust_variance: 1.5,  // Can spike during sandstorms
                dominant_direction: Vec3::new(1.0, 0.0, 0.2),
            },
            BiomeType::Grassland => Self {
                base_strength: 1.2, // Open terrain, moderate winds
                gusty: true,
                gust_frequency: 0.2,
                gust_variance: 0.8,
                dominant_direction: Vec3::new(0.9, 0.0, 0.4),
            },
            BiomeType::Mountain => Self {
                base_strength: 2.5, // Strong altitude winds
                gusty: true,
                gust_frequency: 0.4,                          // Very gusty
                gust_variance: 2.0,                           // Extreme gusts possible
                dominant_direction: Vec3::new(0.6, 0.0, 0.8), // Cross-slope
            },
            BiomeType::Tundra => Self {
                base_strength: 2.0, // Arctic winds
                gusty: true,
                gust_frequency: 0.25,
                gust_variance: 1.2,
                dominant_direction: Vec3::new(0.0, 0.0, 1.0), // From the pole
            },
            BiomeType::Swamp => Self {
                base_strength: 0.3, // Stagnant air
                gusty: false,
                gust_frequency: 0.05,
                gust_variance: 0.2,
                dominant_direction: Vec3::new(0.5, 0.0, 0.5),
            },
            BiomeType::Beach => Self {
                base_strength: 1.5, // Sea breeze
                gusty: true,
                gust_frequency: 0.2,
                gust_variance: 0.9,
                dominant_direction: Vec3::new(-1.0, 0.0, 0.0), // Onshore
            },
            BiomeType::River => Self {
                base_strength: 0.8, // Follows the valley
                gusty: false,
                gust_frequency: 0.1,
                gust_variance: 0.4,
                dominant_direction: Vec3::new(0.7, 0.0, 0.7),
            },
            // Fallback
            _ => Self::default(),
        }
    }

    /// Compute the effective wind strength at a given time, accounting for gusts.
    ///
    /// `time` is total elapsed seconds; the gust pattern is periodic.
    pub fn effective_strength(&self, time: f32) -> f32 {
        if !self.gusty || self.gust_frequency <= 0.0 {
            return self.base_strength;
        }
        // Sine-based gust modulation
        let phase = time * self.gust_frequency * std::f32::consts::TAU;
        let gust = phase.sin().max(0.0); // Only positive gusts
        self.base_strength + gust * self.gust_variance
    }

    /// Compute the effective wind direction at a given time.
    ///
    /// When gusty, the direction oscillates ±30° around the dominant.
    pub fn effective_direction(&self, time: f32) -> Vec3 {
        if !self.gusty || self.gust_frequency <= 0.0 {
            let len = self.dominant_direction.length();
            return if len > 0.001 {
                self.dominant_direction / len
            } else {
                Vec3::X
            };
        }
        let angle_offset = (time * self.gust_frequency * std::f32::consts::TAU * 0.7).sin() * 0.52; // ±30° in radians
        let cos_a = angle_offset.cos();
        let sin_a = angle_offset.sin();
        let d = self.dominant_direction;
        // Rotate in XZ plane
        let rotated = Vec3::new(d.x * cos_a - d.z * sin_a, 0.0, d.x * sin_a + d.z * cos_a);
        let len = rotated.length();
        if len > 0.001 {
            rotated / len
        } else {
            Vec3::X
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── WeatherTransition tests ────────────────────────────────────────

    #[test]
    fn transition_starts_inactive() {
        let wt = WeatherTransition::new(2.0);
        assert!(!wt.is_active());
        assert_eq!(wt.current_kind(), WeatherKind::None);
    }

    #[test]
    fn transition_start_activates() {
        let mut wt = WeatherTransition::new(2.0);
        wt.start(WeatherKind::None, WeatherKind::Rain);
        assert!(wt.is_active());
        assert_eq!(wt.from_kind(), WeatherKind::None);
        assert_eq!(wt.to_kind(), WeatherKind::Rain);
        assert!((wt.progress() - 0.0).abs() < 0.001);
    }

    #[test]
    fn transition_same_kind_noop() {
        let mut wt = WeatherTransition::new(2.0);
        wt.start(WeatherKind::Rain, WeatherKind::Rain);
        assert!(!wt.is_active());
    }

    #[test]
    fn transition_progress_advances() {
        let mut wt = WeatherTransition::new(2.0);
        wt.start(WeatherKind::None, WeatherKind::Snow);
        wt.update(1.0); // 50%
        assert!((wt.progress() - 0.5).abs() < 0.01);
        assert!(wt.is_active());

        wt.update(1.0); // 100%
        assert!((wt.progress() - 1.0).abs() < 0.01);
        assert!(!wt.is_active());
    }

    #[test]
    fn transition_completes_snaps_from() {
        let mut wt = WeatherTransition::new(1.0);
        wt.start(WeatherKind::Rain, WeatherKind::Snow);
        wt.update(1.5); // Overshoot
        assert!(!wt.is_active());
        assert_eq!(wt.from_kind(), WeatherKind::Snow);
        assert_eq!(wt.to_kind(), WeatherKind::Snow);
    }

    #[test]
    fn transition_multipliers_at_t0() {
        let mut wt = WeatherTransition::new(2.0);
        wt.start(WeatherKind::None, WeatherKind::Rain);
        let (fog, ambient) = wt.current_multipliers();
        // At t=0 (eased=0) should be "None" multipliers: 1.0, 1.0
        assert!((fog - 1.0).abs() < 0.01);
        assert!((ambient - 1.0).abs() < 0.01);
    }

    #[test]
    fn transition_multipliers_at_t1() {
        let mut wt = WeatherTransition::new(1.0);
        wt.start(WeatherKind::None, WeatherKind::Rain);
        wt.update(1.0); // Complete
        let (fog, ambient) = wt.current_multipliers();
        // At t=1 (eased=1) should be "Rain" multipliers: 2.5, 0.6
        assert!((fog - 2.5).abs() < 0.01);
        assert!((ambient - 0.6).abs() < 0.01);
    }

    #[test]
    fn transition_multipliers_midpoint() {
        let mut wt = WeatherTransition::new(2.0);
        wt.start(WeatherKind::None, WeatherKind::Sandstorm);
        wt.update(1.0); // 50% linear → smoothstep ≈ 0.5
        let (fog, ambient) = wt.current_multipliers();
        // At smoothstep(0.5) = 0.5 → fog = lerp(1.0, 4.0, 0.5) = 2.5
        assert!((fog - 2.5).abs() < 0.1);
        // ambient = lerp(1.0, 0.4, 0.5) = 0.7
        assert!((ambient - 0.7).abs() < 0.1);
    }

    #[test]
    fn transition_smoothstep_easing() {
        let mut wt = WeatherTransition::new(4.0);
        wt.start(WeatherKind::Rain, WeatherKind::Snow);
        // At 25% linear progress, smoothstep should be less than 0.25
        wt.update(1.0);
        let eased = wt.eased_progress();
        assert!(eased < 0.25 + 0.02, "smoothstep at 0.25 should be < 0.25");
    }

    #[test]
    fn transition_particle_fade() {
        let mut wt = WeatherTransition::new(2.0);
        wt.start(WeatherKind::Rain, WeatherKind::Snow);

        // At t=0
        assert!((wt.outgoing_particle_fade() - 1.0).abs() < 0.01);
        assert!((wt.incoming_particle_fade() - 0.0).abs() < 0.01);

        // Midpoint
        wt.update(1.0);
        let out = wt.outgoing_particle_fade();
        let incoming = wt.incoming_particle_fade();
        assert!(out > 0.3 && out < 0.7, "outgoing at 50% should be ~0.5");
        assert!(
            incoming > 0.3 && incoming < 0.7,
            "incoming at 50% should be ~0.5"
        );
        assert!((out + incoming - 1.0).abs() < 0.01, "should sum to ~1.0");

        // Complete
        wt.update(1.0);
        assert!((wt.outgoing_particle_fade() - 0.0).abs() < 0.01);
        assert!((wt.incoming_particle_fade() - 1.0).abs() < 0.01);
    }

    #[test]
    fn transition_force_complete() {
        let mut wt = WeatherTransition::new(5.0);
        wt.start(WeatherKind::None, WeatherKind::Sandstorm);
        wt.update(0.5); // 10%
        assert!(wt.is_active());

        wt.complete();
        assert!(!wt.is_active());
        assert!((wt.progress() - 1.0).abs() < 0.001);
        let (fog, _) = wt.current_multipliers();
        assert!((fog - 4.0).abs() < 0.01, "Should be sandstorm: {fog}");
    }

    #[test]
    fn transition_duration_clamped() {
        let wt = WeatherTransition::new(-1.0);
        assert!(wt.duration() >= 0.01);
    }

    #[test]
    fn transition_set_duration() {
        let mut wt = WeatherTransition::new(1.0);
        wt.set_duration(5.0);
        assert!((wt.duration() - 5.0).abs() < 0.001);
    }

    #[test]
    fn transition_default() {
        let wt = WeatherTransition::default();
        assert!((wt.duration() - 3.0).abs() < 0.001);
        assert!(!wt.is_active());
    }

    // ── BiomeWeatherMap tests ──────────────────────────────────────────

    #[test]
    fn weights_sum_to_one_for_all_biomes() {
        for biome in BiomeType::all() {
            let table = BiomeWeatherMap::weights(*biome);
            let total: f32 = table.iter().map(|w| w.weight).sum();
            assert!(
                (total - 1.0).abs() < 0.01,
                "{:?} weights sum to {total}, expected ~1.0",
                biome,
            );
        }
    }

    #[test]
    fn pick_returns_valid_weather() {
        for biome in BiomeType::all() {
            for roll in [0.0, 0.25, 0.5, 0.75, 0.99] {
                let kind = BiomeWeatherMap::pick(*biome, roll);
                // Should match one of the entries
                let table = BiomeWeatherMap::weights(*biome);
                assert!(
                    table.iter().any(|w| w.kind == kind),
                    "pick({:?}, {roll}) returned {:?} not in table",
                    biome,
                    kind,
                );
            }
        }
    }

    #[test]
    fn pick_desert_sandstorm_in_range() {
        // Desert has 60% None, 25% Sandstorm → roll [0.60, 0.85) = Sandstorm
        let kind = BiomeWeatherMap::pick(BiomeType::Desert, 0.75);
        assert_eq!(kind, WeatherKind::Sandstorm);
    }

    #[test]
    fn pick_tundra_snow_dominant() {
        // Tundra has 25% None, 50% Snow → roll [0.25, 0.75) = Snow
        let kind = BiomeWeatherMap::pick(BiomeType::Tundra, 0.5);
        assert_eq!(kind, WeatherKind::Snow);
    }

    #[test]
    fn pick_forest_none_dominant() {
        // Forest: 45% None first → roll 0.0-0.44 = None
        let kind = BiomeWeatherMap::pick(BiomeType::Forest, 0.1);
        assert_eq!(kind, WeatherKind::None);
    }

    #[test]
    fn pick_swamp_rain_dominant() {
        // Swamp has 20% None, 55% Rain → roll [0.20, 0.75) = Rain
        let kind = BiomeWeatherMap::pick(BiomeType::Swamp, 0.5);
        assert_eq!(kind, WeatherKind::Rain);
    }

    #[test]
    fn pick_clamps_out_of_range() {
        let kind = BiomeWeatherMap::pick(BiomeType::Grassland, -5.0);
        assert_eq!(kind, WeatherKind::None); // First entry
        let kind2 = BiomeWeatherMap::pick(BiomeType::Grassland, 100.0);
        // Should clamp and still return a valid kind
        assert!(BiomeWeatherMap::weights(BiomeType::Grassland)
            .iter()
            .any(|w| w.kind == kind2));
    }

    #[test]
    fn probability_sums_to_one() {
        for biome in BiomeType::all() {
            let kinds = [
                WeatherKind::None,
                WeatherKind::Rain,
                WeatherKind::Snow,
                WeatherKind::Sandstorm,
                WeatherKind::WindTrails,
            ];
            let sum: f32 = kinds
                .iter()
                .map(|k| BiomeWeatherMap::probability(*biome, *k))
                .sum();
            assert!(
                (sum - 1.0).abs() < 0.02,
                "{:?} probabilities sum to {sum}",
                biome,
            );
        }
    }

    #[test]
    fn most_likely_desert_is_none() {
        assert_eq!(
            BiomeWeatherMap::most_likely(BiomeType::Desert),
            WeatherKind::None
        );
    }

    #[test]
    fn most_likely_tundra_is_snow() {
        assert_eq!(
            BiomeWeatherMap::most_likely(BiomeType::Tundra),
            WeatherKind::Snow
        );
    }

    #[test]
    fn most_likely_swamp_is_rain() {
        assert_eq!(
            BiomeWeatherMap::most_likely(BiomeType::Swamp),
            WeatherKind::Rain
        );
    }

    // ── BiomeWindProfile tests ─────────────────────────────────────────

    #[test]
    fn wind_all_biomes_valid() {
        for biome in BiomeType::all() {
            let profile = BiomeWindProfile::for_biome(*biome);
            assert!(
                profile.base_strength >= 0.0,
                "{:?} has negative base_strength",
                biome,
            );
            assert!(
                profile.dominant_direction.length() > 0.01,
                "{:?} has zero wind direction",
                biome,
            );
        }
    }

    #[test]
    fn wind_mountain_stronger_than_swamp() {
        let mountain = BiomeWindProfile::for_biome(BiomeType::Mountain);
        let swamp = BiomeWindProfile::for_biome(BiomeType::Swamp);
        assert!(mountain.base_strength > swamp.base_strength);
    }

    #[test]
    fn wind_desert_gusty() {
        let desert = BiomeWindProfile::for_biome(BiomeType::Desert);
        assert!(desert.gusty);
        assert!(desert.gust_variance > 1.0);
    }

    #[test]
    fn wind_swamp_calm() {
        let swamp = BiomeWindProfile::for_biome(BiomeType::Swamp);
        assert!(!swamp.gusty);
        assert!(swamp.base_strength < 0.5);
    }

    #[test]
    fn effective_strength_no_gust() {
        let swamp = BiomeWindProfile::for_biome(BiomeType::Swamp);
        // Non-gusty biome should always return base strength
        let s = swamp.effective_strength(100.0);
        assert!((s - swamp.base_strength).abs() < 0.001);
    }

    #[test]
    fn effective_strength_with_gust() {
        let mountain = BiomeWindProfile::for_biome(BiomeType::Mountain);
        // Sample multiple times — peak should exceed base
        let mut max_strength = 0.0_f32;
        for i in 0..100 {
            let s = mountain.effective_strength(i as f32 * 0.1);
            max_strength = max_strength.max(s);
        }
        assert!(
            max_strength > mountain.base_strength,
            "Gusty biome should have moments above base: {} vs {}",
            max_strength,
            mountain.base_strength,
        );
    }

    #[test]
    fn effective_strength_never_negative() {
        for biome in BiomeType::all() {
            let profile = BiomeWindProfile::for_biome(*biome);
            for i in 0..50 {
                let s = profile.effective_strength(i as f32 * 0.13);
                assert!(s >= 0.0, "{:?} at t={} has negative strength", biome, i);
            }
        }
    }

    #[test]
    fn effective_direction_normalized() {
        for biome in BiomeType::all() {
            let profile = BiomeWindProfile::for_biome(*biome);
            for t in [0.0, 1.0, 5.0, 10.0] {
                let dir = profile.effective_direction(t);
                assert!(
                    (dir.length() - 1.0).abs() < 0.01,
                    "{:?} direction not normalized at t={}: len={}",
                    biome,
                    t,
                    dir.length(),
                );
            }
        }
    }

    #[test]
    fn effective_direction_shifts_for_gusty() {
        let mountain = BiomeWindProfile::for_biome(BiomeType::Mountain);
        let d0 = mountain.effective_direction(0.0);
        let d1 = mountain.effective_direction(2.5); // Different phase
                                                    // Should be different unless by coincidence
                                                    // Check that at least some shift occurs across the full cycle
        let mut found_diff = false;
        for i in 0..20 {
            let d = mountain.effective_direction(i as f32 * 0.5);
            if (d - d0).length() > 0.01 {
                found_diff = true;
                break;
            }
        }
        assert!(found_diff, "Gusty wind direction should shift over time");
        let _ = d1; // suppress unused warning
    }

    #[test]
    fn effective_direction_stable_for_calm() {
        let swamp = BiomeWindProfile::for_biome(BiomeType::Swamp);
        let d0 = swamp.effective_direction(0.0);
        let d1 = swamp.effective_direction(10.0);
        assert!(
            (d0 - d1).length() < 0.001,
            "Non-gusty biome should have constant direction",
        );
    }

    // ── WeatherMultipliers tests ───────────────────────────────────────

    #[test]
    fn multipliers_for_none() {
        let m = WeatherMultipliers::for_kind(WeatherKind::None);
        assert!((m.fog - 1.0).abs() < 1e-6);
        assert!((m.ambient - 1.0).abs() < 1e-6);
        assert!((m.particle_density - 0.0).abs() < 1e-6);
    }

    #[test]
    fn multipliers_lerp_midpoint() {
        let a = WeatherMultipliers::for_kind(WeatherKind::None);
        let b = WeatherMultipliers::for_kind(WeatherKind::Rain);
        let mid = WeatherMultipliers::lerp(&a, &b, 0.5);
        assert!((mid.fog - 1.75).abs() < 0.01);
        assert!((mid.ambient - 0.8).abs() < 0.01);
    }

    #[test]
    fn multipliers_lerp_clamped() {
        let a = WeatherMultipliers::for_kind(WeatherKind::None);
        let b = WeatherMultipliers::for_kind(WeatherKind::Snow);
        let over = WeatherMultipliers::lerp(&a, &b, 2.0);
        let snow = WeatherMultipliers::for_kind(WeatherKind::Snow);
        assert!((over.fog - snow.fog).abs() < 0.01);
    }

    // ── Integration: WeatherTransition + BiomeWeatherMap ───────────────

    #[test]
    fn integration_pick_then_transition() {
        let current = WeatherKind::Rain;
        let next = BiomeWeatherMap::pick(BiomeType::Tundra, 0.5);
        assert_eq!(next, WeatherKind::Snow);

        let mut wt = WeatherTransition::new(2.0);
        wt.start(current, next);
        assert!(wt.is_active());

        // Run to completion
        for _ in 0..20 {
            wt.update(0.15);
        }
        assert!(!wt.is_active());
        let (fog, ambient) = wt.current_multipliers();
        let snow_muls = WeatherMultipliers::for_kind(WeatherKind::Snow);
        assert!((fog - snow_muls.fog).abs() < 0.01);
        assert!((ambient - snow_muls.ambient).abs() < 0.01);
    }

    #[test]
    fn integration_wind_profile_with_weather() {
        let biome = BiomeType::Mountain;
        let wind = BiomeWindProfile::for_biome(biome);
        let weather = BiomeWeatherMap::pick(biome, 0.4); // Should be Snow

        // At peak gust, effective strength should be substantial on a mountain
        let peak = wind.effective_strength(1.25); // Quarter cycle of ~0.4 Hz → peak
        assert!(peak > 1.5, "Mountain peak gust should be > 1.5: {peak}");

        // Weather should be appropriate for mountain
        let prob_snow = BiomeWeatherMap::probability(biome, WeatherKind::Snow);
        assert!(prob_snow > 0.2, "Mountains should have >20% snow chance");
        let _ = weather;
    }
}
