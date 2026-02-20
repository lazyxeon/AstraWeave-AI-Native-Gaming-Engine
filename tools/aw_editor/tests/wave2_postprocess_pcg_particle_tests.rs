//! Wave 2 Mutation Remediation — Post-Process + PCG + Particle System per-variant tests
//!
//! Targets the 3 largest editor panels with lowest test density:
//! - post_process_panel.rs (1,386L, 30 inline tests = 2.2/100L)
//! - pcg_panel.rs (1,242L, 30 inline tests = 2.4/100L)
//! - particle_system_panel.rs (2,360L, 64 inline tests = 2.7/100L)

use aw_editor_lib::panels::post_process_panel::{
    AmbientOcclusionSettings, AntiAliasing, AoMethod, BloomSettings, ChromaticAberrationSettings,
    ColorGradingSettings, DepthOfFieldSettings, DofMode, FilmGrainSettings, MotionBlurSettings,
    PostProcessPanel, PostProcessProfile, PostProcessTab, SsrSettings, Tonemapper,
    VignetteSettings,
};
use aw_editor_lib::panels::pcg_panel::{
    DungeonSettings, EncounterConfig, EncounterDifficulty, EnemyType, GenerationType, LootEntry,
    LootRarity, LootTable, PcgPanel, PcgTab, RoomConfig, RoomType,
};
use aw_editor_lib::panels::particle_system_panel::{
    CurveType, EmitterShape, ModuleType, ParticleBlendMode, ParticleRenderMode, ParticleTab,
    RangeValue, SimulationSpace, SortMode, SubEmitterEvent,
};

// ============================================================================
// TONEMAPPER
// ============================================================================

#[test]
fn tonemapper_all_count() {
    assert_eq!(Tonemapper::all().len(), 6);
}

#[test]
fn tonemapper_default_is_aces() {
    assert_eq!(Tonemapper::default(), Tonemapper::ACES);
}

#[test]
fn tonemapper_names() {
    assert_eq!(Tonemapper::None.name(), "None");
    assert_eq!(Tonemapper::Reinhard.name(), "Reinhard");
    assert_eq!(Tonemapper::ACES.name(), "ACES");
    assert_eq!(Tonemapper::Filmic.name(), "Filmic");
    assert_eq!(Tonemapper::AgX.name(), "AgX");
    assert_eq!(Tonemapper::Neutral.name(), "Neutral");
}

#[test]
fn tonemapper_is_cinematic_aces() {
    assert!(Tonemapper::ACES.is_cinematic());
}

#[test]
fn tonemapper_is_cinematic_filmic() {
    assert!(Tonemapper::Filmic.is_cinematic());
}

#[test]
fn tonemapper_is_cinematic_agx() {
    assert!(Tonemapper::AgX.is_cinematic());
}

#[test]
fn tonemapper_not_cinematic_none() {
    assert!(!Tonemapper::None.is_cinematic());
}

#[test]
fn tonemapper_not_cinematic_reinhard() {
    assert!(!Tonemapper::Reinhard.is_cinematic());
}

#[test]
fn tonemapper_not_cinematic_neutral() {
    assert!(!Tonemapper::Neutral.is_cinematic());
}

#[test]
fn tonemapper_display_contains_name() {
    for t in Tonemapper::all() {
        let s = format!("{}", t);
        assert!(s.contains(t.name()), "Display for {:?} = '{}', expected '{}'", t, s, t.name());
    }
}

#[test]
fn tonemapper_icons_nonempty() {
    for t in Tonemapper::all() {
        assert!(!t.icon().is_empty());
    }
}

// ============================================================================
// ANTI-ALIASING
// ============================================================================

#[test]
fn aa_all_count() {
    assert_eq!(AntiAliasing::all().len(), 7);
}

#[test]
fn aa_default_is_smaa() {
    assert_eq!(AntiAliasing::default(), AntiAliasing::SMAA);
}

#[test]
fn aa_names() {
    assert_eq!(AntiAliasing::None.name(), "None");
    assert_eq!(AntiAliasing::FXAA.name(), "FXAA");
    assert_eq!(AntiAliasing::SMAA.name(), "SMAA");
    assert_eq!(AntiAliasing::TAA.name(), "TAA");
    assert_eq!(AntiAliasing::MSAA2x.name(), "MSAA 2x");
    assert_eq!(AntiAliasing::MSAA4x.name(), "MSAA 4x");
    assert_eq!(AntiAliasing::MSAA8x.name(), "MSAA 8x");
}

