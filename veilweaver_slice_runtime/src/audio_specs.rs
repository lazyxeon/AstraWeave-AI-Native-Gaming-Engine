//! Audio descriptors — headless-safe specifications for Veilweaver sound design.
//!
//! Each descriptor is a pure data struct consumed by the audio engine
//! (`astraweave-audio`). The runtime produces these; the audio system reads
//! them to drive music crossfades, spatial SFX, and UI stingers.
//!
//! # Categories
//!
//! | Category | Descriptors |
//! |----------|-------------|
//! | Ambience | [`ZoneAmbienceSpec`] — per-zone ambient loop |
//! | Music | [`BossMusicSpec`] — adaptive boss music layers |
//! | Stinger | [`UiStingerSpec`] — one-shot feedback sounds |
//! | SFX | [`SpatialSfxSpec`] — 3D positional sound effects |

use serde::Serialize;

use crate::vfx_specs::Vec3f;

// ── Zone Ambience ──────────────────────────────────────────────────────

/// Identifier for a zone's ambient audio setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ZoneAmbienceId {
    LoomspireSanctum,
    ThreadhollowRuins,
    StormreachNexus,
    FrayedExpanse,
    BossCourtyard,
}

impl ZoneAmbienceId {
    /// The asset path for this zone's ambient loop.
    pub fn loop_path(&self) -> &'static str {
        match self {
            Self::LoomspireSanctum => "audio/ambience/loomspire_sanctum.ogg",
            Self::ThreadhollowRuins => "audio/ambience/threadhollow_ruins.ogg",
            Self::StormreachNexus => "audio/ambience/stormreach_nexus.ogg",
            Self::FrayedExpanse => "audio/ambience/frayed_expanse.ogg",
            Self::BossCourtyard => "audio/ambience/boss_courtyard.ogg",
        }
    }

    /// From zero-based zone index.
    pub fn from_zone_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::LoomspireSanctum),
            1 => Some(Self::ThreadhollowRuins),
            2 => Some(Self::StormreachNexus),
            3 => Some(Self::FrayedExpanse),
            4 => Some(Self::BossCourtyard),
            _ => None,
        }
    }
}

impl std::fmt::Display for ZoneAmbienceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoomspireSanctum => write!(f, "Loomspire Sanctum"),
            Self::ThreadhollowRuins => write!(f, "Threadhollow Ruins"),
            Self::StormreachNexus => write!(f, "Stormreach Nexus"),
            Self::FrayedExpanse => write!(f, "Frayed Expanse"),
            Self::BossCourtyard => write!(f, "Boss Courtyard"),
        }
    }
}

/// VFX spec for zone ambient audio.
#[derive(Debug, Clone, Serialize)]
pub struct ZoneAmbienceSpec {
    /// Which zone this ambience belongs to.
    pub zone: ZoneAmbienceId,
    /// Target volume (0.0–1.0).
    pub volume: f32,
    /// Crossfade duration when transitioning to this ambience (seconds).
    pub crossfade_sec: f32,
    /// Whether the ambience loops.
    pub looping: bool,
    /// Optional reverb amount (0.0–1.0, for caves/ruins = higher).
    pub reverb: f32,
    /// Low-pass filter cutoff hint (Hz, lower = more muffled, 0 = no filter).
    pub lowpass_hz: f32,
}

impl ZoneAmbienceSpec {
    /// Create a default ambience spec for a zone.
    pub fn for_zone(zone: ZoneAmbienceId) -> Self {
        let (reverb, lowpass_hz) = match zone {
            ZoneAmbienceId::LoomspireSanctum => (0.4, 0.0), // Indoor, reverberant
            ZoneAmbienceId::ThreadhollowRuins => (0.6, 3000.0), // Ruins, muffled
            ZoneAmbienceId::StormreachNexus => (0.1, 0.0),  // Open nexus
            ZoneAmbienceId::FrayedExpanse => (0.05, 0.0),   // Wide open
            ZoneAmbienceId::BossCourtyard => (0.3, 0.0),    // Semi-open courtyard
        };
        Self {
            zone,
            volume: 0.5,
            crossfade_sec: 2.0,
            looping: true,
            reverb,
            lowpass_hz,
        }
    }
}

