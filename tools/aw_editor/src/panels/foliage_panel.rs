//! Foliage Panel for the editor
//!
//! Provides comprehensive vegetation/foliage painting and management:
//! - Foliage painting brushes
//! - Instance management and density control
//! - LOD configuration per foliage type
//! - Wind and interaction settings
//! - Collision and occlusion settings
//! - Procedural placement

use egui::{Color32, RichText, Ui, Vec2};

use crate::panels::Panel;

/// Foliage type category
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FoliageCategory {
    #[default]
    Grass,
    Flowers,
    Shrubs,
    Trees,
    Rocks,
    Custom,
}

impl std::fmt::Display for FoliageCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl FoliageCategory {
    pub fn name(&self) -> &'static str {
        match self {
            FoliageCategory::Grass => "Grass",
            FoliageCategory::Flowers => "Flowers",
            FoliageCategory::Shrubs => "Shrubs",
            FoliageCategory::Trees => "Trees",
            FoliageCategory::Rocks => "Rocks",
            FoliageCategory::Custom => "Custom",
        }
    }

    pub fn all() -> &'static [FoliageCategory] {
        &[
            FoliageCategory::Grass,
            FoliageCategory::Flowers,
            FoliageCategory::Shrubs,
            FoliageCategory::Trees,
            FoliageCategory::Rocks,
            FoliageCategory::Custom,
        ]
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FoliageCategory::Grass => "🌿",
            FoliageCategory::Flowers => "🌸",
            FoliageCategory::Shrubs => "🌳",
            FoliageCategory::Trees => "🌲",
            FoliageCategory::Rocks => "🪨",
            FoliageCategory::Custom => "📦",
        }
    }
}

/// Foliage type definition
#[derive(Debug, Clone)]
pub struct FoliageType {
    pub id: u32,
    pub name: String,
    pub category: FoliageCategory,
    pub mesh_path: String,
    pub enabled: bool,

    // Density
    pub density: f32,          // instances per square meter
    pub min_scale: f32,
    pub max_scale: f32,
    pub random_rotation: bool,
    pub align_to_normal: bool,
    pub normal_alignment: f32, // 0-1

    // Placement
    pub min_slope: f32,
    pub max_slope: f32,
    pub min_altitude: f32,
    pub max_altitude: f32,
    pub placement_jitter: f32,
    pub exclusion_radius: f32,

    // LOD
    pub lod_enabled: bool,
    pub lod_distances: [f32; 4],
    pub cull_distance: f32,

    // Wind
    pub wind_enabled: bool,
    pub wind_strength: f32,
    pub wind_frequency: f32,

    // Collision
    pub collision_enabled: bool,
    pub cast_shadow: bool,
    pub affect_lighting: bool,
    pub affect_navigation: bool,

    // Visuals
    pub preview_color: [f32; 3],
    pub instance_count: u32,
}

impl Default for FoliageType {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Foliage".to_string(),
            category: FoliageCategory::Grass,
            mesh_path: String::new(),
            enabled: true,

            density: 10.0,
            min_scale: 0.8,
            max_scale: 1.2,
            random_rotation: true,
            align_to_normal: true,
            normal_alignment: 1.0,

            min_slope: 0.0,
            max_slope: 45.0,
            min_altitude: -1000.0,
            max_altitude: 1000.0,
            placement_jitter: 0.5,
            exclusion_radius: 0.2,

            lod_enabled: true,
            lod_distances: [50.0, 100.0, 200.0, 500.0],
            cull_distance: 1000.0,

            wind_enabled: true,
            wind_strength: 0.5,
            wind_frequency: 1.0,

            collision_enabled: false,
            cast_shadow: true,
            affect_lighting: true,
            affect_navigation: false,

            preview_color: [0.3, 0.7, 0.3],
            instance_count: 0,
        }
    }
}

/// Brush tool type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum BrushTool {
    #[default]
    Paint,
    Erase,
    Select,
    Reapply,
    SinglePlace,
}

impl std::fmt::Display for BrushTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl BrushTool {
    pub fn all() -> &'static [BrushTool] {
        &[
            BrushTool::Paint,
            BrushTool::Erase,
            BrushTool::Select,
            BrushTool::Reapply,
            BrushTool::SinglePlace,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            BrushTool::Paint => "Paint",
            BrushTool::Erase => "Erase",
            BrushTool::Select => "Select",
            BrushTool::Reapply => "Reapply",
            BrushTool::SinglePlace => "Single Place",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            BrushTool::Paint => "🖌️",
            BrushTool::Erase => "🧹",
            BrushTool::Select => "👆",
            BrushTool::Reapply => "🔄",
            BrushTool::SinglePlace => "📍",
        }
    }
}

/// Brush settings
#[derive(Debug, Clone)]
pub struct BrushSettings {
    pub radius: f32,
    pub falloff: f32,
    pub density: f32,
    pub flow: f32,
    pub use_mask: bool,
    pub mask_channel: u8,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            radius: 5.0,
            falloff: 0.5,
            density: 1.0,
            flow: 1.0,
            use_mask: false,
            mask_channel: 0,
        }
    }
}

/// Procedural placement rule
#[derive(Debug, Clone)]
pub struct ProceduralRule {
    pub id: u32,
    pub name: String,
    pub enabled: bool,

    pub target_types: Vec<u32>,
    pub area_size: [f32; 2],

    // Noise settings
    pub use_noise: bool,
    pub noise_scale: f32,
    pub noise_threshold: f32,

    // Distribution
    pub distribution_type: DistributionType,
    pub clustering: f32,
    pub spacing: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DistributionType {
    #[default]
    Random,
    Uniform,
    Clustered,
    PoissonDisc,
}

impl std::fmt::Display for DistributionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl DistributionType {
    pub fn all() -> &'static [DistributionType] {
        &[
            DistributionType::Random,
            DistributionType::Uniform,
            DistributionType::Clustered,
            DistributionType::PoissonDisc,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            DistributionType::Random => "Random",
            DistributionType::Uniform => "Uniform",
            DistributionType::Clustered => "Clustered",
            DistributionType::PoissonDisc => "Poisson Disc",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            DistributionType::Random => "🎲",
            DistributionType::Uniform => "⊞",
            DistributionType::Clustered => "⚫",
            DistributionType::PoissonDisc => "◎",
        }
    }
}

