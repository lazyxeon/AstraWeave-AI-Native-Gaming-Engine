//! Material Inspector Module - Phase PBR-G Task 2 + Task 3
//!
//! Provides comprehensive material inspection and validation capabilities:
//! - Texture map viewing (albedo, normal, ORM)
//! - Channel isolation (R/G/B/A individual views)
//! - Color space toggling (linear vs sRGB)
//! - Validation integration (from Task 1 validators)
//! - Material parameter display
//! - BRDF preview with interactive lighting (Task 2.2)
//! - Hot-reload with file watching (Task 3)

use anyhow::{Context, Result};
use egui::{Color32, ColorImage, TextureHandle, Ui};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::brdf_preview::BrdfPreview;
use crate::file_watcher::{FileWatcher, ReloadEvent};

/// Material Inspector state
pub struct MaterialInspector {
    /// Currently loaded material TOML path
    pub material_path: Option<PathBuf>,

    /// Parsed material data (from TOML)
    pub material_data: Option<MaterialData>,

    /// Loaded texture images
    pub textures: MaterialTextures,

    /// Display settings
    pub display_mode: DisplayMode,
    pub channel_filter: ChannelFilter,
    pub color_space: ColorSpace,
    pub zoom_level: f32,
    pub pan_offset: (f32, f32),

    /// Validation results (from Task 1 validators)
    pub validation_results: Vec<ValidationResult>,

    /// egui texture handles (cached)
    texture_handles: TextureHandles,

    /// BRDF preview (Task 2.2)
    pub brdf_preview: BrdfPreview,

    /// Status message
    pub status: String,

    /// Task 2.3: Advanced features
    /// Recent material paths history
    pub recent_materials: Vec<PathBuf>,
    /// Available materials discovered in assets directory
    pub available_materials: Vec<PathBuf>,
    /// Current material input text
    pub material_input: String,
    /// Show material browser
    pub show_browser: bool,

    /// Task 3: Hot-reload support
    /// File watcher for automatic reloading
    file_watcher: Option<FileWatcher>,
    /// Last reload time (for UI feedback)
    last_reload_time: Option<std::time::Instant>,
    /// Reload count (for debugging)
    reload_count: usize,

    /// Task 4: Debug UI components
    /// Show UV grid overlay
    pub show_uv_grid: bool,
    /// UV grid density (lines per unit)
    pub uv_grid_density: u32,
    /// Show histogram for current channel
    pub show_histogram: bool,
    /// Histogram data (256 bins for current channel)
    histogram_data: Vec<u32>,
}

/// Material data parsed from TOML
#[derive(Debug, Clone, Deserialize)]
pub struct MaterialData {
    pub name: String,
    #[serde(default)]
    pub layers: Vec<LayerData>,
    #[serde(default)]
    pub metallic: f32,
    #[serde(default)]
    pub roughness: f32,
    #[serde(default)]
    pub base_color: [f32; 4],
}

/// Layer data from terrain materials
#[derive(Debug, Clone, Deserialize)]
pub struct LayerData {
    pub name: String,
    pub albedo: String,
    pub normal: String,
    #[serde(default)]
    pub orm: String,
    #[serde(default)]
    pub mra: String, // Alternative name for ORM
    #[serde(default)]
    pub uv_scale: [f32; 2],
    #[serde(default)]
    pub metallic: f32,
    #[serde(default)]
    pub roughness: f32,
}

/// Loaded texture images
#[derive(Default)]
pub struct MaterialTextures {
    pub albedo: Option<DynamicImage>,
    pub normal: Option<DynamicImage>,
    pub orm: Option<DynamicImage>,
}

/// Display modes for texture viewer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Albedo,
    Normal,
    Orm,
    Split, // Side-by-side comparison
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::Albedo
    }
}

/// Channel isolation filter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelFilter {
    All, // RGB
    Red,
    Green,
    Blue,
    Alpha,
}

impl Default for ChannelFilter {
    fn default() -> Self {
        Self::All
    }
}

/// Color space for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Linear,
    Srgb,
}

