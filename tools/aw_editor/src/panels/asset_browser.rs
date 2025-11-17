use egui::{ColorImage, ImageData, ScrollArea, TextureHandle, Ui};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub size: u64,
}

impl AssetEntry {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let asset_type = AssetType::from_path(&path);

        let size = if path.is_file() {
            fs::metadata(&path).ok()?.len()
        } else {
            0
        };

        Some(AssetEntry {
            path,
            name,
            asset_type,
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
            view_mode: ViewMode::List,
            thumbnail_cache: HashMap::new(),
            thumbnail_size: 64.0,
            dragged_prefab: None,
        };
        browser.scan_current_directory();
        browser
    }

    pub fn take_dragged_prefab(&mut self) -> Option<PathBuf> {
        self.dragged_prefab.take()
    }

    pub fn is_dragging_prefab(&self) -> bool {
        self.dragged_prefab.is_some()
    }

    pub fn cancel_prefab_drag(&mut self) {
        self.dragged_prefab = None;
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
                if !self.show_hidden && entry.name.starts_with('.') {
                    return false;
                }

                if let Some(filter) = &self.filter_type {
                    if &entry.asset_type != filter {
                        return false;
                    }
                }

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
        });

        ui.horizontal(|ui| {
            ui.label("Filter:");

            if ui
                .selectable_label(self.filter_type.is_none(), "All")
                .clicked()
            {
                self.filter_type = None;
                self.scan_current_directory();
            }

            if ui
                .selectable_label(self.filter_type == Some(AssetType::Model), "🎭 Models")
                .clicked()
            {
                self.filter_type = if self.filter_type == Some(AssetType::Model) {
                    None
                } else {
                    Some(AssetType::Model)
                };
                self.scan_current_directory();
            }

            if ui
                .selectable_label(self.filter_type == Some(AssetType::Texture), "🖼️ Textures")
                .clicked()
            {
                self.filter_type = if self.filter_type == Some(AssetType::Texture) {
                    None
                } else {
                    Some(AssetType::Texture)
                };
                self.scan_current_directory();
            }

            if ui
                .selectable_label(self.filter_type == Some(AssetType::Scene), "🌍 Scenes")
                .clicked()
            {
                self.filter_type = if self.filter_type == Some(AssetType::Scene) {
                    None
                } else {
                    Some(AssetType::Scene)
                };
                self.scan_current_directory();
            }
        });

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

        if let Some(selected) = &self.selected_asset {
            ui.separator();
            ui.label("Selected:");
            ui.monospace(selected.display().to_string());
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
    use tempfile::tempdir;

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
        let temp_dir = tempdir().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        let child_path = root_path.join("child");
        std::fs::create_dir_all(&child_path).unwrap();

        let mut browser = AssetBrowser::new(root_path.clone());

        // Already at root, navigate_up should not move outside root
        browser.navigate_up();
        assert_eq!(browser.current_path, root_path);

        // Navigate into child directory and ensure path updates
        browser.navigate_to(child_path.clone());
        assert_eq!(browser.current_path, child_path);

        // Navigating up from child returns to root
        browser.navigate_up();
        assert_eq!(browser.current_path, root_path);
    }

    #[test]
    fn test_asset_entry_format_size() {
        let entry = AssetEntry {
            path: PathBuf::from("test.glb"),
            name: "test.glb".to_string(),
            asset_type: AssetType::Model,
            size: 1024,
        };
        assert_eq!(entry.format_size(), "1.0 KB");

        let entry_large = AssetEntry {
            path: PathBuf::from("large.glb"),
            name: "large.glb".to_string(),
            asset_type: AssetType::Model,
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

    #[test]
    fn test_prefab_drag_helpers() {
        let temp_dir = tempdir().unwrap();
        let mut browser = AssetBrowser::new(temp_dir.path().to_path_buf());
        assert!(!browser.is_dragging_prefab());

        let prefab_path = temp_dir.path().join("example.prefab.ron");
        browser.dragged_prefab = Some(prefab_path.clone());
        assert!(browser.is_dragging_prefab());
        assert_eq!(browser.take_dragged_prefab(), Some(prefab_path));
        assert!(!browser.is_dragging_prefab());

        // Ensure cancel clears any pending drag
        browser.dragged_prefab = Some(temp_dir.path().join("second.prefab.ron"));
        browser.cancel_prefab_drag();
        assert!(!browser.is_dragging_prefab());
    }
}