#[test]
fn aa_is_msaa() {
    assert!(AntiAliasing::MSAA2x.is_msaa());
    assert!(AntiAliasing::MSAA4x.is_msaa());
    assert!(AntiAliasing::MSAA8x.is_msaa());
    assert!(!AntiAliasing::None.is_msaa());
    assert!(!AntiAliasing::FXAA.is_msaa());
    assert!(!AntiAliasing::SMAA.is_msaa());
    assert!(!AntiAliasing::TAA.is_msaa());
}

#[test]
fn aa_is_post_process() {
    assert!(AntiAliasing::FXAA.is_post_process());
    assert!(AntiAliasing::SMAA.is_post_process());
    assert!(AntiAliasing::TAA.is_post_process());
    assert!(!AntiAliasing::None.is_post_process());
    assert!(!AntiAliasing::MSAA2x.is_post_process());
    assert!(!AntiAliasing::MSAA4x.is_post_process());
    assert!(!AntiAliasing::MSAA8x.is_post_process());
}

#[test]
fn aa_display_contains_name() {
    for aa in AntiAliasing::all() {
        let s = format!("{}", aa);
        assert!(s.contains(aa.name()));
    }
}

// ============================================================================
// DOF MODE
// ============================================================================

#[test]
fn dof_all_count() {
    assert_eq!(DofMode::all().len(), 4);
}

#[test]
fn dof_default_disabled() {
    assert_eq!(DofMode::default(), DofMode::Disabled);
}

#[test]
fn dof_names() {
    assert_eq!(DofMode::Disabled.name(), "Disabled");
    assert_eq!(DofMode::Gaussian.name(), "Gaussian");
    assert_eq!(DofMode::Bokeh.name(), "Bokeh");
    assert_eq!(DofMode::CircleOfConfusion.name(), "Circle of Confusion");
}

#[test]
fn dof_is_enabled() {
    assert!(!DofMode::Disabled.is_enabled());
    assert!(DofMode::Gaussian.is_enabled());
    assert!(DofMode::Bokeh.is_enabled());
    assert!(DofMode::CircleOfConfusion.is_enabled());
}

#[test]
fn dof_display_contains_name() {
    for d in DofMode::all() {
        assert!(format!("{}", d).contains(d.name()));
    }
}

// ============================================================================
// AO METHOD
// ============================================================================

#[test]
fn ao_method_all_count() {
    assert_eq!(AoMethod::all().len(), 3);
}

#[test]
fn ao_method_default_ssao() {
    assert_eq!(AoMethod::default(), AoMethod::SSAO);
}

#[test]
fn ao_method_names() {
    assert_eq!(AoMethod::SSAO.name(), "SSAO");
    assert_eq!(AoMethod::HBAO.name(), "HBAO+");
    assert_eq!(AoMethod::GTAO.name(), "GTAO");
}

#[test]
fn ao_method_descriptions_nonempty() {
    for m in AoMethod::all() {
        assert!(!m.description().is_empty());
    }
}

#[test]
fn ao_method_display_contains_name() {
    for m in AoMethod::all() {
        assert!(format!("{}", m).contains(m.name()));
    }
}

// ============================================================================
// POST PROCESS TAB
// ============================================================================

#[test]
fn pp_tab_all_count() {
    assert_eq!(PostProcessTab::all().len(), 7);
}

#[test]
fn pp_tab_default_overview() {
    assert_eq!(PostProcessTab::default(), PostProcessTab::Overview);
}

#[test]
fn pp_tab_names() {
    assert_eq!(PostProcessTab::Overview.name(), "Overview");
    assert_eq!(PostProcessTab::Bloom.name(), "Bloom");
    assert_eq!(PostProcessTab::DepthOfField.name(), "Depth of Field");
    assert_eq!(PostProcessTab::MotionBlur.name(), "Motion Blur");
    assert_eq!(PostProcessTab::ColorGrading.name(), "Color Grading");
    assert_eq!(PostProcessTab::Effects.name(), "Effects");
    assert_eq!(PostProcessTab::Presets.name(), "Presets");
}

#[test]
fn pp_tab_icons_nonempty() {
    for t in PostProcessTab::all() {
        assert!(!t.icon().is_empty());
    }
}

// ============================================================================
// BLOOM SETTINGS DEFAULTS
// ============================================================================

#[test]
fn bloom_defaults() {
    let b = BloomSettings::default();
    assert!(b.enabled);
    assert!((b.intensity - 0.5).abs() < 0.001);
    assert!((b.threshold - 1.0).abs() < 0.001);
    assert!((b.soft_threshold - 0.5).abs() < 0.001);
    assert!((b.radius - 5.0).abs() < 0.001);
    assert!(!b.dirt_mask_enabled);
    assert!((b.dirt_mask_intensity - 1.0).abs() < 0.001);
    assert!(b.dirt_mask_path.is_empty());
}

// ============================================================================
// DEPTH OF FIELD SETTINGS DEFAULTS
// ============================================================================