impl Default for ColorSpace {
    fn default() -> Self {
        Self::Srgb
    }
}

/// Validation result (simplified from Task 1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub asset_path: String,
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

/// Cached egui texture handles
#[derive(Default)]
struct TextureHandles {
    albedo: Option<TextureHandle>,
    normal: Option<TextureHandle>,
    orm: Option<TextureHandle>,
}

impl MaterialInspector {
    pub fn new() -> Self {
        // Try to create file watcher (may fail if assets/materials doesn't exist)
        let file_watcher = FileWatcher::new("assets/materials")
            .map_err(|e| {
                eprintln!("[MaterialInspector] File watcher disabled: {}", e);
                e
            })
            .ok();

        let mut inspector = Self {
            material_path: None,
            material_data: None,
            textures: MaterialTextures::default(),
            display_mode: DisplayMode::default(),
            channel_filter: ChannelFilter::default(),
            color_space: ColorSpace::default(),
            zoom_level: 1.0,
            pan_offset: (0.0, 0.0),
            validation_results: Vec::new(),
            texture_handles: TextureHandles::default(),
            brdf_preview: BrdfPreview::new(),
            status: "No material loaded".to_string(),
            recent_materials: Vec::new(),
            available_materials: Vec::new(),
            material_input: String::new(),
            show_browser: false,
            file_watcher,
            last_reload_time: None,
            reload_count: 0,
            show_uv_grid: false,
            uv_grid_density: 8,
            show_histogram: false,
            histogram_data: vec![0; 256],
        };

        // Discover materials in default assets directory
        inspector.discover_materials();

        inspector
    }

    /// Load a material from TOML file
    pub fn load_material(&mut self, path: &Path) -> Result<()> {
        self.status = format!("Loading {}...", path.display());
        self.validation_results.clear();

        // Parse TOML
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let material: MaterialData = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from {}", path.display()))?;

        // For terrain materials, load textures from first layer
        if !material.layers.is_empty() {
            let layer = &material.layers[0];
            let base_dir = path.parent().unwrap_or(Path::new("."));

            // Load albedo
            let albedo_path = base_dir.join(&layer.albedo);
            if albedo_path.exists() {
                match image::open(&albedo_path) {
                    Ok(img) => {
                        self.textures.albedo = Some(img);
                        self.validation_results.push(ValidationResult {
                            asset_path: albedo_path.display().to_string(),
                            passed: true,
                            errors: Vec::new(),
                            warnings: Vec::new(),
                            info: vec![format!(
                                "Loaded albedo: {}×{}",
                                self.textures.albedo.as_ref().unwrap().width(),
                                self.textures.albedo.as_ref().unwrap().height()
                            )],
                        });
                    }
                    Err(e) => {
                        self.validation_results.push(ValidationResult {
                            asset_path: albedo_path.display().to_string(),
                            passed: false,
                            errors: vec![format!("Failed to load albedo: {}", e)],
                            warnings: Vec::new(),
                            info: Vec::new(),
                        });
                    }
                }
            } else {
                self.validation_results.push(ValidationResult {
                    asset_path: albedo_path.display().to_string(),
                    passed: false,
                    errors: vec!["Albedo texture file not found".to_string()],
                    warnings: Vec::new(),
                    info: Vec::new(),
                });
            }

            // Load normal
            let normal_path = base_dir.join(&layer.normal);
            if normal_path.exists() {
                match image::open(&normal_path) {
                    Ok(img) => {
                        self.textures.normal = Some(img);
                        self.validation_results.push(ValidationResult {
                            asset_path: normal_path.display().to_string(),
                            passed: true,
                            errors: Vec::new(),
                            warnings: Vec::new(),
                            info: vec![format!(
                                "Loaded normal: {}×{}",
                                self.textures.normal.as_ref().unwrap().width(),
                                self.textures.normal.as_ref().unwrap().height()
                            )],
                        });
                    }
                    Err(e) => {
                        self.validation_results.push(ValidationResult {
                            asset_path: normal_path.display().to_string(),
                            passed: false,
                            errors: vec![format!("Failed to load normal: {}", e)],
                            warnings: Vec::new(),
                            info: Vec::new(),
                        });
                    }
                }
            }

            // Load ORM/MRA
            let orm_name = if !layer.orm.is_empty() {
                &layer.orm
            } else {
                &layer.mra
            };
            if !orm_name.is_empty() {
                let orm_path = base_dir.join(orm_name);
                if orm_path.exists() {
                    match image::open(&orm_path) {
                        Ok(img) => {
                            self.textures.orm = Some(img);
                            self.validation_results.push(ValidationResult {
                                asset_path: orm_path.display().to_string(),
                                passed: true,
                                errors: Vec::new(),
                                warnings: Vec::new(),
                                info: vec![format!(
                                    "Loaded ORM: {}×{}",
                                    self.textures.orm.as_ref().unwrap().width(),
                                    self.textures.orm.as_ref().unwrap().height()
                                )],
                            });
                        }
                        Err(e) => {
                            self.validation_results.push(ValidationResult {
                                asset_path: orm_path.display().to_string(),
                                passed: false,
                                errors: vec![format!("Failed to load ORM: {}", e)],
                                warnings: Vec::new(),
                                info: Vec::new(),
                            });
                        }
                    }
                }
            }
        }

        self.material_path = Some(path.to_path_buf());
        self.material_data = Some(material);

        // Invalidate cached textures
        self.texture_handles = TextureHandles::default();

        self.status = format!("Loaded {}", path.display());
        Ok(())
    }