impl Default for ProceduralRule {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Rule".to_string(),
            enabled: true,
            target_types: Vec::new(),
            area_size: [100.0, 100.0],
            use_noise: false,
            noise_scale: 10.0,
            noise_threshold: 0.5,
            distribution_type: DistributionType::Random,
            clustering: 0.0,
            spacing: 1.0,
        }
    }
}

/// Foliage layer (for grouping)
#[derive(Debug, Clone)]
pub struct FoliageLayer {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub foliage_types: Vec<u32>,
}

impl Default for FoliageLayer {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Default Layer".to_string(),
            visible: true,
            locked: false,
            foliage_types: Vec::new(),
        }
    }
}

/// Panel tabs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FoliageTab {
    #[default]
    Paint,
    Types,
    Settings,
    Procedural,
    Layers,
    Statistics,
}

impl std::fmt::Display for FoliageTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl FoliageTab {
    pub fn all() -> &'static [FoliageTab] {
        &[
            FoliageTab::Paint,
            FoliageTab::Types,
            FoliageTab::Settings,
            FoliageTab::Procedural,
            FoliageTab::Layers,
            FoliageTab::Statistics,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            FoliageTab::Paint => "Paint",
            FoliageTab::Types => "Types",
            FoliageTab::Settings => "Settings",
            FoliageTab::Procedural => "Procedural",
            FoliageTab::Layers => "Layers",
            FoliageTab::Statistics => "Statistics",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FoliageTab::Paint => "🖌️",
            FoliageTab::Types => "🌿",
            FoliageTab::Settings => "⚙️",
            FoliageTab::Procedural => "🔧",
            FoliageTab::Layers => "📚",
            FoliageTab::Statistics => "📊",
        }
    }
}

/// Main Foliage Panel
pub struct FoliagePanel {
    active_tab: FoliageTab,

    // Tools
    current_tool: BrushTool,
    brush_settings: BrushSettings,

    // Foliage types
    foliage_types: Vec<FoliageType>,
    selected_types: Vec<u32>,
    current_type: FoliageType,

    // Layers
    layers: Vec<FoliageLayer>,
    selected_layer: Option<u32>,

    // Procedural rules
    procedural_rules: Vec<ProceduralRule>,

    // Editor state
    show_preview: bool,
    preview_density: f32,

    // Statistics
    total_instances: u32,
    total_triangles: u64,
    memory_usage_mb: f32,

    // ID counters
    next_type_id: u32,
    next_layer_id: u32,
    next_rule_id: u32,
}

impl Default for FoliagePanel {
    fn default() -> Self {
        let mut panel = Self {
            active_tab: FoliageTab::Paint,

            current_tool: BrushTool::Paint,
            brush_settings: BrushSettings::default(),

            foliage_types: Vec::new(),
            selected_types: Vec::new(),
            current_type: FoliageType::default(),

            layers: Vec::new(),
            selected_layer: None,

            procedural_rules: Vec::new(),

            show_preview: true,
            preview_density: 1.0,

            total_instances: 0,
            total_triangles: 0,
            memory_usage_mb: 0.0,

            next_type_id: 1,
            next_layer_id: 1,
            next_rule_id: 1,
        };

        panel.create_sample_data();
        panel
    }
}

impl FoliagePanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_sample_data(&mut self) {
        // Create default layer
        let layer_id = self.next_layer_id();
        self.layers.push(FoliageLayer {
            id: layer_id,
            name: "Ground Cover".to_string(),
            ..Default::default()
        });
        self.selected_layer = Some(layer_id);

        // Create sample foliage types
        let samples = [
            ("Short Grass", FoliageCategory::Grass, 50.0, [0.4, 0.6, 0.3]),
            ("Tall Grass", FoliageCategory::Grass, 30.0, [0.3, 0.5, 0.2]),
            ("Wildflowers", FoliageCategory::Flowers, 15.0, [0.8, 0.6, 0.3]),
            ("Bush Small", FoliageCategory::Shrubs, 2.0, [0.2, 0.4, 0.2]),
            ("Pine Tree", FoliageCategory::Trees, 0.1, [0.15, 0.3, 0.15]),
            ("Rock Small", FoliageCategory::Rocks, 0.5, [0.5, 0.5, 0.5]),
        ];

        for (name, category, density, color) in samples {
            let id = self.next_type_id();
            let mut ft = FoliageType {
                id,
                name: name.to_string(),
                category,
                density,
                preview_color: color,
                instance_count: (density * 100.0) as u32,
                ..Default::default()
            };

            // Adjust settings by category
            match category {
                FoliageCategory::Trees => {
                    ft.min_scale = 0.7;
                    ft.max_scale = 1.3;
                    ft.collision_enabled = true;
                    ft.affect_navigation = true;
                    ft.cull_distance = 2000.0;
                }
                FoliageCategory::Shrubs => {
                    ft.min_scale = 0.6;
                    ft.max_scale = 1.4;
                    ft.cull_distance = 500.0;
                }
                FoliageCategory::Rocks => {
                    ft.wind_enabled = false;
                    ft.collision_enabled = true;
                    ft.cull_distance = 800.0;
                }
                _ => {}
            }

            if let Some(layer) = self.layers.first_mut() {
                layer.foliage_types.push(id);
            }

            self.foliage_types.push(ft);
        }

        if !self.foliage_types.is_empty() {
            self.current_type = self.foliage_types[0].clone();
            self.selected_types.push(self.foliage_types[0].id);
        }

