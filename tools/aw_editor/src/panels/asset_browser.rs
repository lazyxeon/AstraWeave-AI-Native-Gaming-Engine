use egui::{ColorImage, ImageData, ScrollArea, TextureHandle, Ui};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// TEXTURE TYPE - Classification of texture maps for PBR workflow
// ============================================================================

/// Texture types for PBR (Physically Based Rendering) material workflow
/// Detected automatically from filename suffixes (e.g., _normal, _albedo, _orm)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureType {
    /// Base color / diffuse map
    Albedo,
    /// Normal map (tangent space)
    Normal,
    /// Combined Occlusion-Roughness-Metallic map (RGB channels)
    ORM,
    /// Combined Metallic-Roughness-AO map (RGB channels)
    MRA,
    /// Roughness map (grayscale)
    Roughness,
    /// Metallic map (grayscale)
    Metallic,
    /// Ambient Occlusion map
    AO,
    /// Emissive/glow map
    Emission,
    /// Height/displacement map
    Height,
    /// Unknown or unclassified texture
    Unknown,
}

impl TextureType {
    /// Detect texture type from filename using common naming conventions
    /// Supports: _n, _normal, _nrm, _orm, _mra, _r, _rough, _roughness,
    /// _m, _metal, _metallic, _ao, _occlusion, _e, _emit, _emission,
    /// _h, _height, _disp, _displacement, _albedo, _diffuse, _basecolor, _color
    pub fn from_filename(name: &str) -> Self {
        let stem = Path::new(name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check suffixes in order of specificity
        if stem.ends_with("_n")
            || stem.ends_with("_normal")
            || stem.ends_with("_nrm")
            || stem.ends_with("_nor")
        {
            TextureType::Normal
        } else if stem.ends_with("_orm") {
            TextureType::ORM
        } else if stem.ends_with("_mra") {
            TextureType::MRA
        } else if stem.ends_with("_r") || stem.ends_with("_rough") || stem.ends_with("_roughness") {
            TextureType::Roughness
        } else if stem.ends_with("_m")
            || stem.ends_with("_metal")
            || stem.ends_with("_metallic")
            || stem.ends_with("_metalness")
        {
            TextureType::Metallic
        } else if stem.ends_with("_ao") || stem.ends_with("_occlusion") {
            TextureType::AO
        } else if stem.ends_with("_e")
            || stem.ends_with("_emit")
            || stem.ends_with("_emission")
            || stem.ends_with("_emissive")
            || stem.ends_with("_glow")
        {
            TextureType::Emission
        } else if stem.ends_with("_h")
            || stem.ends_with("_height")
            || stem.ends_with("_disp")
            || stem.ends_with("_displacement")
            || stem.ends_with("_bump")
        {
            TextureType::Height
        } else if stem.ends_with("_albedo")
            || stem.ends_with("_diffuse")
            || stem.ends_with("_basecolor")
            || stem.ends_with("_base_color")
            || stem.ends_with("_color")
            || stem.ends_with("_col")
            || stem.ends_with("_d")
        {
            TextureType::Albedo
        } else {
            TextureType::Unknown
        }
    }

    /// Icon for display in the UI
    pub fn icon(&self) -> &'static str {
        match self {
            TextureType::Albedo => "🎨",
            TextureType::Normal => "🔵",
            TextureType::ORM => "🔶",
            TextureType::MRA => "🔷",
            TextureType::Roughness => "◽",
            TextureType::Metallic => "⬜",
            TextureType::AO => "⬛",
            TextureType::Emission => "✨",
            TextureType::Height => "📐",
            TextureType::Unknown => "❓",
        }
    }

    /// Display label for the texture type
    pub fn label(&self) -> &'static str {
        match self {
            TextureType::Albedo => "Albedo",
            TextureType::Normal => "Normal",
            TextureType::ORM => "ORM",
            TextureType::MRA => "MRA",
            TextureType::Roughness => "Rough",
            TextureType::Metallic => "Metal",
            TextureType::AO => "AO",
            TextureType::Emission => "Emit",
            TextureType::Height => "Height",
            TextureType::Unknown => "???",
        }
    }

    /// Color for UI badges
    pub fn color(&self) -> egui::Color32 {
        match self {
            TextureType::Albedo => egui::Color32::from_rgb(255, 180, 100),
            TextureType::Normal => egui::Color32::from_rgb(128, 128, 255),
            TextureType::ORM => egui::Color32::from_rgb(255, 165, 0),
            TextureType::MRA => egui::Color32::from_rgb(100, 149, 237),
            TextureType::Roughness => egui::Color32::from_rgb(180, 180, 180),
            TextureType::Metallic => egui::Color32::from_rgb(220, 220, 255),
            TextureType::AO => egui::Color32::from_rgb(80, 80, 80),
            TextureType::Emission => egui::Color32::from_rgb(255, 255, 100),
            TextureType::Height => egui::Color32::from_rgb(150, 100, 200),
            TextureType::Unknown => egui::Color32::from_rgb(128, 128, 128),
        }
    }
}

