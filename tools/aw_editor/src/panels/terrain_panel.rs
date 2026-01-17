//! Terrain Panel - Advanced procedural terrain generation and editing UI
//!
//! Provides comprehensive controls for:
//! - Procedural terrain generation (seed, biome, chunk radius)
//! - Noise parameter tweaking (octaves, lacunarity, persistence)
//! - Advanced erosion simulation (hydraulic, thermal, wind)
//! - Biome blending with smooth transitions
//! - Texture splatting and material rules
//! - Fluid simulation and water body detection
//! - Real-time preview and regeneration
//! - Voxel brush tools for sculpting

use super::Panel;
use crate::terrain_integration::{all_biome_options, TerrainState};
use egui::{Color32, RichText, Ui};

/// Erosion preset types for quick configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErosionPresetType {
    Custom,
    Desert,
    Mountain,
    Coastal,
    Alpine,
    Canyon,
}

impl ErosionPresetType {
    fn name(&self) -> &'static str {
        match self {
            ErosionPresetType::Custom => "Custom",
            ErosionPresetType::Desert => "Desert",
            ErosionPresetType::Mountain => "Mountain",
            ErosionPresetType::Coastal => "Coastal",
            ErosionPresetType::Alpine => "Alpine",
            ErosionPresetType::Canyon => "Canyon",
        }
    }
    
    fn all() -> &'static [ErosionPresetType] {
        &[
            ErosionPresetType::Custom,
            ErosionPresetType::Desert,
            ErosionPresetType::Mountain,
            ErosionPresetType::Coastal,
            ErosionPresetType::Alpine,
            ErosionPresetType::Canyon,
        ]
    }
}

/// Configuration for hydraulic erosion
#[derive(Debug, Clone)]
pub struct HydraulicErosionParams {
    pub enabled: bool,
    pub iterations: u32,
    pub inertia: f32,
    pub capacity: f32,
    pub deposition: f32,
    pub erosion: f32,
    pub evaporation: f32,
    pub min_slope: f32,
    pub gravity: f32,
}

impl Default for HydraulicErosionParams {
    fn default() -> Self {
        Self {
            enabled: true,
            iterations: 50000,
            inertia: 0.3,
            capacity: 8.0,
            deposition: 0.2,
            erosion: 0.5,
            evaporation: 0.02,
            min_slope: 0.01,
            gravity: 10.0,
        }
    }
}

/// Configuration for thermal erosion
#[derive(Debug, Clone)]
pub struct ThermalErosionParams {
    pub enabled: bool,
    pub iterations: u32,
    pub talus_angle: f32,
    pub erosion_rate: f32,
}

impl Default for ThermalErosionParams {
    fn default() -> Self {
        Self {
            enabled: true,
            iterations: 50,
            talus_angle: 40.0,
            erosion_rate: 0.5,
        }
    }
}

/// Configuration for wind erosion
#[derive(Debug, Clone)]
pub struct WindErosionParams {
    pub enabled: bool,
    pub iterations: u32,
    pub wind_direction: [f32; 2],
    pub wind_strength: f32,
    pub suspension: f32,
    pub abrasion: f32,
}

impl Default for WindErosionParams {
    fn default() -> Self {
        Self {
            enabled: false,
            iterations: 20,
            wind_direction: [1.0, 0.0],
            wind_strength: 0.5,
            suspension: 0.3,
            abrasion: 0.2,
        }
    }
}

/// Biome blending configuration
#[derive(Debug, Clone)]
pub struct BiomeBlendParams {
    pub enabled: bool,
    pub blend_radius: f32,
    pub falloff_power: f32,
    pub noise_influence: f32,
    pub secondary_biome: String,
    pub tertiary_biome: String,
    pub show_blend_preview: bool,
}

impl Default for BiomeBlendParams {
    fn default() -> Self {
        Self {
            enabled: true,
            blend_radius: 32.0,
            falloff_power: 2.0,
            noise_influence: 0.3,
            secondary_biome: "desert".to_string(),
            tertiary_biome: "mountains".to_string(),
            show_blend_preview: false,
        }
    }
}

/// Texture splatting configuration
#[derive(Debug, Clone)]
pub struct SplatParams {
    pub enabled: bool,
    pub show_splat_preview: bool,
    pub grass_height_min: f32,
    pub grass_height_max: f32,
    pub rock_slope_threshold: f32,
    pub snow_height_threshold: f32,
    pub sand_height_max: f32,
    pub triplanar_sharpness: f32,
}