// ── Boss Adaptive Music ────────────────────────────────────────────────

/// Music intensity layer for the boss encounter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum BossMusicLayer {
    /// Calm exploration before boss aggro.
    Ambient,
    /// Phase 1: Assessment — building tension.
    Assessment,
    /// Phase 2: Fulcrum Shift — faster tempo, more dissonant.
    FulcrumShift,
    /// Phase 3: Directive Override — full intensity, choir.
    DirectiveOverride,
    /// Victory fanfare.
    Victory,
    /// Defeat / wipe.
    Defeat,
}

impl BossMusicLayer {
    /// The asset path for this music layer.
    pub fn track_path(&self) -> &'static str {
        match self {
            Self::Ambient => "audio/music/boss_ambient.ogg",
            Self::Assessment => "audio/music/boss_assessment.ogg",
            Self::FulcrumShift => "audio/music/boss_fulcrum_shift.ogg",
            Self::DirectiveOverride => "audio/music/boss_directive_override.ogg",
            Self::Victory => "audio/music/boss_victory.ogg",
            Self::Defeat => "audio/music/boss_defeat.ogg",
        }
    }
}

impl std::fmt::Display for BossMusicLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ambient => write!(f, "Ambient"),
            Self::Assessment => write!(f, "Assessment"),
            Self::FulcrumShift => write!(f, "Fulcrum Shift"),
            Self::DirectiveOverride => write!(f, "Directive Override"),
            Self::Victory => write!(f, "Victory"),
            Self::Defeat => write!(f, "Defeat"),
        }
    }
}

/// VFX spec for boss adaptive music.
#[derive(Debug, Clone, Serialize)]
pub struct BossMusicSpec {
    /// Current active layer.
    pub layer: BossMusicLayer,
    /// Target volume (0.0–1.0).
    pub volume: f32,
    /// Crossfade duration when switching layers (seconds).
    pub crossfade_sec: f32,
    /// Whether the layer loops.
    pub looping: bool,
    /// Optional tempo multiplier (1.0 = normal, higher = faster).
    pub tempo_multiplier: f32,
}

impl BossMusicSpec {
    /// Create a music spec for a given layer.
    pub fn for_layer(layer: BossMusicLayer) -> Self {
        let (crossfade, tempo) = match layer {
            BossMusicLayer::Ambient => (3.0, 1.0),
            BossMusicLayer::Assessment => (2.0, 1.0),
            BossMusicLayer::FulcrumShift => (1.5, 1.1),
            BossMusicLayer::DirectiveOverride => (1.0, 1.2),
            BossMusicLayer::Victory => (0.5, 1.0),
            BossMusicLayer::Defeat => (0.5, 0.8),
        };
        let looping = !matches!(layer, BossMusicLayer::Victory | BossMusicLayer::Defeat);
        Self {
            layer,
            volume: 0.7,
            crossfade_sec: crossfade,
            looping,
            tempo_multiplier: tempo,
        }
    }
}

// ── UI Stingers ────────────────────────────────────────────────────────

/// One-shot UI feedback sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum StingerKind {
    /// Successful weaving completed.
    WeavingSuccess,
    /// Weaving failed / thread severed.
    WeavingFailure,
    /// New ability unlocked.
    AbilityUnlock,
    /// Companion affinity rank increased.
    AffinityRankUp,
    /// Echo collected.
    EchoCollected,
    /// Anchor stabilized.
    AnchorStabilized,
    /// Anchor lost (broken).
    AnchorLost,
    /// Boss phase transition.
    PhaseTransition,
    /// Decision prompt appeared.
    DecisionPrompt,
    /// Storm choice made.
    StormChoiceMade,
    /// Boss defeated.
    BossDefeated,
}