// ============================================================================
// ASSET CATEGORY - High-level organization for asset browser
// ============================================================================

/// Asset categories for organizing the asset browser
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetCategory {
    All,
    Models,
    Textures,
    Materials,
    Prefabs,
    Scenes,
    Audio,
    Configs,
}

impl AssetCategory {
    /// Check if an asset type matches this category
    pub fn matches(&self, asset_type: &AssetType) -> bool {
        match self {
            AssetCategory::All => true,
            AssetCategory::Models => *asset_type == AssetType::Model,
            AssetCategory::Textures => *asset_type == AssetType::Texture,
            AssetCategory::Materials => *asset_type == AssetType::Material,
            AssetCategory::Prefabs => *asset_type == AssetType::Prefab,
            AssetCategory::Scenes => *asset_type == AssetType::Scene,
            AssetCategory::Audio => *asset_type == AssetType::Audio,
            AssetCategory::Configs => *asset_type == AssetType::Config,
        }
    }

    /// Icon for category button
    pub fn icon(&self) -> &'static str {
        match self {
            AssetCategory::All => "📦",
            AssetCategory::Models => "🎭",
            AssetCategory::Textures => "🖼️",
            AssetCategory::Materials => "💎",
            AssetCategory::Prefabs => "💾",
            AssetCategory::Scenes => "🌍",
            AssetCategory::Audio => "🔊",
            AssetCategory::Configs => "⚙️",
        }
    }

    /// Display label
    pub fn label(&self) -> &'static str {
        match self {
            AssetCategory::All => "All",
            AssetCategory::Models => "Models",
            AssetCategory::Textures => "Textures",
            AssetCategory::Materials => "Materials",
            AssetCategory::Prefabs => "Prefabs",
            AssetCategory::Scenes => "Scenes",
            AssetCategory::Audio => "Audio",
            AssetCategory::Configs => "Configs",
        }
    }
}

// ============================================================================
// ASSET ACTION - Actions that can be performed on assets
// ============================================================================

/// Actions triggered from the asset browser for processing by the editor
#[derive(Debug, Clone)]
pub enum AssetAction {
    /// Import a 3D model into the scene as a new entity
    ImportModel { path: PathBuf },
    /// Apply a texture to the selected entity's material
    ApplyTexture {
        path: PathBuf,
        texture_type: TextureType,
    },
    /// Apply a material file to the selected entity
    ApplyMaterial { path: PathBuf },
    /// Load a scene file
    LoadScene { path: PathBuf },
    /// Spawn a prefab as a new entity
    SpawnPrefab { path: PathBuf },
    /// Open asset in external application
    OpenExternal { path: PathBuf },
    /// Inspect asset details (for material inspector panel)
    InspectAsset { path: PathBuf },
}

// ============================================================================
// ASSET TYPE - Basic asset classification by file type
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Model,
    Texture,
    Scene,
    Material,
    Audio,
    Config,
    Prefab,
    Directory,
    Unknown,
}