impl Default for SplatParams {
    fn default() -> Self {
        Self {
            enabled: true,
            show_splat_preview: false,
            grass_height_min: 0.0,
            grass_height_max: 0.7,
            rock_slope_threshold: 0.6,
            snow_height_threshold: 0.85,
            sand_height_max: 0.15,
            triplanar_sharpness: 8.0,
        }
    }
}

/// Water body type for fluid placement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterBodyPreset {
    Custom,
    CalmLake,
    MountainStream,
    RagingRiver,
    Ocean,
    Waterfall,
    SwampWetland,
}

impl WaterBodyPreset {
    fn name(&self) -> &'static str {
        match self {
            WaterBodyPreset::Custom => "Custom",
            WaterBodyPreset::CalmLake => "Calm Lake",
            WaterBodyPreset::MountainStream => "Mountain Stream",
            WaterBodyPreset::RagingRiver => "Raging River",
            WaterBodyPreset::Ocean => "Ocean",
            WaterBodyPreset::Waterfall => "Waterfall",
            WaterBodyPreset::SwampWetland => "Swamp/Wetland",
        }
    }
    
    fn all() -> &'static [WaterBodyPreset] {
        &[
            WaterBodyPreset::Custom,
            WaterBodyPreset::CalmLake,
            WaterBodyPreset::MountainStream,
            WaterBodyPreset::RagingRiver,
            WaterBodyPreset::Ocean,
            WaterBodyPreset::Waterfall,
            WaterBodyPreset::SwampWetland,
        ]
    }
}

/// Fluid simulation quality preset
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidQualityPreset {
    Performance,
    Balanced,
    Quality,
    Cinematic,
}

impl FluidQualityPreset {
    fn name(&self) -> &'static str {
        match self {
            FluidQualityPreset::Performance => "Performance",
            FluidQualityPreset::Balanced => "Balanced",
            FluidQualityPreset::Quality => "Quality",
            FluidQualityPreset::Cinematic => "Cinematic",
        }
    }
}

/// Fluid simulation parameters for terrain integration
#[derive(Debug, Clone)]
pub struct FluidSimParams {
    pub enabled: bool,
    pub quality_preset: FluidQualityPreset,
    pub water_body_preset: WaterBodyPreset,
    
    // Physics
    pub particle_count: u32,
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub viscosity: f32,
    pub surface_tension: f32,
    pub gravity: f32,
    pub solver_iterations: u32,
    
    // Flow
    pub flow_enabled: bool,
    pub flow_speed: f32,
    pub flow_direction: [f32; 2],
    pub turbulence: f32,
    
    // Rendering
    pub water_color: [f32; 4],
    pub transparency: f32,
    pub refraction_strength: f32,
    pub caustics_enabled: bool,
    pub caustics_intensity: f32,
    pub foam_enabled: bool,
    pub foam_threshold: f32,
    
    // Thermal
    pub thermal_enabled: bool,
    pub thermal_diffusivity: f32,
    pub buoyancy: f32,
    
    // Detection
    pub auto_detect_water_bodies: bool,
    pub min_river_flow_threshold: f32,
    pub lake_depth_threshold: f32,
    pub waterfall_height_threshold: f32,
    
    // Emitters
    pub emitter_count: u32,
    pub spawn_rate: f32,
    pub initial_velocity: f32,
}

impl Default for FluidSimParams {
    fn default() -> Self {
        Self {
            enabled: true,
            quality_preset: FluidQualityPreset::Balanced,
            water_body_preset: WaterBodyPreset::CalmLake,
            
            // Physics
            particle_count: 65536,
            smoothing_radius: 1.0,
            target_density: 12.0,
            pressure_multiplier: 300.0,
            viscosity: 10.0,
            surface_tension: 0.02,
            gravity: -9.8,
            solver_iterations: 4,
            
            // Flow
            flow_enabled: false,
            flow_speed: 1.0,
            flow_direction: [1.0, 0.0],
            turbulence: 0.1,
            
            // Rendering
            water_color: [0.2, 0.5, 0.8, 0.9],
            transparency: 0.7,
            refraction_strength: 0.5,
            caustics_enabled: true,
            caustics_intensity: 1.0,
            foam_enabled: true,
            foam_threshold: 0.3,
            
            // Thermal
            thermal_enabled: false,
            thermal_diffusivity: 0.1,
            buoyancy: 0.0002,
            
            // Detection
            auto_detect_water_bodies: true,
            min_river_flow_threshold: 500.0,
            lake_depth_threshold: 2.0,
            waterfall_height_threshold: 5.0,
            
            // Emitters
            emitter_count: 1,
            spawn_rate: 1000.0,
            initial_velocity: 0.0,
        }
    }
}