    /// Convert DynamicImage to egui ColorImage with channel filtering
    fn to_color_image(&self, img: &DynamicImage) -> ColorImage {
        let rgba = img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels: Vec<u8> = rgba
            .pixels()
            .flat_map(|p| {
                let [r, g, b, a] = p.0;

                // Apply channel filter
                let (r, g, b, a) = match self.channel_filter {
                    ChannelFilter::All => (r, g, b, a),
                    ChannelFilter::Red => (r, 0, 0, 255),
                    ChannelFilter::Green => (0, g, 0, 255),
                    ChannelFilter::Blue => (0, 0, b, 255),
                    ChannelFilter::Alpha => (a, a, a, 255),
                };

                // Apply color space conversion
                let (r, g, b) = match self.color_space {
                    ColorSpace::Srgb => (r, g, b), // Already in sRGB
                    ColorSpace::Linear => {
                        // sRGB to linear conversion
                        fn srgb_to_linear(c: u8) -> u8 {
                            let c_f = c as f32 / 255.0;
                            let linear = if c_f <= 0.04045 {
                                c_f / 12.92
                            } else {
                                ((c_f + 0.055) / 1.055).powf(2.4)
                            };
                            (linear * 255.0) as u8
                        }
                        (srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b))
                    }
                };

                [r, g, b, a]
            })
            .collect();

