//! VFX descriptors — headless-safe specifications for all Veilweaver visual effects.
//!
//! Each descriptor is a pure data struct consumed by the presentation layer
//! (`astraweave-render`). The runtime produces these; the renderer reads them
//! each frame to drive GPU particle systems, shader parameters, and post-process.
//!
//! # Categories
//!
//! | Category | Descriptors |
//! |----------|-------------|
//! | Weaving thread | [`ThreadVfxSpec`] — spline-based thread glow |
//! | Anchor | [`AnchorVfxSpec`] — stabilization/repair aura |
//! | Echo | [`EchoBurstSpec`] — burst particles on collection |
//! | Boss telegraph | [`TelegraphVfxSpec`] — ground indicators for attacks |
//! | Boss phase | [`PhaseTransitionVfx`] — shockwave + color shift |
//! | Storm | [`StormVfxSpec`] — stabilize vs redirect variants |

use serde::Serialize;
use std::fmt;

// ── Color helpers ──────────────────────────────────────────────────────

/// RGBA color (0.0–1.0 per channel).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct VfxColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl VfxColor {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Linearly interpolate toward `other` by `t` (0.0–1.0).
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    // Veilweaver palette constants
    pub const THREAD_BLUE: Self = Self::rgb(0.2, 0.6, 1.0);
    pub const THREAD_GOLD: Self = Self::rgb(1.0, 0.84, 0.0);
    pub const ANCHOR_STABLE: Self = Self::rgb(0.15, 0.4, 0.7);
    pub const ANCHOR_CRITICAL: Self = Self::rgb(1.0, 0.2, 0.1);
    pub const ANCHOR_REPAIR: Self = Self::rgb(0.3, 0.9, 0.5);
    pub const ECHO_TEAL: Self = Self::rgb(0.0, 0.85, 0.75);
    pub const STORM_STABILIZE: Self = Self::rgb(0.3, 0.7, 1.0);
    pub const STORM_REDIRECT: Self = Self::rgb(1.0, 0.4, 0.2);
    pub const BOSS_ASSESS: Self = Self::rgb(0.3, 0.8, 0.4);
    pub const BOSS_FULCRUM: Self = Self::rgb(0.9, 0.7, 0.2);
    pub const BOSS_OVERRIDE: Self = Self::rgb(0.9, 0.2, 0.2);
    pub const TELEGRAPH_YELLOW: Self = Self::rgb(1.0, 0.9, 0.2);
    pub const TELEGRAPH_RED: Self = Self::rgb(1.0, 0.15, 0.15);
}

impl Default for VfxColor {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}

impl fmt::Display for VfxColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rgba({:.2}, {:.2}, {:.2}, {:.2})",
            self.r, self.g, self.b, self.a
        )
    }
}

// ── 3D Position ────────────────────────────────────────────────────────

/// Simple 3D position (avoids glam dependency in runtime).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3f {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const UP: Self = Self::new(0.0, 1.0, 0.0);
}

impl Default for Vec3f {
    fn default() -> Self {
        Self::ZERO
    }
}

// ── Weaving Thread VFX ─────────────────────────────────────────────────

/// Spline control point for a weaving thread.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct SplinePoint {
    pub position: Vec3f,
    /// Tangent direction at this point.
    pub tangent: Vec3f,
    /// Thread width at this point (meters).
    pub width: f32,
}

/// State of a weaving thread visual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ThreadState {
    /// Thread is stable and glowing steadily.
    Stable,
    /// Thread is being actively woven (pulsing glow).
    Weaving,
    /// Thread is under stress (flickering).
    Stressed,
    /// Thread is severed (particles dissipating).
    Severed,
}

impl std::fmt::Display for ThreadState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "Stable"),
            Self::Weaving => write!(f, "Weaving"),
            Self::Stressed => write!(f, "Stressed"),
            Self::Severed => write!(f, "Severed"),
        }
    }
}

/// VFX spec for a single weaving thread.
#[derive(Debug, Clone, Serialize)]
pub struct ThreadVfxSpec {
    /// Unique identifier for this thread.
    pub thread_id: String,
    /// Spline control points from anchor A to anchor B.
    pub spline: Vec<SplinePoint>,
    /// Current visual state.
    pub state: ThreadState,
    /// Base color (state-dependent).
    pub color: VfxColor,
    /// Emissive intensity (0.0–1.0).
    pub glow_intensity: f32,
    /// Pulse frequency in Hz (0.0 = no pulse).
    pub pulse_hz: f32,
    /// Particle emission rate along the thread (particles/sec).
    pub particle_rate: f32,
}

