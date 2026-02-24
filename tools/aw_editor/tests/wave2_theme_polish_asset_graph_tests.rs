//! Wave 2 mutation remediation tests — Theme Manager + Polish + Asset Store + Graph + Transform panels
//! Covers: EditorTheme, CustomColors, LayoutPreset, LayoutState, EditorPreferences,
//!         TransitionStyle, CreditsConfig, UiPolishSettings, AchievementSettings, GameFeelSettings, Achievement,
//!         ReadinessLevel, ChecklistItem, AssetChecklist, AssetStoreCategory, ReadyAsset,
//!         GraphType, NodeTemplate, GraphStats, TransformPanel

use aw_editor_lib::panels::graph_panel::{GraphType, NodeTemplate};
use aw_editor_lib::panels::polish_panel::{
    Achievement, AchievementSettings, CreditsConfig, GameFeelSettings, TransitionStyle,
    UiPolishSettings,
};
use aw_editor_lib::panels::ready_asset_store_panel::{
    AssetChecklist, AssetStoreCategory, ChecklistItem, ReadinessLevel,
};
use aw_editor_lib::panels::theme_manager::{
    CustomColors, EditorPreferences, EditorTheme, LayoutPreset, LayoutState,
};
use aw_editor_lib::panels::transform_panel::TransformPanel;

// ═══════════════════════════════════════════════════════════════════════════════════
// EDITOR THEME
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn editor_theme_all_count() {
    assert_eq!(EditorTheme::all().len(), 4);
}

#[test]
fn editor_theme_names_non_empty() {
    for t in EditorTheme::all() {
        assert!(!t.name().is_empty(), "{:?} name empty", t);
    }
}

#[test]
fn editor_theme_icons_non_empty() {
    for t in EditorTheme::all() {
        assert!(!t.icon().is_empty(), "{:?} icon empty", t);
    }
}

#[test]
fn editor_theme_is_dark() {
    assert!(EditorTheme::Dark.is_dark());
    assert!(!EditorTheme::Light.is_dark());
    assert!(EditorTheme::HighContrast.is_dark());
    assert!(!EditorTheme::Custom.is_dark());
}

#[test]
fn editor_theme_default_is_dark() {
    assert_eq!(EditorTheme::default(), EditorTheme::Dark);
}

