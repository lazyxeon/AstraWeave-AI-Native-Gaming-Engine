// tools/aw_editor/src/panels/theme_manager.rs - Phase 5.5: Theme & Layout System
//
// Provides dark/light theme toggle, custom color schemes, and layout management
// with presets for different workflows (Modeling, Animation, Scripting).

use egui::{Color32, Context, Ui, Visuals};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Available editor themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EditorTheme {
    #[default]
    Dark,
    Light,
    HighContrast,
    Custom,
}

impl EditorTheme {
    pub fn name(&self) -> &str {
        match self {
            EditorTheme::Dark => "🌙 Dark",
            EditorTheme::Light => "☀️ Light",
            EditorTheme::HighContrast => "👁️ High Contrast",
            EditorTheme::Custom => "🎨 Custom",
        }
    }

    pub const ALL: [EditorTheme; 4] = [
        EditorTheme::Dark,
        EditorTheme::Light,
        EditorTheme::HighContrast,
        EditorTheme::Custom,
    ];

    /// Apply this theme to the egui context
    pub fn apply(&self, ctx: &Context, custom_colors: Option<&CustomColors>) {
        let visuals = match self {
            EditorTheme::Dark => Visuals::dark(),
            EditorTheme::Light => Visuals::light(),
            EditorTheme::HighContrast => Self::high_contrast_visuals(),
            EditorTheme::Custom => {
                if let Some(colors) = custom_colors {
                    colors.to_visuals()
                } else {
                    Visuals::dark()
                }
            }
        };
        ctx.set_visuals(visuals);
    }

    fn high_contrast_visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = Color32::BLACK;
        visuals.widgets.inactive.bg_fill = Color32::from_gray(20);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(0, 80, 160);
        visuals.widgets.active.bg_fill = Color32::from_rgb(0, 120, 200);
        visuals.selection.bg_fill = Color32::from_rgb(0, 150, 255);
        visuals.extreme_bg_color = Color32::BLACK;
        visuals.panel_fill = Color32::from_gray(10);
        visuals
    }
}

/// Custom color configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomColors {
    pub background: [u8; 3],
    pub panel_fill: [u8; 3],
    pub text: [u8; 3],
    pub accent: [u8; 3],
    pub selection: [u8; 3],
    pub warning: [u8; 3],
    pub error: [u8; 3],
    pub success: [u8; 3],
}

impl Default for CustomColors {
    fn default() -> Self {
        Self {
            background: [30, 30, 30],
            panel_fill: [40, 40, 40],
            text: [220, 220, 220],
            accent: [100, 150, 255],
            selection: [60, 100, 180],
            warning: [255, 180, 50],
            error: [255, 80, 80],
            success: [100, 200, 100],
        }
    }
}

impl CustomColors {
    pub fn to_visuals(&self) -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.extreme_bg_color =
            Color32::from_rgb(self.background[0], self.background[1], self.background[2]);
        visuals.panel_fill =
            Color32::from_rgb(self.panel_fill[0], self.panel_fill[1], self.panel_fill[2]);
        visuals.override_text_color =
            Some(Color32::from_rgb(self.text[0], self.text[1], self.text[2]));
        visuals.selection.bg_fill =
            Color32::from_rgb(self.selection[0], self.selection[1], self.selection[2]);
        visuals.hyperlink_color = Color32::from_rgb(self.accent[0], self.accent[1], self.accent[2]);
        visuals.warn_fg_color =
            Color32::from_rgb(self.warning[0], self.warning[1], self.warning[2]);
        visuals.error_fg_color = Color32::from_rgb(self.error[0], self.error[1], self.error[2]);
        visuals
    }

    pub fn color32(&self, field: &str) -> Color32 {
        match field {
            "background" => {
                Color32::from_rgb(self.background[0], self.background[1], self.background[2])
            }
            "panel_fill" => {
                Color32::from_rgb(self.panel_fill[0], self.panel_fill[1], self.panel_fill[2])
            }
            "text" => Color32::from_rgb(self.text[0], self.text[1], self.text[2]),
            "accent" => Color32::from_rgb(self.accent[0], self.accent[1], self.accent[2]),
            "selection" => {
                Color32::from_rgb(self.selection[0], self.selection[1], self.selection[2])
            }
            "warning" => Color32::from_rgb(self.warning[0], self.warning[1], self.warning[2]),
            "error" => Color32::from_rgb(self.error[0], self.error[1], self.error[2]),
            "success" => Color32::from_rgb(self.success[0], self.success[1], self.success[2]),
            _ => Color32::WHITE,
        }
    }
}