#[test]
fn dof_settings_defaults() {
    let d = DepthOfFieldSettings::default();
    assert_eq!(d.mode, DofMode::Disabled);
    assert!((d.focus_distance - 10.0).abs() < 0.001);
    assert!((d.aperture - 5.6).abs() < 0.001);
    assert!((d.focal_length - 50.0).abs() < 0.001);
    assert_eq!(d.blade_count, 6);
    assert!((d.blade_curvature - 0.5).abs() < 0.001);
    assert!((d.max_blur - 1.0).abs() < 0.001);
}

// ============================================================================
// MOTION BLUR SETTINGS DEFAULTS
// ============================================================================

#[test]
fn motion_blur_defaults() {
    let m = MotionBlurSettings::default();
    assert!(!m.enabled);
    assert!((m.intensity - 0.5).abs() < 0.001);
    assert_eq!(m.sample_count, 8);
    assert!((m.max_velocity - 1000.0).abs() < 0.1);
    assert!(m.camera_motion_blur);
    assert!(m.object_motion_blur);
}

// ============================================================================
// COLOR GRADING SETTINGS DEFAULTS
// ============================================================================

#[test]
fn color_grading_defaults() {
    let c = ColorGradingSettings::default();
    assert!(c.enabled);
    assert!((c.temperature).abs() < 0.001);
    assert!((c.tint).abs() < 0.001);
    assert!((c.exposure).abs() < 0.001);
    assert!((c.contrast).abs() < 0.001);
    assert!((c.saturation).abs() < 0.001);
    assert!((c.vibrance).abs() < 0.001);
    assert!((c.hue_shift).abs() < 0.001);
    assert!((c.gamma - 1.0).abs() < 0.001);
    assert!((c.gain - 1.0).abs() < 0.001);
    assert!((c.lift).abs() < 0.001);
    assert!(!c.lut_enabled);
    assert!(c.lut_path.is_empty());
    assert!((c.lut_contribution - 1.0).abs() < 0.001);
}

// ============================================================================
// AO SETTINGS DEFAULTS
// ============================================================================

#[test]
fn ao_settings_defaults() {
    let a = AmbientOcclusionSettings::default();
    assert!(a.enabled);
    assert_eq!(a.method, AoMethod::SSAO);
    assert!((a.intensity - 0.5).abs() < 0.001);
    assert!((a.radius - 0.5).abs() < 0.001);
    assert!((a.bias - 0.025).abs() < 0.001);
    assert_eq!(a.samples, 16);
    assert!((a.direct_lighting_strength).abs() < 0.001);
}

// ============================================================================
// SSR SETTINGS DEFAULTS
// ============================================================================

#[test]
fn ssr_settings_defaults() {
    let s = SsrSettings::default();
    assert!(!s.enabled);
    assert!((s.max_distance - 100.0).abs() < 0.1);
    assert!((s.resolution - 0.5).abs() < 0.001);
    assert!((s.thickness - 0.1).abs() < 0.001);
    assert!((s.max_roughness - 0.5).abs() < 0.001);
}

// ============================================================================
// VIGNETTE / CHROMATIC / FILM GRAIN DEFAULTS
// ============================================================================

#[test]
fn vignette_defaults() {
    let v = VignetteSettings::default();
    assert!(!v.enabled);
    assert!((v.intensity - 0.3).abs() < 0.001);
    assert!((v.smoothness - 0.5).abs() < 0.001);
    assert!((v.roundness - 1.0).abs() < 0.001);
    assert_eq!(v.color, [0.0, 0.0, 0.0]);
}

#[test]
fn chromatic_aberration_defaults() {
    let c = ChromaticAberrationSettings::default();
    assert!(!c.enabled);
    assert!((c.intensity - 0.1).abs() < 0.001);
}

#[test]
fn film_grain_defaults() {
    let f = FilmGrainSettings::default();
    assert!(!f.enabled);
    assert!((f.intensity - 0.3).abs() < 0.001);
    assert!((f.response - 0.8).abs() < 0.001);
}

// ============================================================================
// POST PROCESS PROFILE DEFAULTS
// ============================================================================

#[test]
fn pp_profile_defaults() {
    let p = PostProcessProfile::default();
    assert_eq!(p.id, 0);
    assert_eq!(p.name, "New Profile");
    assert_eq!(p.tonemapper, Tonemapper::ACES);
    assert_eq!(p.anti_aliasing, AntiAliasing::SMAA);
    assert!(p.bloom.enabled);
    assert_eq!(p.dof.mode, DofMode::Disabled);
    assert!(!p.motion_blur.enabled);
    assert!(p.color_grading.enabled);
    assert!(p.ao.enabled);
    assert!(!p.ssr.enabled);
    assert!(!p.vignette.enabled);
    assert!(!p.chromatic_aberration.enabled);
    assert!(!p.film_grain.enabled);
}

