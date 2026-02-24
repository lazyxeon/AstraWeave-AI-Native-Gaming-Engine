//! Wave 2 mutation remediation tests — LOD Config + Material Editor panels
//! Covers: LodBiasMode, FadeMode, LodLevel, LodGroup, GlobalLodSettings,
//!         ReductionMethod, LodGenerationSettings, LodTab,
//!         MaterialType, BlendMode, TextureChannel, TextureSlot, Material,
//!         MaterialPreset

use aw_editor_lib::panels::lod_config_panel::{
    FadeMode, GlobalLodSettings, LodBiasMode, LodGenerationSettings, LodGroup, LodLevel, LodTab,
    ReductionMethod,
};
use aw_editor_lib::panels::material_editor_panel::{
    BlendMode, Material, MaterialPreset, MaterialType, TextureChannel, TextureSlot,
};

// ═══════════════════════════════════════════════════════════════════════════════════
// LOD BIAS MODE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn lod_bias_mode_all_count() {
    assert_eq!(LodBiasMode::all().len(), 5);
}

#[test]
fn lod_bias_mode_names() {
    assert_eq!(LodBiasMode::Auto.name(), "Auto");
    assert_eq!(LodBiasMode::Quality.name(), "Quality");
    assert_eq!(LodBiasMode::Balanced.name(), "Balanced");
    assert_eq!(LodBiasMode::Performance.name(), "Performance");
    assert_eq!(LodBiasMode::Custom.name(), "Custom");
}

#[test]
fn lod_bias_mode_icons_non_empty() {
    for mode in LodBiasMode::all() {
        assert!(!mode.icon().is_empty(), "{:?} icon empty", mode);
    }
}

#[test]
fn lod_bias_mode_display() {
    for mode in LodBiasMode::all() {
        let s = format!("{}", mode);
        assert!(s.contains(mode.name()));
    }
}

#[test]
fn lod_bias_mode_default_is_auto() {
    assert_eq!(LodBiasMode::default(), LodBiasMode::Auto);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// FADE MODE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn fade_mode_all_count() {
    assert_eq!(FadeMode::all().len(), 4);
}

#[test]
fn fade_mode_names() {
    assert_eq!(FadeMode::None.name(), "None");
    assert_eq!(FadeMode::CrossFade.name(), "Cross Fade");
    assert_eq!(FadeMode::SpeedTree.name(), "SpeedTree");
    assert_eq!(FadeMode::Dither.name(), "Dither");
}

#[test]
fn fade_mode_icons_non_empty() {
    for mode in FadeMode::all() {
        assert!(!mode.icon().is_empty(), "{:?} icon empty", mode);
    }
}

#[test]
fn fade_mode_default_is_none() {
    assert_eq!(FadeMode::default(), FadeMode::None);
}

