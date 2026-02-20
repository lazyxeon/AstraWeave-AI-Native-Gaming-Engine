//! Wave 2 mutation remediation tests — Procedural Filler + Charts panels
//! Covers: FillerMode, ScatterCategory, BiomePreset, EnvironmentPreset (filler),
//!         ChartType, DataSource, ExportFormat (charts), ChartStats, ChartsAction

use aw_editor_lib::panels::procedural_filler_panel::{
    BiomePreset, FillerMode, ScatterCategory,
    EnvironmentPreset as FillerEnvPreset,
};
use aw_editor_lib::panels::charts_panel::{
    ChartStats, ChartType, ChartsAction, ChartsPanel, DataSource,
    ExportFormat as ChartExportFormat,
};

// ═══════════════════════════════════════════════════════════════════════════════════
// FILLER MODE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn filler_mode_all_count() {
    assert_eq!(FillerMode::all().len(), 5);
}

#[test]
fn filler_mode_names_non_empty() {
    for m in FillerMode::all() {
        assert!(!m.name().is_empty(), "{:?} name empty", m);
    }
}

#[test]
fn filler_mode_icons_non_empty() {
    for m in FillerMode::all() {
        assert!(!m.icon().is_empty(), "{:?} icon empty", m);
    }
}

#[test]
fn filler_mode_description_non_empty() {
    for m in FillerMode::all() {
        assert!(!m.description().is_empty(), "{:?} desc empty", m);
    }
}

#[test]
fn filler_mode_affects_terrain() {
    assert!(!FillerMode::ScatterFill.affects_terrain());
    assert!(FillerMode::SplineRoad.affects_terrain());
    assert!(FillerMode::TerrainGen.affects_terrain());
    assert!(!FillerMode::EnvironmentPreset.affects_terrain());
    assert!(FillerMode::FullScene.affects_terrain());
}

#[test]
fn filler_mode_spawns_entities() {
    assert!(FillerMode::ScatterFill.spawns_entities());
    assert!(FillerMode::SplineRoad.spawns_entities());
    assert!(!FillerMode::TerrainGen.spawns_entities());
    assert!(!FillerMode::EnvironmentPreset.spawns_entities());
    assert!(FillerMode::FullScene.spawns_entities());
}

#[test]
fn filler_mode_default_is_scatter() {
    assert_eq!(FillerMode::default(), FillerMode::ScatterFill);
}