// ============================================================================
// POST PROCESS PANEL
// ============================================================================

#[test]
fn pp_panel_new_does_not_panic() {
    let _p = PostProcessPanel::new();
    // Panel creates sample profiles on construction without panic
}

// ============================================================================
// GENERATION TYPE (PCG)
// ============================================================================

#[test]
fn gen_type_all_count() {
    assert_eq!(GenerationType::all().len(), 7);
}

#[test]
fn gen_type_default_encounter() {
    assert_eq!(GenerationType::default(), GenerationType::Encounter);
}

#[test]
fn gen_type_names() {
    assert_eq!(GenerationType::Encounter.name(), "Encounter");
    assert_eq!(GenerationType::Dungeon.name(), "Dungeon");
    assert_eq!(GenerationType::Loot.name(), "Loot");
    assert_eq!(GenerationType::Terrain.name(), "Terrain");
    assert_eq!(GenerationType::Vegetation.name(), "Vegetation");
    assert_eq!(GenerationType::Props.name(), "Props");
    assert_eq!(GenerationType::NPC.name(), "NPC");
}

#[test]
fn gen_type_icons_nonempty() {
    for g in GenerationType::all() {
        assert!(!g.icon().is_empty());
    }
}

#[test]
fn gen_type_display_contains_name() {
    for g in GenerationType::all() {
        assert!(format!("{}", g).contains(g.name()));
    }
}

// ============================================================================
// ENCOUNTER DIFFICULTY
// ============================================================================

#[test]
fn difficulty_all_count() {
    assert_eq!(EncounterDifficulty::all().len(), 6);
}

#[test]
fn difficulty_default_medium() {
    assert_eq!(EncounterDifficulty::default(), EncounterDifficulty::Medium);
}

#[test]
fn difficulty_names() {
    assert_eq!(EncounterDifficulty::Trivial.name(), "Trivial");
    assert_eq!(EncounterDifficulty::Easy.name(), "Easy");
    assert_eq!(EncounterDifficulty::Medium.name(), "Medium");
    assert_eq!(EncounterDifficulty::Hard.name(), "Hard");
    assert_eq!(EncounterDifficulty::Deadly.name(), "Deadly");
    assert_eq!(EncounterDifficulty::Boss.name(), "Boss");
}

#[test]
fn difficulty_display_contains_name() {
    for d in EncounterDifficulty::all() {
        assert!(format!("{}", d).contains(d.name()));
    }
}

// ============================================================================
// ROOM TYPE
// ============================================================================

#[test]
fn room_type_all_count() {
    assert_eq!(RoomType::all().len(), 8);
}

#[test]
fn room_type_default_normal() {
    assert_eq!(RoomType::default(), RoomType::Normal);
}

#[test]
fn room_type_names() {
    assert_eq!(RoomType::Normal.name(), "Normal");
    assert_eq!(RoomType::Entrance.name(), "Entrance");
    assert_eq!(RoomType::Exit.name(), "Exit");
    assert_eq!(RoomType::Treasure.name(), "Treasure");
    assert_eq!(RoomType::Boss.name(), "Boss");
    assert_eq!(RoomType::Shop.name(), "Shop");
    assert_eq!(RoomType::Secret.name(), "Secret");
    assert_eq!(RoomType::Corridor.name(), "Corridor");
}

#[test]
fn room_type_is_special() {
    assert!(!RoomType::Normal.is_special());
    assert!(!RoomType::Corridor.is_special());
    assert!(RoomType::Entrance.is_special());
    assert!(RoomType::Exit.is_special());
    assert!(RoomType::Treasure.is_special());
    assert!(RoomType::Boss.is_special());
    assert!(RoomType::Shop.is_special());
    assert!(RoomType::Secret.is_special());
}

#[test]
fn room_type_display_contains_name() {
    for r in RoomType::all() {
        assert!(format!("{}", r).contains(r.name()));
    }
}

// ============================================================================
// LOOT RARITY
// ============================================================================

#[test]
fn loot_rarity_all_count() {
    assert_eq!(LootRarity::all().len(), 5);
}

#[test]
fn loot_rarity_default_common() {
    assert_eq!(LootRarity::default(), LootRarity::Common);
}

#[test]
fn loot_rarity_names() {
    assert_eq!(LootRarity::Common.name(), "Common");
    assert_eq!(LootRarity::Uncommon.name(), "Uncommon");
    assert_eq!(LootRarity::Rare.name(), "Rare");
    assert_eq!(LootRarity::Epic.name(), "Epic");
    assert_eq!(LootRarity::Legendary.name(), "Legendary");
}