impl AssetType {
    pub fn from_path(path: &Path) -> Self {
        if path.is_dir() {
            return AssetType::Directory;
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name.ends_with(".prefab.ron") {
            return AssetType::Prefab;
        }

        match path.extension().and_then(|e| e.to_str()) {
            Some("glb") | Some("gltf") | Some("obj") | Some("fbx") => AssetType::Model,
            Some("png") | Some("jpg") | Some("jpeg") | Some("ktx2") | Some("dds") => {
                AssetType::Texture
            }
            Some("ron") => AssetType::Scene,
            Some("toml") | Some("json") => AssetType::Config,
            Some("wav") | Some("ogg") | Some("mp3") => AssetType::Audio,
            _ => AssetType::Unknown,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AssetType::Model => "🎭",
            AssetType::Texture => "🖼️",
            AssetType::Scene => "🌍",
            AssetType::Material => "💎",
            AssetType::Audio => "🔊",
            AssetType::Config => "⚙️",
            AssetType::Prefab => "💾",
            AssetType::Directory => "📁",
            AssetType::Unknown => "📄",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            AssetType::Model => egui::Color32::from_rgb(100, 200, 255),
            AssetType::Texture => egui::Color32::from_rgb(255, 150, 100),
            AssetType::Scene => egui::Color32::from_rgb(100, 255, 100),
            AssetType::Material => egui::Color32::from_rgb(200, 100, 255),
            AssetType::Audio => egui::Color32::from_rgb(255, 255, 100),
            AssetType::Config => egui::Color32::from_rgb(200, 200, 200),
            AssetType::Prefab => egui::Color32::from_rgb(150, 200, 255),
            AssetType::Directory => egui::Color32::from_rgb(255, 200, 100),
            AssetType::Unknown => egui::Color32::from_rgb(150, 150, 150),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetEntry {
    pub path: PathBuf,
    pub name: String,
    pub asset_type: AssetType,
    pub texture_type: Option<TextureType>,
    pub size: u64,
}

impl AssetEntry {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let asset_type = AssetType::from_path(&path);

        // Detect texture type if this is a texture
        let texture_type = if asset_type == AssetType::Texture {
            Some(TextureType::from_filename(&name))
        } else {
            None
        };

        let size = if path.is_file() {
            fs::metadata(&path).ok()?.len()
        } else {
            0
        };

        Some(AssetEntry {
            path,
            name,
            asset_type,
            texture_type,
            size,
        })
    }

    pub fn format_size(&self) -> String {
        if self.asset_type == AssetType::Directory {
            return String::new();
        }

        let size_kb = self.size as f64 / 1024.0;
        if size_kb < 1024.0 {
            format!("{:.1} KB", size_kb)
        } else {
            format!("{:.1} MB", size_kb / 1024.0)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    List,
    Grid,
}

pub struct AssetBrowser {
    root_path: PathBuf,
    current_path: PathBuf,
    entries: Vec<AssetEntry>,
    selected_asset: Option<PathBuf>,
    show_hidden: bool,
    filter_type: Option<AssetType>,
    search_query: String,
    view_mode: ViewMode,
    thumbnail_cache: HashMap<PathBuf, TextureHandle>,
    thumbnail_size: f32,
    dragged_prefab: Option<PathBuf>,
    // New fields for enhanced organization
    category_filter: AssetCategory,
    texture_type_filter: Option<TextureType>,
    pending_actions: Vec<AssetAction>,
    show_texture_badges: bool,
}

impl AssetBrowser {
    pub fn new(root_path: PathBuf) -> Self {
        let mut browser = Self {
            root_path: root_path.clone(),
            current_path: root_path,
            entries: Vec::new(),
            selected_asset: None,
            show_hidden: false,
            filter_type: None,
            search_query: String::new(),
            view_mode: ViewMode::Grid, // Default to grid for better visual browsing
            thumbnail_cache: HashMap::new(),
            thumbnail_size: 80.0, // Slightly larger for better visibility
            dragged_prefab: None,
            category_filter: AssetCategory::All,
            texture_type_filter: None,
            pending_actions: Vec::new(),
            show_texture_badges: true,
        };
        browser.scan_current_directory();
        browser
    }

    /// Take any pending asset actions for processing by main editor
    pub fn take_pending_actions(&mut self) -> Vec<AssetAction> {
        std::mem::take(&mut self.pending_actions)
    }

    pub fn take_dragged_prefab(&mut self) -> Option<PathBuf> {
        self.dragged_prefab.take()
    }

    fn scan_current_directory(&mut self) {
        self.entries.clear();

        let Ok(read_dir) = fs::read_dir(&self.current_path) else {
            return;
        };

        let mut entries: Vec<AssetEntry> = read_dir
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| AssetEntry::from_path(entry.path()))
            .filter(|entry| {
                // Hidden file filter
                if !self.show_hidden && entry.name.starts_with('.') {
                    return false;
                }

                // Always show directories for navigation
                if entry.asset_type == AssetType::Directory {
                    return true;
                }

                // Category filter
                if !self.category_filter.matches(&entry.asset_type) {
                    return false;
                }

                // Texture type filter (only applies to textures)
                if let Some(tex_filter) = &self.texture_type_filter {
                    if entry.asset_type == AssetType::Texture {
                        if entry.texture_type.as_ref() != Some(tex_filter) {
                            return false;
                        }
                    }
                }

                // Legacy filter_type support
                if let Some(filter) = &self.filter_type {
                    if &entry.asset_type != filter {
                        return false;
                    }
                }

                // Search query filter
                if !self.search_query.is_empty() {
                    if !entry
                        .name
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                    {
                        return false;
                    }
                }

                true
            })
            .collect();

        entries.sort_by(|a, b| match (&a.asset_type, &b.asset_type) {
            (AssetType::Directory, AssetType::Directory) => a.name.cmp(&b.name),
            (AssetType::Directory, _) => std::cmp::Ordering::Less,
            (_, AssetType::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        self.entries = entries;
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_path = path;
            self.scan_current_directory();
        }
    }

    pub fn navigate_up(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            if parent >= self.root_path.as_path() {
                self.current_path = parent.to_path_buf();
                self.scan_current_directory();
            }
        }
    }

    pub fn selected_asset(&self) -> Option<&Path> {
        self.selected_asset.as_deref()
    }

    fn load_thumbnail(&mut self, ctx: &egui::Context, path: &Path) -> Option<TextureHandle> {
        if let Some(texture) = self.thumbnail_cache.get(path) {
            return Some(texture.clone());
        }

        if AssetType::from_path(path) != AssetType::Texture {
            return None;
        }

        let image_data = image::open(path).ok()?;
        let rgba = image_data.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let pixels = rgba.into_raw();

        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

        let texture = ctx.load_texture(
            path.display().to_string(),
            ImageData::Color(std::sync::Arc::new(color_image)),
            egui::TextureOptions::LINEAR,
        );

        self.thumbnail_cache
            .insert(path.to_path_buf(), texture.clone());
        Some(texture)
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.heading("📦 Asset Browser");
        ui.separator();

        // Navigation bar
        ui.horizontal(|ui| {
            if ui.button("⬆️ Up").clicked() {
                self.navigate_up();
            }

            if ui.button("🏠 Root").clicked() {
                self.navigate_to(self.root_path.clone());
            }

            ui.separator();

            ui.label("🔍");
            if ui.text_edit_singleline(&mut self.search_query).changed() {
                self.scan_current_directory();
            }

            ui.separator();

            if ui
                .selectable_label(self.view_mode == ViewMode::List, "📄 List")
                .clicked()
            {
                self.view_mode = ViewMode::List;
            }
            if ui
                .selectable_label(self.view_mode == ViewMode::Grid, "🔲 Grid")
                .clicked()
            {
                self.view_mode = ViewMode::Grid;
            }

            ui.separator();

            ui.checkbox(&mut self.show_texture_badges, "🏷️");
            if ui
                .small_button("⚙️")
                .on_hover_text("Thumbnail size")
                .clicked()
            {
                // Toggle between size presets
                self.thumbnail_size = match self.thumbnail_size as i32 {
                    64 => 80.0,
                    80 => 100.0,
                    100 => 120.0,
                    _ => 64.0,
                };
            }
        });

        // Category filter bar
        ui.horizontal(|ui| {
            ui.label("Category:");

            let categories = [
                AssetCategory::All,
                AssetCategory::Models,
                AssetCategory::Textures,
                AssetCategory::Materials,
                AssetCategory::Prefabs,
                AssetCategory::Scenes,
                AssetCategory::Audio,
                AssetCategory::Configs,
            ];

            for cat in categories {
                let label = format!("{} {}", cat.icon(), cat.label());
                if ui
                    .selectable_label(self.category_filter == cat, label)
                    .clicked()
                {
                    self.category_filter = cat;
                    // Clear texture type filter when changing category
                    if cat != AssetCategory::Textures {
                        self.texture_type_filter = None;
                    }
                    self.scan_current_directory();
                }
            }
        });

        // Texture type sub-filter (only when Textures category selected)
        if self.category_filter == AssetCategory::Textures {
            ui.horizontal(|ui| {
                ui.label("Type:");

                if ui
                    .selectable_label(self.texture_type_filter.is_none(), "All")
                    .clicked()
                {
                    self.texture_type_filter = None;
                    self.scan_current_directory();
                }

                let tex_types = [
                    TextureType::Albedo,
                    TextureType::Normal,
                    TextureType::ORM,
                    TextureType::MRA,
                    TextureType::Roughness,
                    TextureType::Metallic,
                    TextureType::AO,
                    TextureType::Emission,
                    TextureType::Height,
                ];

                for tex_type in tex_types {
                    let label = format!("{} {}", tex_type.icon(), tex_type.label());
                    let is_selected = self.texture_type_filter == Some(tex_type);
                    if ui.selectable_label(is_selected, label).clicked() {
                        self.texture_type_filter = if is_selected { None } else { Some(tex_type) };
                        self.scan_current_directory();
                    }
                }
            });
        }

        // Current path display
        ui.label(format!(
            "📂 {}",
            self.current_path
                .strip_prefix(&self.root_path)
                .unwrap_or(&self.current_path)
                .display()
        ));

        ui.separator();

        let mut path_to_navigate = None;

        match self.view_mode {
            ViewMode::List => {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.style_mut().spacing.item_spacing.y = 2.0;

                        for entry in &self.entries {
                            let is_selected = self.selected_asset.as_ref() == Some(&entry.path);

                            let response = ui.selectable_label(
                                is_selected,
                                format!(
                                    "{} {} {}",
                                    entry.asset_type.icon(),
                                    entry.name,
                                    entry.format_size()
                                ),
                            );

                            if response.clicked() {
                                if entry.asset_type == AssetType::Directory {
                                    path_to_navigate = Some(entry.path.clone());
                                } else {
                                    self.selected_asset = Some(entry.path.clone());
                                }
                            }

                            if response.double_clicked() {
                                if entry.asset_type == AssetType::Directory {
                                    path_to_navigate = Some(entry.path.clone());
                                }
                            }

                            if response.hovered() {
                                response
                                    .clone()
                                    .on_hover_text(entry.path.display().to_string());
                            }

                            if entry.asset_type == AssetType::Prefab {
                                if response.drag_started() {
                                    self.dragged_prefab = Some(entry.path.clone());
                                }
                            }
                        }

                        if self.entries.is_empty() {
                            ui.colored_label(
                                egui::Color32::GRAY,
                                if self.search_query.is_empty() {
                                    "Empty directory"
                                } else {
                                    "No matching assets"
                                },
                            );
                        }
                    });
            }
            ViewMode::Grid => {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let item_spacing = 8.0;
                        let thumbnail_size = self.thumbnail_size;
                        let available_width = ui.available_width();
                        let items_per_row = ((available_width + item_spacing)
                            / (thumbnail_size + item_spacing))
                            .floor()
                            .max(1.0) as usize;

                        ui.style_mut().spacing.item_spacing =
                            egui::vec2(item_spacing, item_spacing);

                        for row_start in (0..self.entries.len()).step_by(items_per_row) {
                            ui.horizontal(|ui| {
                                for i in
                                    row_start..(row_start + items_per_row).min(self.entries.len())
                                {
                                    let entry = &self.entries[i];
                                    let is_selected =
                                        self.selected_asset.as_ref() == Some(&entry.path);
                                    let entry_path = entry.path.clone();
                                    let entry_name = entry.name.clone();
                                    let entry_asset_type = entry.asset_type;
                                    let entry_texture_type = entry.texture_type;
                                    let show_badges = self.show_texture_badges;

                                    let ctx = ui.ctx().clone();
                                    let thumbnail = if entry_asset_type == AssetType::Texture {
                                        self.load_thumbnail(&ctx, &entry_path)
                                    } else {
                                        None
                                    };

                                    ui.vertical(|ui| {
                                        ui.set_width(thumbnail_size);

                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(thumbnail_size, thumbnail_size),
                                            egui::Sense::click(),
                                        );

                                        if ui.is_rect_visible(rect) {
                                            let bg_color = if is_selected {
                                                egui::Color32::from_rgb(60, 120, 180)
                                            } else if response.hovered() {
                                                egui::Color32::from_rgb(50, 50, 55)
                                            } else {
                                                egui::Color32::from_rgb(35, 35, 40)
                                            };

                                            ui.painter().rect_filled(rect, 4.0, bg_color);

                                            if let Some(texture) = thumbnail {
                                                ui.painter().image(
                                                    texture.id(),
                                                    rect.shrink(4.0),
                                                    egui::Rect::from_min_max(
                                                        egui::pos2(0.0, 0.0),
                                                        egui::pos2(1.0, 1.0),
                                                    ),
                                                    egui::Color32::WHITE,
                                                );
                                            } else {
                                                let icon_pos = rect.center();
                                                ui.painter().text(
                                                    icon_pos,
                                                    egui::Align2::CENTER_CENTER,
                                                    entry_asset_type.icon(),
                                                    egui::FontId::proportional(32.0),
                                                    entry_asset_type.color(),
                                                );
                                            }

                                            // Draw texture type badge in bottom-right corner
                                            if show_badges {
                                                if let Some(tex_type) = entry_texture_type {
                                                    let badge_size = 18.0;
                                                    let badge_rect = egui::Rect::from_min_size(
                                                        egui::pos2(
                                                            rect.max.x - badge_size - 4.0,
                                                            rect.max.y - badge_size - 4.0,
                                                        ),
                                                        egui::vec2(badge_size, badge_size),
                                                    );
                                                    ui.painter().rect_filled(
                                                        badge_rect,
                                                        3.0,
                                                        tex_type.color(),
                                                    );
                                                    ui.painter().text(
                                                        badge_rect.center(),
                                                        egui::Align2::CENTER_CENTER,
                                                        tex_type.icon(),
                                                        egui::FontId::proportional(10.0),
                                                        egui::Color32::WHITE,
                                                    );
                                                }
                                            }
                                        }

                                        if response.clicked() {
                                            if entry_asset_type == AssetType::Directory {
                                                path_to_navigate = Some(entry_path.clone());
                                            } else {
                                                self.selected_asset = Some(entry_path.clone());
                                            }
                                        }

                                        if response.double_clicked() {
                                            if entry_asset_type == AssetType::Directory {
                                                path_to_navigate = Some(entry_path.clone());
                                            }
                                        }

                                        if response.hovered() {
                                            response
                                                .clone()
                                                .on_hover_text(entry_path.display().to_string());
                                        }

                                        if entry_asset_type == AssetType::Prefab {
                                            if response.drag_started() {
                                                self.dragged_prefab = Some(entry_path.clone());
                                            }
                                        }

                                        ui.add(
                                            egui::Label::new(&entry_name)
                                                .wrap_mode(egui::TextWrapMode::Truncate),
                                        );
                                    });
                                }
                            });
                        }

                        if self.entries.is_empty() {
                            ui.colored_label(
                                egui::Color32::GRAY,
                                if self.search_query.is_empty() {
                                    "Empty directory"
                                } else {
                                    "No matching assets"
                                },
                            );
                        }
                    });
            }
        }

        if let Some(path) = path_to_navigate {
            self.navigate_to(path);
        }

        // Selected asset details panel with action buttons
        if let Some(selected) = &self.selected_asset.clone() {
            ui.separator();

            let asset_type = AssetType::from_path(selected);
            let texture_type = if asset_type == AssetType::Texture {
                Some(TextureType::from_filename(
                    selected
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or(""),
                ))
            } else {
                None
            };

            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(asset_type.icon());
                    ui.strong(
                        selected
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    );
                    if let Some(tex_type) = texture_type {
                        ui.colored_label(tex_type.color(), format!("[{}]", tex_type.label()));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("📂");
                    ui.monospace(selected.parent().unwrap_or(selected).display().to_string());
                });

                ui.horizontal(|ui| {
                    // Context-appropriate action buttons
                    match asset_type {
                        AssetType::Model => {
                            if ui.button("➕ Import to Scene").clicked() {
                                self.pending_actions.push(AssetAction::ImportModel {
                                    path: selected.clone(),
                                });
                            }
                            if ui.button("🔍 Inspect").clicked() {
                                self.pending_actions.push(AssetAction::InspectAsset {
                                    path: selected.clone(),
                                });
                            }
                        }
                        AssetType::Texture => {
                            let tex_type = texture_type.unwrap_or(TextureType::Albedo);
                            if ui
                                .button(format!("🎨 Apply as {}", tex_type.label()))
                                .clicked()
                            {
                                self.pending_actions.push(AssetAction::ApplyTexture {
                                    path: selected.clone(),
                                    texture_type: tex_type,
                                });
                            }
                            if ui.button("🔍 Inspect").clicked() {
                                self.pending_actions.push(AssetAction::InspectAsset {
                                    path: selected.clone(),
                                });
                            }
                        }
                        AssetType::Material => {
                            if ui.button("💎 Apply Material").clicked() {
                                self.pending_actions.push(AssetAction::ApplyMaterial {
                                    path: selected.clone(),
                                });
                            }
                        }
                        AssetType::Scene => {
                            if ui.button("🌍 Load Scene").clicked() {
                                self.pending_actions.push(AssetAction::LoadScene {
                                    path: selected.clone(),
                                });
                            }
                        }
                        AssetType::Prefab => {
                            if ui.button("💾 Spawn Prefab").clicked() {
                                self.pending_actions.push(AssetAction::SpawnPrefab {
                                    path: selected.clone(),
                                });
                            }
                        }
                        _ => {}
                    }

                    // Common actions
                    if ui.button("📂 Open Folder").clicked() {
                        if let Some(parent) = selected.parent() {
                            self.pending_actions.push(AssetAction::OpenExternal {
                                path: parent.to_path_buf(),
                            });
                        }
                    }
                });
            });
        }
    }

    pub fn get_asset_count(&self) -> usize {
        self.entries.len()
    }

    pub fn get_current_directory(&self) -> &Path {
        &self.current_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_asset_type_from_path() {
        assert_eq!(
            AssetType::from_path(Path::new("test.glb")),
            AssetType::Model
        );
        assert_eq!(
            AssetType::from_path(Path::new("texture.png")),
            AssetType::Texture
        );
        assert_eq!(
            AssetType::from_path(Path::new("scene.ron")),
            AssetType::Scene
        );
        assert_eq!(
            AssetType::from_path(Path::new("config.toml")),
            AssetType::Config
        );
        assert_eq!(
            AssetType::from_path(Path::new("unknown.xyz")),
            AssetType::Unknown
        );
    }

    #[test]
    fn test_asset_type_icon() {
        assert_eq!(AssetType::Model.icon(), "🎭");
        assert_eq!(AssetType::Texture.icon(), "🖼️");
        assert_eq!(AssetType::Scene.icon(), "🌍");
        assert_eq!(AssetType::Directory.icon(), "📁");
    }

    #[test]
    fn test_asset_browser_creation() {
        let temp_dir = env::temp_dir();
        let browser = AssetBrowser::new(temp_dir.clone());
        assert_eq!(browser.root_path, temp_dir);
        assert_eq!(browser.current_path, temp_dir);
    }

    #[test]
    fn test_asset_browser_navigation() {
        let temp_dir = env::temp_dir();
        let mut browser = AssetBrowser::new(temp_dir.clone());

        // Navigate up - may or may not change path depending on temp_dir location
        // On some systems, temp_dir might be at a drive root where navigate_up has no effect
        browser.navigate_up();
        // Just verify navigate_up doesn't panic

        // Navigate back to temp_dir
        browser.navigate_to(temp_dir.clone());
        assert_eq!(browser.current_path, temp_dir);
    }

    #[test]
    fn test_asset_entry_format_size() {
        let entry = AssetEntry {
            path: PathBuf::from("test.glb"),
            name: "test.glb".to_string(),
            asset_type: AssetType::Model,
            texture_type: None,
            size: 1024,
        };
        assert_eq!(entry.format_size(), "1.0 KB");

        let entry_large = AssetEntry {
            path: PathBuf::from("large.glb"),
            name: "large.glb".to_string(),
            asset_type: AssetType::Model,
            texture_type: None,
            size: 1024 * 1024,
        };
        assert_eq!(entry_large.format_size(), "1.0 MB");
    }

    #[test]
    fn test_asset_browser_filter() {
        let temp_dir = env::temp_dir();
        let mut browser = AssetBrowser::new(temp_dir);

        browser.filter_type = Some(AssetType::Model);
        assert_eq!(browser.filter_type, Some(AssetType::Model));

        browser.filter_type = None;
        assert_eq!(browser.filter_type, None);
    }

    #[test]
    fn test_asset_browser_search() {
        let temp_dir = env::temp_dir();
        let mut browser = AssetBrowser::new(temp_dir);

        browser.search_query = "test".to_string();
        assert_eq!(browser.search_query, "test");
    }
}
