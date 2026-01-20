//! Material Editor Panel for the editor UI
//!
//! Provides comprehensive PBR material editing:
//! - Albedo, normal, metallic, roughness, AO maps
//! - Material presets and templates
//! - Shader parameter configuration
//! - Material preview with lighting
//! - Material library management

use egui::{Color32, RichText, Ui, Vec2};

use crate::panels::Panel;

/// Material type/shader
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MaterialType {
    #[default]
    StandardPBR,
    Unlit,
    Subsurface,
    Glass,
    Water,
    Foliage,
    Hair,
    Cloth,
    Terrain,
    Decal,
}

impl MaterialType {
    pub fn all() -> &'static [MaterialType] {
        &[
            MaterialType::StandardPBR,
            MaterialType::Unlit,
            MaterialType::Subsurface,
            MaterialType::Glass,
            MaterialType::Water,
            MaterialType::Foliage,
            MaterialType::Hair,
            MaterialType::Cloth,
            MaterialType::Terrain,
            MaterialType::Decal,
        ]
    }

    pub fn icon(&self) -> &'static str {
        match self {
            MaterialType::StandardPBR => "🎨",
            MaterialType::Unlit => "💡",
            MaterialType::Subsurface => "🧴",
            MaterialType::Glass => "🔮",
            MaterialType::Water => "💧",
            MaterialType::Foliage => "🌿",
            MaterialType::Hair => "💇",
            MaterialType::Cloth => "👕",
            MaterialType::Terrain => "🏔️",
            MaterialType::Decal => "🏷️",
        }
    }
}

/// Blend mode for material
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BlendMode {
    #[default]
    Opaque,
    Masked,
    Translucent,
    Additive,
    Modulate,
}

/// Texture channel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureChannel {
    Albedo,
    Normal,
    Metallic,
    Roughness,
    AO,
    Emissive,
    Height,
    Opacity,
}

impl TextureChannel {
    pub fn name(&self) -> &'static str {
        match self {
            TextureChannel::Albedo => "Albedo",
            TextureChannel::Normal => "Normal",
            TextureChannel::Metallic => "Metallic",
            TextureChannel::Roughness => "Roughness",
            TextureChannel::AO => "Ambient Occlusion",
            TextureChannel::Emissive => "Emissive",
            TextureChannel::Height => "Height",
            TextureChannel::Opacity => "Opacity",
        }
    }
}

/// Texture slot configuration
#[derive(Debug, Clone)]
pub struct TextureSlot {
    pub channel: TextureChannel,
    pub texture_path: String,
    pub tiling: (f32, f32),
    pub offset: (f32, f32),
    pub intensity: f32,
    pub enabled: bool,
}

impl Default for TextureSlot {
    fn default() -> Self {
        Self {
            channel: TextureChannel::Albedo,
            texture_path: String::new(),
            tiling: (1.0, 1.0),
            offset: (0.0, 0.0),
            intensity: 1.0,
            enabled: true,
        }
    }
}

/// Material definition
#[derive(Debug, Clone)]
pub struct Material {
    pub id: u32,
    pub name: String,
    pub material_type: MaterialType,
    pub blend_mode: BlendMode,

    // PBR properties
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive_color: [f32; 3],
    pub emissive_intensity: f32,

    // Textures
    pub texture_slots: Vec<TextureSlot>,

    // Additional properties
    pub two_sided: bool,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub alpha_cutoff: f32,

    // Subsurface (for skin, wax, etc.)
    pub subsurface_color: [f32; 3],
    pub subsurface_radius: f32,

    // Glass/translucent
    pub ior: f32,
    pub transmission: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Material".to_string(),
            material_type: MaterialType::StandardPBR,
            blend_mode: BlendMode::Opaque,

            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive_color: [0.0, 0.0, 0.0],
            emissive_intensity: 0.0,

            texture_slots: Vec::new(),

            two_sided: false,
            cast_shadows: true,
            receive_shadows: true,
            alpha_cutoff: 0.5,

            subsurface_color: [1.0, 0.2, 0.1],
            subsurface_radius: 1.0,

            ior: 1.5,
            transmission: 0.0,
        }
    }
}

/// Material preset/template
#[derive(Debug, Clone)]
pub struct MaterialPreset {
    pub name: String,
    pub category: String,
    pub material_type: MaterialType,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}