#[test]
fn loot_rarity_display_contains_name() {
    for r in LootRarity::all() {
        assert!(format!("{}", r).contains(r.name()));
    }
}

// ============================================================================
// PCG TAB
// ============================================================================

#[test]
fn pcg_tab_all_count() {
    assert_eq!(PcgTab::all().len(), 6);
}

#[test]
fn pcg_tab_default_seeds() {
    assert_eq!(PcgTab::default(), PcgTab::Seeds);
}

#[test]
fn pcg_tab_names() {
    assert_eq!(PcgTab::Seeds.name(), "Seeds");
    assert_eq!(PcgTab::Encounters.name(), "Encounters");
    assert_eq!(PcgTab::Dungeons.name(), "Dungeons");
    assert_eq!(PcgTab::Loot.name(), "Loot");
    assert_eq!(PcgTab::Preview.name(), "Preview");
    assert_eq!(PcgTab::History.name(), "History");
}

#[test]
fn pcg_tab_icons_nonempty() {
    for t in PcgTab::all() {
        assert!(!t.icon().is_empty());
    }
}

// ============================================================================
// ENEMY TYPE DEFAULTS
// ============================================================================

#[test]
fn enemy_type_defaults() {
    let e = EnemyType::default();
    assert!(e.id.is_empty());
    assert_eq!(e.name, "Enemy");
    assert!((e.threat_level - 1.0).abs() < 0.001);
    assert_eq!(e.min_count, 1);
    assert_eq!(e.max_count, 3);
    assert!((e.spawn_weight - 1.0).abs() < 0.001);
}

// ============================================================================
// ENCOUNTER CONFIG DEFAULTS
// ============================================================================

#[test]
fn encounter_config_defaults() {
    let e = EncounterConfig::default();
    assert_eq!(e.id, 0);
    assert_eq!(e.name, "New Encounter");
    assert_eq!(e.difficulty, EncounterDifficulty::Medium);
    assert!(e.enemy_types.is_empty());
    assert_eq!(e.min_enemies, 2);
    assert_eq!(e.max_enemies, 5);
    assert!((e.spawn_radius - 10.0).abs() < 0.001);
    assert!((e.reinforcement_chance - 0.0).abs() < 0.001);
}

// ============================================================================
// ROOM CONFIG DEFAULTS
// ============================================================================

#[test]
fn room_config_defaults() {
    let r = RoomConfig::default();
    assert_eq!(r.room_type, RoomType::Normal);
    assert_eq!(r.min_size, (5, 5));
    assert_eq!(r.max_size, (15, 15));
    assert!((r.spawn_chance - 1.0).abs() < 0.001);
    assert_eq!(r.max_count, 10);
}

// ============================================================================
// DUNGEON SETTINGS DEFAULTS
// ============================================================================

#[test]
fn dungeon_settings_defaults() {
    let d = DungeonSettings::default();
    assert_eq!(d.width, 100);
    assert_eq!(d.height, 100);
    assert_eq!(d.room_count, (8, 15));
    assert_eq!(d.corridor_width, 3);
    assert!((d.branching_factor - 0.3).abs() < 0.001);
    assert!((d.loop_chance - 0.2).abs() < 0.001);
    assert_eq!(d.room_configs.len(), 5);
}

#[test]
fn dungeon_settings_has_entrance_and_exit() {
    let d = DungeonSettings::default();
    assert!(d.room_configs.iter().any(|c| c.room_type == RoomType::Entrance));
    assert!(d.room_configs.iter().any(|c| c.room_type == RoomType::Exit));
}

// ============================================================================
// LOOT TABLE / ENTRY DEFAULTS
// ============================================================================

#[test]
fn loot_entry_defaults() {
    let l = LootEntry::default();
    assert!(l.item_id.is_empty());
    assert_eq!(l.name, "Item");
    assert_eq!(l.rarity, LootRarity::Common);
    assert!((l.drop_weight - 1.0).abs() < 0.001);
    assert_eq!(l.min_quantity, 1);
    assert_eq!(l.max_quantity, 1);
}

#[test]
fn loot_table_defaults() {
    let l = LootTable::default();
    assert_eq!(l.id, 0);
    assert_eq!(l.name, "New Loot Table");
    assert!(l.entries.is_empty());
    assert_eq!(l.guaranteed_drops, 1);
    assert_eq!(l.bonus_rolls, 2);
}

// ============================================================================
// PCG PANEL
// ============================================================================

#[test]
fn pcg_panel_new_does_not_panic() {
    let _p = PcgPanel::new();
    // Panel creates sample data on construction without panic
}

// ============================================================================
// EMITTER SHAPE (PARTICLE)
// ============================================================================