/// Layout preset for different workflows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayoutPreset {
    #[default]
    Default,
    Modeling,
    Animation,
    Scripting,
    Debugging,
    Compact,
}

impl LayoutPreset {
    pub fn name(&self) -> &str {
        match self {
            LayoutPreset::Default => "📐 Default",
            LayoutPreset::Modeling => "🗿 Modeling",
            LayoutPreset::Animation => "🎬 Animation",
            LayoutPreset::Scripting => "💻 Scripting",
            LayoutPreset::Debugging => "🐛 Debugging",
            LayoutPreset::Compact => "📱 Compact",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            LayoutPreset::Default => "Balanced layout for general editing",
            LayoutPreset::Modeling => "Maximized viewport, minimal panels",
            LayoutPreset::Animation => "Timeline visible, animation panels open",
            LayoutPreset::Scripting => "Behavior graph editor prominent",
            LayoutPreset::Debugging => "Console, profiler, and inspector expanded",
            LayoutPreset::Compact => "Minimal UI for small screens",
        }
    }

    pub const ALL: [LayoutPreset; 6] = [
        LayoutPreset::Default,
        LayoutPreset::Modeling,
        LayoutPreset::Animation,
        LayoutPreset::Scripting,
        LayoutPreset::Debugging,
        LayoutPreset::Compact,
    ];
}

/// Layout state for panel visibility and sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutState {
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_panel_height: f32,
    pub left_panel_visible: bool,
    pub right_panel_visible: bool,
    pub bottom_panel_visible: bool,
    pub expanded_sections: HashMap<String, bool>,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            left_panel_width: 300.0,
            right_panel_width: 350.0,
            bottom_panel_height: 200.0,
            left_panel_visible: true,
            right_panel_visible: true,
            bottom_panel_visible: true,
            expanded_sections: HashMap::new(),
        }
    }
}

impl LayoutState {
    pub fn for_preset(preset: LayoutPreset) -> Self {
        match preset {
            LayoutPreset::Default => Self::default(),
            LayoutPreset::Modeling => Self {
                left_panel_width: 200.0,
                right_panel_width: 250.0,
                bottom_panel_height: 100.0,
                left_panel_visible: true,
                right_panel_visible: true,
                bottom_panel_visible: false,
                expanded_sections: HashMap::new(),
            },
            LayoutPreset::Animation => Self {
                left_panel_width: 250.0,
                right_panel_width: 300.0,
                bottom_panel_height: 300.0,
                left_panel_visible: true,
                right_panel_visible: true,
                bottom_panel_visible: true,
                expanded_sections: [
                    ("Animation".to_string(), true),
                    ("Timeline".to_string(), true),
                ]
                .into_iter()
                .collect(),
            },
            LayoutPreset::Scripting => Self {
                left_panel_width: 350.0,
                right_panel_width: 300.0,
                bottom_panel_height: 250.0,
                left_panel_visible: true,
                right_panel_visible: true,
                bottom_panel_visible: true,
                expanded_sections: [
                    ("Behavior Graph".to_string(), true),
                    ("Console".to_string(), true),
                ]
                .into_iter()
                .collect(),
            },
            LayoutPreset::Debugging => Self {
                left_panel_width: 300.0,
                right_panel_width: 400.0,
                bottom_panel_height: 300.0,
                left_panel_visible: true,
                right_panel_visible: true,
                bottom_panel_visible: true,
                expanded_sections: [
                    ("Console".to_string(), true),
                    ("Profiler".to_string(), true),
                    ("Inspector".to_string(), true),
                ]
                .into_iter()
                .collect(),
            },
            LayoutPreset::Compact => Self {
                left_panel_width: 200.0,
                right_panel_width: 200.0,
                bottom_panel_height: 80.0,
                left_panel_visible: false,
                right_panel_visible: true,
                bottom_panel_visible: false,
                expanded_sections: HashMap::new(),
            },
        }
    }
}

