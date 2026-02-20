//! Wave 2 mutation remediation tests — UI Editor + Foliage panels
//! Covers: WidgetType, AnchorPreset, UiWidget, UiCanvas, ScaleMode, UiStyle, EasingType,
//!         FoliageCategory, FoliageType, BrushTool, BrushSettings, DistributionType,
//!         FoliageLayer, FoliageTab, ProceduralRule

use aw_editor_lib::panels::ui_editor_panel::{
    AnchorPreset, EasingType, ScaleMode, UiCanvas, UiStyle, UiWidget, WidgetType,
};
use aw_editor_lib::panels::foliage_panel::{
    BrushSettings, BrushTool, DistributionType, FoliageCategory, FoliageLayer, FoliageTab,
    FoliageType, ProceduralRule,
};

// ═══════════════════════════════════════════════════════════════════════════════════
// WIDGET TYPE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn widget_type_all_count() {
    assert_eq!(WidgetType::all().len(), 13);
}

#[test]
fn widget_type_names() {
    assert_eq!(WidgetType::Panel.name(), "Panel");
    assert_eq!(WidgetType::Button.name(), "Button");
    assert_eq!(WidgetType::Label.name(), "Label");
    assert_eq!(WidgetType::Image.name(), "Image");
    assert_eq!(WidgetType::Slider.name(), "Slider");
    assert_eq!(WidgetType::Toggle.name(), "Toggle");
    assert_eq!(WidgetType::TextField.name(), "Text Field");
    assert_eq!(WidgetType::Dropdown.name(), "Dropdown");
    assert_eq!(WidgetType::ScrollView.name(), "Scroll View");
    assert_eq!(WidgetType::ProgressBar.name(), "Progress Bar");
    assert_eq!(WidgetType::Grid.name(), "Grid");
    assert_eq!(WidgetType::HorizontalLayout.name(), "Horizontal Layout");
    assert_eq!(WidgetType::VerticalLayout.name(), "Vertical Layout");
}

#[test]
fn widget_type_icons_non_empty() {
    for variant in WidgetType::all() {
        assert!(!variant.icon().is_empty(), "{:?} icon empty", variant);
    }
}