#[test]
fn emitter_shape_all_count() {
    assert_eq!(EmitterShape::all().len(), 8);
}

#[test]
fn emitter_shape_default_point() {
    assert_eq!(EmitterShape::default(), EmitterShape::Point);
}

#[test]
fn emitter_shape_names() {
    assert_eq!(EmitterShape::Point.name(), "Point");
    assert_eq!(EmitterShape::Sphere.name(), "Sphere");
    assert_eq!(EmitterShape::Hemisphere.name(), "Hemisphere");
    assert_eq!(EmitterShape::Cone.name(), "Cone");
    assert_eq!(EmitterShape::Box.name(), "Box");
    assert_eq!(EmitterShape::Circle.name(), "Circle");
    assert_eq!(EmitterShape::Edge.name(), "Edge");
    assert_eq!(EmitterShape::Mesh.name(), "Mesh");
}

#[test]
fn emitter_shape_is_volumetric() {
    assert!(EmitterShape::Sphere.is_volumetric());
    assert!(EmitterShape::Hemisphere.is_volumetric());
    assert!(EmitterShape::Cone.is_volumetric());
    assert!(EmitterShape::Box.is_volumetric());
    assert!(!EmitterShape::Point.is_volumetric());
    assert!(!EmitterShape::Circle.is_volumetric());
    assert!(!EmitterShape::Edge.is_volumetric());
    assert!(!EmitterShape::Mesh.is_volumetric());
}

#[test]
fn emitter_shape_display_contains_name() {
    for s in EmitterShape::all() {
        assert!(format!("{}", s).contains(s.name()));
    }
}

// ============================================================================
// SIMULATION SPACE
// ============================================================================

#[test]
fn sim_space_all_count() {
    assert_eq!(SimulationSpace::all().len(), 2);
}

#[test]
fn sim_space_default_local() {
    assert_eq!(SimulationSpace::default(), SimulationSpace::Local);
}

#[test]
fn sim_space_names() {
    assert_eq!(SimulationSpace::Local.name(), "Local");
    assert_eq!(SimulationSpace::World.name(), "World");
}

#[test]
fn sim_space_display() {
    assert!(format!("{}", SimulationSpace::Local).contains("Local"));
    assert!(format!("{}", SimulationSpace::World).contains("World"));
}

// ============================================================================
// PARTICLE BLEND MODE
// ============================================================================

#[test]
fn blend_mode_all_count() {
    assert_eq!(ParticleBlendMode::all().len(), 4);
}

#[test]
fn blend_mode_default_alpha() {
    assert_eq!(ParticleBlendMode::default(), ParticleBlendMode::Alpha);
}

#[test]
fn blend_mode_names() {
    assert_eq!(ParticleBlendMode::Alpha.name(), "Alpha");
    assert_eq!(ParticleBlendMode::Additive.name(), "Additive");
    assert_eq!(ParticleBlendMode::Multiply.name(), "Multiply");
    assert_eq!(ParticleBlendMode::Premultiply.name(), "Premultiply");
}

#[test]
fn blend_mode_is_additive() {
    assert!(ParticleBlendMode::Additive.is_additive());
    assert!(!ParticleBlendMode::Alpha.is_additive());
    assert!(!ParticleBlendMode::Multiply.is_additive());
    assert!(!ParticleBlendMode::Premultiply.is_additive());
}

// ============================================================================
// PARTICLE RENDER MODE
// ============================================================================

#[test]
fn render_mode_all_count() {
    assert_eq!(ParticleRenderMode::all().len(), 6);
}

#[test]
fn render_mode_default_billboard() {
    assert_eq!(ParticleRenderMode::default(), ParticleRenderMode::Billboard);
}

#[test]
fn render_mode_names() {
    assert_eq!(ParticleRenderMode::Billboard.name(), "Billboard");
    assert_eq!(ParticleRenderMode::StretchedBillboard.name(), "Stretched Billboard");
    assert_eq!(ParticleRenderMode::HorizontalBillboard.name(), "Horizontal Billboard");
    assert_eq!(ParticleRenderMode::VerticalBillboard.name(), "Vertical Billboard");
    assert_eq!(ParticleRenderMode::Mesh.name(), "Mesh");
    assert_eq!(ParticleRenderMode::Trail.name(), "Trail");
}

#[test]
fn render_mode_is_billboard() {
    assert!(ParticleRenderMode::Billboard.is_billboard());
    assert!(ParticleRenderMode::StretchedBillboard.is_billboard());
    assert!(ParticleRenderMode::HorizontalBillboard.is_billboard());
    assert!(ParticleRenderMode::VerticalBillboard.is_billboard());
    assert!(!ParticleRenderMode::Mesh.is_billboard());
    assert!(!ParticleRenderMode::Trail.is_billboard());
}

