// tools/aw_editor/src/plugin.rs - Phase 5.3: Plugin System
//
// Provides an extensible plugin API for editor customization.
// Plugins can add custom panels, inspectors, gizmos, importers, and menu items.

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;

use egui::{Context, Ui};

/// Plugin metadata for identification and version management
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Unique plugin identifier (e.g., "com.studio.custom-inspector")
    pub id: String,
    /// Human-readable plugin name
    pub name: String,
    /// Plugin version (semver format)
    pub version: String,
    /// Plugin author/organization
    pub author: String,
    /// Brief description of plugin functionality
    pub description: String,
    /// Minimum editor version required
    pub min_editor_version: String,
}

impl PluginMetadata {
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            author: String::new(),
            description: String::new(),
            min_editor_version: "0.1.0".to_string(),
        }
    }

    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }
}

/// Plugin lifecycle events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginEvent {
    /// Plugin was just loaded
    Loaded,
    /// Plugin is being unloaded
    Unloading,
    /// Editor frame update (called every frame)
    Update,
    /// Scene was loaded
    SceneLoaded,
    /// Scene was saved
    SceneSaved,
    /// Entity was selected
    EntitySelected,
    /// Entity was created
    EntityCreated,
    /// Play mode entered
    PlayModeEnter,
    /// Play mode exited
    PlayModeExit,
}

/// Context provided to plugins during callbacks
pub struct PluginContext<'a> {
    /// egui context for UI rendering
    pub egui_ctx: &'a Context,
    /// Current scene path (if any)
    pub scene_path: Option<&'a PathBuf>,
    /// Selected entity IDs
    pub selected_entities: &'a [u32],
    /// Whether editor is in play mode
    pub is_playing: bool,
    /// Plugin-specific storage (persisted across frames)
    pub storage: &'a mut HashMap<String, Box<dyn Any + Send + Sync>>,
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin-specific error type
#[derive(Debug, Clone)]
pub enum PluginError {
    /// Plugin initialization failed
    InitFailed(String),
    /// Plugin configuration error
    ConfigError(String),
    /// Required dependency missing
    MissingDependency(String),
    /// Incompatible editor version
    IncompatibleVersion { required: String, actual: String },
    /// Generic plugin error
    Other(String),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::InitFailed(msg) => write!(f, "Plugin initialization failed: {}", msg),
            PluginError::ConfigError(msg) => write!(f, "Plugin configuration error: {}", msg),
            PluginError::MissingDependency(dep) => write!(f, "Missing dependency: {}", dep),
            PluginError::IncompatibleVersion { required, actual } => {
                write!(
                    f,
                    "Incompatible version: requires {}, got {}",
                    required, actual
                )
            }
            PluginError::Other(msg) => write!(f, "Plugin error: {}", msg),
        }
    }
}

impl std::error::Error for PluginError {}

/// Trait that all editor plugins must implement
pub trait EditorPlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Called when plugin is loaded
    fn on_load(&mut self, _ctx: &mut PluginContext) -> PluginResult<()> {
        Ok(())
    }

    /// Called when plugin is unloaded
    fn on_unload(&mut self, _ctx: &mut PluginContext) -> PluginResult<()> {
        Ok(())
    }

    /// Called every frame for updates
    fn on_update(&mut self, _ctx: &mut PluginContext) {}

    /// Called to render plugin's custom panel (if any)
    fn show_panel(&mut self, _ui: &mut Ui, _ctx: &mut PluginContext) {}

    /// Whether this plugin has a custom panel
    fn has_panel(&self) -> bool {
        false
    }

    /// Called when editor events occur
    fn on_event(&mut self, _event: PluginEvent, _ctx: &mut PluginContext) {}

    /// Get custom menu items to add to the editor
    fn menu_items(&self) -> Vec<PluginMenuItem> {
        Vec::new()
    }
}

/// Menu item contributed by a plugin
#[derive(Debug, Clone)]
pub struct PluginMenuItem {
    /// Menu path (e.g., "Tools/My Plugin/Action")
    pub path: String,
    /// Keyboard shortcut (e.g., "Ctrl+Shift+P")
    pub shortcut: Option<String>,
    /// Whether item is enabled
    pub enabled: bool,
    /// Action ID for callback
    pub action_id: String,
}