impl ThreadVfxSpec {
    /// Create a stable thread between two positions.
    pub fn stable(thread_id: impl Into<String>, start: Vec3f, end: Vec3f) -> Self {
        Self {
            thread_id: thread_id.into(),
            spline: vec![
                SplinePoint {
                    position: start,
                    tangent: Vec3f::UP,
                    width: 0.05,
                },
                SplinePoint {
                    position: end,
                    tangent: Vec3f::UP,
                    width: 0.05,
                },
            ],
            state: ThreadState::Stable,
            color: VfxColor::THREAD_BLUE,
            glow_intensity: 0.6,
            pulse_hz: 0.0,
            particle_rate: 5.0,
        }
    }

    /// Create an actively weaving thread.
    pub fn weaving(thread_id: impl Into<String>, start: Vec3f, end: Vec3f) -> Self {
        let mut spec = Self::stable(thread_id, start, end);
        spec.state = ThreadState::Weaving;
        spec.color = VfxColor::THREAD_GOLD;
        spec.glow_intensity = 1.0;
        spec.pulse_hz = 2.0;
        spec.particle_rate = 20.0;
        spec
    }
}

// ── Anchor VFX ─────────────────────────────────────────────────────────

/// Anchor visual state (maps to the WGSL shader's `vfx_state` uniform).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AnchorVfxState {
    /// Stability 0.9–1.0: Full blue glow, no distortion.
    Perfect,
    /// Stability 0.6–0.9: Dimmer blue glow.
    Stable,
    /// Stability 0.3–0.6: Yellow warning glow, slight distortion.
    Unstable,
    /// Stability 0.0–0.3: Red danger glow, heavy distortion + flicker.
    Critical,
    /// Stability 0.0 (destroyed): No glow, dead.
    Broken,
}

impl AnchorVfxState {
    /// Derive the visual state from a stability value (0.0–1.0).
    pub fn from_stability(stability: f32) -> Self {
        match stability {
            s if s >= 0.9 => Self::Perfect,
            s if s >= 0.6 => Self::Stable,
            s if s >= 0.3 => Self::Unstable,
            s if s > 0.0 => Self::Critical,
            _ => Self::Broken,
        }
    }

    /// WGSL shader state index (matches `anchor_vfx.wgsl` switch).
    pub fn shader_index(&self) -> u32 {
        match self {
            Self::Perfect => 0,
            Self::Stable => 1,
            Self::Unstable => 2,
            Self::Critical => 3,
            Self::Broken => 4,
        }
    }

    /// Base glow color for this state.
    pub fn color(&self) -> VfxColor {
        match self {
            Self::Perfect => VfxColor::rgb(0.2, 0.6, 1.0),
            Self::Stable => VfxColor::ANCHOR_STABLE,
            Self::Unstable => VfxColor::rgb(0.9, 0.7, 0.2),
            Self::Critical => VfxColor::ANCHOR_CRITICAL,
            Self::Broken => VfxColor::rgb(0.0, 0.0, 0.0),
        }
    }
}

impl std::fmt::Display for AnchorVfxState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Perfect => write!(f, "Perfect"),
            Self::Stable => write!(f, "Stable"),
            Self::Unstable => write!(f, "Unstable"),
            Self::Critical => write!(f, "Critical"),
            Self::Broken => write!(f, "Broken"),
        }
    }
}

/// VFX spec for an anchor's visual aura.
#[derive(Debug, Clone, Serialize)]
pub struct AnchorVfxSpec {
    /// Anchor identifier.
    pub anchor_id: String,
    /// World position of the anchor.
    pub position: Vec3f,
    /// Current stability (0.0–1.0).
    pub stability: f32,
    /// Derived visual state.
    pub vfx_state: AnchorVfxState,
    /// Whether the anchor is being repaired (triggers repair animation).
    pub is_repairing: bool,
    /// Time since repair started (seconds, 0.0 if not repairing).
    pub repair_time: f32,
    /// Proximity radius for the stabilization field (meters).
    pub field_radius: f32,
}