impl MaterialPreset {
    fn presets() -> Vec<MaterialPreset> {
        vec![
            MaterialPreset {
                name: "Polished Metal".to_string(),
                category: "Metals".to_string(),
                material_type: MaterialType::StandardPBR,
                base_color: [0.9, 0.9, 0.9, 1.0],
                metallic: 1.0,
                roughness: 0.1,
            },
            MaterialPreset {
                name: "Brushed Steel".to_string(),
                category: "Metals".to_string(),
                material_type: MaterialType::StandardPBR,
                base_color: [0.7, 0.7, 0.75, 1.0],
                metallic: 1.0,
                roughness: 0.4,
            },
            MaterialPreset {
                name: "Gold".to_string(),
                category: "Metals".to_string(),
                material_type: MaterialType::StandardPBR,
                base_color: [1.0, 0.84, 0.0, 1.0],
                metallic: 1.0,
                roughness: 0.2,
            },
            MaterialPreset {
                name: "Rough Wood".to_string(),
                category: "Organic".to_string(),
                material_type: MaterialType::StandardPBR,
                base_color: [0.55, 0.35, 0.2, 1.0],
                metallic: 0.0,
                roughness: 0.8,
            },
            MaterialPreset {
                name: "Polished Wood".to_string(),
                category: "Organic".to_string(),
                material_type: MaterialType::StandardPBR,
                base_color: [0.6, 0.4, 0.25, 1.0],
                metallic: 0.0,
                roughness: 0.3,
            },
            MaterialPreset {
                name: "Concrete".to_string(),
                category: "Stone".to_string(),
                material_type: MaterialType::StandardPBR,
                base_color: [0.5, 0.5, 0.5, 1.0],
                metallic: 0.0,
                roughness: 0.9,
            },
            MaterialPreset {
                name: "Marble".to_string(),
                category: "Stone".to_string(),
                material_type: MaterialType::StandardPBR,
                base_color: [0.95, 0.95, 0.95, 1.0],
                metallic: 0.0,
                roughness: 0.2,
            },
            MaterialPreset {
                name: "Rubber".to_string(),
                category: "Synthetic".to_string(),
                material_type: MaterialType::StandardPBR,
                base_color: [0.1, 0.1, 0.1, 1.0],
                metallic: 0.0,
                roughness: 0.95,
            },
            MaterialPreset {
                name: "Plastic".to_string(),
                category: "Synthetic".to_string(),
                material_type: MaterialType::StandardPBR,
                base_color: [0.8, 0.2, 0.2, 1.0],
                metallic: 0.0,
                roughness: 0.4,
            },
            MaterialPreset {
                name: "Glass".to_string(),
                category: "Transparent".to_string(),
                material_type: MaterialType::Glass,
                base_color: [1.0, 1.0, 1.0, 0.1],
                metallic: 0.0,
                roughness: 0.0,
            },
            MaterialPreset {
                name: "Skin".to_string(),
                category: "Organic".to_string(),
                material_type: MaterialType::Subsurface,
                base_color: [0.9, 0.7, 0.6, 1.0],
                metallic: 0.0,
                roughness: 0.5,
            },
        ]
    }
}

/// Preview lighting mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PreviewLighting {
    #[default]
    Studio,
    Outdoor,
    Indoor,
    Dramatic,
    Custom,
}

/// Panel tabs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MaterialTab {
    #[default]
    Properties,
    Textures,
    Advanced,
    Presets,
    Preview,
    Library,
}

/// Main Material Editor Panel
pub struct MaterialEditorPanel {
    // Tab state
    active_tab: MaterialTab,

    // Materials
    materials: Vec<Material>,
    selected_material: Option<u32>,
    current_material: Material,

    // Presets
    presets: Vec<MaterialPreset>,
    preset_filter: String,

    // Preview
    preview_lighting: PreviewLighting,
    preview_rotation: f32,
    preview_zoom: f32,

    // Library
    library_path: String,
    library_materials: Vec<String>,

    // ID counter
    next_id: u32,
}

impl Default for MaterialEditorPanel {
    fn default() -> Self {
        let mut panel = Self {
            active_tab: MaterialTab::Properties,

            materials: Vec::new(),
            selected_material: None,
            current_material: Material::default(),

            presets: MaterialPreset::presets(),
            preset_filter: String::new(),

            preview_lighting: PreviewLighting::Studio,
            preview_rotation: 0.0,
            preview_zoom: 1.0,

            library_path: "assets/materials".to_string(),
            library_materials: Vec::new(),

            next_id: 1,
        };

        panel.create_sample_data();
        panel
    }
}