#[test]
fn widget_type_display_contains_name() {
    for variant in WidgetType::all() {
        let s = format!("{}", variant);
        assert!(s.contains(variant.name()), "Display missing name for {:?}", variant);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// ANCHOR PRESET
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn anchor_preset_all_count() {
    assert_eq!(AnchorPreset::all().len(), 12);
}

#[test]
fn anchor_preset_names() {
    assert_eq!(AnchorPreset::TopLeft.name(), "Top Left");
    assert_eq!(AnchorPreset::TopCenter.name(), "Top Center");
    assert_eq!(AnchorPreset::TopRight.name(), "Top Right");
    assert_eq!(AnchorPreset::MiddleLeft.name(), "Middle Left");
    assert_eq!(AnchorPreset::MiddleCenter.name(), "Middle Center");
    assert_eq!(AnchorPreset::MiddleRight.name(), "Middle Right");
    assert_eq!(AnchorPreset::BottomLeft.name(), "Bottom Left");
    assert_eq!(AnchorPreset::BottomCenter.name(), "Bottom Center");
    assert_eq!(AnchorPreset::BottomRight.name(), "Bottom Right");
    assert_eq!(AnchorPreset::StretchHorizontal.name(), "Stretch Horizontal");
    assert_eq!(AnchorPreset::StretchVertical.name(), "Stretch Vertical");
    assert_eq!(AnchorPreset::StretchFull.name(), "Stretch Full");
}

#[test]
fn anchor_preset_icons_non_empty() {
    for preset in AnchorPreset::all() {
        assert!(!preset.icon().is_empty(), "{:?} icon empty", preset);
    }
}

#[test]
fn anchor_preset_display() {
    for preset in AnchorPreset::all() {
        let s = format!("{}", preset);
        assert!(s.contains(preset.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// UI WIDGET DEFAULTS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn ui_widget_default_values() {
    let w = UiWidget::default();
    assert_eq!(w.id, 0);
    assert_eq!(w.name, "New Widget");
    assert_eq!(w.widget_type, WidgetType::Panel);
    assert!(w.enabled);
    assert!(w.visible);
    assert_eq!(w.position, [0.0, 0.0]);
    assert_eq!(w.size, [100.0, 50.0]);
    assert_eq!(w.anchor, AnchorPreset::TopLeft);
    assert_eq!(w.pivot, [0.0, 0.0]);
    assert!((w.rotation - 0.0).abs() < f32::EPSILON);
}

#[test]
fn ui_widget_default_appearance() {
    let w = UiWidget::default();
    assert!((w.border_width - 1.0).abs() < f32::EPSILON);
    assert!((w.corner_radius - 4.0).abs() < f32::EPSILON);
    assert!((w.opacity - 1.0).abs() < f32::EPSILON);
    assert!((w.font_size - 14.0).abs() < f32::EPSILON);
}

#[test]
fn ui_widget_default_layout() {
    let w = UiWidget::default();
    assert_eq!(w.padding, [5.0, 5.0, 5.0, 5.0]);
    assert_eq!(w.margin, [0.0, 0.0, 0.0, 0.0]);
    assert!((w.spacing - 5.0).abs() < f32::EPSILON);
}

#[test]
fn ui_widget_default_value_range() {
    let w = UiWidget::default();
    assert!((w.value - 0.0).abs() < f32::EPSILON);
    assert!((w.min_value - 0.0).abs() < f32::EPSILON);
    assert!((w.max_value - 1.0).abs() < f32::EPSILON);
}

#[test]
fn ui_widget_default_hierarchy() {
    let w = UiWidget::default();
    assert!(w.parent_id.is_none());
    assert!(w.children.is_empty());
}

#[test]
fn ui_widget_default_events() {
    let w = UiWidget::default();
    assert!(w.on_click.is_empty());
    assert!(w.on_value_changed.is_empty());
    assert!(w.on_hover_enter.is_empty());
    assert!(w.on_hover_exit.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════════
// SCALE MODE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn scale_mode_all_count() {
    assert_eq!(ScaleMode::all().len(), 3);
}

#[test]
fn scale_mode_names() {
    assert_eq!(ScaleMode::ConstantPixelSize.name(), "Constant Pixel Size");
    assert_eq!(ScaleMode::ScaleWithScreenSize.name(), "Scale With Screen Size");
    assert_eq!(ScaleMode::ConstantPhysicalSize.name(), "Constant Physical Size");
}

#[test]
fn scale_mode_icons_non_empty() {
    for mode in ScaleMode::all() {
        assert!(!mode.icon().is_empty(), "{:?} icon empty", mode);
    }
}

#[test]
fn scale_mode_default_is_constant_pixel() {
    assert_eq!(ScaleMode::default(), ScaleMode::ConstantPixelSize);
}

#[test]
fn scale_mode_display() {
    for mode in ScaleMode::all() {
        let s = format!("{}", mode);
        assert!(s.contains(mode.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// UI CANVAS DEFAULTS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn ui_canvas_default() {
    let c = UiCanvas::default();
    assert_eq!(c.id, 0);
    assert_eq!(c.name, "New Canvas");
    assert_eq!(c.resolution, [1920, 1080]);
    assert_eq!(c.scale_mode, ScaleMode::ScaleWithScreenSize);
    assert_eq!(c.reference_resolution, [1920, 1080]);
    assert!((c.match_width_or_height - 0.5).abs() < f32::EPSILON);
    assert!(c.widgets.is_empty());
    assert_eq!(c.render_order, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// UI STYLE DEFAULTS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn ui_style_default_typography() {
    let s = UiStyle::default();
    assert_eq!(s.font_family, "Default");
    assert!((s.font_size_small - 12.0).abs() < f32::EPSILON);
    assert!((s.font_size_normal - 14.0).abs() < f32::EPSILON);
    assert!((s.font_size_large - 18.0).abs() < f32::EPSILON);
    assert!((s.font_size_heading - 24.0).abs() < f32::EPSILON);
}

#[test]
fn ui_style_default_spacing() {
    let s = UiStyle::default();
    assert!((s.padding_small - 4.0).abs() < f32::EPSILON);
    assert!((s.padding_normal - 8.0).abs() < f32::EPSILON);
    assert!((s.padding_large - 16.0).abs() < f32::EPSILON);
    assert!((s.corner_radius - 4.0).abs() < f32::EPSILON);
    assert!((s.border_width - 1.0).abs() < f32::EPSILON);
}

#[test]
fn ui_style_default_transitions() {
    let s = UiStyle::default();
    assert!((s.transition_duration - 0.15).abs() < f32::EPSILON);
    assert!((s.hover_scale - 1.02).abs() < f32::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// EASING TYPE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn easing_type_all_count() {
    assert_eq!(EasingType::all().len(), 7);
}

#[test]
fn easing_type_names() {
    assert_eq!(EasingType::Linear.name(), "Linear");
    assert_eq!(EasingType::EaseIn.name(), "Ease In");
    assert_eq!(EasingType::EaseOut.name(), "Ease Out");
    assert_eq!(EasingType::EaseInOut.name(), "Ease In-Out");
    assert_eq!(EasingType::Bounce.name(), "Bounce");
    assert_eq!(EasingType::Elastic.name(), "Elastic");
    assert_eq!(EasingType::Back.name(), "Back");
}

#[test]
fn easing_type_icons_non_empty() {
    for e in EasingType::all() {
        assert!(!e.icon().is_empty(), "{:?} icon empty", e);
    }
}

#[test]
fn easing_type_default() {
    assert_eq!(EasingType::default(), EasingType::Linear);
}

#[test]
fn easing_type_display() {
    for e in EasingType::all() {
        let s = format!("{}", e);
        assert!(s.contains(e.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// FOLIAGE CATEGORY
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn foliage_category_all_count() {
    assert_eq!(FoliageCategory::all().len(), 6);
}

#[test]
fn foliage_category_names() {
    assert_eq!(FoliageCategory::Trees.name(), "Trees");
    assert_eq!(FoliageCategory::Shrubs.name(), "Shrubs");
    assert_eq!(FoliageCategory::Grass.name(), "Grass");
    assert_eq!(FoliageCategory::Flowers.name(), "Flowers");
    assert_eq!(FoliageCategory::Rocks.name(), "Rocks");
    assert_eq!(FoliageCategory::Custom.name(), "Custom");
}

#[test]
fn foliage_category_icons_non_empty() {
    for cat in FoliageCategory::all() {
        assert!(!cat.icon().is_empty(), "{:?} icon empty", cat);
    }
}

#[test]
fn foliage_category_display() {
    for cat in FoliageCategory::all() {
        let s = format!("{}", cat);
        assert!(s.contains(cat.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// FOLIAGE TYPE DEFAULTS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn foliage_type_default_values() {
    let ft = FoliageType::default();
    assert_eq!(ft.id, 0);
    assert_eq!(ft.name, "New Foliage");
    assert_eq!(ft.category, FoliageCategory::Grass);
    assert!(ft.enabled);
    assert!((ft.density - 10.0).abs() < f32::EPSILON);
    assert!((ft.min_scale - 0.8).abs() < f32::EPSILON);
    assert!((ft.max_scale - 1.2).abs() < f32::EPSILON);
}

#[test]
fn foliage_type_default_placement() {
    let ft = FoliageType::default();
    assert!((ft.min_slope - 0.0).abs() < f32::EPSILON);
    assert!((ft.max_slope - 45.0).abs() < f32::EPSILON);
    assert!((ft.min_altitude - -1000.0).abs() < f32::EPSILON);
    assert!((ft.max_altitude - 1000.0).abs() < f32::EPSILON);
}

#[test]
fn foliage_type_default_rendering() {
    let ft = FoliageType::default();
    assert!(ft.cast_shadow);
    assert!((ft.cull_distance - 1000.0).abs() < f32::EPSILON);
    assert_eq!(ft.lod_distances.len(), 4);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// BRUSH TOOL
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn brush_tool_all_count() {
    assert_eq!(BrushTool::all().len(), 5);
}

#[test]
fn brush_tool_names() {
    assert_eq!(BrushTool::Paint.name(), "Paint");
    assert_eq!(BrushTool::Erase.name(), "Erase");
    assert_eq!(BrushTool::Select.name(), "Select");
    assert_eq!(BrushTool::Reapply.name(), "Reapply");
    assert_eq!(BrushTool::SinglePlace.name(), "Single Place");
}

#[test]
fn brush_tool_icons_non_empty() {
    for tool in BrushTool::all() {
        assert!(!tool.icon().is_empty(), "{:?} icon empty", tool);
    }
}

#[test]
fn brush_tool_display() {
    for tool in BrushTool::all() {
        let s = format!("{}", tool);
        assert!(s.contains(tool.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// BRUSH SETTINGS
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn brush_settings_defaults() {
    let s = BrushSettings::default();
    assert!((s.radius - 5.0).abs() < f32::EPSILON);
    assert!((s.falloff - 0.5).abs() < f32::EPSILON);
    assert!((s.density - 1.0).abs() < f32::EPSILON);
    assert!((s.flow - 1.0).abs() < f32::EPSILON);
    assert!(!s.use_mask);
    assert_eq!(s.mask_channel, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// DISTRIBUTION TYPE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn distribution_type_all_count() {
    assert_eq!(DistributionType::all().len(), 4);
}

#[test]
fn distribution_type_names() {
    assert_eq!(DistributionType::Random.name(), "Random");
    assert_eq!(DistributionType::Uniform.name(), "Uniform");
    assert_eq!(DistributionType::Clustered.name(), "Clustered");
    assert_eq!(DistributionType::PoissonDisc.name(), "Poisson Disc");
}

#[test]
fn distribution_type_icons_non_empty() {
    for dt in DistributionType::all() {
        assert!(!dt.icon().is_empty(), "{:?} icon empty", dt);
    }
}

#[test]
fn distribution_type_default() {
    assert_eq!(DistributionType::default(), DistributionType::Random);
}

#[test]
fn distribution_type_display() {
    for dt in DistributionType::all() {
        let s = format!("{}", dt);
        assert!(s.contains(dt.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// FOLIAGE LAYER
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn foliage_layer_defaults() {
    let l = FoliageLayer::default();
    assert_eq!(l.id, 0);
    assert_eq!(l.name, "Default Layer");
    assert!(l.visible);
    assert!(!l.locked);
    assert!(l.foliage_types.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════════
// FOLIAGE TAB
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn foliage_tab_all_count() {
    assert_eq!(FoliageTab::all().len(), 6);
}

#[test]
fn foliage_tab_names() {
    assert_eq!(FoliageTab::Paint.name(), "Paint");
    assert_eq!(FoliageTab::Types.name(), "Types");
    assert_eq!(FoliageTab::Settings.name(), "Settings");
    assert_eq!(FoliageTab::Procedural.name(), "Procedural");
    assert_eq!(FoliageTab::Layers.name(), "Layers");
    assert_eq!(FoliageTab::Statistics.name(), "Statistics");
}

#[test]
fn foliage_tab_icons_non_empty() {
    for tab in FoliageTab::all() {
        assert!(!tab.icon().is_empty(), "{:?} icon empty", tab);
    }
}

#[test]
fn foliage_tab_default_is_paint() {
    assert_eq!(FoliageTab::default(), FoliageTab::Paint);
}

#[test]
fn foliage_tab_display() {
    for tab in FoliageTab::all() {
        let s = format!("{}", tab);
        assert!(s.contains(tab.name()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// PROCEDURAL RULE
// ═══════════════════════════════════════════════════════════════════════════════════

#[test]
fn procedural_rule_defaults() {
    let r = ProceduralRule::default();
    assert_eq!(r.id, 0);
    assert_eq!(r.name, "New Rule");
    assert!(r.enabled);
    assert!(r.target_types.is_empty());
    assert_eq!(r.area_size, [100.0, 100.0]);
    assert!(!r.use_noise);
    assert!((r.noise_scale - 10.0).abs() < f32::EPSILON);
    assert!((r.noise_threshold - 0.5).abs() < f32::EPSILON);
    assert_eq!(r.distribution_type, DistributionType::Random);
    assert!((r.clustering - 0.0).abs() < f32::EPSILON);
    assert!((r.spacing - 1.0).abs() < f32::EPSILON);
}
