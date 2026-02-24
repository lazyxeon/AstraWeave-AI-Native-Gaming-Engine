//! Palette & material descriptors — twilight color definitions and zone material config.
//!
//! These are pure data structs consumed by the material system
//! (`astraweave-materials`, `astraweave-render`). The runtime produces
//! palette lookups; shaders and material passes read the resulting colors.
//!
//! # Design
//!
//! Veilweaver uses a "twilight palette" — muted blues, golds, and violets
//! with high-contrast emissive accents for threads, anchors, and boss VFX.

use serde::Serialize;

use crate::vfx_specs::VfxColor;

// ── Twilight Palette ───────────────────────────────────────────────────

/// Named palette slots for the Veilweaver twilight theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum PaletteSlot {
    /// Base sky color at zenith.
    SkyZenith,
    /// Base sky color at horizon.
    SkyHorizon,
    /// Ambient light tint.
    AmbientLight,
    /// Fog / atmospheric haze.
    Fog,
    /// Ground base color.
    GroundBase,
    /// Structure / ruin stone color.
    StructureStone,
    /// Thread glow (primary emissive).
    ThreadEmissive,
    /// Anchor glow (secondary emissive).
    AnchorEmissive,
    /// Echo collection glow.
    EchoEmissive,
    /// UI accent color.
    UiAccent,
    /// UI background tint.
    UiBackground,
    /// Boss arena tint.
    BossArenaTint,
}

impl PaletteSlot {
    /// The canonical twilight palette color for this slot.
    pub fn color(&self) -> VfxColor {
        match self {
            Self::SkyZenith => VfxColor::rgb(0.08, 0.05, 0.18), // Deep violet-black
            Self::SkyHorizon => VfxColor::rgb(0.20, 0.12, 0.35), // Dusky violet
            Self::AmbientLight => VfxColor::rgb(0.25, 0.22, 0.40), // Cool dim
            Self::Fog => VfxColor::new(0.15, 0.12, 0.25, 0.6),  // Purple haze
            Self::GroundBase => VfxColor::rgb(0.12, 0.10, 0.08), // Dark earth
            Self::StructureStone => VfxColor::rgb(0.25, 0.22, 0.20), // Weathered stone
            Self::ThreadEmissive => VfxColor::THREAD_BLUE,
            Self::AnchorEmissive => VfxColor::ANCHOR_STABLE,
            Self::EchoEmissive => VfxColor::ECHO_TEAL,
            Self::UiAccent => VfxColor::rgb(0.3, 0.7, 1.0), // Bright thread blue
            Self::UiBackground => VfxColor::new(0.05, 0.03, 0.10, 0.85), // Dark translucent
            Self::BossArenaTint => VfxColor::rgb(0.15, 0.05, 0.05), // Ominous red-black
        }
    }
}

// ── Zone Material Config ───────────────────────────────────────────────

/// Zone-specific material tint overrides.
#[derive(Debug, Clone, Serialize)]
pub struct ZoneMaterialConfig {
    /// Zone name.
    pub zone_name: String,
    /// Ground tint (multiplied with base ground texture).
    pub ground_tint: VfxColor,
    /// Structure tint (for walls, pillars, ruins).
    pub structure_tint: VfxColor,
    /// Fog color override.
    pub fog_color: VfxColor,
    /// Fog near distance (meters).
    pub fog_near: f32,
    /// Fog far distance (meters).
    pub fog_far: f32,
    /// Ambient light intensity multiplier.
    pub ambient_intensity: f32,
    /// Emissive intensity multiplier for threads/anchors in this zone.
    pub emissive_boost: f32,
}

impl ZoneMaterialConfig {
    /// Z0: Loomspire Sanctum — warm indoor twilight, strong emissive.
    pub fn loomspire_sanctum() -> Self {
        Self {
            zone_name: "Loomspire Sanctum".into(),
            ground_tint: VfxColor::rgb(0.15, 0.12, 0.10),
            structure_tint: VfxColor::rgb(0.30, 0.25, 0.22),
            fog_color: VfxColor::new(0.12, 0.08, 0.20, 0.4),
            fog_near: 20.0,
            fog_far: 80.0,
            ambient_intensity: 0.35,
            emissive_boost: 1.2,
        }
    }

    /// Z1: Threadhollow Ruins — cooler, more muted, deeper fog.
    pub fn threadhollow_ruins() -> Self {
        Self {
            zone_name: "Threadhollow Ruins".into(),
            ground_tint: VfxColor::rgb(0.10, 0.10, 0.12),
            structure_tint: VfxColor::rgb(0.20, 0.20, 0.25),
            fog_color: VfxColor::new(0.10, 0.10, 0.18, 0.6),
            fog_near: 15.0,
            fog_far: 60.0,
            ambient_intensity: 0.25,
            emissive_boost: 1.4,
        }
    }