impl MaterialEditorPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_sample_data(&mut self) {
        // Create sample materials
        let id = self.next_id();
        self.materials.push(Material {
            id,
            name: "Ground Dirt".to_string(),
            material_type: MaterialType::StandardPBR,
            base_color: [0.4, 0.3, 0.2, 1.0],
            metallic: 0.0,
            roughness: 0.9,
            texture_slots: vec![
                TextureSlot { channel: TextureChannel::Albedo, texture_path: "textures/dirt_albedo.png".to_string(), ..Default::default() },
                TextureSlot { channel: TextureChannel::Normal, texture_path: "textures/dirt_normal.png".to_string(), ..Default::default() },
            ],
            ..Default::default()
        });
        self.next_id += 1;

        let id = self.next_id();
        self.materials.push(Material {
            id,
            name: "Metal Plate".to_string(),
            material_type: MaterialType::StandardPBR,
            base_color: [0.8, 0.8, 0.85, 1.0],
            metallic: 1.0,
            roughness: 0.3,
            ..Default::default()
        });
        self.next_id += 1;

        let id = self.next_id();
        self.materials.push(Material {
            id,
            name: "Glowing Crystal".to_string(),
            material_type: MaterialType::StandardPBR,
            base_color: [0.2, 0.8, 1.0, 0.8],
            blend_mode: BlendMode::Translucent,
            metallic: 0.0,
            roughness: 0.1,
            emissive_color: [0.2, 0.8, 1.0],
            emissive_intensity: 2.0,
            ..Default::default()
        });
        self.next_id += 1;

        // Sample library materials
        self.library_materials = vec![
            "ground/grass.mat".to_string(),
            "ground/dirt.mat".to_string(),
            "ground/stone.mat".to_string(),
            "metal/steel.mat".to_string(),
            "metal/copper.mat".to_string(),
            "wood/oak.mat".to_string(),
            "wood/pine.mat".to_string(),
            "fabric/cloth.mat".to_string(),
            "fabric/leather.mat".to_string(),
        ];

        self.current_material = self.materials[0].clone();
        self.selected_material = Some(self.materials[0].id);
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (MaterialTab::Properties, "🎨 Properties"),
                (MaterialTab::Textures, "🖼️ Textures"),
                (MaterialTab::Advanced, "⚙️ Advanced"),
                (MaterialTab::Presets, "📋 Presets"),
                (MaterialTab::Preview, "👁️ Preview"),
                (MaterialTab::Library, "📁 Library"),
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

        // Material info
        ui.horizontal(|ui| {
            ui.label(format!("{} {}", self.current_material.material_type.icon(), self.current_material.name));
            ui.separator();
            ui.label(format!("Type: {:?}", self.current_material.material_type));
            ui.label(format!("| Blend: {:?}", self.current_material.blend_mode));
        });