/// Complete theme and layout preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorPreferences {
    pub theme: EditorTheme,
    pub custom_colors: CustomColors,
    pub layout_preset: LayoutPreset,
    pub layout_state: LayoutState,
    pub font_size: f32,
    pub animations_enabled: bool,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            theme: EditorTheme::Dark,
            custom_colors: CustomColors::default(),
            layout_preset: LayoutPreset::Default,
            layout_state: LayoutState::default(),
            font_size: 14.0,
            animations_enabled: true,
        }
    }
}

impl EditorPreferences {
    /// Load preferences from file
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(prefs) = toml::from_str(&content) {
                    return prefs;
                }
            }
        }
        Self::default()
    }

    /// Save preferences to file
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).unwrap_or_default();
        std::fs::write(path, content)
    }

    fn config_path() -> PathBuf {
        PathBuf::from("editor_preferences.toml")
    }

    /// Apply the current theme to an egui context
    pub fn apply_theme(&self, ctx: &Context) {
        self.theme.apply(ctx, Some(&self.custom_colors));
    }
}

/// Theme Manager Panel - Phase 5.5
pub struct ThemeManagerPanel {
    preferences: EditorPreferences,
    unsaved_changes: bool,
    show_custom_colors: bool,
}

impl Default for ThemeManagerPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManagerPanel {
    pub fn new() -> Self {
        Self {
            preferences: EditorPreferences::load(),
            unsaved_changes: false,
            show_custom_colors: false,
        }
    }

    pub fn preferences(&self) -> &EditorPreferences {
        &self.preferences
    }

    pub fn preferences_mut(&mut self) -> &mut EditorPreferences {
        self.unsaved_changes = true;
        &mut self.preferences
    }

    pub fn apply_theme(&self, ctx: &Context) {
        self.preferences.apply_theme(ctx);
    }

