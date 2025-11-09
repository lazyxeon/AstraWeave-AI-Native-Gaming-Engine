use egui::{ScrollArea, Ui};
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetType {
    Model,
    Texture,
    Scene,
    Material,
    Audio,
    Config,
    Directory,
    Unknown,
}

impl AssetType {
    pub fn from_path(path: &Path) -> Self {
        if path.is_dir() {
            return AssetType::Directory;
        }

        match path.extension().and_then(|e| e.to_str()) {
            Some("glb") | Some("gltf") | Some("obj") | Some("fbx") => AssetType::Model,
            Some("png") | Some("jpg") | Some("jpeg") | Some("ktx2") | Some("dds") => AssetType::Texture,
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

pub struct AssetBrowser {
    root_path: PathBuf,
    current_path: PathBuf,
    entries: Vec<AssetEntry>,
    selected_asset: Option<PathBuf>,
    show_hidden: bool,
    filter_type: Option<AssetType>,
    search_query: String,
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
        };
        browser.scan_current_directory();
        browser
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
                    if !entry.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                        return false;
                    }
                }

                true
            })
            .collect();

        entries.sort_by(|a, b| {
            match (&a.asset_type, &b.asset_type) {
                (AssetType::Directory, AssetType::Directory) => a.name.cmp(&b.name),
                (AssetType::Directory, _) => std::cmp::Ordering::Less,
                (_, AssetType::Directory) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
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
        });

        ui.horizontal(|ui| {
            ui.label("Filter:");
            
            if ui.selectable_label(self.filter_type.is_none(), "All").clicked() {
                self.filter_type = None;
                self.scan_current_directory();
            }

            if ui.selectable_label(self.filter_type == Some(AssetType::Model), "🎭 Models").clicked() {
                self.filter_type = if self.filter_type == Some(AssetType::Model) {
                    None
                } else {
                    Some(AssetType::Model)
                };
                self.scan_current_directory();
            }

            if ui.selectable_label(self.filter_type == Some(AssetType::Texture), "🖼️ Textures").clicked() {
                self.filter_type = if self.filter_type == Some(AssetType::Texture) {
                    None
                } else {
                    Some(AssetType::Texture)
                };
                self.scan_current_directory();
            }

            if ui.selectable_label(self.filter_type == Some(AssetType::Scene), "🌍 Scenes").clicked() {
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
            self.current_path.strip_prefix(&self.root_path)
                .unwrap_or(&self.current_path)
                .display()
        ));

        ui.separator();

        let mut path_to_navigate = None;
        
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
                        response.on_hover_text(entry.path.display().to_string());
                    }
                }

                if self.entries.is_empty() {
                    ui.colored_label(
                        egui::Color32::GRAY,
                        if self.search_query.is_empty() {
                            "Empty directory"
                        } else {
                            "No matching assets"
                        }
                    );
                }
            });
        
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

    #[test]
    fn test_asset_type_from_path() {
        assert_eq!(AssetType::from_path(Path::new("test.glb")), AssetType::Model);
        assert_eq!(AssetType::from_path(Path::new("texture.png")), AssetType::Texture);
        assert_eq!(AssetType::from_path(Path::new("scene.ron")), AssetType::Scene);
        assert_eq!(AssetType::from_path(Path::new("config.toml")), AssetType::Config);
        assert_eq!(AssetType::from_path(Path::new("unknown.xyz")), AssetType::Unknown);
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
        
        browser.navigate_up();
        assert!(browser.current_path != temp_dir);
        
        browser.navigate_to(temp_dir.clone());
        assert_eq!(browser.current_path, temp_dir);
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
}