#[test]
fn render_mode_display_contains_name() {
    for r in ParticleRenderMode::all() {
        assert!(format!("{}", r).contains(r.name()));
    }
}

// ============================================================================
// CURVE TYPE
// ============================================================================

#[test]
fn curve_type_all_count() {
    assert_eq!(CurveType::all().len(), 7);
}

#[test]
fn curve_type_default_constant() {
    assert_eq!(CurveType::default(), CurveType::Constant);
}

#[test]
fn curve_type_names() {
    assert_eq!(CurveType::Constant.name(), "Constant");
    assert_eq!(CurveType::Linear.name(), "Linear");
    assert_eq!(CurveType::EaseIn.name(), "Ease In");
    assert_eq!(CurveType::EaseOut.name(), "Ease Out");
    assert_eq!(CurveType::EaseInOut.name(), "Ease In Out");
    assert_eq!(CurveType::Random.name(), "Random");
    assert_eq!(CurveType::Curve.name(), "Curve");
}

#[test]
fn curve_type_is_easing() {
    assert!(CurveType::EaseIn.is_easing());
    assert!(CurveType::EaseOut.is_easing());
    assert!(CurveType::EaseInOut.is_easing());
    assert!(!CurveType::Constant.is_easing());
    assert!(!CurveType::Linear.is_easing());
    assert!(!CurveType::Random.is_easing());
    assert!(!CurveType::Curve.is_easing());
}

#[test]
fn curve_type_display_contains_name() {
    for c in CurveType::all() {
        assert!(format!("{}", c).contains(c.name()));
    }
}

// ============================================================================
// RANGE VALUE
// ============================================================================

#[test]
fn range_value_default() {
    let r = RangeValue::default();
    assert!((r.min - 1.0).abs() < 0.001);
    assert!((r.max - 1.0).abs() < 0.001);
}

#[test]
fn range_value_constant() {
    let r = RangeValue::constant(5.0);
    assert!((r.min - 5.0).abs() < 0.001);
    assert!((r.max - 5.0).abs() < 0.001);
}

#[test]
fn range_value_range() {
    let r = RangeValue::range(2.0, 8.0);
    assert!((r.min - 2.0).abs() < 0.001);
    assert!((r.max - 8.0).abs() < 0.001);
}

// ============================================================================
// SUB-EMITTER EVENT
// ============================================================================

#[test]
fn sub_emitter_event_all_count() {
    assert_eq!(SubEmitterEvent::all().len(), 4);
}

#[test]
fn sub_emitter_event_default_birth() {
    assert_eq!(SubEmitterEvent::default(), SubEmitterEvent::Birth);
}

#[test]
fn sub_emitter_event_names() {
    assert_eq!(SubEmitterEvent::Birth.name(), "Birth");
    assert_eq!(SubEmitterEvent::Death.name(), "Death");
    assert_eq!(SubEmitterEvent::Collision.name(), "Collision");
    assert_eq!(SubEmitterEvent::Trigger.name(), "Trigger");
}

#[test]
fn sub_emitter_event_display() {
    for e in SubEmitterEvent::all() {
        assert!(format!("{}", e).contains(e.name()));
    }
}

// ============================================================================
// SORT MODE
// ============================================================================

#[test]
fn sort_mode_all_count() {
    assert_eq!(SortMode::all().len(), 4);
}

#[test]
fn sort_mode_default_none() {
    assert_eq!(SortMode::default(), SortMode::None);
}

#[test]
fn sort_mode_names() {
    assert_eq!(SortMode::None.name(), "None");
    assert_eq!(SortMode::ByDistance.name(), "By Distance");
    assert_eq!(SortMode::OldestFirst.name(), "Oldest First");
    assert_eq!(SortMode::YoungestFirst.name(), "Youngest First");
}

#[test]
fn sort_mode_is_sorted() {
    assert!(!SortMode::None.is_sorted());
    assert!(SortMode::ByDistance.is_sorted());
    assert!(SortMode::OldestFirst.is_sorted());
    assert!(SortMode::YoungestFirst.is_sorted());
}

#[test]
fn sort_mode_display() {
    for s in SortMode::all() {
        assert!(format!("{}", s).contains(s.name()));
    }
}

// ============================================================================
// MODULE TYPE — IS_PHYSICS / IS_VISUAL / IS_SPAWNER
// ============================================================================

#[test]
fn module_type_velocity_is_physics() {
    let m = ModuleType::Velocity {
        direction: [0.0, 1.0, 0.0],
        speed: RangeValue::constant(1.0),
    };
    assert!(m.is_physics());
    assert!(!m.is_visual());
    assert!(!m.is_spawner());
}