/// Detected water body information for display
#[derive(Debug, Clone)]
pub struct DetectedWaterBodyInfo {
    pub name: String,
    pub body_type: String,
    pub center: [f32; 3],
    pub volume: f32,
    pub particle_count: u32,
    pub flow_speed: Option<f32>,
    pub selected: bool,
}

/// Statistics for fluid simulation
#[derive(Default, Clone)]
pub struct FluidStats {
    pub active_particles: u32,
    pub emitter_count: u32,
    pub detected_bodies: u32,
    pub simulation_time_ms: f32,
    pub render_time_ms: f32,
}

/// Terrain generation and editing panel
pub struct TerrainPanel {
    /// Terrain generation state
    terrain_state: TerrainState,
    
    /// Generation parameters
    seed: u64,
    seed_string: String,
    primary_biome: String,
    chunk_radius: i32,
    
    /// Noise parameters
    octaves: u32,
    lacunarity: f32,
    persistence: f32,
    base_amplitude: f32,
    
    /// Erosion parameters
    erosion_preset: ErosionPresetType,
    hydraulic_erosion: HydraulicErosionParams,
    thermal_erosion: ThermalErosionParams,
    wind_erosion: WindErosionParams,
    
    /// Biome blending parameters
    biome_blend: BiomeBlendParams,
    
    /// Texture splatting parameters
    splat_params: SplatParams,
    
    /// Fluid simulation parameters
    fluid_params: FluidSimParams,
    fluid_stats: FluidStats,
    detected_water_bodies: Vec<DetectedWaterBodyInfo>,
    show_fluid_debug: bool,
    
    /// UI state
    auto_regenerate: bool,
    show_advanced: bool,
    last_generation_time_ms: f32,
    generation_stats: GenerationStats,
    
    /// Brush settings for voxel editing
    brush_mode: BrushMode,
    brush_radius: f32,
    brush_strength: f32,
    selected_material: usize,
}

/// Brush modes for terrain sculpting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushMode {
    Sculpt,
    Smooth,
    Flatten,
    Paint,
    Erode,
}

#[derive(Default, Clone)]
struct GenerationStats {
    chunks_generated: usize,
    total_vertices: usize,
    total_triangles: usize,
    memory_estimate_mb: f32,
    erosion_time_ms: f32,
    splatmap_time_ms: f32,
}

impl Default for TerrainPanel {
    fn default() -> Self {
        Self {
            terrain_state: TerrainState::new(),
            seed: 12345,
            seed_string: "12345".to_string(),
            primary_biome: "grassland".to_string(),
            chunk_radius: 2,
            octaves: 6,
            lacunarity: 2.0,
            persistence: 0.5,
            base_amplitude: 50.0,
            erosion_preset: ErosionPresetType::Mountain,
            hydraulic_erosion: HydraulicErosionParams::default(),
            thermal_erosion: ThermalErosionParams::default(),
            wind_erosion: WindErosionParams::default(),
            biome_blend: BiomeBlendParams::default(),
            splat_params: SplatParams::default(),
            fluid_params: FluidSimParams::default(),
            fluid_stats: FluidStats::default(),
            detected_water_bodies: Vec::new(),
            show_fluid_debug: false,
            auto_regenerate: false,
            show_advanced: false,
            last_generation_time_ms: 0.0,
            generation_stats: GenerationStats::default(),
            brush_mode: BrushMode::Sculpt,
            brush_radius: 5.0,
            brush_strength: 0.5,
            selected_material: 0,
        }
    }
}

impl TerrainPanel {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get reference to terrain state for rendering
    pub fn terrain_state(&self) -> &TerrainState {
        &self.terrain_state
    }
    
    /// Get mutable reference to terrain state
    pub fn terrain_state_mut(&mut self) -> &mut TerrainState {
        &mut self.terrain_state
    }
    
    /// Check if terrain needs regeneration
    pub fn needs_regeneration(&self) -> bool {
        self.terrain_state.is_dirty()
    }
    