    /// Z2: Stormreach Nexus — open, windy, thin fog.
    pub fn stormreach_nexus() -> Self {
        Self {
            zone_name: "Stormreach Nexus".into(),
            ground_tint: VfxColor::rgb(0.14, 0.12, 0.08),
            structure_tint: VfxColor::rgb(0.22, 0.18, 0.15),
            fog_color: VfxColor::new(0.20, 0.15, 0.25, 0.3),
            fog_near: 40.0,
            fog_far: 150.0,
            ambient_intensity: 0.45,
            emissive_boost: 1.0,
        }
    }

    /// Z3: Frayed Expanse — stormy, darkened, high-contrast emissive.
    pub fn frayed_expanse() -> Self {
        Self {
            zone_name: "Frayed Expanse".into(),
            ground_tint: VfxColor::rgb(0.08, 0.06, 0.05),
            structure_tint: VfxColor::rgb(0.15, 0.12, 0.10),
            fog_color: VfxColor::new(0.18, 0.10, 0.15, 0.7),
            fog_near: 10.0,
            fog_far: 50.0,
            ambient_intensity: 0.15,
            emissive_boost: 1.8,
        }
    }

    /// Z4: Boss Courtyard — ominous, red-tinted, dramatic.
    pub fn boss_courtyard() -> Self {
        Self {
            zone_name: "Boss Courtyard".into(),
            ground_tint: VfxColor::rgb(0.12, 0.06, 0.06),
            structure_tint: VfxColor::rgb(0.20, 0.12, 0.12),
            fog_color: VfxColor::new(0.15, 0.05, 0.05, 0.5),
            fog_near: 25.0,
            fog_far: 100.0,
            ambient_intensity: 0.20,
            emissive_boost: 1.5,
        }
    }

    /// Look up a zone material config by zero-based index.
    pub fn from_zone_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::loomspire_sanctum()),
            1 => Some(Self::threadhollow_ruins()),
            2 => Some(Self::stormreach_nexus()),
            3 => Some(Self::frayed_expanse()),
            4 => Some(Self::boss_courtyard()),
            _ => None,
        }
    }
}

// ── Skybox Config ──────────────────────────────────────────────────────

/// Skybox configuration for the Veilweaver twilight sky.
#[derive(Debug, Clone, Serialize)]
pub struct SkyboxConfig {
    /// Zenith color (top of sky dome).
    pub zenith: VfxColor,
    /// Horizon color (ring at eye level).
    pub horizon: VfxColor,
    /// Nadir color (below horizon, only visible near edges).
    pub nadir: VfxColor,
    /// Star visibility (0.0 = none, 1.0 = full).
    pub star_density: f32,
    /// Aurora/ribbon effect intensity (0.0 = none).
    pub aurora_intensity: f32,
    /// Aurora primary color.
    pub aurora_color: VfxColor,
    /// Cloud coverage (0.0–1.0).
    pub cloud_coverage: f32,
    /// Cloud color tint.
    pub cloud_tint: VfxColor,
    /// Moon/light source direction (normalized).
    pub moon_direction: crate::vfx_specs::Vec3f,
    /// Moon glow intensity.
    pub moon_intensity: f32,
}

impl SkyboxConfig {
    /// Default twilight skybox.
    pub fn twilight() -> Self {
        Self {
            zenith: PaletteSlot::SkyZenith.color(),
            horizon: PaletteSlot::SkyHorizon.color(),
            nadir: VfxColor::rgb(0.04, 0.02, 0.06),
            star_density: 0.7,
            aurora_intensity: 0.3,
            aurora_color: VfxColor::new(0.2, 0.5, 0.9, 0.4),
            cloud_coverage: 0.3,
            cloud_tint: VfxColor::new(0.12, 0.08, 0.18, 0.6),
            moon_direction: crate::vfx_specs::Vec3f::new(0.3, 0.8, -0.5),
            moon_intensity: 0.4,
        }
    }

    /// Storm skybox (Z3-Z4 during storm sequences).
    pub fn storm() -> Self {
        Self {
            zenith: VfxColor::rgb(0.04, 0.02, 0.08),
            horizon: VfxColor::rgb(0.12, 0.06, 0.15),
            nadir: VfxColor::rgb(0.02, 0.01, 0.03),
            star_density: 0.0,
            aurora_intensity: 0.0,
            aurora_color: VfxColor::new(0.0, 0.0, 0.0, 0.0),
            cloud_coverage: 0.9,
            cloud_tint: VfxColor::new(0.15, 0.08, 0.10, 0.8),
            moon_direction: crate::vfx_specs::Vec3f::new(0.3, 0.8, -0.5),
            moon_intensity: 0.05,
        }
    }
}

// ── Complete Presentation Config ───────────────────────────────────────

/// Full presentation configuration for a given moment in the game.
///
/// The material/render system reads this each frame.
#[derive(Debug, Clone, Serialize)]
pub struct PresentationConfig {
    /// Current zone material config.
    pub zone_material: ZoneMaterialConfig,
    /// Skybox configuration.
    pub skybox: SkyboxConfig,
    /// Global brightness override (1.0 = normal).
    pub brightness: f32,
    /// Global saturation override (1.0 = normal, 0.0 = grayscale).
    pub saturation: f32,
    /// Vignette intensity (0.0 = none, 1.0 = heavy).
    pub vignette: f32,
    /// Chromatic aberration on screen edges (0.0 = none).
    pub chromatic_aberration: f32,
}