#[test]
fn fade_mode_display() {
    for mode in FadeMode::all() {
        let s = format!("{}", mode);
        assert!(s.contains(mode.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// LOD LEVEL
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn lod_level_defaults() {
    let l = LodLevel::default();
    assert_eq!(l.level, 0);
    assert!(l.mesh_path.is_empty());
    assert!((l.distance - 0.0).abs() < f32::EPSILON);
    assert!((l.screen_coverage - 1.0).abs() < f32::EPSILON);
    assert_eq!(l.triangle_count, 0);
    assert_eq!(l.vertex_count, 0);
    assert!((l.reduction_percent - 0.0).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// LOD GROUP
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn lod_group_defaults() {
    let g = LodGroup::default();
    assert_eq!(g.id, 0);
    assert_eq!(g.name, "New LOD Group");
    assert!(g.asset_path.is_empty());
    assert!(g.enabled);
    assert!(g.levels.is_empty());
    assert_eq!(g.fade_mode, FadeMode::CrossFade);
    assert!((g.fade_width - 0.1).abs() < f32::EPSILON);
    assert!(g.cross_fade);
    assert!((g.cull_distance - 1000.0).abs() < f32::EPSILON);
    assert_eq!(g.shadow_lod_offset, 1);
    assert_eq!(g.base_triangles, 0);
    assert_eq!(g.current_level, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GLOBAL LOD SETTINGS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn global_lod_settings_defaults() {
    let s = GlobalLodSettings::default();
    assert_eq!(s.bias_mode, LodBiasMode::Balanced);
    assert!((s.custom_bias - 1.0).abs() < f32::EPSILON);
    assert_eq!(s.maximum_lod_level, 4);
    assert!((s.lod_cross_fade_time - 0.5).abs() < f32::EPSILON);
    assert!(!s.screen_coverage_enabled);
    assert!((s.min_screen_coverage - 0.01).abs() < f32::EPSILON);
    assert_eq!(s.shadow_lod_bias, 1);
    assert!((s.shadow_cull_distance - 500.0).abs() < f32::EPSILON);
}

#[test]
fn global_lod_settings_quality_distances() {
    let s = GlobalLodSettings::default();
    assert_eq!(s.quality_distances, [20.0, 50.0, 100.0, 200.0]);
}

#[test]
fn global_lod_settings_balanced_distances() {
    let s = GlobalLodSettings::default();
    assert_eq!(s.balanced_distances, [15.0, 35.0, 70.0, 150.0]);
}

#[test]
fn global_lod_settings_performance_distances() {
    let s = GlobalLodSettings::default();
    assert_eq!(s.performance_distances, [10.0, 25.0, 50.0, 100.0]);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// REDUCTION METHOD
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn reduction_method_all_count() {
    assert_eq!(ReductionMethod::all().len(), 4);
}

#[test]
fn reduction_method_names() {
    assert_eq!(
        ReductionMethod::QuadricErrorMetric.name(),
        "Quadric Error Metric"
    );
    assert_eq!(ReductionMethod::EdgeCollapse.name(), "Edge Collapse");
    assert_eq!(
        ReductionMethod::VertexClustering.name(),
        "Vertex Clustering"
    );
    assert_eq!(ReductionMethod::Simplygon.name(), "Simplygon");
}

#[test]
fn reduction_method_icons_non_empty() {
    for m in ReductionMethod::all() {
        assert!(!m.icon().is_empty(), "{:?} icon empty", m);
    }
}

#[test]
fn reduction_method_default() {
    assert_eq!(
        ReductionMethod::default(),
        ReductionMethod::QuadricErrorMetric
    );
}

#[test]
fn reduction_method_display() {
    for m in ReductionMethod::all() {
        let s = format!("{}", m);
        assert!(s.contains(m.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// LOD GENERATION SETTINGS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn lod_generation_settings_defaults() {
    let s = LodGenerationSettings::default();
    assert!(s.auto_generate);
    assert_eq!(s.num_levels, 4);
    assert_eq!(s.reduction_method, ReductionMethod::QuadricErrorMetric);
    assert_eq!(s.target_reductions, [50.0, 75.0, 90.0, 95.0]);
    assert!(s.preserve_uvs);
    assert!(s.preserve_normals);
    assert!(s.preserve_borders);
    assert!((s.weld_threshold - 0.001).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// LOD TAB
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn lod_tab_all_count() {
    assert_eq!(LodTab::all().len(), 4);
}

#[test]
fn lod_tab_names() {
    assert_eq!(LodTab::Groups.name(), "Groups");
    assert_eq!(LodTab::Global.name(), "Global");
    assert_eq!(LodTab::Generation.name(), "Generation");
    assert_eq!(LodTab::Statistics.name(), "Statistics");
}

#[test]
fn lod_tab_icons_non_empty() {
    for tab in LodTab::all() {
        assert!(!tab.icon().is_empty(), "{:?} icon empty", tab);
    }
}

#[test]
fn lod_tab_default_is_groups() {
    assert_eq!(LodTab::default(), LodTab::Groups);
}

#[test]
fn lod_tab_display() {
    for tab in LodTab::all() {
        let s = format!("{}", tab);
        assert!(s.contains(tab.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// MATERIAL TYPE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn material_type_all_count() {
    assert_eq!(MaterialType::all().len(), 10);
}

#[test]
fn material_type_names() {
    assert_eq!(MaterialType::StandardPBR.name(), "Standard PBR");
    assert_eq!(MaterialType::Unlit.name(), "Unlit");
    assert_eq!(MaterialType::Subsurface.name(), "Subsurface");
    assert_eq!(MaterialType::Glass.name(), "Glass");
    assert_eq!(MaterialType::Water.name(), "Water");
    assert_eq!(MaterialType::Foliage.name(), "Foliage");
    assert_eq!(MaterialType::Hair.name(), "Hair");
    assert_eq!(MaterialType::Cloth.name(), "Cloth");
    assert_eq!(MaterialType::Terrain.name(), "Terrain");
    assert_eq!(MaterialType::Decal.name(), "Decal");
}

#[test]
fn material_type_icons_non_empty() {
    for t in MaterialType::all() {
        assert!(!t.icon().is_empty(), "{:?} icon empty", t);
    }
}

#[test]
fn material_type_default() {
    assert_eq!(MaterialType::default(), MaterialType::StandardPBR);
}

#[test]
fn material_type_display() {
    for t in MaterialType::all() {
        let s = format!("{}", t);
        assert!(s.contains(t.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// BLEND MODE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn blend_mode_all_count() {
    assert_eq!(BlendMode::all().len(), 5);
}

#[test]
fn blend_mode_names() {
    assert_eq!(BlendMode::Opaque.name(), "Opaque");
    assert_eq!(BlendMode::Masked.name(), "Masked");
    assert_eq!(BlendMode::Translucent.name(), "Translucent");
    assert_eq!(BlendMode::Additive.name(), "Additive");
    assert_eq!(BlendMode::Modulate.name(), "Modulate");
}

#[test]
fn blend_mode_is_transparent() {
    assert!(!BlendMode::Opaque.is_transparent());
    assert!(!BlendMode::Masked.is_transparent());
    assert!(BlendMode::Translucent.is_transparent());
    assert!(BlendMode::Additive.is_transparent());
    assert!(BlendMode::Modulate.is_transparent());
}

#[test]
fn blend_mode_icons_non_empty() {
    for mode in BlendMode::all() {
        assert!(!mode.icon().is_empty(), "{:?} icon empty", mode);
    }
}

#[test]
fn blend_mode_default() {
    assert_eq!(BlendMode::default(), BlendMode::Opaque);
}

#[test]
fn blend_mode_display() {
    for mode in BlendMode::all() {
        let s = format!("{}", mode);
        assert!(s.contains(mode.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// TEXTURE CHANNEL
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn texture_channel_all_count() {
    assert_eq!(TextureChannel::all().len(), 8);
}

#[test]
fn texture_channel_names() {
    assert_eq!(TextureChannel::Albedo.name(), "Albedo");
    assert_eq!(TextureChannel::Normal.name(), "Normal");
    assert_eq!(TextureChannel::Metallic.name(), "Metallic");
    assert_eq!(TextureChannel::Roughness.name(), "Roughness");
    assert_eq!(TextureChannel::AO.name(), "Ambient Occlusion");
    assert_eq!(TextureChannel::Emissive.name(), "Emissive");
    assert_eq!(TextureChannel::Height.name(), "Height");
    assert_eq!(TextureChannel::Opacity.name(), "Opacity");
}

#[test]
fn texture_channel_icons_non_empty() {
    for ch in TextureChannel::all() {
        assert!(!ch.icon().is_empty(), "{:?} icon empty", ch);
    }
}

#[test]
fn texture_channel_display() {
    for ch in TextureChannel::all() {
        let s = format!("{}", ch);
        assert!(s.contains(ch.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// TEXTURE SLOT
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn texture_slot_defaults() {
    let s = TextureSlot::default();
    assert_eq!(s.channel, TextureChannel::Albedo);
    assert!(s.texture_path.is_empty());
    assert_eq!(s.tiling, (1.0, 1.0));
    assert_eq!(s.offset, (0.0, 0.0));
    assert!((s.intensity - 1.0).abs() < f32::EPSILON);
    assert!(s.enabled);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// MATERIAL DEFAULTS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn material_default_identity() {
    let m = Material::default();
    assert_eq!(m.id, 0);
    assert_eq!(m.name, "New Material");
    assert_eq!(m.material_type, MaterialType::StandardPBR);
    assert_eq!(m.blend_mode, BlendMode::Opaque);
}

#[test]
fn material_default_pbr() {
    let m = Material::default();
    assert_eq!(m.base_color, [1.0, 1.0, 1.0, 1.0]);
    assert!((m.metallic - 0.0).abs() < f32::EPSILON);
    assert!((m.roughness - 0.5).abs() < f32::EPSILON);
    assert_eq!(m.emissive_color, [0.0, 0.0, 0.0]);
    assert!((m.emissive_intensity - 0.0).abs() < f32::EPSILON);
}

#[test]
fn material_default_rendering() {
    let m = Material::default();
    assert!(!m.two_sided);
    assert!(m.cast_shadows);
    assert!(m.receive_shadows);
    assert!((m.alpha_cutoff - 0.5).abs() < f32::EPSILON);
}

#[test]
fn material_default_subsurface() {
    let m = Material::default();
    assert_eq!(m.subsurface_color, [1.0, 0.2, 0.1]);
    assert!((m.subsurface_radius - 1.0).abs() < f32::EPSILON);
}

#[test]
fn material_default_glass() {
    let m = Material::default();
    assert!((m.ior - 1.5).abs() < f32::EPSILON);
    assert!((m.transmission - 0.0).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// MATERIAL PRESET (struct fields — presets() is private, so test construction)
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn material_preset_construction() {
    let mp = MaterialPreset {
        name: "Test Metal".to_string(),
        category: "Metals".to_string(),
        material_type: MaterialType::StandardPBR,
        base_color: [0.9, 0.9, 0.9, 1.0],
        metallic: 1.0,
        roughness: 0.1,
    };
    assert_eq!(mp.name, "Test Metal");
    assert_eq!(mp.category, "Metals");
    assert_eq!(mp.material_type, MaterialType::StandardPBR);
    assert!((mp.metallic - 1.0).abs() < f32::EPSILON);
    assert!((mp.roughness - 0.1).abs() < f32::EPSILON);
}

#[test]
fn material_preset_glass_type() {
    let mp = MaterialPreset {
        name: "Glass".to_string(),
        category: "Special".to_string(),
        material_type: MaterialType::Glass,
        base_color: [1.0, 1.0, 1.0, 0.3],
        metallic: 0.0,
        roughness: 0.0,
    };
    assert_eq!(mp.material_type, MaterialType::Glass);
    assert!((mp.roughness - 0.0).abs() < f32::EPSILON);
}

#[test]
fn material_preset_subsurface_type() {
    let mp = MaterialPreset {
        name: "Skin".to_string(),
        category: "Organic".to_string(),
        material_type: MaterialType::Subsurface,
        base_color: [0.9, 0.7, 0.6, 1.0],
        metallic: 0.0,
        roughness: 0.5,
    };
    assert_eq!(mp.material_type, MaterialType::Subsurface);
    assert_eq!(mp.category, "Organic");
}

#[test]
fn material_preset_base_color_channels() {
    let mp = MaterialPreset {
        name: "Red".to_string(),
        category: "Test".to_string(),
        material_type: MaterialType::StandardPBR,
        base_color: [1.0, 0.0, 0.0, 1.0],
        metallic: 0.0,
        roughness: 0.5,
    };
    assert!((mp.base_color[0] - 1.0).abs() < f32::EPSILON);
    assert!((mp.base_color[1] - 0.0).abs() < f32::EPSILON);
    assert!((mp.base_color[2] - 0.0).abs() < f32::EPSILON);
    assert!((mp.base_color[3] - 1.0).abs() < f32::EPSILON);
}

#[test]
fn material_preset_clone() {
    let mp = MaterialPreset {
        name: "Original".to_string(),
        category: "Test".to_string(),
        material_type: MaterialType::StandardPBR,
        base_color: [0.5, 0.5, 0.5, 1.0],
        metallic: 0.8,
        roughness: 0.3,
    };
    let cloned = mp.clone();
    assert_eq!(cloned.name, mp.name);
    assert_eq!(cloned.material_type, mp.material_type);
    assert!((cloned.metallic - mp.metallic).abs() < f32::EPSILON);
}