impl AnchorVfxSpec {
    /// Creates an anchor VFX spec from an anchor id, world position, and stability.
    #[must_use]
    pub fn new(anchor_id: impl Into<String>, position: Vec3f, stability: f32) -> Self {
        let state = AnchorVfxState::from_stability(stability);
        Self {
            anchor_id: anchor_id.into(),
            position,
            stability,
            vfx_state: state,
            is_repairing: false,
            repair_time: 0.0,
            field_radius: 5.0,
        }
    }

    /// Update stability and recompute visual state.
    pub fn set_stability(&mut self, stability: f32) {
        self.stability = stability.clamp(0.0, 1.0);
        self.vfx_state = AnchorVfxState::from_stability(self.stability);
    }

    /// Start the repair animation.
    pub fn begin_repair(&mut self) {
        self.is_repairing = true;
        self.repair_time = 0.0;
    }

    /// Advance repair animation timer.
    pub fn tick_repair(&mut self, dt: f32) {
        if self.is_repairing {
            self.repair_time += dt;
        }
    }
}

// ── Echo Collection VFX ────────────────────────────────────────────────

/// Shape of the echo burst particle emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BurstShape {
    /// Particles expand outward in a sphere.
    Spherical,
    /// Particles spiral upward.
    Spiral,
    /// Particles streak toward the player.
    Directional,
}

impl std::fmt::Display for BurstShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spherical => write!(f, "Spherical"),
            Self::Spiral => write!(f, "Spiral"),
            Self::Directional => write!(f, "Directional"),
        }
    }
}

/// VFX spec for echo collection burst particles.
#[derive(Debug, Clone, Serialize)]
pub struct EchoBurstSpec {
    /// World position of the echo pickup.
    pub position: Vec3f,
    /// Burst emission shape.
    pub shape: BurstShape,
    /// Particle count.
    pub particle_count: u32,
    /// Base color.
    pub color: VfxColor,
    /// Particle lifetime (seconds).
    pub lifetime: f32,
    /// Initial velocity magnitude (m/s).
    pub speed: f32,
    /// Scale of individual particles.
    pub particle_scale: f32,
}

impl EchoBurstSpec {
    /// Standard echo pickup burst.
    pub fn standard(position: Vec3f) -> Self {
        Self {
            position,
            shape: BurstShape::Spiral,
            particle_count: 24,
            color: VfxColor::ECHO_TEAL,
            lifetime: 1.2,
            speed: 3.0,
            particle_scale: 0.08,
        }
    }

    /// Large echo burst (for rare/valuable echoes).
    pub fn large(position: Vec3f) -> Self {
        Self {
            position,
            shape: BurstShape::Spherical,
            particle_count: 48,
            color: VfxColor::THREAD_GOLD,
            lifetime: 1.8,
            speed: 5.0,
            particle_scale: 0.12,
        }
    }
}

// ── Boss Telegraph VFX ─────────────────────────────────────────────────

/// Shape of a boss telegraph ground indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TelegraphShape {
    /// Circular area of effect.
    Circle,
    /// Conical/fan sweep.
    Cone,
    /// Linear charge path.
    Line,
    /// Chain connecting multiple targets.
    Chain,
}

impl std::fmt::Display for TelegraphShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Circle => write!(f, "Circle"),
            Self::Cone => write!(f, "Cone"),
            Self::Line => write!(f, "Line"),
            Self::Chain => write!(f, "Chain"),
        }
    }
}

/// VFX spec for a boss attack telegraph (ground warning indicator).
#[derive(Debug, Clone, Serialize)]
pub struct TelegraphVfxSpec {
    /// Name of the attack.
    pub attack_name: String,
    /// Center position.
    pub origin: Vec3f,
    /// Shape of the danger zone.
    pub shape: TelegraphShape,
    /// Radius (for Circle) or length (for Line/Cone) in meters.
    pub size: f32,
    /// Arc angle in radians (for Cone shape). Always TAU/4 for 90° sweep.
    pub arc_angle: f32,
    /// Direction vector (for Cone/Line).
    pub direction: Vec3f,
    /// Fill progress (0.0 = just appeared, 1.0 = about to fire).
    pub progress: f32,
    /// Base fill color (yellow at start, red near fire).
    pub color: VfxColor,
    /// Edge glow color.
    pub edge_color: VfxColor,
    /// Edge pulse frequency (Hz).
    pub edge_pulse_hz: f32,
}