        self.calculate_statistics();
    }

    fn next_type_id(&mut self) -> u32 {
        let id = self.next_type_id;
        self.next_type_id += 1;
        id
    }

    fn next_layer_id(&mut self) -> u32 {
        let id = self.next_layer_id;
        self.next_layer_id += 1;
        id
    }

    #[allow(dead_code)]
    fn next_rule_id(&mut self) -> u32 {
        let id = self.next_rule_id;
        self.next_rule_id += 1;
        id
    }

    fn calculate_statistics(&mut self) {
        self.total_instances = self.foliage_types.iter().map(|ft| ft.instance_count).sum();
        self.total_triangles = self.total_instances as u64 * 500; // Estimate
        self.memory_usage_mb = self.total_instances as f32 * 0.0001; // Estimate
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (FoliageTab::Paint, "🖌️ Paint"),
                (FoliageTab::Types, "🌿 Types"),
                (FoliageTab::Settings, "⚙️ Settings"),
                (FoliageTab::Procedural, "🎲 Procedural"),
                (FoliageTab::Layers, "📚 Layers"),
                (FoliageTab::Statistics, "📊 Stats"),
            ];

            for (tab, label) in tabs {
                let is_selected = self.active_tab == tab;
                let button = egui::Button::new(label).fill(if is_selected {
                    Color32::from_rgb(60, 100, 160)
                } else {
                    Color32::from_rgb(50, 50, 55)
                });

                if ui.add(button).clicked() {
                    self.active_tab = tab;
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!("📊 {} instances", self.total_instances));
            ui.separator();
            ui.label(format!("🔺 {} tris", self.total_triangles));
            ui.separator();
            ui.label(format!("💾 {:.1} MB", self.memory_usage_mb));
        });

        ui.separator();
    }

    fn show_paint_tab(&mut self, ui: &mut Ui) {
        ui.heading("🖌️ Foliage Painting");
        ui.add_space(10.0);

        // Tool selection
        ui.group(|ui| {
            ui.label(RichText::new("🔧 Tools").strong());
            ui.horizontal(|ui| {
                let tools = [
                    BrushTool::Paint,
                    BrushTool::Erase,
                    BrushTool::Select,
                    BrushTool::Reapply,
                    BrushTool::SinglePlace,
                ];

                for tool in tools {
                    let is_selected = self.current_tool == tool;
                    let button = egui::Button::new(format!("{} {:?}", tool.icon(), tool))
                        .fill(if is_selected { Color32::from_rgb(80, 120, 180) } else { Color32::from_rgb(50, 50, 55) });

                    if ui.add(button).clicked() {
                        self.current_tool = tool;
                    }
                }
            });
        });

        ui.add_space(10.0);

        // Brush settings
        ui.group(|ui| {
            ui.label(RichText::new("🎨 Brush Settings").strong());

            egui::Grid::new("brush_settings")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Radius:");
                    ui.add(egui::Slider::new(&mut self.brush_settings.radius, 0.5..=50.0).suffix("m"));
                    ui.end_row();

                    ui.label("Falloff:");
                    ui.add(egui::Slider::new(&mut self.brush_settings.falloff, 0.0..=1.0));
                    ui.end_row();

                    ui.label("Density:");
                    ui.add(egui::Slider::new(&mut self.brush_settings.density, 0.1..=2.0));
                    ui.end_row();

                    ui.label("Flow:");
                    ui.add(egui::Slider::new(&mut self.brush_settings.flow, 0.1..=1.0));
                    ui.end_row();
                });
        });

        ui.add_space(10.0);

        // Foliage type selection
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🌿 Active Foliage Types").strong());
                if ui.button("Select All").clicked() {
                    self.selected_types = self.foliage_types.iter().map(|ft| ft.id).collect();
                }
                if ui.button("Clear").clicked() {
                    self.selected_types.clear();
                }
            });

            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for ft in &self.foliage_types {
                        let is_selected = self.selected_types.contains(&ft.id);
                        let label = format!("{} {} ({})", ft.category.icon(), ft.name, ft.instance_count);

                        let mut selected = is_selected;
                        if ui.checkbox(&mut selected, label).changed() {
                            if selected {
                                if !self.selected_types.contains(&ft.id) {
                                    self.selected_types.push(ft.id);
                                }
                            } else {
                                self.selected_types.retain(|&id| id != ft.id);
                            }
                        }
                    }
                });
        });
    }

    fn show_types_tab(&mut self, ui: &mut Ui) {
        ui.heading("🌿 Foliage Types");
        ui.add_space(10.0);

        // Type list
        ui.horizontal(|ui| {
            if ui.button("+ Add Type").clicked() {
                let id = self.next_type_id();
                let new_type = FoliageType {
                    id,
                    name: format!("Foliage {}", id),
                    ..Default::default()
                };
                self.foliage_types.push(new_type.clone());
                self.current_type = new_type;
                self.selected_types = vec![id];
            }

            egui::ComboBox::from_id_salt("category_filter")
                .selected_text("All Categories")
                .show_ui(ui, |ui| {
                    for cat in FoliageCategory::all() {
                        let _ = ui.selectable_label(false, format!("{} {:?}", cat.icon(), cat));
                    }
                });
        });

        ui.add_space(10.0);

        // Type grid
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                egui::Grid::new("foliage_types_grid")
                    .num_columns(3)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        for (i, ft) in self.foliage_types.clone().iter().enumerate() {
                            let is_selected = self.selected_types.first() == Some(&ft.id);

                            let button_color = if is_selected {
                                Color32::from_rgb(80, 120, 180)
                            } else {
                                Color32::from_rgb(
                                    (ft.preview_color[0] * 100.0 + 30.0) as u8,
                                    (ft.preview_color[1] * 100.0 + 30.0) as u8,
                                    (ft.preview_color[2] * 100.0 + 30.0) as u8,
                                )
                            };

                            let btn = egui::Button::new(format!("{}\n{}", ft.category.icon(), ft.name))
                                .min_size(Vec2::new(80.0, 60.0))
                                .fill(button_color);

                            if ui.add(btn).clicked() {
                                self.current_type = ft.clone();
                                self.selected_types = vec![ft.id];
                            }

                            if (i + 1) % 3 == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });

        ui.add_space(10.0);

        // Current type properties
        ui.group(|ui| {
            ui.label(RichText::new(format!("{} {}", self.current_type.category.icon(), self.current_type.name)).strong());

            egui::Grid::new("type_props")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.current_type.name);
                    ui.end_row();

                    ui.label("Category:");
                    egui::ComboBox::from_id_salt("type_category")
                        .selected_text(format!("{} {:?}", self.current_type.category.icon(), self.current_type.category))
                        .show_ui(ui, |ui| {
                            for cat in FoliageCategory::all() {
                                ui.selectable_value(&mut self.current_type.category, *cat, format!("{} {:?}", cat.icon(), cat));
                            }
                        });
                    ui.end_row();

                    ui.label("Mesh:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.current_type.mesh_path);
                        if ui.button("📂").clicked() {}
                    });
                    ui.end_row();

                    ui.label("Enabled:");
                    ui.checkbox(&mut self.current_type.enabled, "");
                    ui.end_row();
                });
        });
    }

    fn show_settings_tab(&mut self, ui: &mut Ui) {
        ui.heading("⚙️ Foliage Settings");
        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(350.0)
            .show(ui, |ui| {
                // Density & Scale
                ui.group(|ui| {
                    ui.label(RichText::new("📐 Density & Scale").strong());

                    egui::Grid::new("density_settings")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Density (per m²):");
                            ui.add(egui::Slider::new(&mut self.current_type.density, 0.1..=100.0).logarithmic(true));
                            ui.end_row();

                            ui.label("Scale Range:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_type.min_scale).speed(0.01).prefix("Min:"));
                                ui.add(egui::DragValue::new(&mut self.current_type.max_scale).speed(0.01).prefix("Max:"));
                            });
                            ui.end_row();

                            ui.label("Random Rotation:");
                            ui.checkbox(&mut self.current_type.random_rotation, "");
                            ui.end_row();

                            ui.label("Align to Normal:");
                            ui.checkbox(&mut self.current_type.align_to_normal, "");
                            ui.end_row();

                            if self.current_type.align_to_normal {
                                ui.label("Normal Weight:");
                                ui.add(egui::Slider::new(&mut self.current_type.normal_alignment, 0.0..=1.0));
                                ui.end_row();
                            }
                        });
                });

                ui.add_space(10.0);

                // Placement constraints
                ui.group(|ui| {
                    ui.label(RichText::new("🏔️ Placement Constraints").strong());

                    egui::Grid::new("placement_settings")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Slope Range:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_type.min_slope).speed(1.0).suffix("°").prefix("Min:"));
                                ui.add(egui::DragValue::new(&mut self.current_type.max_slope).speed(1.0).suffix("°").prefix("Max:"));
                            });
                            ui.end_row();

                            ui.label("Altitude Range:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_type.min_altitude).speed(1.0).prefix("Min:"));
                                ui.add(egui::DragValue::new(&mut self.current_type.max_altitude).speed(1.0).prefix("Max:"));
                            });
                            ui.end_row();

                            ui.label("Exclusion Radius:");
                            ui.add(egui::Slider::new(&mut self.current_type.exclusion_radius, 0.0..=5.0).suffix("m"));
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // LOD
                ui.group(|ui| {
                    ui.label(RichText::new("📏 Level of Detail").strong());

                    ui.checkbox(&mut self.current_type.lod_enabled, "Enable LOD");

                    if self.current_type.lod_enabled {
                        egui::Grid::new("lod_settings")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                for (i, dist) in self.current_type.lod_distances.iter_mut().enumerate() {
                                    ui.label(format!("LOD {} Distance:", i));
                                    ui.add(egui::DragValue::new(dist).speed(1.0).suffix("m"));
                                    ui.end_row();
                                }

                                ui.label("Cull Distance:");
                                ui.add(egui::DragValue::new(&mut self.current_type.cull_distance).speed(10.0).suffix("m"));
                                ui.end_row();
                            });
                    }
                });

                ui.add_space(10.0);

                // Wind
                ui.group(|ui| {
                    ui.label(RichText::new("🌬️ Wind").strong());

                    ui.checkbox(&mut self.current_type.wind_enabled, "Enable Wind");

                    if self.current_type.wind_enabled {
                        egui::Grid::new("wind_settings")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Strength:");
                                ui.add(egui::Slider::new(&mut self.current_type.wind_strength, 0.0..=2.0));
                                ui.end_row();

                                ui.label("Frequency:");
                                ui.add(egui::Slider::new(&mut self.current_type.wind_frequency, 0.1..=5.0));
                                ui.end_row();
                            });
                    }
                });

                ui.add_space(10.0);

                // Collision & Rendering
                ui.group(|ui| {
                    ui.label(RichText::new("🎯 Collision & Rendering").strong());

                    ui.checkbox(&mut self.current_type.collision_enabled, "Enable Collision");
                    ui.checkbox(&mut self.current_type.cast_shadow, "Cast Shadow");
                    ui.checkbox(&mut self.current_type.affect_lighting, "Affect Lighting");
                    ui.checkbox(&mut self.current_type.affect_navigation, "Affect Navigation");
                });
            });
    }

    fn show_procedural_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎲 Procedural Placement");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("+ New Rule").clicked() {
                let id = self.next_rule_id;
                self.next_rule_id += 1;
                self.procedural_rules.push(ProceduralRule {
                    id,
                    name: format!("Rule {}", id),
                    ..Default::default()
                });
            }

            if ui.button("Generate All").clicked() {
                // Trigger procedural generation
            }
        });

        ui.add_space(10.0);

        if self.procedural_rules.is_empty() {
            ui.label("No procedural rules defined.");
            ui.label("Click '+ New Rule' to create one.");
        } else {
            for rule in &mut self.procedural_rules {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut rule.enabled, "");
                        ui.text_edit_singleline(&mut rule.name);
                        if ui.button("🗑️").clicked() {
                            // Remove rule
                        }
                    });

                    egui::Grid::new(format!("rule_{}", rule.id))
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Distribution:");
                            egui::ComboBox::from_id_salt(format!("dist_{}", rule.id))
                                .selected_text(format!("{:?}", rule.distribution_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut rule.distribution_type, DistributionType::Random, "Random");
                                    ui.selectable_value(&mut rule.distribution_type, DistributionType::Uniform, "Uniform");
                                    ui.selectable_value(&mut rule.distribution_type, DistributionType::Clustered, "Clustered");
                                    ui.selectable_value(&mut rule.distribution_type, DistributionType::PoissonDisc, "Poisson Disc");
                                });
                            ui.end_row();

                            ui.label("Spacing:");
                            ui.add(egui::Slider::new(&mut rule.spacing, 0.1..=10.0).suffix("m"));
                            ui.end_row();

                            ui.label("Use Noise:");
                            ui.checkbox(&mut rule.use_noise, "");
                            ui.end_row();

                            if rule.use_noise {
                                ui.label("Noise Scale:");
                                ui.add(egui::Slider::new(&mut rule.noise_scale, 1.0..=100.0));
                                ui.end_row();

                                ui.label("Threshold:");
                                ui.add(egui::Slider::new(&mut rule.noise_threshold, 0.0..=1.0));
                                ui.end_row();
                            }
                        });
                });
            }
        }
    }

    fn show_layers_tab(&mut self, ui: &mut Ui) {
        ui.heading("📚 Foliage Layers");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("+ New Layer").clicked() {
                let id = self.next_layer_id();
                self.layers.push(FoliageLayer {
                    id,
                    name: format!("Layer {}", id),
                    ..Default::default()
                });
            }
        });

        ui.add_space(10.0);

        for layer in &mut self.layers {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let vis_icon = if layer.visible { "👁️" } else { "👁️‍🗨️" };
                    if ui.button(vis_icon).clicked() {
                        layer.visible = !layer.visible;
                    }

                    let lock_icon = if layer.locked { "🔒" } else { "🔓" };
                    if ui.button(lock_icon).clicked() {
                        layer.locked = !layer.locked;
                    }

                    ui.text_edit_singleline(&mut layer.name);

                    ui.label(format!("({} types)", layer.foliage_types.len()));
                });
            });
        }
    }

    fn show_statistics_tab(&mut self, ui: &mut Ui) {
        ui.heading("📊 Foliage Statistics");
        ui.add_space(10.0);

        // Overall stats
        ui.group(|ui| {
            ui.label(RichText::new("📈 Overview").strong());

            egui::Grid::new("overall_stats")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Total Instances:");
                    ui.label(RichText::new(format!("{}", self.total_instances)).strong());
                    ui.end_row();

                    ui.label("Total Triangles:");
                    ui.label(RichText::new(format!("{}", self.total_triangles)).strong());
                    ui.end_row();

                    ui.label("Memory Usage:");
                    ui.label(RichText::new(format!("{:.2} MB", self.memory_usage_mb)).strong());
                    ui.end_row();

                    ui.label("Foliage Types:");
                    ui.label(RichText::new(format!("{}", self.foliage_types.len())).strong());
                    ui.end_row();

                    ui.label("Layers:");
                    ui.label(RichText::new(format!("{}", self.layers.len())).strong());
                    ui.end_row();
                });
        });

        ui.add_space(10.0);

        // Per-type breakdown
        ui.group(|ui| {
            ui.label(RichText::new("🌿 Per-Type Breakdown").strong());

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for ft in &self.foliage_types {
                        ui.horizontal(|ui| {
                            ui.label(format!("{} {}", ft.category.icon(), ft.name));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("{} instances", ft.instance_count));
                            });
                        });

                        let fraction = if self.total_instances > 0 {
                            ft.instance_count as f32 / self.total_instances as f32
                        } else {
                            0.0
                        };

                        ui.add(egui::ProgressBar::new(fraction).show_percentage());
                        ui.add_space(5.0);
                    }
                });
        });

        ui.add_space(10.0);

        // Actions
        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh").clicked() {
                self.calculate_statistics();
            }
            if ui.button("🗑️ Clear All Instances").clicked() {
                for ft in &mut self.foliage_types {
                    ft.instance_count = 0;
                }
                self.calculate_statistics();
            }
        });
    }

    // Getters for testing
    pub fn foliage_type_count(&self) -> usize {
        self.foliage_types.len()
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn selected_type_count(&self) -> usize {
        self.selected_types.len()
    }

    pub fn total_instance_count(&self) -> u32 {
        self.total_instances
    }

    pub fn add_foliage_type(&mut self, name: &str, category: FoliageCategory) -> u32 {
        let id = self.next_type_id();
        self.foliage_types.push(FoliageType {
            id,
            name: name.to_string(),
            category,
            ..Default::default()
        });
        self.calculate_statistics();
        id
    }

    pub fn set_brush_radius(&mut self, radius: f32) {
        self.brush_settings.radius = radius;
    }
}