impl PluginMenuItem {
    pub fn new(path: &str, action_id: &str) -> Self {
        Self {
            path: path.to_string(),
            shortcut: None,
            enabled: true,
            action_id: action_id.to_string(),
        }
    }

    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }
}

/// Plugin state tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Plugin is loaded but not yet initialized
    Loaded,
    /// Plugin is active and running
    Active,
    /// Plugin encountered an error
    Error,
    /// Plugin is disabled by user
    Disabled,
}

/// Entry for a loaded plugin
struct PluginEntry {
    plugin: Box<dyn EditorPlugin>,
    state: PluginState,
    error: Option<String>,
    storage: HashMap<String, Box<dyn Any + Send + Sync>>,
}

/// Plugin manager handles loading, unloading, and updating plugins
pub struct PluginManager {
    plugins: HashMap<String, PluginEntry>,
    plugins_dir: PathBuf,
    editor_version: String,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new("plugins", "0.1.0")
    }
}

impl PluginManager {
    pub fn new(plugins_dir: &str, editor_version: &str) -> Self {
        Self {
            plugins: HashMap::new(),
            plugins_dir: PathBuf::from(plugins_dir),
            editor_version: editor_version.to_string(),
        }
    }

    /// Register a plugin with the manager
    pub fn register(&mut self, plugin: Box<dyn EditorPlugin>) -> PluginResult<()> {
        let metadata = plugin.metadata();
        let id = metadata.id.clone();

        if self.plugins.contains_key(&id) {
            return Err(PluginError::Other(format!(
                "Plugin '{}' already registered",
                id
            )));
        }

        // Version compatibility check
        if !self.is_version_compatible(&metadata.min_editor_version) {
            return Err(PluginError::IncompatibleVersion {
                required: metadata.min_editor_version.clone(),
                actual: self.editor_version.clone(),
            });
        }

        tracing::info!(
            "Registering plugin: {} v{}",
            metadata.name,
            metadata.version
        );

        self.plugins.insert(
            id,
            PluginEntry {
                plugin,
                state: PluginState::Loaded,
                error: None,
                storage: HashMap::new(),
            },
        );

        Ok(())
    }

    /// Initialize a registered plugin
    pub fn initialize(&mut self, plugin_id: &str, ctx: &Context) -> PluginResult<()> {
        let entry = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::Other(format!("Plugin '{}' not found", plugin_id)))?;

        if entry.state != PluginState::Loaded && entry.state != PluginState::Error {
            return Err(PluginError::Other("Plugin already initialized".to_string()));
        }

        let mut plugin_ctx = PluginContext {
            egui_ctx: ctx,
            scene_path: None,
            selected_entities: &[],
            is_playing: false,
            storage: &mut entry.storage,
        };