#[test]
fn editor_theme_display() {
    for t in EditorTheme::all() {
        let s = format!("{}", t);
        assert!(s.contains(t.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CUSTOM COLORS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn custom_colors_defaults() {
    let c = CustomColors::default();
    assert_eq!(c.background, [30, 30, 30]);
    assert_eq!(c.panel_fill, [40, 40, 40]);
    assert_eq!(c.text, [220, 220, 220]);
    assert_eq!(c.accent, [100, 150, 255]);
    assert_eq!(c.selection, [60, 100, 180]);
    assert_eq!(c.warning, [255, 180, 50]);
    assert_eq!(c.error, [255, 80, 80]);
    assert_eq!(c.success, [100, 200, 100]);
}

#[test]
fn custom_colors_color32_known_fields() {
    let c = CustomColors::default();
    let fields = [
        "background",
        "panel_fill",
        "text",
        "accent",
        "selection",
        "warning",
        "error",
        "success",
    ];
    for field in &fields {
        let color = c.color32(field);
        // Just verify it doesn't panic and returns non-transparent
        assert_ne!(color.a(), 0);
    }
}

#[test]
fn custom_colors_color32_unknown_is_white() {
    let c = CustomColors::default();
    let color = c.color32("nonexistent");
    assert_eq!(color, egui::Color32::WHITE);
}

#[test]
fn custom_colors_to_visuals_no_panic() {
    let c = CustomColors::default();
    let _visuals = c.to_visuals();
}

// ═══════════════════════════════════════════════════════════════════════════════════
// LAYOUT PRESET
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn layout_preset_all_count() {
    assert_eq!(LayoutPreset::all().len(), 6);
}

#[test]
fn layout_preset_names_non_empty() {
    for p in LayoutPreset::all() {
        assert!(!p.name().is_empty(), "{:?} name empty", p);
    }
}

#[test]
fn layout_preset_icons_non_empty() {
    for p in LayoutPreset::all() {
        assert!(!p.icon().is_empty(), "{:?} icon empty", p);
    }
}

#[test]
fn layout_preset_description_non_empty() {
    for p in LayoutPreset::all() {
        assert!(!p.description().is_empty(), "{:?} desc empty", p);
    }
}

#[test]
fn layout_preset_is_development() {
    assert!(!LayoutPreset::Default.is_development());
    assert!(!LayoutPreset::Modeling.is_development());
    assert!(LayoutPreset::Scripting.is_development());
    assert!(LayoutPreset::Debugging.is_development());
    assert!(!LayoutPreset::Compact.is_development());
}

#[test]
fn layout_preset_is_minimal() {
    assert!(!LayoutPreset::Default.is_minimal());
    assert!(LayoutPreset::Modeling.is_minimal());
    assert!(!LayoutPreset::Animation.is_minimal());
    assert!(LayoutPreset::Compact.is_minimal());
}

#[test]
fn layout_preset_default_is_default() {
    assert_eq!(LayoutPreset::default(), LayoutPreset::Default);
}

#[test]
fn layout_preset_display() {
    for p in LayoutPreset::all() {
        let s = format!("{}", p);
        assert!(s.contains(p.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// LAYOUT STATE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn layout_state_defaults() {
    let s = LayoutState::default();
    assert!((s.left_panel_width - 300.0).abs() < f32::EPSILON);
    assert!((s.right_panel_width - 350.0).abs() < f32::EPSILON);
    assert!((s.bottom_panel_height - 200.0).abs() < f32::EPSILON);
    assert!(s.left_panel_visible);
    assert!(s.right_panel_visible);
    assert!(s.bottom_panel_visible);
}

#[test]
fn layout_state_for_preset_compact() {
    let s = LayoutState::for_preset(LayoutPreset::Compact);
    assert!(!s.left_panel_visible);
    assert!(!s.bottom_panel_visible);
}

#[test]
fn layout_state_for_preset_modeling() {
    let s = LayoutState::for_preset(LayoutPreset::Modeling);
    assert!(!s.bottom_panel_visible);
}

#[test]
fn layout_state_for_preset_debugging() {
    let s = LayoutState::for_preset(LayoutPreset::Debugging);
    assert!(s.bottom_panel_visible);
    assert!(s.expanded_sections.contains_key("Console"));
    assert!(s.expanded_sections.contains_key("Profiler"));
}

// ═══════════════════════════════════════════════════════════════════════════════════
// EDITOR PREFERENCES
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn editor_preferences_defaults() {
    let p = EditorPreferences::default();
    assert_eq!(p.theme, EditorTheme::Dark);
    assert_eq!(p.layout_preset, LayoutPreset::Default);
    assert!((p.font_size - 14.0).abs() < f32::EPSILON);
    assert!(p.animations_enabled);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// TRANSITION STYLE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn transition_style_all_count() {
    assert_eq!(TransitionStyle::all().len(), 7);
}

#[test]
fn transition_style_names_non_empty() {
    for t in TransitionStyle::all() {
        assert!(!t.name().is_empty(), "{:?} name empty", t);
    }
}

#[test]
fn transition_style_icons_non_empty() {
    for t in TransitionStyle::all() {
        assert!(!t.icon().is_empty(), "{:?} icon empty", t);
    }
}

#[test]
fn transition_style_is_slide() {
    assert!(!TransitionStyle::Fade.is_slide());
    assert!(TransitionStyle::SlideRight.is_slide());
    assert!(TransitionStyle::SlideLeft.is_slide());
    assert!(TransitionStyle::SlideTop.is_slide());
    assert!(TransitionStyle::SlideBottom.is_slide());
    assert!(!TransitionStyle::Dissolve.is_slide());
    assert!(!TransitionStyle::Instant.is_slide());
}

#[test]
fn transition_style_is_immediate() {
    for t in TransitionStyle::all() {
        if *t == TransitionStyle::Instant {
            assert!(t.is_immediate());
        } else {
            assert!(!t.is_immediate());
        }
    }
}

#[test]
fn transition_style_default_is_fade() {
    assert_eq!(TransitionStyle::default(), TransitionStyle::Fade);
}

#[test]
fn transition_style_display() {
    for t in TransitionStyle::all() {
        let s = format!("{}", t);
        assert!(s.contains(t.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// POLISH — CreditsConfig
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn credits_config_defaults() {
    let c = CreditsConfig::default();
    assert_eq!(c.game_title, "Your Game");
    assert_eq!(c.subtitle, "Made with AstraWeave");
    assert!(!c.entries.is_empty());
    assert!((c.scroll_speed - 50.0).abs() < f32::EPSILON);
    assert!(c.music_enabled);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// POLISH — UiPolishSettings
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn ui_polish_defaults() {
    let s = UiPolishSettings::default();
    assert!(s.animations_enabled);
    assert!((s.animation_speed - 1.0).abs() < f32::EPSILON);
    assert!(s.sounds_enabled);
    assert!(s.transitions_enabled);
    assert!((s.transition_duration - 0.3).abs() < f32::EPSILON);
    assert_eq!(s.transition_style, TransitionStyle::Fade);
    assert!(s.button_hover_enabled);
    assert!(s.tooltips_enabled);
    assert!((s.tooltip_delay - 0.5).abs() < f32::EPSILON);
    assert!(s.notifications_enabled);
    assert!((s.notification_duration - 3.0).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// POLISH — AchievementSettings
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn achievement_settings_defaults() {
    let s = AchievementSettings::default();
    assert!(s.enabled);
    assert!(s.achievements.is_empty());
    assert!(s.show_notifications);
    assert!((s.notification_duration - 5.0).abs() < f32::EPSILON);
    assert!(s.unlock_sound_enabled);
    assert!(!s.steam_integration);
}

#[test]
fn achievement_new() {
    let a = Achievement::new("first_kill", "First Kill", "Defeat an enemy");
    assert_eq!(a.id, "first_kill");
    assert_eq!(a.name, "First Kill");
    assert_eq!(a.description, "Defeat an enemy");
    assert!(!a.hidden);
    assert_eq!(a.points, 10);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// POLISH — GameFeelSettings
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn game_feel_defaults() {
    let s = GameFeelSettings::default();
    assert!(s.screen_shake_enabled);
    assert!((s.screen_shake_intensity - 1.0).abs() < f32::EPSILON);
    assert!(s.hit_stop_enabled);
    assert_eq!(s.hit_stop_duration, 50);
    assert!(s.camera_zoom_enabled);
    assert!(s.particles_enabled);
    assert!((s.particle_density - 1.0).abs() < f32::EPSILON);
    assert!(!s.chromatic_aberration);
    assert!(!s.motion_blur);
    assert_eq!(s.motion_blur_samples, 8);
    assert!(s.vignette_enabled);
    assert!((s.vignette_intensity - 0.3).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// ASSET STORE — ReadinessLevel
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn readiness_level_all_count() {
    assert_eq!(ReadinessLevel::all().len(), 5);
}

#[test]
fn readiness_level_names_non_empty() {
    for r in ReadinessLevel::all() {
        assert!(!r.name().is_empty(), "{:?} name empty", r);
    }
}

#[test]
fn readiness_level_icons_non_empty() {
    for r in ReadinessLevel::all() {
        assert!(!r.icon().is_empty(), "{:?} icon empty", r);
    }
}

#[test]
fn readiness_level_description_non_empty() {
    for r in ReadinessLevel::all() {
        assert!(!r.description().is_empty(), "{:?} desc empty", r);
    }
}

#[test]
fn readiness_level_min_requirements_monotonic() {
    let reqs: Vec<u32> = ReadinessLevel::all()
        .iter()
        .map(|r| r.min_requirements())
        .collect();
    for w in reqs.windows(2) {
        assert!(w[0] <= w[1], "Requirements not monotonic: {:?}", reqs);
    }
}

#[test]
fn readiness_level_ordering() {
    assert!(ReadinessLevel::NotReady < ReadinessLevel::Basic);
    assert!(ReadinessLevel::Basic < ReadinessLevel::Standard);
    assert!(ReadinessLevel::Standard < ReadinessLevel::Production);
    assert!(ReadinessLevel::Production < ReadinessLevel::Premium);
}

#[test]
fn readiness_level_default_is_not_ready() {
    assert_eq!(ReadinessLevel::default(), ReadinessLevel::NotReady);
}

#[test]
fn readiness_level_display() {
    for r in ReadinessLevel::all() {
        let s = format!("{}", r);
        assert!(s.contains(r.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// ASSET STORE — ChecklistItem
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn checklist_item_all_count() {
    assert_eq!(ChecklistItem::all().len(), 14);
}

#[test]
fn checklist_item_core_count() {
    assert_eq!(ChecklistItem::core_items().len(), 7);
}

#[test]
fn checklist_item_names_non_empty() {
    for i in ChecklistItem::all() {
        assert!(!i.name().is_empty(), "{:?} name empty", i);
    }
}

#[test]
fn checklist_item_icons_non_empty() {
    for i in ChecklistItem::all() {
        assert!(!i.icon().is_empty(), "{:?} icon empty", i);
    }
}

#[test]
fn checklist_item_description_non_empty() {
    for i in ChecklistItem::all() {
        assert!(!i.description().is_empty(), "{:?} desc empty", i);
    }
}

#[test]
fn checklist_item_is_core() {
    assert!(ChecklistItem::HasMesh.is_core());
    assert!(ChecklistItem::HasMaterial.is_core());
    assert!(ChecklistItem::HasCollider.is_core());
    assert!(!ChecklistItem::HasThumbnail.is_core());
    assert!(!ChecklistItem::HasVariants.is_core());
    assert!(!ChecklistItem::HasAnimations.is_core());
}

#[test]
fn checklist_item_display() {
    for i in ChecklistItem::all() {
        let s = format!("{}", i);
        assert!(s.contains(i.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// ASSET STORE — AssetChecklist
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn asset_checklist_new_empty() {
    let c = AssetChecklist::new();
    assert_eq!(c.passed_count(), 0);
    assert_eq!(c.failed_count(), 0);
    assert!(c.warnings.is_empty());
}

#[test]
fn asset_checklist_pass_item() {
    let mut c = AssetChecklist::new();
    c.pass(ChecklistItem::HasMesh);
    assert_eq!(c.passed_count(), 1);
    assert!(c.is_passed(ChecklistItem::HasMesh));
}

#[test]
fn asset_checklist_fail_item() {
    let mut c = AssetChecklist::new();
    c.fail(ChecklistItem::HasCollider);
    assert_eq!(c.failed_count(), 1);
    assert!(!c.is_passed(ChecklistItem::HasCollider));
}

#[test]
fn asset_checklist_pass_removes_from_failed() {
    let mut c = AssetChecklist::new();
    c.fail(ChecklistItem::HasMesh);
    assert_eq!(c.failed_count(), 1);
    c.pass(ChecklistItem::HasMesh);
    assert_eq!(c.failed_count(), 0);
    assert!(c.is_passed(ChecklistItem::HasMesh));
}

#[test]
fn asset_checklist_fail_removes_from_passed() {
    let mut c = AssetChecklist::new();
    c.pass(ChecklistItem::HasMesh);
    assert!(c.is_passed(ChecklistItem::HasMesh));
    c.fail(ChecklistItem::HasMesh);
    assert!(!c.is_passed(ChecklistItem::HasMesh));
    assert_eq!(c.passed_count(), 0);
}

#[test]
fn asset_checklist_readiness_not_ready() {
    let c = AssetChecklist::new();
    assert_eq!(c.readiness(), ReadinessLevel::NotReady);
}

#[test]
fn asset_checklist_readiness_basic() {
    let mut c = AssetChecklist::new();
    for item in ChecklistItem::all().iter().take(3) {
        c.pass(*item);
    }
    assert_eq!(c.readiness(), ReadinessLevel::Basic);
}

#[test]
fn asset_checklist_readiness_premium() {
    let mut c = AssetChecklist::new();
    for item in ChecklistItem::all().iter().take(9) {
        c.pass(*item);
    }
    assert_eq!(c.readiness(), ReadinessLevel::Premium);
}

#[test]
fn asset_checklist_core_passed_count() {
    let mut c = AssetChecklist::new();
    c.pass(ChecklistItem::HasMesh);
    c.pass(ChecklistItem::HasMaterial);
    c.pass(ChecklistItem::HasThumbnail); // non-core
    assert_eq!(c.core_passed_count(), 2);
}

#[test]
fn asset_checklist_completion_percentage() {
    let c = AssetChecklist::new();
    assert!((c.completion_percentage() - 0.0).abs() < f32::EPSILON);

    let mut c2 = AssetChecklist::new();
    for item in ChecklistItem::all() {
        c2.pass(*item);
    }
    assert!((c2.completion_percentage() - 100.0).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// ASSET STORE — AssetStoreCategory
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn asset_store_category_all_count() {
    assert_eq!(AssetStoreCategory::all().len(), 13);
}

#[test]
fn asset_store_category_names_non_empty() {
    for c in AssetStoreCategory::all() {
        assert!(!c.name().is_empty(), "{:?} name empty", c);
    }
}

#[test]
fn asset_store_category_icons_non_empty() {
    for c in AssetStoreCategory::all() {
        assert!(!c.icon().is_empty(), "{:?} icon empty", c);
    }
}

#[test]
fn asset_store_category_related_tags() {
    assert!(AssetStoreCategory::All.related_tags().is_empty());
    assert!(!AssetStoreCategory::Nature.related_tags().is_empty());
    assert!(AssetStoreCategory::Nature.related_tags().contains(&"tree"));
    assert!(AssetStoreCategory::Weapons
        .related_tags()
        .contains(&"sword"));
}

#[test]
fn asset_store_category_default_is_all() {
    assert_eq!(AssetStoreCategory::default(), AssetStoreCategory::All);
}

#[test]
fn asset_store_category_display() {
    for c in AssetStoreCategory::all() {
        let s = format!("{}", c);
        assert!(s.contains(c.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GRAPH PANEL — GraphType
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn graph_type_all_count() {
    assert_eq!(GraphType::all().len(), 6);
}

#[test]
fn graph_type_names_non_empty() {
    for g in GraphType::all() {
        assert!(!g.name().is_empty(), "{:?} name empty", g);
    }
}

#[test]
fn graph_type_icons_non_empty() {
    for g in GraphType::all() {
        assert!(!g.icon().is_empty(), "{:?} icon empty", g);
    }
}

#[test]
fn graph_type_display() {
    for g in GraphType::all() {
        let s = format!("{}", g);
        assert!(s.contains(g.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// GRAPH PANEL — NodeTemplate
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn node_template_new() {
    let t = NodeTemplate::new("Test", "Category");
    assert_eq!(t.name, "Test");
    assert_eq!(t.category, "Category");
    assert!(t.inputs.is_empty());
    assert!(t.outputs.is_empty());
    assert!(t.color.is_none());
}

#[test]
fn node_template_with_color() {
    let t = NodeTemplate::new("Test", "Cat").with_color(egui::Color32::RED);
    assert_eq!(t.color, Some(egui::Color32::RED));
}

// ═══════════════════════════════════════════════════════════════════════════════════
// TRANSFORM PANEL
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn transform_panel_new() {
    let panel = TransformPanel::new();
    assert!(panel.get_transform().is_none());
}

#[test]
fn transform_panel_add_and_select_entity() {
    let mut panel = TransformPanel::new();
    let default_transform = Default::default();
    let id = panel.add_entity("TestEntity".to_string(), default_transform);
    panel.select_entity(id);
    assert!(panel.get_transform().is_some());
}

#[test]
fn transform_panel_remove_entity_clears_selection() {
    let mut panel = TransformPanel::new();
    let default_transform = Default::default();
    let id = panel.add_entity("TestEntity".to_string(), default_transform);
    panel.select_entity(id);
    assert!(panel.get_transform().is_some());
    panel.remove_entity(id);
    assert!(panel.get_transform().is_none());
}

#[test]
fn transform_panel_clear_selection() {
    let mut panel = TransformPanel::new();
    let default_transform = Default::default();
    let id = panel.add_entity("E".to_string(), default_transform);
    panel.select_entity(id);
    panel.clear_selection();
    assert!(panel.get_transform().is_none());
}

#[test]
fn transform_panel_start_translate_no_selection() {
    let mut panel = TransformPanel::new();
    panel.clear_selection();
    panel.start_translate(); // Should not panic
}

#[test]
fn transform_panel_start_rotate_no_selection() {
    let mut panel = TransformPanel::new();
    panel.clear_selection();
    panel.start_rotate(); // Should not panic
}

#[test]
fn transform_panel_start_scale_no_selection() {
    let mut panel = TransformPanel::new();
    panel.clear_selection();
    panel.start_scale(); // Should not panic
}