        ui.separator();
    }

    fn show_properties_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎨 Material Properties");
        ui.add_space(10.0);

        // Material selector
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("material_select")
                .selected_text(&self.current_material.name)
                .show_ui(ui, |ui| {
                    for mat in &self.materials.clone() {
                        if ui.selectable_value(&mut self.selected_material, Some(mat.id), &mat.name).clicked() {
                            self.current_material = mat.clone();
                        }
                    }
                });

            if ui.button("+ New").clicked() {
                let id = self.next_id();
                let new_mat = Material {
                    id,
                    name: format!("Material {}", id),
                    ..Default::default()
                };
                self.materials.push(new_mat.clone());
                self.current_material = new_mat;
                self.selected_material = Some(id);
            }
        });

        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                // Basic properties
                ui.group(|ui| {
                    ui.label(RichText::new("📝 Basic").strong());

                    egui::Grid::new("basic_props")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.current_material.name);
                            ui.end_row();

                            ui.label("Type:");
                            egui::ComboBox::from_id_salt("mat_type")
                                .selected_text(format!("{} {:?}", self.current_material.material_type.icon(), self.current_material.material_type))
                                .show_ui(ui, |ui| {
                                    for t in MaterialType::all() {
                                        ui.selectable_value(&mut self.current_material.material_type, *t, format!("{} {:?}", t.icon(), t));
                                    }
                                });
                            ui.end_row();

                            ui.label("Blend Mode:");
                            egui::ComboBox::from_id_salt("blend_mode")
                                .selected_text(format!("{:?}", self.current_material.blend_mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.current_material.blend_mode, BlendMode::Opaque, "Opaque");
                                    ui.selectable_value(&mut self.current_material.blend_mode, BlendMode::Masked, "Masked");
                                    ui.selectable_value(&mut self.current_material.blend_mode, BlendMode::Translucent, "Translucent");
                                    ui.selectable_value(&mut self.current_material.blend_mode, BlendMode::Additive, "Additive");
                                });
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // PBR properties
                ui.group(|ui| {
                    ui.label(RichText::new("🎨 PBR Properties").strong());

                    egui::Grid::new("pbr_props")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Base Color:");
                            let mut color = Color32::from_rgba_unmultiplied(
                                (self.current_material.base_color[0] * 255.0) as u8,
                                (self.current_material.base_color[1] * 255.0) as u8,
                                (self.current_material.base_color[2] * 255.0) as u8,
                                (self.current_material.base_color[3] * 255.0) as u8,
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                self.current_material.base_color = [
                                    color.r() as f32 / 255.0,
                                    color.g() as f32 / 255.0,
                                    color.b() as f32 / 255.0,
                                    color.a() as f32 / 255.0,
                                ];
                            }
                            ui.end_row();

                            ui.label("Metallic:");
                            ui.add(egui::Slider::new(&mut self.current_material.metallic, 0.0..=1.0));
                            ui.end_row();

                            ui.label("Roughness:");
                            ui.add(egui::Slider::new(&mut self.current_material.roughness, 0.0..=1.0));
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // Emissive
                ui.group(|ui| {
                    ui.label(RichText::new("✨ Emissive").strong());

                    egui::Grid::new("emissive_props")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Color:");
                            let mut color = Color32::from_rgb(
                                (self.current_material.emissive_color[0] * 255.0) as u8,
                                (self.current_material.emissive_color[1] * 255.0) as u8,
                                (self.current_material.emissive_color[2] * 255.0) as u8,
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                self.current_material.emissive_color = [
                                    color.r() as f32 / 255.0,
                                    color.g() as f32 / 255.0,
                                    color.b() as f32 / 255.0,
                                ];
                            }
                            ui.end_row();

                            ui.label("Intensity:");
                            ui.add(egui::Slider::new(&mut self.current_material.emissive_intensity, 0.0..=10.0));
                            ui.end_row();
                        });
                });
            });
    }

    fn show_textures_tab(&mut self, ui: &mut Ui) {
        ui.heading("🖼️ Texture Slots");
        ui.add_space(10.0);

        // Add texture slot
        ui.horizontal(|ui| {
            if ui.button("+ Add Texture Slot").clicked() {
                self.current_material.texture_slots.push(TextureSlot::default());
            }
        });

        ui.add_space(10.0);

        if self.current_material.texture_slots.is_empty() {
            ui.label("No texture slots. Click '+ Add Texture Slot' to add one.");
        } else {
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    let mut to_remove = None;

                    for (idx, slot) in self.current_material.texture_slots.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut slot.enabled, "");
                                ui.label(RichText::new(slot.channel.name()).strong());

                                if ui.button("🗑️").clicked() {
                                    to_remove = Some(idx);
                                }
                            });

                            if slot.enabled {
                                egui::Grid::new(format!("texture_{}", idx))
                                    .num_columns(2)
                                    .spacing([10.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Channel:");
                                        egui::ComboBox::from_id_salt(format!("channel_{}", idx))
                                            .selected_text(slot.channel.name())
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(&mut slot.channel, TextureChannel::Albedo, "Albedo");
                                                ui.selectable_value(&mut slot.channel, TextureChannel::Normal, "Normal");
                                                ui.selectable_value(&mut slot.channel, TextureChannel::Metallic, "Metallic");
                                                ui.selectable_value(&mut slot.channel, TextureChannel::Roughness, "Roughness");
                                                ui.selectable_value(&mut slot.channel, TextureChannel::AO, "Ambient Occlusion");
                                                ui.selectable_value(&mut slot.channel, TextureChannel::Emissive, "Emissive");
                                                ui.selectable_value(&mut slot.channel, TextureChannel::Height, "Height");
                                                ui.selectable_value(&mut slot.channel, TextureChannel::Opacity, "Opacity");
                                            });
                                        ui.end_row();

                                        ui.label("Path:");
                                        ui.horizontal(|ui| {
                                            ui.text_edit_singleline(&mut slot.texture_path);
                                            if ui.button("📂").clicked() {
                                                // Open file dialog
                                            }
                                        });
                                        ui.end_row();

                                        ui.label("Tiling:");
                                        ui.horizontal(|ui| {
                                            ui.add(egui::DragValue::new(&mut slot.tiling.0).prefix("U:").speed(0.1));
                                            ui.add(egui::DragValue::new(&mut slot.tiling.1).prefix("V:").speed(0.1));
                                        });
                                        ui.end_row();

                                        ui.label("Offset:");
                                        ui.horizontal(|ui| {
                                            ui.add(egui::DragValue::new(&mut slot.offset.0).prefix("U:").speed(0.01));
                                            ui.add(egui::DragValue::new(&mut slot.offset.1).prefix("V:").speed(0.01));
                                        });
                                        ui.end_row();

                                        ui.label("Intensity:");
                                        ui.add(egui::Slider::new(&mut slot.intensity, 0.0..=2.0));
                                        ui.end_row();
                                    });
                            }
                        });
                    }

                    if let Some(idx) = to_remove {
                        self.current_material.texture_slots.remove(idx);
                    }
                });
        }
    }

    fn show_advanced_tab(&mut self, ui: &mut Ui) {
        ui.heading("⚙️ Advanced Properties");
        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                // Rendering options
                ui.group(|ui| {
                    ui.label(RichText::new("🖥️ Rendering").strong());

                    ui.checkbox(&mut self.current_material.two_sided, "Two Sided");
                    ui.checkbox(&mut self.current_material.cast_shadows, "Cast Shadows");
                    ui.checkbox(&mut self.current_material.receive_shadows, "Receive Shadows");

                    if self.current_material.blend_mode == BlendMode::Masked {
                        ui.horizontal(|ui| {
                            ui.label("Alpha Cutoff:");
                            ui.add(egui::Slider::new(&mut self.current_material.alpha_cutoff, 0.0..=1.0));
                        });
                    }
                });

                ui.add_space(10.0);

                // Subsurface scattering (for skin, wax, etc.)
                if self.current_material.material_type == MaterialType::Subsurface {
                    ui.group(|ui| {
                        ui.label(RichText::new("🧴 Subsurface Scattering").strong());

                        egui::Grid::new("sss_props")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Color:");
                                let mut color = Color32::from_rgb(
                                    (self.current_material.subsurface_color[0] * 255.0) as u8,
                                    (self.current_material.subsurface_color[1] * 255.0) as u8,
                                    (self.current_material.subsurface_color[2] * 255.0) as u8,
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    self.current_material.subsurface_color = [
                                        color.r() as f32 / 255.0,
                                        color.g() as f32 / 255.0,
                                        color.b() as f32 / 255.0,
                                    ];
                                }
                                ui.end_row();

                                ui.label("Radius:");
                                ui.add(egui::Slider::new(&mut self.current_material.subsurface_radius, 0.0..=5.0));
                                ui.end_row();
                            });
                    });

                    ui.add_space(10.0);
                }

                // Glass/Translucent properties
                if self.current_material.material_type == MaterialType::Glass || 
                   self.current_material.blend_mode == BlendMode::Translucent {
                    ui.group(|ui| {
                        ui.label(RichText::new("🔮 Transmission").strong());

                        egui::Grid::new("trans_props")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("IOR:");
                                ui.add(egui::Slider::new(&mut self.current_material.ior, 1.0..=3.0));
                                ui.end_row();

                                ui.label("Transmission:");
                                ui.add(egui::Slider::new(&mut self.current_material.transmission, 0.0..=1.0));
                                ui.end_row();
                            });
                    });
                }
            });
    }

    fn show_presets_tab(&mut self, ui: &mut Ui) {
        ui.heading("📋 Material Presets");
        ui.add_space(10.0);

        // Filter
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.preset_filter);
        });

        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                let mut current_category = String::new();

                for preset in &self.presets {
                    // Filter
                    if !self.preset_filter.is_empty() && 
                       !preset.name.to_lowercase().contains(&self.preset_filter.to_lowercase()) {
                        continue;
                    }

                    // Category header
                    if preset.category != current_category {
                        current_category = preset.category.clone();
                        ui.add_space(5.0);
                        ui.label(RichText::new(&current_category).strong().color(Color32::from_rgb(150, 150, 200)));
                    }

                    ui.horizontal(|ui| {
                        // Color preview
                        let color = Color32::from_rgba_unmultiplied(
                            (preset.base_color[0] * 255.0) as u8,
                            (preset.base_color[1] * 255.0) as u8,
                            (preset.base_color[2] * 255.0) as u8,
                            255,
                        );
                        let color_rect = ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::hover()).0;
                        ui.painter().rect_filled(color_rect, 3.0, color);

                        ui.label(format!("{} {}", preset.material_type.icon(), preset.name));

                        if ui.button("Apply").clicked() {
                            self.current_material.material_type = preset.material_type;
                            self.current_material.base_color = preset.base_color;
                            self.current_material.metallic = preset.metallic;
                            self.current_material.roughness = preset.roughness;
                        }
                    });
                }
            });
    }

    fn show_preview_tab(&mut self, ui: &mut Ui) {
        ui.heading("👁️ Material Preview");
        ui.add_space(10.0);

        // Preview controls
        ui.horizontal(|ui| {
            ui.label("Lighting:");
            egui::ComboBox::from_id_salt("preview_lighting")
                .selected_text(format!("{:?}", self.preview_lighting))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.preview_lighting, PreviewLighting::Studio, "Studio");
                    ui.selectable_value(&mut self.preview_lighting, PreviewLighting::Outdoor, "Outdoor");
                    ui.selectable_value(&mut self.preview_lighting, PreviewLighting::Indoor, "Indoor");
                    ui.selectable_value(&mut self.preview_lighting, PreviewLighting::Dramatic, "Dramatic");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Rotation:");
            ui.add(egui::Slider::new(&mut self.preview_rotation, 0.0..=360.0).suffix("°"));
        });

        ui.horizontal(|ui| {
            ui.label("Zoom:");
            ui.add(egui::Slider::new(&mut self.preview_zoom, 0.5..=3.0));
        });

        ui.add_space(10.0);

        // Preview area (placeholder)
        let preview_size = Vec2::new(ui.available_width(), 200.0);
        let (rect, _response) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

        let painter = ui.painter();
        painter.rect_filled(rect, 5.0, Color32::from_rgb(30, 30, 35));

        // Draw a simple sphere representation
        let center = rect.center();
        let radius = 60.0 * self.preview_zoom;

        let base_color = Color32::from_rgba_unmultiplied(
            (self.current_material.base_color[0] * 255.0) as u8,
            (self.current_material.base_color[1] * 255.0) as u8,
            (self.current_material.base_color[2] * 255.0) as u8,
            (self.current_material.base_color[3] * 255.0) as u8,
        );

        painter.circle_filled(center, radius, base_color);

        // Highlight (simulating metallic/roughness)
        let highlight_intensity = (1.0 - self.current_material.roughness) * 0.7;
        let highlight_color = Color32::from_rgba_unmultiplied(
            255,
            255,
            255,
            (highlight_intensity * 200.0) as u8,
        );
        painter.circle_filled(
            egui::Pos2::new(center.x - radius * 0.3, center.y - radius * 0.3),
            radius * 0.3,
            highlight_color,
        );

        // Material info overlay
        painter.text(
            egui::Pos2::new(rect.min.x + 10.0, rect.max.y - 20.0),
            egui::Align2::LEFT_BOTTOM,
            format!("M: {:.2}  R: {:.2}", self.current_material.metallic, self.current_material.roughness),
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
    }

    fn show_library_tab(&mut self, ui: &mut Ui) {
        ui.heading("📁 Material Library");
        ui.add_space(10.0);

        // Library path
        ui.horizontal(|ui| {
            ui.label("Path:");
            ui.text_edit_singleline(&mut self.library_path);
            if ui.button("🔄 Refresh").clicked() {
                // Refresh library
            }
        });

        ui.add_space(10.0);

        // Actions
        ui.horizontal(|ui| {
            if ui.button("💾 Save Current").clicked() {
                // Save current material
            }
            if ui.button("📁 Load").clicked() {
                // Load material
            }
            if ui.button("📋 Duplicate").clicked() {
                // Duplicate material
            }
        });

        ui.add_space(10.0);

        // Library list
        ui.group(|ui| {
            ui.label(RichText::new("Available Materials").strong());

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for mat_path in &self.library_materials {
                        ui.horizontal(|ui| {
                            ui.label("📄");
                            if ui.selectable_label(false, mat_path).clicked() {
                                // Load this material
                            }
                        });
                    }
                });
        });
    }

    // Getters for testing
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub fn preset_count(&self) -> usize {
        self.presets.len()
    }

    pub fn current_material_name(&self) -> &str {
        &self.current_material.name
    }

    pub fn texture_slot_count(&self) -> usize {
        self.current_material.texture_slots.len()
    }

    pub fn add_material(&mut self, name: &str) -> u32 {
        let id = self.next_id();
        self.materials.push(Material {
            id,
            name: name.to_string(),
            ..Default::default()
        });
        id
    }

    pub fn set_metallic(&mut self, value: f32) {
        self.current_material.metallic = value.clamp(0.0, 1.0);
    }

    pub fn set_roughness(&mut self, value: f32) {
        self.current_material.roughness = value.clamp(0.0, 1.0);
    }

    pub fn add_texture_slot(&mut self, channel: TextureChannel, path: &str) {
        self.current_material.texture_slots.push(TextureSlot {
            channel,
            texture_path: path.to_string(),
            ..Default::default()
        });
    }
}