#[test]
fn filler_mode_display() {
    for m in FillerMode::all() {
        let s = format!("{}", m);
        assert!(s.contains(m.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// SCATTER CATEGORY
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn scatter_category_all_count() {
    assert_eq!(ScatterCategory::all().len(), 8);
}

#[test]
fn scatter_category_names_non_empty() {
    for c in ScatterCategory::all() {
        assert!(!c.name().is_empty(), "{:?} name empty", c);
    }
}

#[test]
fn scatter_category_icons_non_empty() {
    for c in ScatterCategory::all() {
        assert!(!c.icon().is_empty(), "{:?} icon empty", c);
    }
}

#[test]
fn scatter_category_default_density_positive() {
    for c in ScatterCategory::all() {
        assert!(c.default_density() > 0.0, "{:?} density not positive", c);
    }
}

#[test]
fn scatter_category_grass_has_highest_density() {
    let max_density = ScatterCategory::all()
        .iter()
        .filter(|c| **c != ScatterCategory::All)
        .map(|c| c.default_density())
        .fold(0.0f32, f32::max);
    assert!((max_density - ScatterCategory::Grass.default_density()).abs() < f32::EPSILON);
}

#[test]
fn scatter_category_needs_lod() {
    assert!(ScatterCategory::Trees.needs_lod());
    assert!(ScatterCategory::Rocks.needs_lod());
    assert!(ScatterCategory::Bushes.needs_lod());
    assert!(!ScatterCategory::Grass.needs_lod());
    assert!(!ScatterCategory::Flowers.needs_lod());
}

#[test]
fn scatter_category_casts_shadows() {
    assert!(ScatterCategory::Trees.casts_shadows());
    assert!(ScatterCategory::Rocks.casts_shadows());
    assert!(ScatterCategory::Props.casts_shadows());
    assert!(!ScatterCategory::Grass.casts_shadows());
    assert!(!ScatterCategory::Flowers.casts_shadows());
}

#[test]
fn scatter_category_default_is_all() {
    assert_eq!(ScatterCategory::default(), ScatterCategory::All);
}

#[test]
fn scatter_category_display() {
    for c in ScatterCategory::all() {
        let s = format!("{}", c);
        assert!(s.contains(c.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// BIOME PRESET
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn biome_preset_all_count() {
    assert_eq!(BiomePreset::all().len(), 11);
}

#[test]
fn biome_preset_names_non_empty() {
    for b in BiomePreset::all() {
        assert!(!b.name().is_empty(), "{:?} name empty", b);
    }
}

#[test]
fn biome_preset_icons_non_empty() {
    for b in BiomePreset::all() {
        assert!(!b.icon().is_empty(), "{:?} icon empty", b);
    }
}

#[test]
fn biome_preset_tree_density_non_negative() {
    for b in BiomePreset::all() {
        assert!(b.tree_density() >= 0.0, "{:?} tree density negative", b);
    }
}

#[test]
fn biome_preset_rock_density_non_negative() {
    for b in BiomePreset::all() {
        assert!(b.rock_density() >= 0.0, "{:?} rock density negative", b);
    }
}

#[test]
fn biome_preset_jungle_densest_trees() {
    assert!(BiomePreset::TropicalJungle.tree_density() > BiomePreset::TemperateForest.tree_density());
}

#[test]
fn biome_preset_volcanic_no_trees() {
    assert!((BiomePreset::VolcanicWasteland.tree_density() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn biome_preset_primary_color_in_range() {
    for b in BiomePreset::all() {
        let c = b.primary_color();
        for channel in &c {
            assert!(*channel >= 0.0 && *channel <= 1.0, "{:?} color out of range", b);
        }
    }
}

#[test]
fn biome_preset_has_water() {
    assert!(BiomePreset::MediterraneanCoast.has_water());
    assert!(BiomePreset::Swampland.has_water());
    assert!(BiomePreset::TropicalJungle.has_water());
    assert!(!BiomePreset::DesertDunes.has_water());
    assert!(!BiomePreset::ArcticTundra.has_water());
}

#[test]
fn biome_preset_default_is_custom() {
    assert_eq!(BiomePreset::default(), BiomePreset::Custom);
}

#[test]
fn biome_preset_display() {
    for b in BiomePreset::all() {
        let s = format!("{}", b);
        assert!(s.contains(b.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// FILLER ENVIRONMENT PRESET
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn filler_env_preset_all_count() {
    assert_eq!(FillerEnvPreset::all().len(), 15);
}

#[test]
fn filler_env_preset_names_non_empty() {
    for p in FillerEnvPreset::all() {
        assert!(!p.name().is_empty(), "{:?} name empty", p);
    }
}

#[test]
fn filler_env_preset_sun_intensity_range() {
    for p in FillerEnvPreset::all() {
        let i = p.sun_intensity();
        assert!(i >= 0.0 && i <= 2.0, "{:?} sun intensity {} out of range", p, i);
    }
}

#[test]
fn filler_env_preset_night_no_sun() {
    assert!((FillerEnvPreset::Night.sun_intensity() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn filler_env_preset_ambient_intensity_range() {
    for p in FillerEnvPreset::all() {
        let i = p.ambient_intensity();
        assert!(i >= 0.0 && i <= 1.0, "{:?} ambient intensity {} out of range", p, i);
    }
}

#[test]
fn filler_env_preset_fog_density_range() {
    for p in FillerEnvPreset::all() {
        let d = p.fog_density();
        assert!(d >= 0.0 && d <= 1.0, "{:?} fog density {} out of range", p, d);
    }
}

#[test]
fn filler_env_preset_sky_color_in_range() {
    for p in FillerEnvPreset::all() {
        let c = p.sky_color();
        for ch in &c {
            assert!(*ch >= 0.0 && *ch <= 1.0, "{:?} sky color out of range", p);
        }
    }
}

#[test]
fn filler_env_preset_fog_color_in_range() {
    for p in FillerEnvPreset::all() {
        let c = p.fog_color();
        for ch in &c {
            assert!(*ch >= 0.0 && *ch <= 1.0, "{:?} fog color out of range", p);
        }
    }
}

#[test]
fn filler_env_preset_default_is_custom() {
    assert_eq!(FillerEnvPreset::default(), FillerEnvPreset::Custom);
}

#[test]
fn filler_env_preset_display() {
    for p in FillerEnvPreset::all() {
        let s = format!("{}", p);
        assert!(s.contains(p.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CHARTS PANEL — ChartType
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn chart_type_all_count() {
    assert_eq!(ChartType::all().len(), 5);
}

#[test]
fn chart_type_names_non_empty() {
    for t in ChartType::all() {
        assert!(!t.name().is_empty(), "{:?} name empty", t);
    }
}

#[test]
fn chart_type_icons_non_empty() {
    for t in ChartType::all() {
        assert!(!t.icon().is_empty(), "{:?} icon empty", t);
    }
}

#[test]
fn chart_type_is_bar_variant() {
    assert!(!ChartType::Line.is_bar_variant());
    assert!(ChartType::Bar.is_bar_variant());
    assert!(!ChartType::Scatter.is_bar_variant());
    assert!(ChartType::Stacked.is_bar_variant());
    assert!(ChartType::Normalized.is_bar_variant());
}

#[test]
fn chart_type_display() {
    for t in ChartType::all() {
        let s = format!("{}", t);
        assert!(s.contains(t.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CHARTS PANEL — DataSource
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn data_source_all_count() {
    assert_eq!(DataSource::all().len(), 7);
}

#[test]
fn data_source_names_non_empty() {
    for d in DataSource::all() {
        assert!(!d.name().is_empty(), "{:?} name empty", d);
    }
}

#[test]
fn data_source_is_hardware_metric() {
    assert!(!DataSource::FrameTiming.is_hardware_metric());
    assert!(!DataSource::EntityCounts.is_hardware_metric());
    assert!(DataSource::MemoryUsage.is_hardware_metric());
    assert!(DataSource::CpuLoad.is_hardware_metric());
    assert!(DataSource::GpuUtilization.is_hardware_metric());
    assert!(!DataSource::Custom.is_hardware_metric());
}

#[test]
fn data_source_display() {
    for d in DataSource::all() {
        let s = format!("{}", d);
        assert!(s.contains(d.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CHARTS PANEL — ExportFormat
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn chart_export_format_all_count() {
    assert_eq!(ChartExportFormat::all().len(), 3);
}

#[test]
fn chart_export_format_extensions() {
    assert_eq!(ChartExportFormat::Csv.extension(), "csv");
    assert_eq!(ChartExportFormat::Json.extension(), "json");
    assert_eq!(ChartExportFormat::Png.extension(), "png");
}

#[test]
fn chart_export_format_is_text() {
    assert!(ChartExportFormat::Csv.is_text_format());
    assert!(ChartExportFormat::Json.is_text_format());
    assert!(!ChartExportFormat::Png.is_text_format());
}

#[test]
fn chart_export_format_display() {
    for f in ChartExportFormat::all() {
        let s = format!("{}", f);
        assert!(s.contains(f.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CHARTS PANEL — ChartStats
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn chart_stats_default_all_zero() {
    let s = ChartStats::default();
    assert!((s.min - 0.0).abs() < f64::EPSILON);
    assert!((s.max - 0.0).abs() < f64::EPSILON);
    assert!((s.avg - 0.0).abs() < f64::EPSILON);
    assert!((s.stddev - 0.0).abs() < f64::EPSILON);
    assert_eq!(s.count, 0);
}

#[test]
fn chart_stats_from_empty_data() {
    let s = ChartStats::from_data(&[]);
    assert_eq!(s.count, 0);
}

#[test]
fn chart_stats_from_single_point() {
    let s = ChartStats::from_data(&[(0.0, 42.0)]);
    assert_eq!(s.count, 1);
    assert!((s.min - 42.0).abs() < f64::EPSILON);
    assert!((s.max - 42.0).abs() < f64::EPSILON);
    assert!((s.avg - 42.0).abs() < f64::EPSILON);
}

#[test]
fn chart_stats_from_multiple_points() {
    let data = vec![(0.0, 10.0), (1.0, 20.0), (2.0, 30.0), (3.0, 40.0)];
    let s = ChartStats::from_data(&data);
    assert_eq!(s.count, 4);
    assert!((s.min - 10.0).abs() < f64::EPSILON);
    assert!((s.max - 40.0).abs() < f64::EPSILON);
    assert!((s.avg - 25.0).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CHARTS PANEL — ChartsAction
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn charts_action_is_export() {
    assert!(ChartsAction::ExportCsv { data: "x".to_string() }.is_export());
    assert!(ChartsAction::ExportJson { data: "x".to_string() }.is_export());
    assert!(ChartsAction::ExportPng.is_export());
    assert!(!ChartsAction::ClearHistory.is_export());
    assert!(!ChartsAction::SetChartType { chart_type: ChartType::Line }.is_export());
}

#[test]
fn charts_action_name_non_empty() {
    let actions = [
        ChartsAction::ExportCsv { data: String::new() },
        ChartsAction::ExportJson { data: String::new() },
        ChartsAction::ExportPng,
        ChartsAction::SetChartType { chart_type: ChartType::Bar },
        ChartsAction::SetDataSource { source: DataSource::FrameTiming },
        ChartsAction::ClearHistory,
    ];
    for a in &actions {
        assert!(!a.name().is_empty());
    }
}

#[test]
fn charts_action_display() {
    let a = ChartsAction::ClearHistory;
    let s = format!("{}", a);
    assert!(!s.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CHARTS PANEL construction
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn charts_panel_new() {
    let panel = ChartsPanel::new();
    assert_eq!(panel.chart_type(), ChartType::Line);
    assert_eq!(panel.data_source(), DataSource::FrameTiming);
    assert!(!panel.has_pending_actions());
}

#[test]
fn charts_panel_take_actions_empty() {
    let mut panel = ChartsPanel::new();
    let actions = panel.take_actions();
    assert!(actions.is_empty());
}