    fn show_generation_section(&mut self, ui: &mut Ui) {
        ui.heading("🏔️ Terrain Generation");
        ui.separator();
        
        // Seed input
        ui.horizontal(|ui| {
            ui.label("Seed:");
            if ui.text_edit_singleline(&mut self.seed_string).changed() {
                if let Ok(new_seed) = self.seed_string.parse::<u64>() {
                    self.seed = new_seed;
                    self.terrain_state.configure(self.seed, &self.primary_biome);
                }
            }
            if ui.button("🎲").on_hover_text("Random seed").clicked() {
                self.seed = rand::random();
                self.seed_string = self.seed.to_string();
                self.terrain_state.configure(self.seed, &self.primary_biome);
            }
        });
        
        // Primary biome selection
        ui.horizontal(|ui| {
            ui.label("Primary Biome:");
            egui::ComboBox::from_id_salt("primary_biome")
                .selected_text(&self.primary_biome)
                .show_ui(ui, |ui| {
                    for (value, display) in all_biome_options() {
                        if ui.selectable_value(&mut self.primary_biome, value.to_string(), *display).clicked() {
                            self.terrain_state.configure(self.seed, &self.primary_biome);
                        }
                    }
                });
        });
        
        // Chunk radius slider
        ui.horizontal(|ui| {
            ui.label("Chunk Radius:");
            if ui.add(egui::Slider::new(&mut self.chunk_radius, 1..=5)).changed() {
                self.terrain_state.configure(self.seed, &self.primary_biome);
            }
            ui.label(format!("({} chunks)", (self.chunk_radius * 2 + 1).pow(2)));
        });
        
        ui.add_space(10.0);
        
        // Generate button
        let generate_text = if self.terrain_state.is_dirty() {
            RichText::new("🔄 Generate Terrain").color(Color32::YELLOW)
        } else {
            RichText::new("✅ Generate Terrain")
        };
        
        if ui.button(generate_text).clicked() {
            self.regenerate_terrain();
        }
        
        ui.checkbox(&mut self.auto_regenerate, "Auto-regenerate on change");
        
        // Stats
        if self.generation_stats.chunks_generated > 0 {
            ui.add_space(5.0);
            ui.group(|ui| {
                ui.label(RichText::new("Generation Stats").strong());
                ui.label(format!("Chunks: {}", self.generation_stats.chunks_generated));
                ui.label(format!("Vertices: {}", self.generation_stats.total_vertices));
                ui.label(format!("Triangles: {}", self.generation_stats.total_triangles));
                ui.label(format!("Memory: {:.2} MB", self.generation_stats.memory_estimate_mb));
                ui.label(format!("Time: {:.1} ms", self.last_generation_time_ms));
            });
        }
    }
    
    fn show_noise_section(&mut self, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.collapsing("🎛️ Noise Parameters", |ui| {
            let mut changed = false;
            
            ui.horizontal(|ui| {
                ui.label("Octaves:");
                changed |= ui.add(egui::Slider::new(&mut self.octaves, 1..=8)).changed();
            });
            
            ui.horizontal(|ui| {
                ui.label("Lacunarity:");
                changed |= ui.add(egui::Slider::new(&mut self.lacunarity, 1.5..=3.0)).changed();
            });
            
            ui.horizontal(|ui| {
                ui.label("Persistence:");
                changed |= ui.add(egui::Slider::new(&mut self.persistence, 0.1..=0.9)).changed();
            });
            
            ui.horizontal(|ui| {
                ui.label("Amplitude:");
                changed |= ui.add(egui::Slider::new(&mut self.base_amplitude, 10.0..=200.0)).changed();
            });
            
            if changed {
                self.terrain_state.configure(self.seed, &self.primary_biome);
                if self.auto_regenerate {
                    self.regenerate_terrain();
                }
            }
            
            if ui.button("Reset to Defaults").clicked() {
                self.octaves = 6;
                self.lacunarity = 2.0;
                self.persistence = 0.5;
                self.base_amplitude = 50.0;
                self.terrain_state.configure(self.seed, &self.primary_biome);
            }
        });
    }
    
    fn show_brush_section(&mut self, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.collapsing("🖌️ Sculpting Brushes", |ui| {
            ui.horizontal(|ui| {
                ui.label("Mode:");
                ui.selectable_value(&mut self.brush_mode, BrushMode::Sculpt, "Sculpt");
                ui.selectable_value(&mut self.brush_mode, BrushMode::Smooth, "Smooth");
                ui.selectable_value(&mut self.brush_mode, BrushMode::Flatten, "Flatten");
                ui.selectable_value(&mut self.brush_mode, BrushMode::Paint, "Paint");
                ui.selectable_value(&mut self.brush_mode, BrushMode::Erode, "Erode");
            });
            
            ui.horizontal(|ui| {
                ui.label("Radius:");
                ui.add(egui::Slider::new(&mut self.brush_radius, 1.0..=50.0));
            });
            
            ui.horizontal(|ui| {
                ui.label("Strength:");
                ui.add(egui::Slider::new(&mut self.brush_strength, 0.0..=1.0));
            });
            
            if self.brush_mode == BrushMode::Paint {
                ui.horizontal(|ui| {
                    ui.label("Material:");
                    egui::ComboBox::from_id_salt("brush_material")
                        .selected_text(Self::material_name(self.selected_material))
                        .show_ui(ui, |ui| {
                            for i in 0..8 {
                                ui.selectable_value(&mut self.selected_material, i, Self::material_name(i));
                            }
                        });
                });
            }
            
            if self.brush_mode == BrushMode::Erode {
                ui.label(RichText::new("Applies localized hydraulic erosion").small().italics());
            }
            
            ui.add_space(5.0);
            ui.label(RichText::new("Tip: Use in viewport with Shift+Click").small().italics());
        });
    }
    