        ColorImage {
            size,
            pixels: pixels
                .chunks(4)
                .map(|c| Color32::from_rgba_premultiplied(c[0], c[1], c[2], c[3]))
                .collect(),
            source_size: egui::Vec2::new(size[0] as f32, size[1] as f32),
        }
    }

    // Task 2.3: Advanced Inspector Features

    /// Discover materials in assets directory
    fn discover_materials(&mut self) {
        self.available_materials.clear();

        // Start from assets/materials directory
        let materials_dir = Path::new("assets/materials");
        if !materials_dir.exists() {
            return;
        }

        // Walk directory recursively
        if let Ok(walker) = walkdir::WalkDir::new(materials_dir)
            .follow_links(false)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
        {
            for entry in walker {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    self.available_materials.push(path.to_path_buf());
                }
            }
        }

        // Sort alphabetically for consistent display
        self.available_materials.sort();
    }

    /// Add material to recent history (LRU cache)
    fn add_to_history(&mut self, path: PathBuf) {
        // Remove if already in history
        self.recent_materials.retain(|p| p != &path);

        // Add to front (most recent)
        self.recent_materials.insert(0, path);

        // Truncate to max 10
        if self.recent_materials.len() > 10 {
            self.recent_materials.truncate(10);
        }
    }

    /// Load material and update history
    fn load_material_with_history(&mut self, path: &Path) {
        if let Err(e) = self.load_material(path) {
            self.status = format!("Error: {}", e);
        } else {
            self.add_to_history(path.to_path_buf());
        }
    }

    /// Process hot-reload events from file watcher (Task 3)
    ///
    /// Call this frequently (e.g., in show() or update loop) to process
    /// file system changes and automatically reload materials/textures.
    pub fn process_hot_reload(&mut self) {
        let Some(ref watcher) = self.file_watcher else {
            return; // File watcher not available
        };

        // Collect all pending events first (to avoid borrow issues)
        let mut events = Vec::new();
        while let Ok(event) = watcher.try_recv() {
            events.push(event);
        }

        // Process collected events
        for event in events {
            match event {
                ReloadEvent::Material(path) => {
                    // Only reload if this is the currently loaded material
                    if let Some(ref current_path) = self.material_path {
                        if current_path == &path {
                            println!(
                                "[MaterialInspector] Hot-reloading material: {}",
                                path.display()
                            );
                            if let Err(e) = self.load_material(&path) {
                                self.status = format!("⚠ Hot-reload failed: {}", e);
                            } else {
                                self.status = format!(
                                    "✅ Hot-reloaded: {} ({})",
                                    path.file_name().unwrap_or_default().to_string_lossy(),
                                    self.reload_count + 1
                                );
                                self.last_reload_time = Some(std::time::Instant::now());
                                self.reload_count += 1;
                            }
                        }
                    }
                }
                ReloadEvent::Texture(path) => {
                    // Check if this texture belongs to the current material
                    let should_reload = if let Some(ref data) = self.material_data {
                        // Check if texture is referenced by current material
                        data.layers.iter().any(|layer| {
                            let base_dir = self
                                .material_path
                                .as_ref()
                                .and_then(|p| p.parent())
                                .unwrap_or(Path::new("."));

                            base_dir.join(&layer.albedo) == path
                                || base_dir.join(&layer.normal) == path
                                || base_dir.join(&layer.orm) == path
                                || base_dir.join(&layer.mra) == path
                        })
                    } else {
                        false
                    };

                    if should_reload {
                        println!(
                            "[MaterialInspector] Hot-reloading texture: {}",
                            path.display()
                        );

                        // Reload the entire material (simplest approach)
                        if let Some(mat_path) = self.material_path.clone() {
                            if let Err(e) = self.load_material(&mat_path) {
                                self.status = format!("⚠ Texture reload failed: {}", e);
                            } else {
                                self.status = format!(
                                    "✅ Texture hot-reloaded: {}",
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                );
                                self.last_reload_time = Some(std::time::Instant::now());
                                self.reload_count += 1;
                            }
                        }
                    }
                }
                ReloadEvent::Prefab(_) => {}
                ReloadEvent::Model(_) => {}
            }
        }
    }

    // Task 4: Debug UI Components

    /// Update histogram data for current texture and channel
    fn update_histogram(&mut self, img: &DynamicImage) {
        // Reset histogram
        self.histogram_data.fill(0);

        let rgba = img.to_rgba8();

        // Count pixel values based on current channel filter
        for pixel in rgba.pixels() {
            let [r, g, b, a] = pixel.0;

            // Get value based on channel filter
            let value = match self.channel_filter {
                ChannelFilter::All => {
                    // Use luminance for "All" mode
                    (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) as u8
                }
                ChannelFilter::Red => r,
                ChannelFilter::Green => g,
                ChannelFilter::Blue => b,
                ChannelFilter::Alpha => a,
            };

            self.histogram_data[value as usize] += 1;
        }
    }

    /// Draw histogram visualization
    fn draw_histogram(&self, ui: &mut egui::Ui) {
        let max_count = *self.histogram_data.iter().max().unwrap_or(&1);

        // Calculate statistics
        let mut min_val = 255;
        let mut max_val = 0;
        let mut sum = 0u64;
        let mut total_pixels = 0u64;

        for (val, &count) in self.histogram_data.iter().enumerate() {
            if count > 0 {
                if val < min_val {
                    min_val = val;
                }
                if val > max_val {
                    max_val = val;
                }
                sum += (val as u64) * (count as u64);
                total_pixels += count as u64;
            }
        }

        let avg = if total_pixels > 0 {
            (sum / total_pixels) as u8
        } else {
            0
        };

        // Display statistics
        ui.label(format!(
            "Min: {} | Max: {} | Avg: {} | Pixels: {}",
            min_val, max_val, avg, total_pixels
        ));

        ui.add_space(4.0);

        // Draw histogram bars
        let desired_height = 100.0;
        let desired_width = 256.0;

        let (response, painter) = ui.allocate_painter(
            egui::vec2(desired_width, desired_height),
            egui::Sense::hover(),
        );

        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));

        // Draw histogram bars
        for (i, &count) in self.histogram_data.iter().enumerate() {
            if count > 0 {
                let normalized = (count as f32) / (max_count as f32);
                let bar_height = normalized * desired_height;

                let x = rect.left() + (i as f32 / 256.0) * desired_width;
                let bar_rect = egui::Rect::from_min_max(
                    egui::pos2(x, rect.bottom() - bar_height),
                    egui::pos2(x + (desired_width / 256.0), rect.bottom()),
                );

                // Color based on channel filter
                let color = match self.channel_filter {
                    ChannelFilter::Red => egui::Color32::from_rgb(200, 50, 50),
                    ChannelFilter::Green => egui::Color32::from_rgb(50, 200, 50),
                    ChannelFilter::Blue => egui::Color32::from_rgb(50, 50, 200),
                    ChannelFilter::Alpha => egui::Color32::from_gray(200),
                    ChannelFilter::All => egui::Color32::from_gray(150),
                };

                painter.rect_filled(bar_rect, 0.0, color);
            }
        }

        // Draw grid lines (quartiles)
        for i in 0..=4 {
            let x = rect.left() + (i as f32 / 4.0) * desired_width;
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
            );
        }
    }

    /// Draw UV grid overlay on texture
    fn draw_uv_grid_overlay(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        _tex_width: u32,
        _tex_height: u32,
    ) {
        let painter = ui.painter();

        let grid_color = egui::Color32::from_rgba_premultiplied(255, 255, 0, 128); // Semi-transparent yellow
        let stroke = egui::Stroke::new(1.0, grid_color);

        // Draw vertical lines
        for i in 0..=self.uv_grid_density {
            let u = (i as f32) / (self.uv_grid_density as f32);
            let x = rect.left() + u * rect.width();
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                stroke,
            );
        }

        // Draw horizontal lines
        for i in 0..=self.uv_grid_density {
            let v = (i as f32) / (self.uv_grid_density as f32);
            let y = rect.top() + v * rect.height();
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                stroke,
            );
        }

        // Draw corner labels (UV coordinates)
        let font_id = egui::FontId::proportional(10.0);
        let text_color = egui::Color32::YELLOW;

        // (0, 0) - top left
        painter.text(
            egui::pos2(rect.left() + 4.0, rect.top() + 4.0),
            egui::Align2::LEFT_TOP,
            "(0,0)",
            font_id.clone(),
            text_color,
        );

        // (1, 0) - top right
        painter.text(
            egui::pos2(rect.right() - 4.0, rect.top() + 4.0),
            egui::Align2::RIGHT_TOP,
            "(1,0)",
            font_id.clone(),
            text_color,
        );

        // (0, 1) - bottom left
        painter.text(
            egui::pos2(rect.left() + 4.0, rect.bottom() - 4.0),
            egui::Align2::LEFT_BOTTOM,
            "(0,1)",
            font_id.clone(),
            text_color,
        );

        // (1, 1) - bottom right
        painter.text(
            egui::pos2(rect.right() - 4.0, rect.bottom() - 4.0),
            egui::Align2::RIGHT_BOTTOM,
            "(1,1)",
            font_id,
            text_color,
        );
    }

    /// Render the inspector UI
    pub fn show(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        // Process hot-reload events first
        self.process_hot_reload();

        ui.heading("Material Inspector");
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(8.0);

        // Task 2.3: Material Browser & History
        ui.collapsing("📁 Material Browser", |ui| {
            ui.add_space(4.0);

            // History dropdown
            if !self.recent_materials.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Recent:")
                        .on_hover_text("Last 10 loaded materials (most recent first)");
                    egui::ComboBox::from_label("")
                        .width(300.0)
                        .selected_text(
                            self.recent_materials
                                .first()
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .unwrap_or("Select..."),
                        )
                        .show_ui(ui, |ui| {
                            for path in self.recent_materials.clone() {
                                let name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown");
                                if ui.selectable_label(false, name).clicked() {
                                    self.load_material_with_history(&path);
                                }
                            }
                        });
                });
                ui.add_space(4.0);
            }

            // Browser toggle
            ui.horizontal(|ui| {
                if ui
                    .button(if self.show_browser {
                        "▼ Hide Browser"
                    } else {
                        "▶ Show Browser"
                    })
                    .on_hover_text("Toggle material list visibility")
                    .clicked()
                {
                    self.show_browser = !self.show_browser;
                }
                if ui
                    .button("🔄 Refresh")
                    .on_hover_text("Rescan assets/materials/ directory")
                    .clicked()
                {
                    self.discover_materials();
                }

                // Show count
                if !self.available_materials.is_empty() {
                    ui.label(format!("({} materials)", self.available_materials.len()));
                }
            });

            // Material list (when expanded)
            if self.show_browser {
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if self.available_materials.is_empty() {
                            ui.colored_label(
                                egui::Color32::from_rgb(200, 150, 100),
                                "⚠ No materials found in assets/materials/",
                            );
                            ui.label("Create .toml files or click Refresh to scan again.");
                        } else {
                            for path in &self.available_materials.clone() {
                                let name = path
                                    .strip_prefix("assets/materials/")
                                    .unwrap_or(path)
                                    .display()
                                    .to_string();
                                if ui
                                    .selectable_label(false, &name)
                                    .on_hover_text(format!("Load: {}", name))
                                    .clicked()
                                {
                                    self.load_material_with_history(path);
                                }
                            }
                        }
                    });
            }

            // Manual path input
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Path:")
                    .on_hover_text("Enter relative path to material TOML");
                let response = ui.text_edit_singleline(&mut self.material_input);
                response.on_hover_text("Example: assets/materials/terrain/grassland_demo.toml");

                let load_button = ui
                    .button("Load")
                    .on_hover_text("Load material from typed path");
                if load_button.clicked() && !self.material_input.is_empty() {
                    let path = PathBuf::from(&self.material_input);
                    self.load_material_with_history(&path);
                }
            });
            ui.add_space(4.0);
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // File picker (legacy, kept for compatibility)
        ui.horizontal(|ui| {
            if ui
                .button("📂 Load Demo Material")
                .on_hover_text("Load grassland_demo.toml (for quick testing)")
                .clicked()
            {
                // For now, hardcoded to demo materials
                if let Err(e) =
                    self.load_material(Path::new("assets/materials/terrain/grassland_demo.toml"))
                {
                    self.status = format!("❌ Error: {}", e);
                } else {
                    self.status = "✅ Loaded: grassland_demo.toml".to_string();
                }
            }

            // Status with color coding
            let status_color = if self.status.starts_with("✅") {
                egui::Color32::from_rgb(100, 200, 100)
            } else if self.status.starts_with("⚠") {
                egui::Color32::from_rgb(200, 150, 100)
            } else if self.status.starts_with("❌") {
                egui::Color32::from_rgb(200, 100, 100)
            } else {
                egui::Color32::GRAY
            };
            ui.colored_label(status_color, &self.status);

            // Hot-reload indicator (Task 3)
            if self.file_watcher.is_some() {
                ui.add_space(8.0);
                ui.label("🔄").on_hover_text(format!(
                    "Hot-reload: ENABLED\nReload count: {}\nLast reload: {}",
                    self.reload_count,
                    self.last_reload_time
                        .map(|t| format!("{:.1}s ago", t.elapsed().as_secs_f32()))
                        .unwrap_or_else(|| "Never".to_string())
                ));
            } else {
                ui.add_space(8.0);
                ui.label("⭕")
                    .on_hover_text("Hot-reload: DISABLED\n(assets/materials directory not found)");
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Display controls
        ui.horizontal(|ui| {
            ui.label("Display Mode:")
                .on_hover_text("Select which texture to view");
            ui.radio_value(&mut self.display_mode, DisplayMode::Albedo, "Albedo")
                .on_hover_text("Base color (sRGB)");
            ui.radio_value(&mut self.display_mode, DisplayMode::Normal, "Normal")
                .on_hover_text("Tangent-space normal map (Linear)");
            ui.radio_value(&mut self.display_mode, DisplayMode::Orm, "ORM")
                .on_hover_text("Occlusion (R), Roughness (G), Metallic (B)");
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Channel:")
                .on_hover_text("Isolate individual color channels");
            ui.radio_value(&mut self.channel_filter, ChannelFilter::All, "All (RGB)");
            ui.radio_value(&mut self.channel_filter, ChannelFilter::Red, "R");
            ui.radio_value(&mut self.channel_filter, ChannelFilter::Green, "G");
            ui.radio_value(&mut self.channel_filter, ChannelFilter::Blue, "B");
            ui.radio_value(&mut self.channel_filter, ChannelFilter::Alpha, "A");
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Color Space:")
                .on_hover_text("Toggle between sRGB (gamma-corrected) and Linear");
            ui.radio_value(&mut self.color_space, ColorSpace::Srgb, "sRGB")
                .on_hover_text("Standard display color space");
            ui.radio_value(&mut self.color_space, ColorSpace::Linear, "Linear")
                .on_hover_text("Raw texture values (darker)");
        });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Zoom:")
                .on_hover_text("Texture magnification (0.1x to 4.0x)");
            ui.add(
                egui::Slider::new(&mut self.zoom_level, 0.1..=4.0)
                    .step_by(0.1)
                    .text("×"),
            );
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Task 4: Debug UI components
        ui.collapsing("🔧 Debug Tools", |ui| {
            ui.add_space(4.0);

            // UV Grid overlay
            ui.checkbox(&mut self.show_uv_grid, "Show UV Grid")
                .on_hover_text("Overlay UV coordinate grid (0-1 range)");

            if self.show_uv_grid {
                ui.horizontal(|ui| {
                    ui.label("Grid Density:");
                    ui.add(egui::Slider::new(&mut self.uv_grid_density, 2..=32))
                        .on_hover_text("Number of grid lines per UV unit");
                });
            }

            ui.add_space(4.0);

            // Histogram
            ui.checkbox(&mut self.show_histogram, "Show Histogram")
                .on_hover_text("Display value distribution for current channel");

            if self.show_histogram {
                // Calculate histogram for current texture
                // Get image reference based on display mode
                let img_opt = match self.display_mode {
                    DisplayMode::Albedo => self.textures.albedo.as_ref(),
                    DisplayMode::Normal => self.textures.normal.as_ref(),
                    DisplayMode::Orm => self.textures.orm.as_ref(),
                    DisplayMode::Split => self.textures.albedo.as_ref(),
                };

                if let Some(img) = img_opt {
                    // Clone image to avoid borrow issues (only clones pointer, not data)
                    let img_clone = img.clone();
                    self.update_histogram(&img_clone);

                    // Draw histogram
                    self.draw_histogram(ui);
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Texture viewer
        if let Some(img) = match self.display_mode {
            DisplayMode::Albedo => self.textures.albedo.as_ref(),
            DisplayMode::Normal => self.textures.normal.as_ref(),
            DisplayMode::Orm => self.textures.orm.as_ref(),
            DisplayMode::Split => self.textures.albedo.as_ref(), // TODO: Split view
        } {
            // Convert to ColorImage with channel filtering
            let color_image = self.to_color_image(img);

            // Get or create texture handle
            let texture_handle = match self.display_mode {
                DisplayMode::Albedo => &mut self.texture_handles.albedo,
                DisplayMode::Normal => &mut self.texture_handles.normal,
                DisplayMode::Orm => &mut self.texture_handles.orm,
                DisplayMode::Split => &mut self.texture_handles.albedo,
            };

            let handle = texture_handle.get_or_insert_with(|| {
                ctx.load_texture(
                    format!("{:?}", self.display_mode),
                    color_image.clone(),
                    Default::default(),
                )
            });

            // Update texture if channel filter or color space changed
            // Note: This is inefficient but works for MVP
            // TODO: Only update when filter/colorspace changes
            *handle = ctx.load_texture(
                format!(
                    "{:?}_{:?}_{:?}",
                    self.display_mode, self.channel_filter, self.color_space
                ),
                color_image,
                Default::default(),
            );

            // Display texture with zoom
            let size = egui::vec2(
                img.width() as f32 * self.zoom_level,
                img.height() as f32 * self.zoom_level,
            );

            let response = ui.image((handle.id(), size));

            // Task 4: Draw UV grid overlay
            if self.show_uv_grid {
                self.draw_uv_grid_overlay(ui, response.rect, img.width(), img.height());
            }

            // Texture info
            ui.label(format!(
                "Size: {}×{} | Zoom: {:.1}x | Format: {:?}",
                img.width(),
                img.height(),
                self.zoom_level,
                img.color()
            ));
        } else {
            ui.label("No texture loaded for this mode");
        }

        ui.separator();

        // Validation results
        ui.collapsing("Validation Results", |ui| {
            if self.validation_results.is_empty() {
                ui.label("No validation results");
            } else {
                for result in &self.validation_results {
                    let icon = if result.passed { "✅" } else { "❌" };
                    ui.label(format!("{} {}", icon, result.asset_path));

                    for error in &result.errors {
                        ui.colored_label(egui::Color32::RED, format!("  ERROR: {}", error));
                    }
                    for warning in &result.warnings {
                        ui.colored_label(egui::Color32::YELLOW, format!("  WARN: {}", warning));
                    }
                    for info in &result.info {
                        ui.label(format!("  INFO: {}", info));
                    }
                }
            }
        });

        // Material data
        if let Some(data) = &self.material_data {
            ui.collapsing("Material Data", |ui| {
                ui.label(format!("Name: {}", data.name));
                ui.label(format!("Layers: {}", data.layers.len()));

                for (i, layer) in data.layers.iter().enumerate() {
                    ui.collapsing(format!("Layer {}: {}", i, layer.name), |ui| {
                        ui.label(format!("  Albedo: {}", layer.albedo));
                        ui.label(format!("  Normal: {}", layer.normal));
                        if !layer.orm.is_empty() {
                            ui.label(format!("  ORM: {}", layer.orm));
                        }
                        if !layer.mra.is_empty() {
                            ui.label(format!("  MRA: {}", layer.mra));
                        }
                        ui.label(format!("  UV Scale: {:?}", layer.uv_scale));
                        ui.label(format!("  Metallic: {:.2}", layer.metallic));
                        ui.label(format!("  Roughness: {:.2}", layer.roughness));
                    });
                }
            });
        }

        ui.separator();

        // BRDF Preview (Task 2.2)
        ui.collapsing("BRDF Preview", |ui| {
            // Update BRDF preview with current material parameters
            if let Some(data) = &self.material_data {
                if let Some(layer) = data.layers.first() {
                    // Extract albedo from first layer (or use material defaults)
                    let albedo = [data.base_color[0], data.base_color[1], data.base_color[2]];
                    let metallic = if layer.metallic >= 0.0 {
                        layer.metallic
                    } else {
                        data.metallic
                    };
                    let roughness = if layer.roughness >= 0.0 {
                        layer.roughness
                    } else {
                        data.roughness
                    };

                    self.brdf_preview.set_material(albedo, metallic, roughness);
                }
            }

            self.brdf_preview.show(ui, ctx);
        });
    }
}

impl Default for MaterialInspector {
    fn default() -> Self {
        Self::new()
    }
}