impl TelegraphVfxSpec {
    /// Cleave telegraph — wide frontal cone.
    pub fn cleave(origin: Vec3f, direction: Vec3f) -> Self {
        Self {
            attack_name: "Oathbound Cleave".into(),
            origin,
            shape: TelegraphShape::Cone,
            size: 8.0,
            arc_angle: std::f32::consts::FRAC_PI_2, // 90°
            direction,
            progress: 0.0,
            color: VfxColor::TELEGRAPH_YELLOW,
            edge_color: VfxColor::TELEGRAPH_RED,
            edge_pulse_hz: 3.0,
        }
    }

    /// Chain Lash telegraph — linear charge.
    pub fn chain_lash(origin: Vec3f, target: Vec3f) -> Self {
        let dx = target.x - origin.x;
        let dy = target.y - origin.y;
        let dz = target.z - origin.z;
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(0.001);
        Self {
            attack_name: "Chain Lash".into(),
            origin,
            shape: TelegraphShape::Chain,
            size: len,
            arc_angle: 0.0,
            direction: Vec3f::new(dx / len, dy / len, dz / len),
            progress: 0.0,
            color: VfxColor::new(0.8, 0.4, 1.0, 0.6),
            edge_color: VfxColor::TELEGRAPH_RED,
            edge_pulse_hz: 5.0,
        }
    }

    /// Anchor Rupture telegraph — ground circle.
    pub fn anchor_rupture(position: Vec3f, radius: f32) -> Self {
        Self {
            attack_name: "Anchor Rupture".into(),
            origin: position,
            shape: TelegraphShape::Circle,
            size: radius,
            arc_angle: 0.0,
            direction: Vec3f::UP,
            progress: 0.0,
            color: VfxColor::new(0.7, 0.0, 0.0, 0.4),
            edge_color: VfxColor::ANCHOR_CRITICAL,
            edge_pulse_hz: 8.0,
        }
    }

    /// Update fill progress. Color interpolates yellow → red as progress increases.
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.color = VfxColor::TELEGRAPH_YELLOW.lerp(&VfxColor::TELEGRAPH_RED, self.progress);
        // Pulse frequency increases near detonation
        self.edge_pulse_hz = 3.0 + self.progress * 7.0; // 3–10 Hz
    }
}

// ── Boss Phase Transition VFX ──────────────────────────────────────────

/// VFX spec for a boss phase transition (shockwave + color shift).
#[derive(Debug, Clone, Serialize)]
pub struct PhaseTransitionVfx {
    /// Phase index (0→1, 1→2).
    pub from_phase: u32,
    pub to_phase: u32,
    /// Shockwave origin.
    pub origin: Vec3f,
    /// Shockwave radius (expands over time).
    pub shockwave_radius: f32,
    /// Maximum radius.
    pub max_radius: f32,
    /// Expansion speed (m/s).
    pub expansion_speed: f32,
    /// Color shifting from old phase to new.
    pub from_color: VfxColor,
    pub to_color: VfxColor,
    /// Current transition progress (0.0–1.0).
    pub progress: f32,
    /// Flash intensity (bright flash at trigger, decays).
    pub flash_intensity: f32,
    /// Screen shake intensity.
    pub screen_shake: f32,
}

impl PhaseTransitionVfx {
    /// Create a phase transition VFX.
    #[must_use]
    pub fn new(from_phase: u32, to_phase: u32, origin: Vec3f) -> Self {
        let from_color = match from_phase {
            0 => VfxColor::BOSS_ASSESS,
            1 => VfxColor::BOSS_FULCRUM,
            _ => VfxColor::BOSS_OVERRIDE,
        };
        let to_color = match to_phase {
            1 => VfxColor::BOSS_FULCRUM,
            2 => VfxColor::BOSS_OVERRIDE,
            _ => VfxColor::BOSS_ASSESS,
        };
        Self {
            from_phase,
            to_phase,
            origin,
            shockwave_radius: 0.0,
            max_radius: 25.0,
            expansion_speed: 20.0,
            from_color,
            to_color,
            progress: 0.0,
            flash_intensity: 1.0,
            screen_shake: 0.8,
        }
    }

    /// Advance the transition animation.
    pub fn tick(&mut self, dt: f32) {
        // Expand shockwave
        self.shockwave_radius =
            (self.shockwave_radius + self.expansion_speed * dt).min(self.max_radius);
        self.progress = if self.max_radius > 0.0 {
            self.shockwave_radius / self.max_radius
        } else {
            1.0
        };

        // Decay flash
        self.flash_intensity = (self.flash_intensity - dt * 3.0).max(0.0);

        // Decay screen shake
        self.screen_shake = (self.screen_shake - dt * 2.0).max(0.0);
    }