    fn show_erosion_section(&mut self, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.collapsing("🌊 Erosion Simulation", |ui| {
            // Preset selection
            ui.horizontal(|ui| {
                ui.label("Preset:");
                egui::ComboBox::from_id_salt("erosion_preset")
                    .selected_text(self.erosion_preset.name())
                    .show_ui(ui, |ui| {
                        for preset in ErosionPresetType::all() {
                            if ui.selectable_value(&mut self.erosion_preset, *preset, preset.name()).clicked() {
                                self.apply_erosion_preset(*preset);
                            }
                        }
                    });
            });
            
            ui.separator();
            
            // Hydraulic erosion
            ui.collapsing("💧 Hydraulic Erosion", |ui| {
                ui.checkbox(&mut self.hydraulic_erosion.enabled, "Enabled");
                
                if self.hydraulic_erosion.enabled {
                    ui.horizontal(|ui| {
                        ui.label("Iterations:");
                        ui.add(egui::Slider::new(&mut self.hydraulic_erosion.iterations, 1000..=200000).logarithmic(true));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Inertia:");
                        ui.add(egui::Slider::new(&mut self.hydraulic_erosion.inertia, 0.0..=1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Capacity:");
                        ui.add(egui::Slider::new(&mut self.hydraulic_erosion.capacity, 1.0..=20.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Deposition:");
                        ui.add(egui::Slider::new(&mut self.hydraulic_erosion.deposition, 0.0..=1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Erosion Rate:");
                        ui.add(egui::Slider::new(&mut self.hydraulic_erosion.erosion, 0.0..=1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Evaporation:");
                        ui.add(egui::Slider::new(&mut self.hydraulic_erosion.evaporation, 0.0..=0.1));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Min Slope:");
                        ui.add(egui::Slider::new(&mut self.hydraulic_erosion.min_slope, 0.001..=0.1).logarithmic(true));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Gravity:");
                        ui.add(egui::Slider::new(&mut self.hydraulic_erosion.gravity, 1.0..=20.0));
                    });
                }
            });
            
            // Thermal erosion
            ui.collapsing("🔥 Thermal Erosion", |ui| {
                ui.checkbox(&mut self.thermal_erosion.enabled, "Enabled");
                
                if self.thermal_erosion.enabled {
                    ui.horizontal(|ui| {
                        ui.label("Iterations:");
                        ui.add(egui::Slider::new(&mut self.thermal_erosion.iterations, 1..=200));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Talus Angle (°):");
                        ui.add(egui::Slider::new(&mut self.thermal_erosion.talus_angle, 20.0..=60.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Erosion Rate:");
                        ui.add(egui::Slider::new(&mut self.thermal_erosion.erosion_rate, 0.0..=1.0));
                    });
                }
            });
            
            // Wind erosion
            ui.collapsing("💨 Wind Erosion", |ui| {
                ui.checkbox(&mut self.wind_erosion.enabled, "Enabled");
                
                if self.wind_erosion.enabled {
                    ui.horizontal(|ui| {
                        ui.label("Iterations:");
                        ui.add(egui::Slider::new(&mut self.wind_erosion.iterations, 1..=100));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Direction X:");
                        ui.add(egui::Slider::new(&mut self.wind_erosion.wind_direction[0], -1.0..=1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Direction Y:");
                        ui.add(egui::Slider::new(&mut self.wind_erosion.wind_direction[1], -1.0..=1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Strength:");
                        ui.add(egui::Slider::new(&mut self.wind_erosion.wind_strength, 0.0..=1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Suspension:");
                        ui.add(egui::Slider::new(&mut self.wind_erosion.suspension, 0.0..=1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Abrasion:");
                        ui.add(egui::Slider::new(&mut self.wind_erosion.abrasion, 0.0..=1.0));
                    });
                }
            });
            
            ui.add_space(5.0);
            
            // Apply erosion button
            if ui.button(RichText::new("⚡ Apply Erosion").color(Color32::LIGHT_BLUE)).clicked() {
                self.apply_erosion();
            }
            
            if self.generation_stats.erosion_time_ms > 0.0 {
                ui.label(format!("Last erosion: {:.1} ms", self.generation_stats.erosion_time_ms));
            }
        });
    }
    
    fn show_biome_blend_section(&mut self, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.collapsing("🌍 Biome Blending", |ui| {
            ui.checkbox(&mut self.biome_blend.enabled, "Enable Biome Blending");
            
            if self.biome_blend.enabled {
                // Secondary biome
                ui.horizontal(|ui| {
                    ui.label("Secondary Biome:");
                    egui::ComboBox::from_id_salt("secondary_biome")
                        .selected_text(&self.biome_blend.secondary_biome)
                        .show_ui(ui, |ui| {
                            for (value, display) in all_biome_options() {
                                ui.selectable_value(&mut self.biome_blend.secondary_biome, value.to_string(), *display);
                            }
                        });
                });
                
                // Tertiary biome
                ui.horizontal(|ui| {
                    ui.label("Tertiary Biome:");
                    egui::ComboBox::from_id_salt("tertiary_biome")
                        .selected_text(&self.biome_blend.tertiary_biome)
                        .show_ui(ui, |ui| {
                            for (value, display) in all_biome_options() {
                                ui.selectable_value(&mut self.biome_blend.tertiary_biome, value.to_string(), *display);
                            }
                        });
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Blend Radius:");
                    ui.add(egui::Slider::new(&mut self.biome_blend.blend_radius, 4.0..=128.0).logarithmic(true));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Falloff Power:");
                    ui.add(egui::Slider::new(&mut self.biome_blend.falloff_power, 0.5..=4.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Noise Influence:");
                    ui.add(egui::Slider::new(&mut self.biome_blend.noise_influence, 0.0..=1.0));
                });
                
                ui.checkbox(&mut self.biome_blend.show_blend_preview, "Show Blend Preview");
                
                if self.biome_blend.show_blend_preview {
                    ui.group(|ui| {
                        ui.label(RichText::new("Blend Preview").strong());
                        // Preview visualization would be rendered in viewport
                        ui.label("Preview overlay enabled in viewport");
                        
                        // Color legend
                        ui.horizontal(|ui| {
                            ui.colored_label(Color32::from_rgb(100, 200, 100), "■");
                            ui.label(&self.primary_biome);
                        });
                        ui.horizontal(|ui| {
                            ui.colored_label(Color32::from_rgb(200, 180, 100), "■");
                            ui.label(&self.biome_blend.secondary_biome);
                        });
                        ui.horizontal(|ui| {
                            ui.colored_label(Color32::from_rgb(150, 150, 180), "■");
                            ui.label(&self.biome_blend.tertiary_biome);
                        });
                    });
                }
            }
        });
    }
    
    fn show_splatting_section(&mut self, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.collapsing("🎨 Texture Splatting", |ui| {
            ui.checkbox(&mut self.splat_params.enabled, "Enable Texture Splatting");
            
            if self.splat_params.enabled {
                ui.separator();
                ui.label(RichText::new("Material Rules").strong());
                
                // Grass rules
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(100, 180, 80), "🌿");
                        ui.label("Grass");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height Range:");
                        ui.add(egui::DragValue::new(&mut self.splat_params.grass_height_min).speed(0.01).prefix("min: "));
                        ui.add(egui::DragValue::new(&mut self.splat_params.grass_height_max).speed(0.01).prefix("max: "));
                    });
                });
                
                // Rock rules
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(120, 110, 100), "🪨");
                        ui.label("Rock");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Slope Threshold:");
                        ui.add(egui::Slider::new(&mut self.splat_params.rock_slope_threshold, 0.0..=1.0));
                    });
                });
                
                // Snow rules
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::WHITE, "❄️");
                        ui.label("Snow");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height Threshold:");
                        ui.add(egui::Slider::new(&mut self.splat_params.snow_height_threshold, 0.0..=1.0));
                    });
                });
                
                // Sand rules
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(220, 200, 150), "🏖️");
                        ui.label("Sand");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Max Height:");
                        ui.add(egui::Slider::new(&mut self.splat_params.sand_height_max, 0.0..=0.5));
                    });
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Triplanar Sharpness:");
                    ui.add(egui::Slider::new(&mut self.splat_params.triplanar_sharpness, 1.0..=16.0));
                });
                
                ui.checkbox(&mut self.splat_params.show_splat_preview, "Show Splat Preview");
                
                if self.splat_params.show_splat_preview {
                    ui.label(RichText::new("Splatmap visualization enabled in viewport").small().italics());
                }
                
                ui.add_space(5.0);
                
                if ui.button(RichText::new("🔄 Regenerate Splatmaps")).clicked() {
                    self.regenerate_splatmaps();
                }
                
                if self.generation_stats.splatmap_time_ms > 0.0 {
                    ui.label(format!("Last splatmap: {:.1} ms", self.generation_stats.splatmap_time_ms));
                }
            }
        });
    }
    