impl Panel for FoliagePanel {
    fn name(&self) -> &'static str {
        "Foliage"
    }

    fn show(&mut self, ui: &mut Ui) {
        self.show_tab_bar(ui);

        match self.active_tab {
            FoliageTab::Paint => self.show_paint_tab(ui),
            FoliageTab::Types => self.show_types_tab(ui),
            FoliageTab::Settings => self.show_settings_tab(ui),
            FoliageTab::Procedural => self.show_procedural_tab(ui),
            FoliageTab::Layers => self.show_layers_tab(ui),
            FoliageTab::Statistics => self.show_statistics_tab(ui),
        }
    }

    fn update(&mut self) {
        // Sync current type back to list
        if let Some(&id) = self.selected_types.first() {
            if let Some(ft) = self.foliage_types.iter_mut().find(|f| f.id == id) {
                *ft = self.current_type.clone();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // FOLIAGE CATEGORY TESTS
    // ============================================================

    #[test]
    fn test_foliage_category_default() {
        let cat = FoliageCategory::default();
        assert_eq!(cat, FoliageCategory::Grass);
    }

    #[test]
    fn test_foliage_category_all() {
        let all = FoliageCategory::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_foliage_category_icon_grass() {
        assert_eq!(FoliageCategory::Grass.icon(), "🌿");
    }

    #[test]
    fn test_foliage_category_icon_flowers() {
        assert_eq!(FoliageCategory::Flowers.icon(), "🌸");
    }

    #[test]
    fn test_foliage_category_icon_shrubs() {
        assert_eq!(FoliageCategory::Shrubs.icon(), "🌳");
    }

    #[test]
    fn test_foliage_category_icon_trees() {
        assert_eq!(FoliageCategory::Trees.icon(), "🌲");
    }

    #[test]
    fn test_foliage_category_icon_rocks() {
        assert_eq!(FoliageCategory::Rocks.icon(), "🪨");
    }

    #[test]
    fn test_foliage_category_icon_custom() {
        assert_eq!(FoliageCategory::Custom.icon(), "📦");
    }

    #[test]
    fn test_foliage_category_all_have_icons() {
        for cat in FoliageCategory::all() {
            assert!(!cat.icon().is_empty());
        }
    }

    // ============================================================
    // Session 5: Enum Enhancement Tests
    // ============================================================

    // FoliageCategory tests (7 tests)
    #[test]
    fn test_foliage_category_display() {
        assert!(format!("{}", FoliageCategory::Grass).contains("Grass"));
        assert!(format!("{}", FoliageCategory::Flowers).contains("Flowers"));
        assert!(format!("{}", FoliageCategory::Trees).contains("Trees"));
        assert!(format!("{}", FoliageCategory::Rocks).contains("Rocks"));
    }

    #[test]
    fn test_foliage_category_name() {
        assert_eq!(FoliageCategory::Grass.name(), "Grass");
        assert_eq!(FoliageCategory::Flowers.name(), "Flowers");
        assert_eq!(FoliageCategory::Shrubs.name(), "Shrubs");
        assert_eq!(FoliageCategory::Custom.name(), "Custom");
    }

    #[test]
    fn test_foliage_category_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for cat in FoliageCategory::all() {
            assert!(set.insert(*cat));
        }
        assert_eq!(set.len(), 6);
    }

    #[test]
    fn test_foliage_category_default_value() {
        assert_eq!(FoliageCategory::default(), FoliageCategory::Grass);
    }

    #[test]
    fn test_foliage_category_all_unique() {
        let all = FoliageCategory::all();
        for (i, cat1) in all.iter().enumerate() {
            for (j, cat2) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(cat1, cat2);
                }
            }
        }
    }

    #[test]
    fn test_foliage_category_all_have_names() {
        for cat in FoliageCategory::all() {
            assert!(!cat.name().is_empty());
        }
    }

    #[test]
    fn test_foliage_category_icon_not_empty() {
        for cat in FoliageCategory::all() {
            assert!(!cat.icon().is_empty());
        }
    }

    // BrushTool tests (7 tests)
    #[test]
    fn test_brush_tool_display() {
        assert!(format!("{}", BrushTool::Paint).contains("Paint"));
        assert!(format!("{}", BrushTool::Erase).contains("Erase"));
        assert!(format!("{}", BrushTool::Select).contains("Select"));
        assert!(format!("{}", BrushTool::SinglePlace).contains("Single Place"));
    }

    #[test]
    fn test_brush_tool_name() {
        assert_eq!(BrushTool::Paint.name(), "Paint");
        assert_eq!(BrushTool::Erase.name(), "Erase");
        assert_eq!(BrushTool::Reapply.name(), "Reapply");
        assert_eq!(BrushTool::SinglePlace.name(), "Single Place");
    }

    #[test]
    fn test_brush_tool_icon_present() {
        assert!(!BrushTool::Paint.icon().is_empty());
        assert!(!BrushTool::Erase.icon().is_empty());
        assert!(!BrushTool::Select.icon().is_empty());
        assert!(!BrushTool::Reapply.icon().is_empty());
    }

    #[test]
    fn test_brush_tool_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for tool in BrushTool::all() {
            assert!(set.insert(*tool));
        }
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn test_brush_tool_default_value() {
        assert_eq!(BrushTool::default(), BrushTool::Paint);
    }

    #[test]
    fn test_brush_tool_all_have_names() {
        for tool in BrushTool::all() {
            assert!(!tool.name().is_empty());
        }
    }

    #[test]
    fn test_brush_tool_all_have_icons() {
        for tool in BrushTool::all() {
            assert!(!tool.icon().is_empty());
        }
    }

    // DistributionType tests (7 tests)
    #[test]
    fn test_distribution_type_display() {
        assert!(format!("{}", DistributionType::Random).contains("Random"));
        assert!(format!("{}", DistributionType::Uniform).contains("Uniform"));
        assert!(format!("{}", DistributionType::Clustered).contains("Clustered"));
        assert!(format!("{}", DistributionType::PoissonDisc).contains("Poisson Disc"));
    }

    #[test]
    fn test_distribution_type_name() {
        assert_eq!(DistributionType::Random.name(), "Random");
        assert_eq!(DistributionType::Uniform.name(), "Uniform");
        assert_eq!(DistributionType::Clustered.name(), "Clustered");
        assert_eq!(DistributionType::PoissonDisc.name(), "Poisson Disc");
    }

    #[test]
    fn test_distribution_type_icon_present() {
        assert!(!DistributionType::Random.icon().is_empty());
        assert!(!DistributionType::Uniform.icon().is_empty());
        assert!(!DistributionType::Clustered.icon().is_empty());
        assert!(!DistributionType::PoissonDisc.icon().is_empty());
    }

    #[test]
    fn test_distribution_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for dist in DistributionType::all() {
            assert!(set.insert(*dist));
        }
        assert_eq!(set.len(), 4);
    }

    #[test]
    fn test_distribution_type_default_value() {
        assert_eq!(DistributionType::default(), DistributionType::Random);
    }

    #[test]
    fn test_distribution_type_all_have_names() {
        for dist in DistributionType::all() {
            assert!(!dist.name().is_empty());
        }
    }

    #[test]
    fn test_distribution_type_all_have_icons() {
        for dist in DistributionType::all() {
            assert!(!dist.icon().is_empty());
        }
    }

    // FoliageTab tests (7 tests)
    #[test]
    fn test_foliage_tab_display() {
        assert!(format!("{}", FoliageTab::Paint).contains("Paint"));
        assert!(format!("{}", FoliageTab::Types).contains("Types"));
        assert!(format!("{}", FoliageTab::Settings).contains("Settings"));
        assert!(format!("{}", FoliageTab::Statistics).contains("Statistics"));
    }

    #[test]
    fn test_foliage_tab_name() {
        assert_eq!(FoliageTab::Paint.name(), "Paint");
        assert_eq!(FoliageTab::Types.name(), "Types");
        assert_eq!(FoliageTab::Procedural.name(), "Procedural");
        assert_eq!(FoliageTab::Layers.name(), "Layers");
    }

    #[test]
    fn test_foliage_tab_icon_present() {
        assert!(!FoliageTab::Paint.icon().is_empty());
        assert!(!FoliageTab::Types.icon().is_empty());
        assert!(!FoliageTab::Settings.icon().is_empty());
        assert!(!FoliageTab::Statistics.icon().is_empty());
    }

    #[test]
    fn test_foliage_tab_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for tab in FoliageTab::all() {
            assert!(set.insert(*tab));
        }
        assert_eq!(set.len(), 6);
    }

    #[test]
    fn test_foliage_tab_default_value() {
        assert_eq!(FoliageTab::default(), FoliageTab::Paint);
    }

    #[test]
    fn test_foliage_tab_all_have_names() {
        for tab in FoliageTab::all() {
            assert!(!tab.name().is_empty());
        }
    }

    #[test]
    fn test_foliage_tab_all_have_icons() {
        for tab in FoliageTab::all() {
            assert!(!tab.icon().is_empty());
        }
    }

    // ============================================================
    // BRUSH TOOL TESTS
    // ============================================================

    #[test]
    fn test_brush_tool_default() {
        let tool = BrushTool::default();
        assert_eq!(tool, BrushTool::Paint);
    }

    #[test]
    fn test_brush_tool_all_variants() {
        let variants = [
            BrushTool::Paint,
            BrushTool::Erase,
            BrushTool::Select,
            BrushTool::Reapply,
            BrushTool::SinglePlace,
        ];
        assert_eq!(variants.len(), 5);
    }

    #[test]
    fn test_brush_tool_icon_paint() {
        assert_eq!(BrushTool::Paint.icon(), "🖌️");
    }

    #[test]
    fn test_brush_tool_icon_erase() {
        assert_eq!(BrushTool::Erase.icon(), "🧹");
    }

    #[test]
    fn test_brush_tool_icon_select() {
        assert_eq!(BrushTool::Select.icon(), "👆");
    }

    #[test]
    fn test_brush_tool_icon_reapply() {
        assert_eq!(BrushTool::Reapply.icon(), "🔄");
    }

    #[test]
    fn test_brush_tool_icon_single_place() {
        assert_eq!(BrushTool::SinglePlace.icon(), "📍");
    }

    #[test]
    fn test_brush_tool_icon_not_empty() {
        let tools = [
            BrushTool::Paint,
            BrushTool::Erase,
            BrushTool::Select,
            BrushTool::Reapply,
            BrushTool::SinglePlace,
        ];
        for tool in tools {
            assert!(!tool.icon().is_empty());
        }
    }

    // ============================================================
    // DISTRIBUTION TYPE TESTS
    // ============================================================

    #[test]
    fn test_distribution_type_default() {
        let dist = DistributionType::default();
        assert_eq!(dist, DistributionType::Random);
    }

    #[test]
    fn test_distribution_type_all_variants() {
        let variants = [
            DistributionType::Random,
            DistributionType::Uniform,
            DistributionType::Clustered,
            DistributionType::PoissonDisc,
        ];
        assert_eq!(variants.len(), 4);
    }

    #[test]
    fn test_distribution_type_clone() {
        let dist = DistributionType::PoissonDisc;
        let cloned = dist;
        assert_eq!(cloned, DistributionType::PoissonDisc);
    }

    // ============================================================
    // FOLIAGE TAB TESTS
    // ============================================================

    #[test]
    fn test_foliage_tab_default() {
        let tab = FoliageTab::default();
        assert_eq!(tab, FoliageTab::Paint);
    }

    #[test]
    fn test_foliage_tab_all_variants() {
        let variants = [
            FoliageTab::Paint,
            FoliageTab::Types,
            FoliageTab::Settings,
            FoliageTab::Procedural,
            FoliageTab::Layers,
            FoliageTab::Statistics,
        ];
        assert_eq!(variants.len(), 6);
    }

    // ============================================================
    // FOLIAGE TYPE TESTS
    // ============================================================

    #[test]
    fn test_foliage_type_default() {
        let ft = FoliageType::default();
        assert_eq!(ft.id, 0);
        assert_eq!(ft.name, "New Foliage");
        assert_eq!(ft.category, FoliageCategory::Grass);
    }

    #[test]
    fn test_foliage_type_default_density() {
        let ft = FoliageType::default();
        assert!((ft.density - 10.0).abs() < 0.001);
        assert!((ft.min_scale - 0.8).abs() < 0.001);
        assert!((ft.max_scale - 1.2).abs() < 0.001);
    }

    #[test]
    fn test_foliage_type_default_rotation() {
        let ft = FoliageType::default();
        assert!(ft.random_rotation);
        assert!(ft.align_to_normal);
        assert!((ft.normal_alignment - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_foliage_type_default_slope() {
        let ft = FoliageType::default();
        assert!((ft.min_slope - 0.0).abs() < 0.001);
        assert!((ft.max_slope - 45.0).abs() < 0.001);
    }

    #[test]
    fn test_foliage_type_default_altitude() {
        let ft = FoliageType::default();
        assert!((ft.min_altitude - (-1000.0)).abs() < 0.001);
        assert!((ft.max_altitude - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_foliage_type_default_lod() {
        let ft = FoliageType::default();
        assert!(ft.lod_enabled);
        assert_eq!(ft.lod_distances.len(), 4);
        assert!((ft.cull_distance - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_foliage_type_default_wind() {
        let ft = FoliageType::default();
        assert!(ft.wind_enabled);
        assert!((ft.wind_strength - 0.5).abs() < 0.001);
        assert!((ft.wind_frequency - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_foliage_type_default_collision() {
        let ft = FoliageType::default();
        assert!(!ft.collision_enabled);
        assert!(ft.cast_shadow);
        assert!(ft.affect_lighting);
        assert!(!ft.affect_navigation);
    }

    #[test]
    fn test_foliage_type_clone() {
        let ft = FoliageType::default();
        let cloned = ft.clone();
        assert_eq!(cloned.name, "New Foliage");
        assert_eq!(cloned.category, FoliageCategory::Grass);
    }

    // ============================================================
    // BRUSH SETTINGS TESTS
    // ============================================================

    #[test]
    fn test_brush_settings_default() {
        let bs = BrushSettings::default();
        assert!((bs.radius - 5.0).abs() < 0.001);
        assert!((bs.falloff - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_brush_settings_default_density() {
        let bs = BrushSettings::default();
        assert!((bs.density - 1.0).abs() < 0.001);
        assert!((bs.flow - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_brush_settings_default_mask() {
        let bs = BrushSettings::default();
        assert!(!bs.use_mask);
        assert_eq!(bs.mask_channel, 0);
    }

    #[test]
    fn test_brush_settings_clone() {
        let bs = BrushSettings::default();
        let cloned = bs.clone();
        assert!((cloned.radius - 5.0).abs() < 0.001);
    }

    // ============================================================
    // PROCEDURAL RULE TESTS
    // ============================================================

    #[test]
    fn test_procedural_rule_default() {
        let rule = ProceduralRule::default();
        assert_eq!(rule.id, 0);
        assert_eq!(rule.name, "New Rule");
        assert!(rule.enabled);
    }

    #[test]
    fn test_procedural_rule_default_noise() {
        let rule = ProceduralRule::default();
        assert!(!rule.use_noise);
        assert!((rule.noise_scale - 10.0).abs() < 0.001);
        assert!((rule.noise_threshold - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_procedural_rule_default_distribution() {
        let rule = ProceduralRule::default();
        assert_eq!(rule.distribution_type, DistributionType::Random);
        assert!((rule.clustering - 0.0).abs() < 0.001);
        assert!((rule.spacing - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_procedural_rule_default_area() {
        let rule = ProceduralRule::default();
        assert!((rule.area_size[0] - 100.0).abs() < 0.001);
        assert!((rule.area_size[1] - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_procedural_rule_clone() {
        let rule = ProceduralRule::default();
        let cloned = rule.clone();
        assert_eq!(cloned.name, "New Rule");
    }

    // ============================================================
    // FOLIAGE LAYER TESTS
    // ============================================================

    #[test]
    fn test_foliage_layer_default() {
        let layer = FoliageLayer::default();
        assert_eq!(layer.id, 0);
        assert_eq!(layer.name, "Default Layer");
        assert!(layer.visible);
        assert!(!layer.locked);
    }

    #[test]
    fn test_foliage_layer_default_types() {
        let layer = FoliageLayer::default();
        assert!(layer.foliage_types.is_empty());
    }

    #[test]
    fn test_foliage_layer_clone() {
        let layer = FoliageLayer::default();
        let cloned = layer.clone();
        assert_eq!(cloned.name, "Default Layer");
        assert!(cloned.visible);
    }

    // ============================================================
    // FOLIAGE PANEL TESTS
    // ============================================================

    #[test]
    fn test_foliage_panel_creation() {
        let panel = FoliagePanel::new();
        assert!(panel.foliage_type_count() >= 6);
    }

    #[test]
    fn test_default_layers() {
        let panel = FoliagePanel::new();
        assert!(panel.layer_count() >= 1);
    }

    #[test]
    fn test_selected_types() {
        let panel = FoliagePanel::new();
        assert!(panel.selected_type_count() >= 1);
    }

    #[test]
    fn test_add_foliage_type() {
        let mut panel = FoliagePanel::new();
        let initial = panel.foliage_type_count();
        let id = panel.add_foliage_type("Test Grass", FoliageCategory::Grass);
        assert!(id > 0);
        assert_eq!(panel.foliage_type_count(), initial + 1);
    }

    #[test]
    fn test_add_multiple_foliage_types() {
        let mut panel = FoliagePanel::new();
        let initial = panel.foliage_type_count();
        let id1 = panel.add_foliage_type("Grass A", FoliageCategory::Grass);
        let id2 = panel.add_foliage_type("Tree B", FoliageCategory::Trees);
        let id3 = panel.add_foliage_type("Rock C", FoliageCategory::Rocks);
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_eq!(panel.foliage_type_count(), initial + 3);
    }

    #[test]
    fn test_brush_settings() {
        let mut panel = FoliagePanel::new();
        panel.set_brush_radius(25.0);
        assert!((panel.brush_settings.radius - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_brush_radius_range() {
        let mut panel = FoliagePanel::new();
        panel.set_brush_radius(50.0);
        assert!((panel.brush_settings.radius - 50.0).abs() < 0.001);
        panel.set_brush_radius(1.0);
        assert!((panel.brush_settings.radius - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_statistics_calculation() {
        let panel = FoliagePanel::new();
        assert!(panel.total_instance_count() > 0);
    }

    #[test]
    fn test_panel_trait() {
        let panel = FoliagePanel::new();
        assert_eq!(panel.name(), "Foliage");
    }

    // ============================================================
    // INTEGRATION TESTS
    // ============================================================

    #[test]
    fn test_category_variants_coverage() {
        let categories = FoliageCategory::all();
        assert!(categories.contains(&FoliageCategory::Grass));
        assert!(categories.contains(&FoliageCategory::Flowers));
        assert!(categories.contains(&FoliageCategory::Shrubs));
        assert!(categories.contains(&FoliageCategory::Trees));
        assert!(categories.contains(&FoliageCategory::Rocks));
        assert!(categories.contains(&FoliageCategory::Custom));
    }

    #[test]
    fn test_foliage_type_enabled_by_default() {
        let ft = FoliageType::default();
        assert!(ft.enabled);
    }

    #[test]
    fn test_foliage_type_preview_color() {
        let ft = FoliageType::default();
        assert_eq!(ft.preview_color.len(), 3);
        for c in ft.preview_color {
            assert!((0.0..=1.0).contains(&c));
        }
    }

    #[test]
    fn test_lod_distances_order() {
        let ft = FoliageType::default();
        for i in 0..ft.lod_distances.len() - 1 {
            assert!(ft.lod_distances[i] <= ft.lod_distances[i + 1]);
        }
    }

    #[test]
    fn test_scale_range_valid() {
        let ft = FoliageType::default();
        assert!(ft.min_scale <= ft.max_scale);
        assert!(ft.min_scale > 0.0);
    }

    #[test]
    fn test_slope_range_valid() {
        let ft = FoliageType::default();
        assert!(ft.min_slope <= ft.max_slope);
        assert!(ft.min_slope >= 0.0);
        assert!(ft.max_slope <= 90.0);
    }
}