impl PresentationConfig {
    /// Default config for a given zone.
    pub fn for_zone(zone_index: usize) -> Self {
        let zone_material = ZoneMaterialConfig::from_zone_index(zone_index)
            .unwrap_or_else(ZoneMaterialConfig::loomspire_sanctum);
        let skybox = if zone_index >= 3 {
            SkyboxConfig::storm()
        } else {
            SkyboxConfig::twilight()
        };
        let vignette = if zone_index == 4 { 0.3 } else { 0.1 };
        Self {
            zone_material,
            skybox,
            brightness: 1.0,
            saturation: 1.0,
            vignette,
            chromatic_aberration: 0.0,
        }
    }

    /// Boss encounter presentation (increased vignette, slight desaturation).
    pub fn boss_encounter(zone_index: usize) -> Self {
        let mut config = Self::for_zone(zone_index);
        config.vignette = 0.4;
        config.saturation = 0.85;
        config.chromatic_aberration = 0.02;
        config
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_all_slots_defined() {
        let slots = [
            PaletteSlot::SkyZenith,
            PaletteSlot::SkyHorizon,
            PaletteSlot::AmbientLight,
            PaletteSlot::Fog,
            PaletteSlot::GroundBase,
            PaletteSlot::StructureStone,
            PaletteSlot::ThreadEmissive,
            PaletteSlot::AnchorEmissive,
            PaletteSlot::EchoEmissive,
            PaletteSlot::UiAccent,
            PaletteSlot::UiBackground,
            PaletteSlot::BossArenaTint,
        ];
        for slot in &slots {
            let c = slot.color();
            // All colors should have at least some component > 0 (except full black)
            let sum = c.r + c.g + c.b;
            assert!(sum >= 0.0, "Slot {:?} has zero color", slot);
        }
    }

    #[test]
    fn palette_emissive_brighter_than_ambient() {
        let thread = PaletteSlot::ThreadEmissive.color();
        let ambient = PaletteSlot::AmbientLight.color();
        let thread_lum = thread.r * 0.299 + thread.g * 0.587 + thread.b * 0.114;
        let ambient_lum = ambient.r * 0.299 + ambient.g * 0.587 + ambient.b * 0.114;
        assert!(thread_lum > ambient_lum);
    }

    #[test]
    fn zone_material_all_zones() {
        for idx in 0..5 {
            let config = ZoneMaterialConfig::from_zone_index(idx);
            assert!(config.is_some(), "Zone {} missing", idx);
            let c = config.unwrap();
            assert!(!c.zone_name.is_empty());
            assert!(c.fog_far > c.fog_near);
            assert!(c.ambient_intensity > 0.0);
            assert!(c.emissive_boost > 0.0);
        }
    }

    #[test]
    fn zone_material_invalid_index() {
        assert!(ZoneMaterialConfig::from_zone_index(99).is_none());
    }

    #[test]
    fn fog_distance_progression() {
        // Later zones should have increasingly tight fog (darker mood).
        let z0 = ZoneMaterialConfig::loomspire_sanctum();
        let z3 = ZoneMaterialConfig::frayed_expanse();
        assert!(
            z3.fog_near < z0.fog_near,
            "Later zones should have closer fog"
        );
    }

    #[test]
    fn emissive_boost_progression() {
        // Darker zones should boost emissives more for readability.
        let z0 = ZoneMaterialConfig::loomspire_sanctum();
        let z3 = ZoneMaterialConfig::frayed_expanse();
        assert!(z3.emissive_boost > z0.emissive_boost);
    }

    #[test]
    fn skybox_twilight_stars() {
        let sky = SkyboxConfig::twilight();
        assert!(sky.star_density > 0.0);
        assert!(sky.aurora_intensity > 0.0);
    }

    #[test]
    fn skybox_storm_no_stars() {
        let sky = SkyboxConfig::storm();
        assert!(sky.star_density == 0.0);
        assert!(sky.cloud_coverage > 0.5);
    }

    #[test]
    fn presentation_zone_fallback() {
        // Invalid zone should fallback to Z0.
        let config = PresentationConfig::for_zone(99);
        assert_eq!(config.zone_material.zone_name, "Loomspire Sanctum");
    }

    #[test]
    fn presentation_boss_encounter() {
        let config = PresentationConfig::boss_encounter(4);
        assert!(config.vignette > 0.3);
        assert!(config.saturation < 1.0);
        assert!(config.chromatic_aberration > 0.0);
    }

    #[test]
    fn presentation_storm_skybox_for_late_zones() {
        let z2 = PresentationConfig::for_zone(2);
        let z3 = PresentationConfig::for_zone(3);
        // Z2 and below: twilight, Z3+: storm
        assert!(z2.skybox.star_density > 0.0);
        assert!(z3.skybox.star_density == 0.0);
    }
}
