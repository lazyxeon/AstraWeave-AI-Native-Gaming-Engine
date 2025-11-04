use astract::advanced::{ColorPicker, RangeSlider, TreeNode, TreeView};
use egui::{Color32, Ui};

/// Panel demonstrating advanced widgets with game engine use cases
pub struct AdvancedWidgetsPanel {
    // Color picker examples
    ambient_color: ColorPicker,
    directional_light_color: ColorPicker,
    fog_color: ColorPicker,

    // Tree view examples
    scene_hierarchy: TreeView,
    asset_browser: TreeView,

    // Range slider examples
    camera_distance: RangeSlider,
    player_level_range: RangeSlider,
    audio_frequency: RangeSlider,

    // Demo state
    initialized: bool,
}

impl Default for AdvancedWidgetsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl AdvancedWidgetsPanel {
    pub fn new() -> Self {
        Self {
            // Color pickers with game engine defaults
            ambient_color: ColorPicker::new()
                .with_color(Color32::from_rgb(50, 50, 70))
                .show_alpha(false)
                .width(260.0),

            directional_light_color: ColorPicker::new()
                .with_color(Color32::from_rgb(255, 244, 214))
                .show_alpha(false)
                .width(260.0),

            fog_color: ColorPicker::new()
                .with_color(Color32::from_rgb(180, 180, 200))
                .show_alpha(true)
                .width(260.0),

            // Tree views (will be populated in init)
            scene_hierarchy: TreeView::new().with_indent(16.0),
            asset_browser: TreeView::new().with_indent(16.0),

            // Range sliders with game-relevant ranges
            camera_distance: RangeSlider::new(1.0, 100.0)
                .with_min(5.0)
                .with_max(25.0)
                .step(1.0)
                .suffix(" m")
                .width(280.0),

            player_level_range: RangeSlider::new(1.0, 100.0)
                .with_min(10.0)
                .with_max(50.0)
                .step(1.0)
                .prefix("Lv ")
                .width(280.0),

            audio_frequency: RangeSlider::new(20.0, 20000.0)
                .with_min(200.0)
                .with_max(8000.0)
                .step(100.0)
                .suffix(" Hz")
                .width(280.0),

            initialized: false,
        }
    }

    /// Initialize tree view data (only once)
    fn initialize_tree_views(&mut self) {
        if self.initialized {
            return;
        }

        // Scene Hierarchy: Typical game scene structure
        let world = self
            .scene_hierarchy
            .add_node(TreeNode::new(1, "World").with_icon("🌍"));

        let environment = self
            .scene_hierarchy
            .add_node(TreeNode::new(2, "Environment").with_icon("🌄"));
        self.scene_hierarchy
            .add_child(environment, TreeNode::new(10, "Skybox").with_icon("🌌"));
        self.scene_hierarchy
            .add_child(environment, TreeNode::new(11, "Sun").with_icon("☀️"));
        self.scene_hierarchy
            .add_child(environment, TreeNode::new(12, "Fog").with_icon("🌫️"));

        let entities = self
            .scene_hierarchy
            .add_node(TreeNode::new(3, "Entities").with_icon("🎮"));

        let player = self
            .scene_hierarchy
            .add_child(entities, TreeNode::new(20, "Player").with_icon("👤"))
            .unwrap();
        self.scene_hierarchy
            .add_child(player, TreeNode::new(21, "Camera").with_icon("📷"));
        self.scene_hierarchy
            .add_child(player, TreeNode::new(22, "Weapon").with_icon("🔫"));

        let enemies = self
            .scene_hierarchy
            .add_child(entities, TreeNode::new(23, "Enemies").with_icon("👾"))
            .unwrap();
        self.scene_hierarchy
            .add_child(enemies, TreeNode::new(24, "Enemy_1").with_icon("🤖"));
        self.scene_hierarchy
            .add_child(enemies, TreeNode::new(25, "Enemy_2").with_icon("🤖"));
        self.scene_hierarchy
            .add_child(enemies, TreeNode::new(26, "Enemy_3").with_icon("🤖"));

        let npcs = self
            .scene_hierarchy
            .add_child(entities, TreeNode::new(27, "NPCs").with_icon("🧑"))
            .unwrap();
        self.scene_hierarchy
            .add_child(npcs, TreeNode::new(28, "Merchant").with_icon("🏪"));
        self.scene_hierarchy
            .add_child(npcs, TreeNode::new(29, "Guard").with_icon("🛡️"));

        // Asset Browser: Game asset organization
        let assets = self
            .asset_browser
            .add_node(TreeNode::new(100, "Assets").with_icon("📦"));

        let models = self
            .asset_browser
            .add_child(assets, TreeNode::new(101, "Models").with_icon("🗿"))
            .unwrap();
        self.asset_browser
            .add_child(models, TreeNode::new(110, "character.fbx").with_icon("📄"));
        self.asset_browser
            .add_child(models, TreeNode::new(111, "weapon.fbx").with_icon("📄"));
        self.asset_browser.add_child(
            models,
            TreeNode::new(112, "environment.fbx").with_icon("📄"),
        );

        let textures = self
            .asset_browser
            .add_child(assets, TreeNode::new(102, "Textures").with_icon("🖼️"))
            .unwrap();
        self.asset_browser
            .add_child(textures, TreeNode::new(120, "albedo.png").with_icon("📄"));
        self.asset_browser
            .add_child(textures, TreeNode::new(121, "normal.png").with_icon("📄"));
        self.asset_browser
            .add_child(textures, TreeNode::new(122, "metallic.png").with_icon("📄"));

        let audio = self
            .asset_browser
            .add_child(assets, TreeNode::new(103, "Audio").with_icon("🔊"))
            .unwrap();
        self.asset_browser
            .add_child(audio, TreeNode::new(130, "music.ogg").with_icon("🎵"));
        self.asset_browser
            .add_child(audio, TreeNode::new(131, "sfx_shot.wav").with_icon("🔊"));
        self.asset_browser
            .add_child(audio, TreeNode::new(132, "sfx_step.wav").with_icon("🔊"));

        let scripts = self
            .asset_browser
            .add_child(assets, TreeNode::new(104, "Scripts").with_icon("📜"))
            .unwrap();
        self.asset_browser.add_child(
            scripts,
            TreeNode::new(140, "player_controller.rs").with_icon("🦀"),
        );
        self.asset_browser
            .add_child(scripts, TreeNode::new(141, "enemy_ai.rs").with_icon("🦀"));

        self.initialized = true;
    }