        match entry.plugin.on_load(&mut plugin_ctx) {
            Ok(()) => {
                entry.state = PluginState::Active;
                entry.error = None;
                tracing::info!("Plugin '{}' initialized", plugin_id);
                Ok(())
            }
            Err(e) => {
                entry.state = PluginState::Error;
                entry.error = Some(e.to_string());
                Err(e)
            }
        }
    }

    /// Unload a plugin
    pub fn unload(&mut self, plugin_id: &str, ctx: &Context) -> PluginResult<()> {
        let entry = self
            .plugins
            .get_mut(plugin_id)
            .ok_or_else(|| PluginError::Other(format!("Plugin '{}' not found", plugin_id)))?;

        let mut plugin_ctx = PluginContext {
            egui_ctx: ctx,
            scene_path: None,
            selected_entities: &[],
            is_playing: false,
            storage: &mut entry.storage,
        };

        entry.plugin.on_unload(&mut plugin_ctx)?;
        self.plugins.remove(plugin_id);

        tracing::info!("Plugin '{}' unloaded", plugin_id);
        Ok(())
    }

    /// Update all active plugins
    pub fn update(
        &mut self,
        ctx: &Context,
        scene_path: Option<&PathBuf>,
        selected_entities: &[u32],
        is_playing: bool,
    ) {
        for entry in self.plugins.values_mut() {
            if entry.state == PluginState::Active {
                let mut plugin_ctx = PluginContext {
                    egui_ctx: ctx,
                    scene_path,
                    selected_entities,
                    is_playing,
                    storage: &mut entry.storage,
                };
                entry.plugin.on_update(&mut plugin_ctx);
            }
        }
    }

    /// Broadcast an event to all active plugins
    pub fn broadcast_event(
        &mut self,
        event: PluginEvent,
        ctx: &Context,
        scene_path: Option<&PathBuf>,
        selected_entities: &[u32],
        is_playing: bool,
    ) {
        for entry in self.plugins.values_mut() {
            if entry.state == PluginState::Active {
                let mut plugin_ctx = PluginContext {
                    egui_ctx: ctx,
                    scene_path,
                    selected_entities,
                    is_playing,
                    storage: &mut entry.storage,
                };
                entry.plugin.on_event(event, &mut plugin_ctx);
            }
        }
    }

    /// Show plugin panel for a specific plugin
    pub fn show_plugin_panel(
        &mut self,
        plugin_id: &str,
        ui: &mut Ui,
        ctx: &Context,
        scene_path: Option<&PathBuf>,
        selected_entities: &[u32],
        is_playing: bool,
    ) {
        if let Some(entry) = self.plugins.get_mut(plugin_id) {
            if entry.state == PluginState::Active && entry.plugin.has_panel() {
                let mut plugin_ctx = PluginContext {
                    egui_ctx: ctx,
                    scene_path,
                    selected_entities,
                    is_playing,
                    storage: &mut entry.storage,
                };
                entry.plugin.show_panel(ui, &mut plugin_ctx);
            }
        }
    }

    /// Get list of all registered plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|(id, entry)| PluginInfo {
                id: id.clone(),
                metadata: entry.plugin.metadata(),
                state: entry.state,
                error: entry.error.clone(),
                has_panel: entry.plugin.has_panel(),
            })
            .collect()
    }

    /// Enable/disable a plugin
    pub fn set_enabled(&mut self, plugin_id: &str, enabled: bool) {
        if let Some(entry) = self.plugins.get_mut(plugin_id) {
            if enabled && entry.state == PluginState::Disabled {
                entry.state = PluginState::Active;
            } else if !enabled && entry.state == PluginState::Active {
                entry.state = PluginState::Disabled;
            }
        }
    }

    /// Collect all menu items from plugins
    pub fn collect_menu_items(&self) -> Vec<(String, PluginMenuItem)> {
        let mut items = Vec::new();
        for (id, entry) in &self.plugins {
            if entry.state == PluginState::Active {
                for item in entry.plugin.menu_items() {
                    items.push((id.clone(), item));
                }
            }
        }
        items
    }

    fn is_version_compatible(&self, required: &str) -> bool {
        // Simple semver comparison (major.minor.patch)
        let parse_version = |v: &str| -> (u32, u32, u32) {
            let parts: Vec<_> = v.split('.').collect();
            (
                parts.first().and_then(|s| s.parse().ok()).unwrap_or(0),
                parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
            )
        };

        let req = parse_version(required);
        let actual = parse_version(&self.editor_version);

        // Editor version must be >= required version
        actual >= req
    }
}

/// Plugin info for UI display
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub metadata: PluginMetadata,
    pub state: PluginState,
    pub error: Option<String>,
    pub has_panel: bool,
}

/// Panel for managing plugins in the editor UI
pub struct PluginManagerPanel {
    filter_text: String,
    last_error: Option<String>,
}