    fn apply_erosion_preset(&mut self, preset: ErosionPresetType) {
        match preset {
            ErosionPresetType::Custom => {
                // Don't modify anything for custom
            }
            ErosionPresetType::Desert => {
                self.hydraulic_erosion.enabled = false;
                self.thermal_erosion.enabled = true;
                self.thermal_erosion.iterations = 100;
                self.thermal_erosion.talus_angle = 35.0;
                self.wind_erosion.enabled = true;
                self.wind_erosion.iterations = 50;
                self.wind_erosion.wind_strength = 0.7;
            }
            ErosionPresetType::Mountain => {
                self.hydraulic_erosion.enabled = true;
                self.hydraulic_erosion.iterations = 100000;
                self.hydraulic_erosion.capacity = 10.0;
                self.thermal_erosion.enabled = true;
                self.thermal_erosion.iterations = 80;
                self.thermal_erosion.talus_angle = 45.0;
                self.wind_erosion.enabled = false;
            }
            ErosionPresetType::Coastal => {
                self.hydraulic_erosion.enabled = true;
                self.hydraulic_erosion.iterations = 80000;
                self.hydraulic_erosion.capacity = 6.0;
                self.hydraulic_erosion.erosion = 0.7;
                self.thermal_erosion.enabled = false;
                self.wind_erosion.enabled = true;
                self.wind_erosion.wind_strength = 0.4;
            }
            ErosionPresetType::Alpine => {
                self.hydraulic_erosion.enabled = true;
                self.hydraulic_erosion.iterations = 150000;
                self.hydraulic_erosion.capacity = 12.0;
                self.thermal_erosion.enabled = true;
                self.thermal_erosion.iterations = 60;
                self.thermal_erosion.talus_angle = 50.0;
                self.wind_erosion.enabled = false;
            }
            ErosionPresetType::Canyon => {
                self.hydraulic_erosion.enabled = true;
                self.hydraulic_erosion.iterations = 200000;
                self.hydraulic_erosion.capacity = 15.0;
                self.hydraulic_erosion.erosion = 0.8;
                self.thermal_erosion.enabled = true;
                self.thermal_erosion.iterations = 40;
                self.thermal_erosion.talus_angle = 55.0;
                self.wind_erosion.enabled = false;
            }
        }
    }
    