    /// Update panel state
    pub fn update(&mut self) {
        self.initialize_tree_views();
    }

    /// Show the advanced widgets panel
    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("🎨 Advanced Widgets");
        ui.add_space(8.0);

        // Color Pickers Section
        ui.collapsing("🎨 Color Pickers", |ui| {
            ui.label("Game engine lighting and atmosphere controls");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.label("🌙 Ambient Light Color");
                ui.add_space(4.0);
                if self.ambient_color.show(ui) {
                    let color = self.ambient_color.color();
                    ui.label(format!(
                        "RGB: ({}, {}, {})",
                        color.r(),
                        color.g(),
                        color.b()
                    ));
                }
            });

            ui.add_space(8.0);

            ui.group(|ui| {
                ui.label("☀️ Directional Light Color");
                ui.add_space(4.0);
                if self.directional_light_color.show(ui) {
                    let color = self.directional_light_color.color();
                    ui.label(format!(
                        "RGB: ({}, {}, {})",
                        color.r(),
                        color.g(),
                        color.b()
                    ));
                }
            });

            ui.add_space(8.0);

            ui.group(|ui| {
                ui.label("🌫️ Fog Color (with Alpha)");
                ui.add_space(4.0);
                if self.fog_color.show(ui) {
                    let color = self.fog_color.color();
                    ui.label(format!(
                        "RGBA: ({}, {}, {}, {})",
                        color.r(),
                        color.g(),
                        color.b(),
                        color.a()
                    ));
                }
            });
        });

        ui.add_space(8.0);

        // Tree Views Section
        ui.collapsing("🌳 Tree Views", |ui| {
            ui.label("Hierarchical data visualization");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.label("🌍 Scene Hierarchy");
                ui.separator();
                ui.add_space(4.0);

                if let Some(clicked_id) = self.scene_hierarchy.show(ui) {
                    if let Some(node) = self.scene_hierarchy.get_node(clicked_id) {
                        ui.separator();
                        ui.colored_label(
                            Color32::from_rgb(100, 180, 255),
                            format!("Selected: {} (ID: {})", node.label, node.id),
                        );
                    }
                }
            });

            ui.add_space(8.0);

            ui.group(|ui| {
                ui.label("📦 Asset Browser");
                ui.separator();
                ui.add_space(4.0);

                if let Some(clicked_id) = self.asset_browser.show(ui) {
                    if let Some(node) = self.asset_browser.get_node(clicked_id) {
                        ui.separator();
                        ui.colored_label(
                            Color32::from_rgb(100, 180, 255),
                            format!("Selected: {} (ID: {})", node.label, node.id),
                        );
                    }
                }
            });
        });

        ui.add_space(8.0);

        // Range Sliders Section
        ui.collapsing("📏 Range Sliders", |ui| {
            ui.label("Dual-handle range selection controls");
            ui.add_space(4.0);

            ui.group(|ui| {
                ui.label("📷 Camera Distance Range");
                ui.add_space(4.0);
                if self.camera_distance.show(ui) {
                    ui.label(format!(
                        "LOD switching: {} - {}",
                        self.camera_distance
                            .format_value(self.camera_distance.min_value()),
                        self.camera_distance
                            .format_value(self.camera_distance.max_value())
                    ));
                }
            });

            ui.add_space(8.0);

            ui.group(|ui| {
                ui.label("🎮 Player Level Range Filter");
                ui.add_space(4.0);
                if self.player_level_range.show(ui) {
                    ui.label(format!(
                        "Matchmaking range: {} - {}",
                        self.player_level_range
                            .format_value(self.player_level_range.min_value()),
                        self.player_level_range
                            .format_value(self.player_level_range.max_value())
                    ));
                }
            });

            ui.add_space(8.0);

            ui.group(|ui| {
                ui.label("🔊 Audio Frequency Filter");
                ui.add_space(4.0);
                if self.audio_frequency.show(ui) {
                    ui.label(format!(
                        "EQ range: {} - {}",
                        self.audio_frequency
                            .format_value(self.audio_frequency.min_value()),
                        self.audio_frequency
                            .format_value(self.audio_frequency.max_value())
                    ));
                }
            });
        });

        ui.add_space(8.0);

        // Stats
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!(
                "Scene Nodes: {}",
                self.scene_hierarchy.node_count()
            ));
            ui.separator();
            ui.label(format!("Assets: {}", self.asset_browser.node_count()));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_creation() {
        let panel = AdvancedWidgetsPanel::new();
        assert!(!panel.initialized);
    }

    #[test]
    fn test_panel_initialization() {
        let mut panel = AdvancedWidgetsPanel::new();
        panel.initialize_tree_views();

        assert!(panel.initialized);
        assert!(panel.scene_hierarchy.node_count() > 0);
        assert!(panel.asset_browser.node_count() > 0);
    }

    #[test]
    fn test_color_picker_defaults() {
        let panel = AdvancedWidgetsPanel::new();

        // Ambient color should be dark blue-ish
        let ambient = panel.ambient_color.color();
        assert_eq!(ambient.r(), 50);
        assert_eq!(ambient.g(), 50);
        assert_eq!(ambient.b(), 70);

        // Directional light should be warm white
        let light = panel.directional_light_color.color();
        assert_eq!(light.r(), 255);
        assert_eq!(light.g(), 244);
        assert_eq!(light.b(), 214);
    }

    #[test]
    fn test_range_slider_defaults() {
        let panel = AdvancedWidgetsPanel::new();

        // Camera distance
        assert_eq!(panel.camera_distance.min_value(), 5.0);
        assert_eq!(panel.camera_distance.max_value(), 25.0);

        // Player level
        assert_eq!(panel.player_level_range.min_value(), 10.0);
        assert_eq!(panel.player_level_range.max_value(), 50.0);

        // Audio frequency
        assert_eq!(panel.audio_frequency.min_value(), 200.0);
        assert_eq!(panel.audio_frequency.max_value(), 8000.0);
    }

    #[test]
    fn test_scene_hierarchy_structure() {
        let mut panel = AdvancedWidgetsPanel::new();
        panel.initialize_tree_views();

        // Should have root nodes
        assert!(panel.scene_hierarchy.node_count() >= 3); // World, Environment, Entities

        // Check specific nodes exist
        assert!(panel.scene_hierarchy.get_node(1).is_some()); // World
        assert!(panel.scene_hierarchy.get_node(2).is_some()); // Environment
        assert!(panel.scene_hierarchy.get_node(3).is_some()); // Entities
        assert!(panel.scene_hierarchy.get_node(20).is_some()); // Player
    }

    #[test]
    fn test_asset_browser_structure() {
        let mut panel = AdvancedWidgetsPanel::new();
        panel.initialize_tree_views();

        // Should have asset nodes
        assert!(panel.asset_browser.node_count() > 0);

        // Check asset categories exist
        assert!(panel.asset_browser.get_node(100).is_some()); // Assets root
        assert!(panel.asset_browser.get_node(101).is_some()); // Models
        assert!(panel.asset_browser.get_node(102).is_some()); // Textures
        assert!(panel.asset_browser.get_node(103).is_some()); // Audio
        assert!(panel.asset_browser.get_node(104).is_some()); // Scripts
    }
}