impl Panel for MaterialEditorPanel {
    fn name(&self) -> &'static str {
        "Material Editor"
    }

    fn show(&mut self, ui: &mut Ui) {
        self.show_tab_bar(ui);

        match self.active_tab {
            MaterialTab::Properties => self.show_properties_tab(ui),
            MaterialTab::Textures => self.show_textures_tab(ui),
            MaterialTab::Advanced => self.show_advanced_tab(ui),
            MaterialTab::Presets => self.show_presets_tab(ui),
            MaterialTab::Preview => self.show_preview_tab(ui),
            MaterialTab::Library => self.show_library_tab(ui),
        }
    }

    fn update(&mut self) {
        // Could auto-update preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_editor_panel_creation() {
        let panel = MaterialEditorPanel::new();
        assert!(!panel.current_material_name().is_empty());
    }

    #[test]
    fn test_default_sample_data() {
        let panel = MaterialEditorPanel::new();
        assert!(panel.material_count() >= 3);
        assert!(panel.preset_count() >= 10);
    }

    #[test]
    fn test_add_material() {
        let mut panel = MaterialEditorPanel::new();
        let initial_count = panel.material_count();

        let id = panel.add_material("Test Material");
        assert!(id > 0);
        assert_eq!(panel.material_count(), initial_count + 1);
    }

    #[test]
    fn test_set_metallic() {
        let mut panel = MaterialEditorPanel::new();
        panel.set_metallic(0.75);
        assert!((panel.current_material.metallic - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_set_roughness() {
        let mut panel = MaterialEditorPanel::new();
        panel.set_roughness(0.3);
        assert!((panel.current_material.roughness - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_add_texture_slot() {
        let mut panel = MaterialEditorPanel::new();
        let initial_count = panel.texture_slot_count();

        panel.add_texture_slot(TextureChannel::Normal, "textures/normal.png");
        assert_eq!(panel.texture_slot_count(), initial_count + 1);
    }

    #[test]
    fn test_material_type_icons() {
        assert_eq!(MaterialType::StandardPBR.icon(), "🎨");
        assert_eq!(MaterialType::Glass.icon(), "🔮");
        assert_eq!(MaterialType::Water.icon(), "💧");
    }

    #[test]
    fn test_texture_channel_names() {
        assert_eq!(TextureChannel::Albedo.name(), "Albedo");
        assert_eq!(TextureChannel::Normal.name(), "Normal");
        assert_eq!(TextureChannel::Roughness.name(), "Roughness");
    }

    #[test]
    fn test_metallic_clamping() {
        let mut panel = MaterialEditorPanel::new();
        panel.set_metallic(1.5);
        assert!((panel.current_material.metallic - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_panel_trait_implementation() {
        let panel = MaterialEditorPanel::new();
        assert_eq!(panel.name(), "Material Editor");
    }
}