#[test]
fn module_type_force_is_physics() {
    let m = ModuleType::Force {
        force: [0.0, -9.81, 0.0],
        space: SimulationSpace::World,
    };
    assert!(m.is_physics());
}

#[test]
fn module_type_gravity_is_physics() {
    let m = ModuleType::Gravity { multiplier: 1.0 };
    assert!(m.is_physics());
}

#[test]
fn module_type_collision_is_physics() {
    let m = ModuleType::Collision {
        bounce: 0.5,
        lifetime_loss: 0.1,
        radius_scale: 1.0,
    };
    assert!(m.is_physics());
}

#[test]
fn module_type_rotation_is_physics() {
    let m = ModuleType::Rotation {
        speed: RangeValue::constant(90.0),
        random_start: false,
    };
    assert!(m.is_physics());
}

#[test]
fn module_type_texture_anim_is_visual() {
    let m = ModuleType::TextureAnimation {
        tiles_x: 4,
        tiles_y: 4,
        fps: 30.0,
    };
    assert!(m.is_visual());
    assert!(!m.is_physics());
}

#[test]
fn module_type_trail_is_visual() {
    let m = ModuleType::Trail {
        width: RangeValue::constant(0.1),
        lifetime: 1.0,
        min_vertex_distance: 0.1,
    };
    assert!(m.is_visual());
}

#[test]
fn module_type_light_is_visual() {
    let m = ModuleType::Light {
        color: [1.0, 1.0, 1.0],
        intensity: RangeValue::constant(1.0),
        range: RangeValue::constant(5.0),
    };
    assert!(m.is_visual());
}

#[test]
fn module_type_sub_emitter_is_spawner() {
    let m = ModuleType::SubEmitter {
        event: SubEmitterEvent::Death,
        emitter_id: 1,
    };
    assert!(m.is_spawner());
    assert!(!m.is_physics());
    assert!(!m.is_visual());
}

#[test]
fn module_type_noise_is_neither() {
    let m = ModuleType::Noise {
        strength: 1.0,
        frequency: 1.0,
        scroll_speed: 0.5,
    };
    assert!(!m.is_physics());
    assert!(!m.is_visual());
    assert!(!m.is_spawner());
}

#[test]
fn module_type_all_variants_count() {
    assert_eq!(ModuleType::all_variants().len(), 10);
}

#[test]
fn module_type_names_nonempty() {
    let modules: Vec<ModuleType> = vec![
        ModuleType::Velocity { direction: [0.0; 3], speed: RangeValue::default() },
        ModuleType::Force { force: [0.0; 3], space: SimulationSpace::Local },
        ModuleType::Gravity { multiplier: 1.0 },
        ModuleType::Noise { strength: 1.0, frequency: 1.0, scroll_speed: 0.0 },
        ModuleType::Collision { bounce: 0.5, lifetime_loss: 0.0, radius_scale: 1.0 },
        ModuleType::SubEmitter { event: SubEmitterEvent::Birth, emitter_id: 0 },
        ModuleType::TextureAnimation { tiles_x: 1, tiles_y: 1, fps: 1.0 },
        ModuleType::Trail { width: RangeValue::default(), lifetime: 1.0, min_vertex_distance: 0.1 },
        ModuleType::Light { color: [1.0; 3], intensity: RangeValue::default(), range: RangeValue::default() },
        ModuleType::Rotation { speed: RangeValue::default(), random_start: false },
    ];
    for m in &modules {
        assert!(!m.name().is_empty());
        assert!(!m.icon().is_empty());
        let display = format!("{}", m);
        assert!(display.contains(m.name()));
    }
}

// ============================================================================
// PARTICLE TAB
// ============================================================================

#[test]
fn particle_tab_all_count() {
    assert_eq!(ParticleTab::all().len(), 8);
}

#[test]
fn particle_tab_default_emitter() {
    assert_eq!(ParticleTab::default(), ParticleTab::Emitter);
}

#[test]
fn particle_tab_names() {
    assert_eq!(ParticleTab::Emitter.name(), "Emitter");
    assert_eq!(ParticleTab::Shape.name(), "Shape");
    assert_eq!(ParticleTab::Particles.name(), "Particles");
    assert_eq!(ParticleTab::Lifetime.name(), "Lifetime");
    assert_eq!(ParticleTab::Rendering.name(), "Rendering");
    assert_eq!(ParticleTab::Modules.name(), "Modules");
    assert_eq!(ParticleTab::Presets.name(), "Presets");
    assert_eq!(ParticleTab::Stats.name(), "Stats");
}

#[test]
fn particle_tab_display() {
    for t in ParticleTab::all() {
        assert!(format!("{}", t).contains(t.name()));
    }
}