    pub fn set_layout_preset(&mut self, preset: LayoutPreset) {
        self.preferences.layout_preset = preset;
        self.preferences.layout_state = LayoutState::for_preset(preset);
        self.unsaved_changes = true;
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("🎨 Theme & Layout");
        ui.add_space(8.0);

        // Theme selection
        ui.group(|ui| {
            ui.label(egui::RichText::new("Theme").strong());
            ui.horizontal(|ui| {
                for theme in EditorTheme::ALL {
                    if ui
                        .selectable_label(self.preferences.theme == theme, theme.name())
                        .clicked()
                    {
                        self.preferences.theme = theme;
                        self.unsaved_changes = true;
                        self.show_custom_colors = theme == EditorTheme::Custom;
                    }
                }
            });
        });

        // Custom colors (only show when Custom theme is selected)
        if self.preferences.theme == EditorTheme::Custom || self.show_custom_colors {
            ui.add_space(4.0);
            ui.collapsing("🎨 Custom Colors", |ui| {
                let colors = &mut self.preferences.custom_colors;

                Self::color_picker(ui, "Background", &mut colors.background);
                Self::color_picker(ui, "Panel Fill", &mut colors.panel_fill);
                Self::color_picker(ui, "Text", &mut colors.text);
                Self::color_picker(ui, "Accent", &mut colors.accent);
                Self::color_picker(ui, "Selection", &mut colors.selection);
                Self::color_picker(ui, "Warning", &mut colors.warning);
                Self::color_picker(ui, "Error", &mut colors.error);
                Self::color_picker(ui, "Success", &mut colors.success);

                self.unsaved_changes = true;
            });
        }

        ui.add_space(8.0);

        // Layout presets
        ui.group(|ui| {
            ui.label(egui::RichText::new("Layout Preset").strong());

            for preset in LayoutPreset::ALL {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.preferences.layout_preset == preset, preset.name())
                        .clicked()
                    {
                        self.set_layout_preset(preset);
                    }
                    ui.label(
                        egui::RichText::new(preset.description())
                            .small()
                            .color(Color32::GRAY),
                    );
                });
            }
        });

        ui.add_space(8.0);

        // UI Settings
        ui.group(|ui| {
            ui.label(egui::RichText::new("UI Settings").strong());

            ui.horizontal(|ui| {
                ui.label("Font Size:");
                if ui
                    .add(
                        egui::Slider::new(&mut self.preferences.font_size, 10.0..=24.0)
                            .suffix("px"),
                    )
                    .changed()
                {
                    self.unsaved_changes = true;
                }
            });

            if ui
                .checkbox(
                    &mut self.preferences.animations_enabled,
                    "Enable animations",
                )
                .changed()
            {
                self.unsaved_changes = true;
            }
        });

        ui.add_space(8.0);

        // Panel visibility toggles
        ui.collapsing("📐 Panel Visibility", |ui| {
            let layout = &mut self.preferences.layout_state;

            if ui
                .checkbox(&mut layout.left_panel_visible, "Left Panel")
                .changed()
            {
                self.unsaved_changes = true;
            }
            if ui
                .checkbox(&mut layout.right_panel_visible, "Right Panel")
                .changed()
            {
                self.unsaved_changes = true;
            }
            if ui
                .checkbox(&mut layout.bottom_panel_visible, "Bottom Panel")
                .changed()
            {
                self.unsaved_changes = true;
            }

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Left Width:");
                if ui
                    .add(
                        egui::DragValue::new(&mut layout.left_panel_width)
                            .speed(1.0)
                            .range(100.0..=500.0),
                    )
                    .changed()
                {
                    self.unsaved_changes = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Right Width:");
                if ui
                    .add(
                        egui::DragValue::new(&mut layout.right_panel_width)
                            .speed(1.0)
                            .range(100.0..=500.0),
                    )
                    .changed()
                {
                    self.unsaved_changes = true;
                }
            });
        });

        ui.add_space(12.0);

        // Save/Reset buttons
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.unsaved_changes,
                    egui::Button::new("💾 Save Preferences"),
                )
                .clicked()
            {
                if let Err(e) = self.preferences.save() {
                    tracing::error!("Failed to save preferences: {}", e);
                } else {
                    self.unsaved_changes = false;
                }
            }

            if ui.button("🔄 Reset to Default").clicked() {
                self.preferences = EditorPreferences::default();
                self.unsaved_changes = true;
            }
        });

        if self.unsaved_changes {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("⚠️ Unsaved changes")
                    .color(Color32::YELLOW)
                    .small(),
            );
        }
    }

    fn color_picker(ui: &mut Ui, label: &str, color: &mut [u8; 3]) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            let mut color32 = Color32::from_rgb(color[0], color[1], color[2]);
            if ui.color_edit_button_srgba(&mut color32).changed() {
                *color = [color32.r(), color32.g(), color32.b()];
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_enum_variants() {
        assert_eq!(EditorTheme::Dark.name(), "🌙 Dark");
        assert_eq!(EditorTheme::Light.name(), "☀️ Light");
        assert_eq!(EditorTheme::ALL.len(), 4);
    }

    #[test]
    fn test_layout_preset_variants() {
        assert_eq!(LayoutPreset::Default.name(), "📐 Default");
        assert_eq!(LayoutPreset::ALL.len(), 6);
    }

    #[test]
    fn test_custom_colors_default() {
        let colors = CustomColors::default();
        assert_eq!(colors.background, [30, 30, 30]);
        assert_eq!(colors.accent, [100, 150, 255]);
    }

    #[test]
    fn test_layout_state_for_preset() {
        let default_state = LayoutState::for_preset(LayoutPreset::Default);
        assert!(default_state.left_panel_visible);

        let compact_state = LayoutState::for_preset(LayoutPreset::Compact);
        assert!(!compact_state.left_panel_visible);
    }

    #[test]
    fn test_editor_preferences_default() {
        let prefs = EditorPreferences::default();
        assert_eq!(prefs.theme, EditorTheme::Dark);
        assert_eq!(prefs.font_size, 14.0);
        assert!(prefs.animations_enabled);
    }
}