    /// Returns `true` when the transition animation is complete.
    pub fn is_complete(&self) -> bool {
        self.progress >= 1.0 && self.flash_intensity <= 0.0
    }

    /// Current interpolated color.
    pub fn current_color(&self) -> VfxColor {
        self.from_color.lerp(&self.to_color, self.progress)
    }
}

// ── Storm VFX ──────────────────────────────────────────────────────────

/// Storm visual variant based on player choice at Z3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StormVariant {
    /// Clear arena — stabilized threads, blue glow.
    Stabilized,
    /// Redirected storm — fog, orange energy, environmental hazards.
    Redirected,
}

impl std::fmt::Display for StormVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stabilized => write!(f, "Stabilized"),
            Self::Redirected => write!(f, "Redirected"),
        }
    }
}

/// VFX spec for the storm environment effect in the boss arena.
#[derive(Debug, Clone, Serialize)]
pub struct StormVfxSpec {
    /// Which variant the player chose.
    pub variant: StormVariant,
    /// Fog density (0.0 = clear, 1.0 = thick).
    pub fog_density: f32,
    /// Fog color.
    pub fog_color: VfxColor,
    /// Ambient light multiplier (lower = darker).
    pub ambient_multiplier: f32,
    /// Particle emission rate for environmental particles.
    pub particle_rate: f32,
    /// Particle color.
    pub particle_color: VfxColor,
    /// Wind strength for directional particle movement (m/s).
    pub wind_strength: f32,
    /// Wind direction (normalized XZ).
    pub wind_direction: Vec3f,
    /// Lightning flash frequency (flashes per minute, 0 = none).
    pub lightning_frequency: f32,
    /// Thread glow intensity in the arena.
    pub thread_glow: f32,
}

impl StormVfxSpec {
    /// Create the stabilized storm VFX (clear arena, blue glow).
    pub fn stabilized() -> Self {
        Self {
            variant: StormVariant::Stabilized,
            fog_density: 0.05,
            fog_color: VfxColor::new(0.6, 0.7, 0.9, 1.0),
            ambient_multiplier: 0.9,
            particle_rate: 10.0,
            particle_color: VfxColor::new(0.3, 0.7, 1.0, 0.3),
            wind_strength: 2.0,
            wind_direction: Vec3f::new(1.0, 0.0, 0.0),
            lightning_frequency: 0.0,
            thread_glow: 1.0,
        }
    }

    /// Create the redirected storm VFX (foggy arena, orange energy).
    pub fn redirected() -> Self {
        Self {
            variant: StormVariant::Redirected,
            fog_density: 0.6,
            fog_color: VfxColor::new(0.3, 0.2, 0.15, 1.0),
            ambient_multiplier: 0.4,
            particle_rate: 80.0,
            particle_color: VfxColor::new(1.0, 0.4, 0.1, 0.5),
            wind_strength: 12.0,
            wind_direction: Vec3f::new(0.7, 0.0, 0.7),
            lightning_frequency: 8.0,
            thread_glow: 0.3,
        }
    }

    /// Create the appropriate variant from the storm choice.
    pub fn from_choice(choice: &crate::storm_choice::StormChoice) -> Self {
        match choice {
            crate::storm_choice::StormChoice::Stabilize => Self::stabilized(),
            crate::storm_choice::StormChoice::Redirect => Self::redirected(),
        }
    }
}

// ── Active VFX Scene ───────────────────────────────────────────────────

/// Complete VFX state for the current frame.
///
/// The presentation layer reads this every frame to update all VFX systems.
#[derive(Debug, Clone, Default, Serialize)]
pub struct VfxScene {
    /// Active weaving threads.
    pub threads: Vec<ThreadVfxSpec>,
    /// Anchor auras.
    pub anchors: Vec<AnchorVfxSpec>,
    /// Pending echo burst effects (drain after spawning particles).
    pub echo_bursts: Vec<EchoBurstSpec>,
    /// Active boss telegraph indicators.
    pub telegraphs: Vec<TelegraphVfxSpec>,
    /// Active phase transition (if any).
    pub phase_transition: Option<PhaseTransitionVfx>,
    /// Storm environment effect (if in boss arena).
    pub storm: Option<StormVfxSpec>,
}