    fn apply_erosion(&mut self) {
        let start = std::time::Instant::now();
        
        // In a real implementation, this would call the erosion systems
        // For now, just track the timing
        tracing::info!("Applying erosion with preset: {:?}", self.erosion_preset);
        tracing::info!("Hydraulic: enabled={}, iterations={}", 
            self.hydraulic_erosion.enabled, self.hydraulic_erosion.iterations);
        tracing::info!("Thermal: enabled={}, iterations={}", 
            self.thermal_erosion.enabled, self.thermal_erosion.iterations);
        tracing::info!("Wind: enabled={}, iterations={}", 
            self.wind_erosion.enabled, self.wind_erosion.iterations);
        
        self.generation_stats.erosion_time_ms = start.elapsed().as_secs_f32() * 1000.0;
    }
    
    fn regenerate_splatmaps(&mut self) {
        let start = std::time::Instant::now();
        
        // In a real implementation, this would regenerate splatmaps
        tracing::info!("Regenerating splatmaps with params: {:?}", self.splat_params);
        
        self.generation_stats.splatmap_time_ms = start.elapsed().as_secs_f32() * 1000.0;
    }
    
    fn material_name(id: usize) -> &'static str {
        match id {
            0 => "Grass",
            1 => "Sand",
            2 => "Rock",
            3 => "Snow",
            4 => "Dirt",
            5 => "Mud",
            6 => "Gravel",
            7 => "Clay",
            _ => "Unknown",
        }
    }
    
    fn regenerate_terrain(&mut self) {
        let start = std::time::Instant::now();
        
        match self.terrain_state.generate_terrain(self.chunk_radius) {
            Ok(count) => {
                self.last_generation_time_ms = start.elapsed().as_secs_f32() * 1000.0;
                
                let all_vertices = self.terrain_state.get_all_vertices();
                let vertex_count = all_vertices.len();
                let triangle_count = self.terrain_state.get_all_indices(0).len() / 3;
                
                self.generation_stats = GenerationStats {
                    chunks_generated: count,
                    total_vertices: vertex_count,
                    total_triangles: triangle_count,
                    memory_estimate_mb: (vertex_count * std::mem::size_of::<crate::terrain_integration::TerrainVertex>()) as f32 / (1024.0 * 1024.0),
                    erosion_time_ms: 0.0,
                    splatmap_time_ms: 0.0,
                };
            }
            Err(e) => {
                tracing::error!("Terrain generation failed: {}", e);
            }
        }
    }
}