impl Default for PluginManagerPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManagerPanel {
    pub fn new() -> Self {
        Self {
            filter_text: String::new(),
            last_error: None,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, manager: &mut PluginManager, ctx: &Context) {
        ui.heading("🔌 Plugin Manager");
        ui.add_space(8.0);

        // Show global error if any
        let mut dismiss = false;
        if let Some(error) = &self.last_error {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠️").color(egui::Color32::RED));
                    ui.label(egui::RichText::new(error).color(egui::Color32::RED));
                    if ui.button("Dismiss").clicked() {
                        dismiss = true;
                    }
                });
            });
            ui.add_space(8.0);
        }
        if dismiss {
            self.last_error = None;
        }

        // Search filter
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.filter_text);
        });

        ui.add_space(8.0);
        ui.separator();

        // Plugin list
        let plugins = manager.list_plugins();

        if plugins.is_empty() {
            ui.label(
                egui::RichText::new("No plugins loaded")
                    .italics()
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(8.0);
            ui.label("Place plugins in the 'plugins/' directory");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for info in plugins {
                    // Apply filter
                    if !self.filter_text.is_empty() {
                        let filter_lower = self.filter_text.to_lowercase();
                        if !info.metadata.name.to_lowercase().contains(&filter_lower)
                            && !info
                                .metadata
                                .description
                                .to_lowercase()
                                .contains(&filter_lower)
                        {
                            continue;
                        }
                    }

                    self.show_plugin_entry(ui, &info, manager, ctx);
                }
            });
        }
    }

    fn show_plugin_entry(
        &mut self,
        ui: &mut Ui,
        info: &PluginInfo,
        manager: &mut PluginManager,
        ctx: &Context,
    ) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // State indicator
                let (state_icon, state_color) = match info.state {
                    PluginState::Active => ("✅", egui::Color32::GREEN),
                    PluginState::Loaded => ("⏳", egui::Color32::YELLOW),
                    PluginState::Disabled => ("⏸️", egui::Color32::GRAY),
                    PluginState::Error => ("❌", egui::Color32::RED),
                };
                ui.label(egui::RichText::new(state_icon).color(state_color));

                // Plugin name and version
                ui.label(egui::RichText::new(&info.metadata.name).strong());
                ui.label(
                    egui::RichText::new(format!("v{}", info.metadata.version))
                        .small()
                        .color(egui::Color32::GRAY),
                );

                // Panel indicator
                if info.has_panel {
                    ui.label(egui::RichText::new("📋").small());
                }
            });

            // Description
            if !info.metadata.description.is_empty() {
                ui.label(egui::RichText::new(&info.metadata.description).small());
            }

            // Author
            if !info.metadata.author.is_empty() {
                ui.label(
                    egui::RichText::new(format!("by {}", info.metadata.author))
                        .small()
                        .color(egui::Color32::GRAY),
                );
            }

            // Error message
            if let Some(error) = &info.error {
                ui.label(
                    egui::RichText::new(format!("Error: {}", error))
                        .small()
                        .color(egui::Color32::RED),
                );
            }

            // Actions
            ui.horizontal(|ui| {
                match info.state {
                    PluginState::Loaded => {
                        if ui.button("Initialize").clicked() {
                            if let Err(e) = manager.initialize(&info.id, ctx) {
                                self.last_error = Some(format!(
                                    "Failed to initialize plugin '{}': {}",
                                    info.metadata.name, e
                                ));
                            }
                        }
                    }
                    PluginState::Active => {
                        if ui.button("Disable").clicked() {
                            manager.set_enabled(&info.id, false);
                        }
                    }
                    PluginState::Disabled => {
                        if ui.button("Enable").clicked() {
                            manager.set_enabled(&info.id, true);
                        }
                    }
                    PluginState::Error => {
                        if ui.button("Retry").clicked() {
                            if let Err(e) = manager.initialize(&info.id, ctx) {
                                self.last_error = Some(format!(
                                    "Retry failed for plugin '{}': {}",
                                    info.metadata.name, e
                                ));
                            }
                        }
                    }
                }

                if ui.button("🗑️").clicked() {
                    if let Err(e) = manager.unload(&info.id, ctx) {
                        self.last_error = Some(format!(
                            "Failed to unload plugin '{}': {}",
                            info.metadata.name, e
                        ));
                    }
                }
            });
        });

        ui.add_space(4.0);
    }
}

// ============================================================================
// Example Plugin Implementation
// ============================================================================