impl StingerKind {
    /// The asset path for this stinger.
    pub fn sound_path(&self) -> &'static str {
        match self {
            Self::WeavingSuccess => "audio/stinger/weaving_success.ogg",
            Self::WeavingFailure => "audio/stinger/weaving_failure.ogg",
            Self::AbilityUnlock => "audio/stinger/ability_unlock.ogg",
            Self::AffinityRankUp => "audio/stinger/affinity_rank_up.ogg",
            Self::EchoCollected => "audio/stinger/echo_collected.ogg",
            Self::AnchorStabilized => "audio/stinger/anchor_stabilized.ogg",
            Self::AnchorLost => "audio/stinger/anchor_lost.ogg",
            Self::PhaseTransition => "audio/stinger/phase_transition.ogg",
            Self::DecisionPrompt => "audio/stinger/decision_prompt.ogg",
            Self::StormChoiceMade => "audio/stinger/storm_choice_made.ogg",
            Self::BossDefeated => "audio/stinger/boss_defeated.ogg",
        }
    }
}

impl std::fmt::Display for StingerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WeavingSuccess => write!(f, "Weaving Success"),
            Self::WeavingFailure => write!(f, "Weaving Failure"),
            Self::AbilityUnlock => write!(f, "Ability Unlock"),
            Self::AffinityRankUp => write!(f, "Affinity Rank Up"),
            Self::EchoCollected => write!(f, "Echo Collected"),
            Self::AnchorStabilized => write!(f, "Anchor Stabilized"),
            Self::AnchorLost => write!(f, "Anchor Lost"),
            Self::PhaseTransition => write!(f, "Phase Transition"),
            Self::DecisionPrompt => write!(f, "Decision Prompt"),
            Self::StormChoiceMade => write!(f, "Storm Choice Made"),
            Self::BossDefeated => write!(f, "Boss Defeated"),
        }
    }
}

/// VFX spec for a UI stinger.
#[derive(Debug, Clone, Serialize)]
pub struct UiStingerSpec {
    /// Which stinger to play.
    pub kind: StingerKind,
    /// Volume (0.0–1.0).
    pub volume: f32,
    /// Playback pitch multiplier (1.0 = normal).
    pub pitch: f32,
    /// Optional delay before playing (seconds).
    pub delay_sec: f32,
}

impl UiStingerSpec {
    /// Create a stinger with default settings.
    #[must_use]
    pub fn new(kind: StingerKind) -> Self {
        Self {
            kind,
            volume: 0.8,
            pitch: 1.0,
            delay_sec: 0.0,
        }
    }

    /// Create a stinger with custom volume.
    pub fn with_volume(kind: StingerKind, volume: f32) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
            ..Self::new(kind)
        }
    }
}

// ── Spatial SFX ────────────────────────────────────────────────────────

/// One-shot or looping spatial (3D) sound effect.
#[derive(Debug, Clone, Serialize)]
pub struct SpatialSfxSpec {
    /// Sound effect name.
    pub name: String,
    /// Asset path.
    pub path: String,
    /// Emitter world position.
    pub position: Vec3f,
    /// Volume (0.0–1.0).
    pub volume: f32,
    /// Maximum audible distance (meters).
    pub max_distance: f32,
    /// Whether the SFX loops.
    pub looping: bool,
    /// Pitch multiplier.
    pub pitch: f32,
}

impl SpatialSfxSpec {
    /// Anchor humming loop.
    pub fn anchor_hum(position: Vec3f) -> Self {
        Self {
            name: "anchor_hum".into(),
            path: "audio/sfx/anchor_hum.ogg".into(),
            position,
            volume: 0.3,
            max_distance: 15.0,
            looping: true,
            pitch: 1.0,
        }
    }

    /// Thread weaving sound.
    pub fn thread_weave(position: Vec3f) -> Self {
        Self {
            name: "thread_weave".into(),
            path: "audio/sfx/thread_weave.ogg".into(),
            position,
            volume: 0.5,
            max_distance: 10.0,
            looping: false,
            pitch: 1.0,
        }
    }

    /// Boss impact sound.
    pub fn boss_impact(position: Vec3f) -> Self {
        Self {
            name: "boss_impact".into(),
            path: "audio/sfx/boss_impact.ogg".into(),
            position,
            volume: 0.9,
            max_distance: 30.0,
            looping: false,
            pitch: 1.0,
        }
    }