impl Panel for TerrainPanel {
    fn name(&self) -> &str {
        "Terrain"
    }
    
    fn show(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_generation_section(ui);
            self.show_noise_section(ui);
            self.show_erosion_section(ui);
            self.show_biome_blend_section(ui);
            self.show_splatting_section(ui);
            self.show_brush_section(ui);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_terrain_panel_creation() {
        let panel = TerrainPanel::new();
        assert_eq!(panel.seed, 12345);
        assert_eq!(panel.primary_biome, "grassland");
        assert_eq!(panel.chunk_radius, 2);
    }
    
    #[test]
    fn test_default_noise_params() {
        let panel = TerrainPanel::new();
        assert_eq!(panel.octaves, 6);
        assert!((panel.lacunarity - 2.0).abs() < 0.01);
        assert!((panel.persistence - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn test_brush_modes() {
        let mut panel = TerrainPanel::new();
        assert_eq!(panel.brush_mode, BrushMode::Sculpt);
        
        panel.brush_mode = BrushMode::Smooth;
        assert_eq!(panel.brush_mode, BrushMode::Smooth);
        
        panel.brush_mode = BrushMode::Erode;
        assert_eq!(panel.brush_mode, BrushMode::Erode);
    }
    
    #[test]
    fn test_material_names() {
        assert_eq!(TerrainPanel::material_name(0), "Grass");
        assert_eq!(TerrainPanel::material_name(2), "Rock");
        assert_eq!(TerrainPanel::material_name(99), "Unknown");
    }
    
    #[test]
    fn test_erosion_presets() {
        let mut panel = TerrainPanel::new();
        assert_eq!(panel.erosion_preset, ErosionPresetType::Mountain);
        
        panel.apply_erosion_preset(ErosionPresetType::Desert);
        assert!(!panel.hydraulic_erosion.enabled);
        assert!(panel.thermal_erosion.enabled);
        assert!(panel.wind_erosion.enabled);
        
        panel.apply_erosion_preset(ErosionPresetType::Coastal);
        assert!(panel.hydraulic_erosion.enabled);
        assert!(!panel.thermal_erosion.enabled);
        assert!(panel.wind_erosion.enabled);
    }
    
    #[test]
    fn test_default_erosion_params() {
        let panel = TerrainPanel::new();
        assert!(panel.hydraulic_erosion.enabled);
        assert_eq!(panel.hydraulic_erosion.iterations, 50000);
        assert!(panel.thermal_erosion.enabled);
        assert!(!panel.wind_erosion.enabled);
    }
    
    #[test]
    fn test_biome_blend_defaults() {
        let panel = TerrainPanel::new();
        assert!(panel.biome_blend.enabled);
        assert!((panel.biome_blend.blend_radius - 32.0).abs() < 0.01);
        assert_eq!(panel.biome_blend.secondary_biome, "desert");
        assert_eq!(panel.biome_blend.tertiary_biome, "mountains");
    }
    
    #[test]
    fn test_splat_params_defaults() {
        let panel = TerrainPanel::new();
        assert!(panel.splat_params.enabled);
        assert!((panel.splat_params.rock_slope_threshold - 0.6).abs() < 0.01);
        assert!((panel.splat_params.snow_height_threshold - 0.85).abs() < 0.01);
        assert!((panel.splat_params.triplanar_sharpness - 8.0).abs() < 0.01);
    }
    
    #[test]
    fn test_erosion_preset_all() {
        let presets = ErosionPresetType::all();
        assert_eq!(presets.len(), 6);
        assert!(presets.contains(&ErosionPresetType::Custom));
        assert!(presets.contains(&ErosionPresetType::Canyon));
    }
}