impl VfxScene {
    /// Creates an empty VFX scene with default layers.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Remove all expired effects after the renderer has consumed them.
    pub fn drain_bursts(&mut self) -> Vec<EchoBurstSpec> {
        std::mem::take(&mut self.echo_bursts)
    }

    /// Advance any ticking VFX (phase transitions, telegraphs).
    pub fn tick(&mut self, dt: f32) {
        // Tick phase transition
        if let Some(ref mut pt) = self.phase_transition {
            pt.tick(dt);
            if pt.is_complete() {
                self.phase_transition = None;
            }
        }

        // Tick anchor repair timers
        for anchor in &mut self.anchors {
            anchor.tick_repair(dt);
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vfx_color_lerp() {
        let a = VfxColor::rgb(0.0, 0.0, 0.0);
        let b = VfxColor::rgb(1.0, 1.0, 1.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.r - 0.5).abs() < 0.001);
        assert!((mid.g - 0.5).abs() < 0.001);
        assert!((mid.b - 0.5).abs() < 0.001);
    }

    #[test]
    fn vfx_color_lerp_clamped() {
        let a = VfxColor::rgb(0.0, 0.0, 0.0);
        let b = VfxColor::rgb(1.0, 1.0, 1.0);
        let over = a.lerp(&b, 2.0);
        assert!((over.r - 1.0).abs() < 0.001);
        let under = a.lerp(&b, -1.0);
        assert!((under.r - 0.0).abs() < 0.001);
    }

    #[test]
    fn vfx_color_display() {
        let c = VfxColor::rgb(1.0, 0.5, 0.0);
        assert!(format!("{}", c).contains("1.00"));
    }

    #[test]
    fn anchor_state_from_stability() {
        assert_eq!(AnchorVfxState::from_stability(1.0), AnchorVfxState::Perfect);
        assert_eq!(
            AnchorVfxState::from_stability(0.95),
            AnchorVfxState::Perfect
        );
        assert_eq!(AnchorVfxState::from_stability(0.7), AnchorVfxState::Stable);
        assert_eq!(
            AnchorVfxState::from_stability(0.4),
            AnchorVfxState::Unstable
        );
        assert_eq!(
            AnchorVfxState::from_stability(0.1),
            AnchorVfxState::Critical
        );
        assert_eq!(AnchorVfxState::from_stability(0.0), AnchorVfxState::Broken);
    }

    #[test]
    fn anchor_shader_index() {
        assert_eq!(AnchorVfxState::Perfect.shader_index(), 0);
        assert_eq!(AnchorVfxState::Broken.shader_index(), 4);
    }

    #[test]
    fn thread_stable_factory() {
        let spec = ThreadVfxSpec::stable(
            "thread_01",
            Vec3f::new(0.0, 0.0, 0.0),
            Vec3f::new(10.0, 0.0, 0.0),
        );
        assert_eq!(spec.state, ThreadState::Stable);
        assert_eq!(spec.spline.len(), 2);
        assert!(spec.pulse_hz == 0.0);
    }

    #[test]
    fn thread_weaving_factory() {
        let spec = ThreadVfxSpec::weaving(
            "thread_01",
            Vec3f::new(0.0, 0.0, 0.0),
            Vec3f::new(10.0, 0.0, 0.0),
        );
        assert_eq!(spec.state, ThreadState::Weaving);
        assert!(spec.pulse_hz > 0.0);
        assert!(spec.glow_intensity > 0.6);
    }

    #[test]
    fn echo_burst_standard() {
        let burst = EchoBurstSpec::standard(Vec3f::ZERO);
        assert_eq!(burst.shape, BurstShape::Spiral);
        assert_eq!(burst.particle_count, 24);
    }

    #[test]
    fn echo_burst_large() {
        let burst = EchoBurstSpec::large(Vec3f::ZERO);
        assert_eq!(burst.shape, BurstShape::Spherical);
        assert!(burst.particle_count > 24);
    }

    #[test]
    fn telegraph_cleave() {
        let spec = TelegraphVfxSpec::cleave(Vec3f::ZERO, Vec3f::new(1.0, 0.0, 0.0));
        assert_eq!(spec.shape, TelegraphShape::Cone);
        assert!(spec.arc_angle > 1.0);
    }

    #[test]
    fn telegraph_chain_lash() {
        let spec = TelegraphVfxSpec::chain_lash(Vec3f::ZERO, Vec3f::new(10.0, 0.0, 0.0));
        assert_eq!(spec.shape, TelegraphShape::Chain);
        assert!((spec.size - 10.0).abs() < 0.1);
    }

    #[test]
    fn telegraph_anchor_rupture() {
        let spec = TelegraphVfxSpec::anchor_rupture(Vec3f::ZERO, 6.0);
        assert_eq!(spec.shape, TelegraphShape::Circle);
        assert_eq!(spec.size, 6.0);
    }

    #[test]
    fn telegraph_progress_color_interpolation() {
        let mut spec = TelegraphVfxSpec::cleave(Vec3f::ZERO, Vec3f::new(1.0, 0.0, 0.0));
        spec.set_progress(0.0);
        let yellow = spec.color;
        spec.set_progress(1.0);
        let red = spec.color;
        // Red channel stays high, green decreases
        assert!(red.g < yellow.g);
    }

    #[test]
    fn phase_transition_ticks() {
        let mut vfx = PhaseTransitionVfx::new(0, 1, Vec3f::ZERO);
        assert!(!vfx.is_complete());
        // Tick enough to complete
        for _ in 0..200 {
            vfx.tick(1.0 / 60.0);
        }
        assert!(vfx.is_complete());
    }

    #[test]
    fn phase_transition_color() {
        let vfx = PhaseTransitionVfx::new(0, 1, Vec3f::ZERO);
        let start = vfx.current_color();
        assert!((start.r - VfxColor::BOSS_ASSESS.r).abs() < 0.01);
    }

    #[test]
    fn storm_stabilized() {
        let storm = StormVfxSpec::stabilized();
        assert_eq!(storm.variant, StormVariant::Stabilized);
        assert!(storm.fog_density < 0.1);
        assert!(storm.lightning_frequency == 0.0);
        assert!(storm.thread_glow > 0.5);
    }

    #[test]
    fn storm_redirected() {
        let storm = StormVfxSpec::redirected();
        assert_eq!(storm.variant, StormVariant::Redirected);
        assert!(storm.fog_density > 0.3);
        assert!(storm.lightning_frequency > 0.0);
        assert!(storm.wind_strength > 5.0);
    }

    #[test]
    fn storm_from_choice() {
        use crate::storm_choice::StormChoice;
        let stab = StormVfxSpec::from_choice(&StormChoice::Stabilize);
        assert_eq!(stab.variant, StormVariant::Stabilized);
        let redir = StormVfxSpec::from_choice(&StormChoice::Redirect);
        assert_eq!(redir.variant, StormVariant::Redirected);
    }

    #[test]
    fn anchor_vfx_repair() {
        let mut anchor = AnchorVfxSpec::new("anchor_01", Vec3f::ZERO, 0.5);
        assert_eq!(anchor.vfx_state, AnchorVfxState::Unstable);
        assert!(!anchor.is_repairing);

        anchor.begin_repair();
        assert!(anchor.is_repairing);

        anchor.tick_repair(1.0);
        assert!(anchor.repair_time > 0.0);

        anchor.set_stability(1.0);
        assert_eq!(anchor.vfx_state, AnchorVfxState::Perfect);
    }

    #[test]
    fn vfx_scene_tick_clears_completed_transition() {
        let mut scene = VfxScene::new();
        scene.phase_transition = Some(PhaseTransitionVfx::new(0, 1, Vec3f::ZERO));

        // Tick until complete
        for _ in 0..300 {
            scene.tick(1.0 / 60.0);
        }
        assert!(scene.phase_transition.is_none());
    }

    #[test]
    fn vfx_scene_drain_bursts() {
        let mut scene = VfxScene::new();
        scene.echo_bursts.push(EchoBurstSpec::standard(Vec3f::ZERO));
        scene
            .echo_bursts
            .push(EchoBurstSpec::large(Vec3f::new(5.0, 0.0, 0.0)));
        assert_eq!(scene.echo_bursts.len(), 2);

        let drained = scene.drain_bursts();
        assert_eq!(drained.len(), 2);
        assert!(scene.echo_bursts.is_empty());
    }

    #[test]
    fn vfx_color_default_is_opaque_black() {
        let c = VfxColor::default();
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn vec3f_default_is_zero() {
        let v = Vec3f::default();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
        assert_eq!(v.z, 0.0);
    }
}