    /// Storm ambient crackle.
    pub fn storm_crackle(position: Vec3f) -> Self {
        Self {
            name: "storm_crackle".into(),
            path: "audio/sfx/storm_crackle.ogg".into(),
            position,
            volume: 0.4,
            max_distance: 25.0,
            looping: true,
            pitch: 1.0,
        }
    }
}

// ── Audio Scene ────────────────────────────────────────────────────────

/// Complete audio state for the current frame.
///
/// The audio engine reads this to update all audio systems.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AudioScene {
    /// Active zone ambience (crossfades between zones automatically).
    pub ambience: Option<ZoneAmbienceSpec>,
    /// Boss music (if in boss encounter).
    pub boss_music: Option<BossMusicSpec>,
    /// Pending stingers (drain after queuing to audio engine).
    pub pending_stingers: Vec<UiStingerSpec>,
    /// Active spatial SFX emitters.
    pub spatial_sfx: Vec<SpatialSfxSpec>,
}

impl AudioScene {
    /// Creates an empty audio scene with default layers.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a stinger for playback.
    pub fn queue_stinger(&mut self, stinger: UiStingerSpec) {
        self.pending_stingers.push(stinger);
    }

    /// Drain pending stingers after the audio engine has processed them.
    pub fn drain_stingers(&mut self) -> Vec<UiStingerSpec> {
        std::mem::take(&mut self.pending_stingers)
    }

    /// Set the zone ambience, creating a crossfade if switching zones.
    pub fn set_zone(&mut self, zone: ZoneAmbienceId) {
        let should_update = self.ambience.as_ref().is_none_or(|a| a.zone != zone);
        if should_update {
            self.ambience = Some(ZoneAmbienceSpec::for_zone(zone));
        }
    }