/// Example plugin demonstrating the plugin API
pub struct ExamplePlugin {
    counter: u32,
    show_window: bool,
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            counter: 0,
            show_window: false,
        }
    }
}

impl EditorPlugin for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new("com.astraweave.example-plugin", "Example Plugin", "1.0.0")
            .with_author("AstraWeave Team")
            .with_description("Demonstrates the plugin API with a simple counter")
    }

    fn on_load(&mut self, _ctx: &mut PluginContext) -> PluginResult<()> {
        tracing::info!("Example plugin loaded!");
        Ok(())
    }

    fn on_update(&mut self, _ctx: &mut PluginContext) {
        // Called every frame
    }

    fn has_panel(&self) -> bool {
        true
    }

    fn show_panel(&mut self, ui: &mut Ui, _ctx: &mut PluginContext) {
        ui.heading("🔌 Example Plugin");

        ui.label(format!("Counter: {}", self.counter));

        ui.horizontal(|ui| {
            if ui.button("➕ Increment").clicked() {
                self.counter += 1;
            }
            if ui.button("➖ Decrement").clicked() {
                self.counter = self.counter.saturating_sub(1);
            }
            if ui.button("🔄 Reset").clicked() {
                self.counter = 0;
            }
        });

        ui.checkbox(&mut self.show_window, "Show floating window");

        if self.show_window {
            ui.label("(Floating window would appear here in full implementation)");
        }
    }

    fn on_event(&mut self, event: PluginEvent, _ctx: &mut PluginContext) {
        match event {
            PluginEvent::EntitySelected => {
                tracing::debug!("Example plugin: Entity selected");
            }
            PluginEvent::PlayModeEnter => {
                tracing::info!("Example plugin: Entering play mode, resetting counter");
                self.counter = 0;
            }
            _ => {}
        }
    }

    fn menu_items(&self) -> Vec<PluginMenuItem> {
        vec![
            PluginMenuItem::new("Tools/Example Plugin/Reset Counter", "reset_counter"),
            PluginMenuItem::new("Tools/Example Plugin/Show Window", "show_window")
                .with_shortcut("Ctrl+Shift+E"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let meta = PluginMetadata::new("test.plugin", "Test Plugin", "1.0.0")
            .with_author("Test Author")
            .with_description("A test plugin");

        assert_eq!(meta.id, "test.plugin");
        assert_eq!(meta.name, "Test Plugin");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.author, "Test Author");
    }

    #[test]
    fn test_plugin_menu_item() {
        let item = PluginMenuItem::new("Tools/Test", "test_action").with_shortcut("Ctrl+T");

        assert_eq!(item.path, "Tools/Test");
        assert_eq!(item.action_id, "test_action");
        assert_eq!(item.shortcut, Some("Ctrl+T".to_string()));
        assert!(item.enabled);
    }

    #[test]
    fn test_example_plugin() {
        let plugin = ExamplePlugin::new();
        assert_eq!(plugin.counter, 0);

        let meta = plugin.metadata();
        assert_eq!(meta.id, "com.astraweave.example-plugin");
        assert!(plugin.has_panel());
        assert_eq!(plugin.menu_items().len(), 2);
    }

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new("plugins", "0.1.0");
        assert!(manager.list_plugins().is_empty());
    }

    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new("plugins", "1.0.0");
        let plugin = Box::new(ExamplePlugin::new());

        let result = manager.register(plugin);
        assert!(result.is_ok());
        assert_eq!(manager.list_plugins().len(), 1);
    }

    #[test]
    fn test_plugin_version_compatibility() {
        let mut manager = PluginManager::new("plugins", "0.1.0");

        // Plugin requiring newer version should fail
        struct NewVersionPlugin;
        impl EditorPlugin for NewVersionPlugin {
            fn metadata(&self) -> PluginMetadata {
                let mut meta = PluginMetadata::new("new.plugin", "New Plugin", "1.0.0");
                meta.min_editor_version = "2.0.0".to_string();
                meta
            }
        }

        let result = manager.register(Box::new(NewVersionPlugin));
        assert!(matches!(
            result,
            Err(PluginError::IncompatibleVersion { .. })
        ));
    }
}