    /// Set the boss music layer.
    pub fn set_boss_layer(&mut self, layer: BossMusicLayer) {
        let should_update = self.boss_music.as_ref().is_none_or(|m| m.layer != layer);
        if should_update {
            self.boss_music = Some(BossMusicSpec::for_layer(layer));
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_ambience_from_index() {
        assert_eq!(
            ZoneAmbienceId::from_zone_index(0),
            Some(ZoneAmbienceId::LoomspireSanctum)
        );
        assert_eq!(
            ZoneAmbienceId::from_zone_index(4),
            Some(ZoneAmbienceId::BossCourtyard)
        );
        assert_eq!(ZoneAmbienceId::from_zone_index(99), None);
    }

    #[test]
    fn zone_ambience_paths() {
        for idx in 0..5 {
            let zone = ZoneAmbienceId::from_zone_index(idx).unwrap();
            assert!(zone.loop_path().ends_with(".ogg"));
        }
    }

    #[test]
    fn zone_ambience_spec_defaults() {
        let spec = ZoneAmbienceSpec::for_zone(ZoneAmbienceId::LoomspireSanctum);
        assert!(spec.looping);
        assert!(spec.volume > 0.0);
        assert!(spec.crossfade_sec > 0.0);
        assert!(spec.reverb > 0.0); // indoor = reverberant
    }

    #[test]
    fn zone_ambience_ruins_muffled() {
        let spec = ZoneAmbienceSpec::for_zone(ZoneAmbienceId::ThreadhollowRuins);
        assert!(spec.lowpass_hz > 0.0);
        assert!(spec.reverb > 0.3);
    }

    #[test]
    fn boss_music_layer_paths() {
        for layer in [
            BossMusicLayer::Ambient,
            BossMusicLayer::Assessment,
            BossMusicLayer::FulcrumShift,
            BossMusicLayer::DirectiveOverride,
            BossMusicLayer::Victory,
            BossMusicLayer::Defeat,
        ] {
            assert!(layer.track_path().ends_with(".ogg"));
        }
    }

    #[test]
    fn boss_music_victory_not_looping() {
        let spec = BossMusicSpec::for_layer(BossMusicLayer::Victory);
        assert!(!spec.looping);
    }

    #[test]
    fn boss_music_assessment_looping() {
        let spec = BossMusicSpec::for_layer(BossMusicLayer::Assessment);
        assert!(spec.looping);
    }

    #[test]
    fn boss_music_tempo_escalation() {
        let assess = BossMusicSpec::for_layer(BossMusicLayer::Assessment);
        let fulcrum = BossMusicSpec::for_layer(BossMusicLayer::FulcrumShift);
        let directive = BossMusicSpec::for_layer(BossMusicLayer::DirectiveOverride);
        assert!(fulcrum.tempo_multiplier > assess.tempo_multiplier);
        assert!(directive.tempo_multiplier > fulcrum.tempo_multiplier);
    }

    #[test]
    fn stinger_kind_paths() {
        for kind in [
            StingerKind::WeavingSuccess,
            StingerKind::WeavingFailure,
            StingerKind::AbilityUnlock,
            StingerKind::AffinityRankUp,
            StingerKind::EchoCollected,
            StingerKind::AnchorStabilized,
            StingerKind::AnchorLost,
            StingerKind::PhaseTransition,
            StingerKind::DecisionPrompt,
            StingerKind::StormChoiceMade,
            StingerKind::BossDefeated,
        ] {
            assert!(kind.sound_path().starts_with("audio/stinger/"));
        }
    }

    #[test]
    fn stinger_volume_clamped() {
        let stinger = UiStingerSpec::with_volume(StingerKind::WeavingSuccess, 5.0);
        assert!((stinger.volume - 1.0).abs() < 0.001);
    }

    #[test]
    fn spatial_sfx_factories() {
        let hum = SpatialSfxSpec::anchor_hum(Vec3f::ZERO);
        assert!(hum.looping);
        assert!(hum.max_distance > 0.0);

        let weave = SpatialSfxSpec::thread_weave(Vec3f::ZERO);
        assert!(!weave.looping);

        let impact = SpatialSfxSpec::boss_impact(Vec3f::ZERO);
        assert!(impact.volume > 0.5);

        let crackle = SpatialSfxSpec::storm_crackle(Vec3f::ZERO);
        assert!(crackle.looping);
    }

    #[test]
    fn audio_scene_zone_switching() {
        let mut scene = AudioScene::new();
        assert!(scene.ambience.is_none());

        scene.set_zone(ZoneAmbienceId::LoomspireSanctum);
        assert!(scene.ambience.is_some());
        assert_eq!(
            scene.ambience.as_ref().unwrap().zone,
            ZoneAmbienceId::LoomspireSanctum
        );

        // Setting the same zone should not change the spec.
        let orig_reverb = scene.ambience.as_ref().unwrap().reverb;
        scene.set_zone(ZoneAmbienceId::LoomspireSanctum);
        assert!((scene.ambience.as_ref().unwrap().reverb - orig_reverb).abs() < 0.001);

        // Switching zone should update.
        scene.set_zone(ZoneAmbienceId::BossCourtyard);
        assert_eq!(
            scene.ambience.as_ref().unwrap().zone,
            ZoneAmbienceId::BossCourtyard
        );
    }

    #[test]
    fn audio_scene_boss_layer_switching() {
        let mut scene = AudioScene::new();
        scene.set_boss_layer(BossMusicLayer::Assessment);
        assert_eq!(
            scene.boss_music.as_ref().unwrap().layer,
            BossMusicLayer::Assessment
        );

        scene.set_boss_layer(BossMusicLayer::DirectiveOverride);
        assert_eq!(
            scene.boss_music.as_ref().unwrap().layer,
            BossMusicLayer::DirectiveOverride
        );
    }

    #[test]
    fn audio_scene_stinger_queue() {
        let mut scene = AudioScene::new();
        scene.queue_stinger(UiStingerSpec::new(StingerKind::EchoCollected));
        scene.queue_stinger(UiStingerSpec::new(StingerKind::WeavingSuccess));
        assert_eq!(scene.pending_stingers.len(), 2);

        let drained = scene.drain_stingers();
        assert_eq!(drained.len(), 2);
        assert!(scene.pending_stingers.is_empty());
    }
}
