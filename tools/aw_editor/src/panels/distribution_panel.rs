//! Distribution panel for the editor UI
//!
//! Provides a comprehensive GUI for creating platform-specific distributions
//! (Windows, macOS, Linux, Steam) with build profiles, validation, and history.

#![allow(clippy::upper_case_acronyms)]

use egui::{Color32, RichText, Ui};
use std::collections::VecDeque;
use std::path::PathBuf;

use aw_editor_lib::distribution::{
    DistributionBuilder, DistributionConfig, DistributionFormat, DistributionResult,
};
use crate::panels::Panel;

// ═══════════════════════════════════════════════════════════════════════════════════
// BUILD PROFILE
// ═══════════════════════════════════════════════════════════════════════════════════

/// Build optimization profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildProfile {
    /// Debug build with symbols
    Debug,
    /// Optimized release build
    #[default]
    Release,
    /// Maximum optimization with LTO
    ReleaseOptimized,
    /// Minimal size build
    MinSize,
}

impl BuildProfile {
    pub fn name(&self) -> &'static str {
        match self {
            BuildProfile::Debug => "Debug",
            BuildProfile::Release => "Release",
            BuildProfile::ReleaseOptimized => "Release (Optimized)",
            BuildProfile::MinSize => "Minimum Size",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BuildProfile::Debug => "Debug symbols, no optimization, fast build",
            BuildProfile::Release => "Standard release, balanced optimization",
            BuildProfile::ReleaseOptimized => "LTO, maximum optimization, slow build",
            BuildProfile::MinSize => "Size optimized, smaller binary",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            BuildProfile::Debug => "🔧",
            BuildProfile::Release => "📦",
            BuildProfile::ReleaseOptimized => "⚡",
            BuildProfile::MinSize => "📎",
        }
    }

    pub fn cargo_profile(&self) -> &'static str {
        match self {
            BuildProfile::Debug => "debug",
            BuildProfile::Release => "release",
            BuildProfile::ReleaseOptimized => "release-lto",
            BuildProfile::MinSize => "release-small",
        }
    }

    pub fn all() -> &'static [BuildProfile] {
        &[
            BuildProfile::Debug,
            BuildProfile::Release,
            BuildProfile::ReleaseOptimized,
            BuildProfile::MinSize,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// TARGET PLATFORM
// ═══════════════════════════════════════════════════════════════════════════════════

/// Target platform for cross-compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetPlatform {
    #[default]
    Native,
    Windows64,
    Windows32,
    MacOSArm64,
    MacOSx64,
    MacOSUniversal,
    LinuxX64,
    LinuxArm64,
}

impl TargetPlatform {
    pub fn name(&self) -> &'static str {
        match self {
            TargetPlatform::Native => "Native",
            TargetPlatform::Windows64 => "Windows x64",
            TargetPlatform::Windows32 => "Windows x86",
            TargetPlatform::MacOSArm64 => "macOS ARM64",
            TargetPlatform::MacOSx64 => "macOS x64",
            TargetPlatform::MacOSUniversal => "macOS Universal",
            TargetPlatform::LinuxX64 => "Linux x64",
            TargetPlatform::LinuxArm64 => "Linux ARM64",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TargetPlatform::Native => "🖥️",
            TargetPlatform::Windows64 | TargetPlatform::Windows32 => "🪟",
            TargetPlatform::MacOSArm64 | TargetPlatform::MacOSx64 | TargetPlatform::MacOSUniversal => "🍎",
            TargetPlatform::LinuxX64 | TargetPlatform::LinuxArm64 => "🐧",
        }
    }

    pub fn rust_target(&self) -> Option<&'static str> {
        match self {
            TargetPlatform::Native => None,
            TargetPlatform::Windows64 => Some("x86_64-pc-windows-msvc"),
            TargetPlatform::Windows32 => Some("i686-pc-windows-msvc"),
            TargetPlatform::MacOSArm64 => Some("aarch64-apple-darwin"),
            TargetPlatform::MacOSx64 => Some("x86_64-apple-darwin"),
            TargetPlatform::MacOSUniversal => None, // Special handling
            TargetPlatform::LinuxX64 => Some("x86_64-unknown-linux-gnu"),
            TargetPlatform::LinuxArm64 => Some("aarch64-unknown-linux-gnu"),
        }
    }

    pub fn all() -> &'static [TargetPlatform] {
        &[
            TargetPlatform::Native,
            TargetPlatform::Windows64,
            TargetPlatform::Windows32,
            TargetPlatform::MacOSArm64,
            TargetPlatform::MacOSx64,
            TargetPlatform::MacOSUniversal,
            TargetPlatform::LinuxX64,
            TargetPlatform::LinuxArm64,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// BUILD OPTIONS
// ═══════════════════════════════════════════════════════════════════════════════════

/// Additional build options
#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub strip_symbols: bool,
    pub compress_assets: bool,
    pub embed_runtime: bool,
    pub sign_binary: bool,
    pub notarize_macos: bool,
    pub create_installer: bool,
    pub generate_checksums: bool,
    pub include_debug_symbols: bool,
    pub run_tests_before_build: bool,
    pub clean_before_build: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            strip_symbols: true,
            compress_assets: true,
            embed_runtime: false,
            sign_binary: false,
            notarize_macos: false,
            create_installer: true,
            generate_checksums: true,
            include_debug_symbols: false,
            run_tests_before_build: true,
            clean_before_build: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// ASSET OPTIONS
// ═══════════════════════════════════════════════════════════════════════════════════

/// Asset packaging options
#[derive(Debug, Clone)]
pub struct AssetOptions {
    pub compress_textures: bool,
    pub compress_audio: bool,
    pub compress_meshes: bool,
    pub pack_into_archives: bool,
    pub encrypt_assets: bool,
    pub generate_manifests: bool,
    pub texture_format: TextureFormat,
    pub audio_format: AudioFormat,
    pub max_texture_size: u32,
}

impl Default for AssetOptions {
    fn default() -> Self {
        Self {
            compress_textures: true,
            compress_audio: true,
            compress_meshes: true,
            pack_into_archives: true,
            encrypt_assets: false,
            generate_manifests: true,
            texture_format: TextureFormat::BC7,
            audio_format: AudioFormat::Vorbis,
            max_texture_size: 4096,
        }
    }
}

/// Texture compression format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureFormat {
    None,
    #[default]
    BC7,
    BC5,
    ASTC,
    ETC2,
}

impl TextureFormat {
    pub fn name(&self) -> &'static str {
        match self {
            TextureFormat::None => "Uncompressed",
            TextureFormat::BC7 => "BC7 (DX11+)",
            TextureFormat::BC5 => "BC5 (Normals)",
            TextureFormat::ASTC => "ASTC (Mobile)",
            TextureFormat::ETC2 => "ETC2 (OpenGL ES)",
        }
    }

    pub fn all() -> &'static [TextureFormat] {
        &[
            TextureFormat::None,
            TextureFormat::BC7,
            TextureFormat::BC5,
            TextureFormat::ASTC,
            TextureFormat::ETC2,
        ]
    }
}

/// Audio compression format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioFormat {
    None,
    #[default]
    Vorbis,
    Opus,
    MP3,
    AAC,
}

impl AudioFormat {
    pub fn name(&self) -> &'static str {
        match self {
            AudioFormat::None => "Uncompressed",
            AudioFormat::Vorbis => "Vorbis (OGG)",
            AudioFormat::Opus => "Opus",
            AudioFormat::MP3 => "MP3",
            AudioFormat::AAC => "AAC",
        }
    }

    pub fn all() -> &'static [AudioFormat] {
        &[
            AudioFormat::None,
            AudioFormat::Vorbis,
            AudioFormat::Opus,
            AudioFormat::MP3,
            AudioFormat::AAC,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// BUILD STEP
// ═══════════════════════════════════════════════════════════════════════════════════

/// Build pipeline step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStep {
    Preparing,
    CleaningBuild,
    RunningTests,
    CompilingCode,
    ProcessingAssets,
    PackagingFiles,
    SigningBinary,
    GeneratingChecksums,
    CreatingInstaller,
    Finalizing,
    Complete,
    Failed,
}

impl BuildStep {
    pub fn name(&self) -> &'static str {
        match self {
            BuildStep::Preparing => "Preparing",
            BuildStep::CleaningBuild => "Cleaning",
            BuildStep::RunningTests => "Running Tests",
            BuildStep::CompilingCode => "Compiling",
            BuildStep::ProcessingAssets => "Processing Assets",
            BuildStep::PackagingFiles => "Packaging",
            BuildStep::SigningBinary => "Signing",
            BuildStep::GeneratingChecksums => "Checksums",
            BuildStep::CreatingInstaller => "Installer",
            BuildStep::Finalizing => "Finalizing",
            BuildStep::Complete => "Complete",
            BuildStep::Failed => "Failed",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            BuildStep::Preparing => "🔄",
            BuildStep::CleaningBuild => "🧹",
            BuildStep::RunningTests => "🧪",
            BuildStep::CompilingCode => "🔨",
            BuildStep::ProcessingAssets => "🎨",
            BuildStep::PackagingFiles => "📦",
            BuildStep::SigningBinary => "🔐",
            BuildStep::GeneratingChecksums => "✓",
            BuildStep::CreatingInstaller => "💿",
            BuildStep::Finalizing => "✨",
            BuildStep::Complete => "✅",
            BuildStep::Failed => "❌",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, BuildStep::Complete | BuildStep::Failed)
    }

    pub fn all_steps() -> &'static [BuildStep] {
        &[
            BuildStep::Preparing,
            BuildStep::CleaningBuild,
            BuildStep::RunningTests,
            BuildStep::CompilingCode,
            BuildStep::ProcessingAssets,
            BuildStep::PackagingFiles,
            BuildStep::SigningBinary,
            BuildStep::GeneratingChecksums,
            BuildStep::CreatingInstaller,
            BuildStep::Finalizing,
            BuildStep::Complete,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// BUILD PROGRESS
// ═══════════════════════════════════════════════════════════════════════════════════

/// Build progress tracker
#[derive(Debug, Clone)]
pub struct BuildProgress {
    pub current_step: BuildStep,
    pub step_progress: f32,
    pub overall_progress: f32,
    pub status_message: String,
    pub start_time: Option<std::time::Instant>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl Default for BuildProgress {
    fn default() -> Self {
        Self {
            current_step: BuildStep::Preparing,
            step_progress: 0.0,
            overall_progress: 0.0,
            status_message: String::new(),
            start_time: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl BuildProgress {
    pub fn elapsed_secs(&self) -> f32 {
        self.start_time.map(|t| t.elapsed().as_secs_f32()).unwrap_or(0.0)
    }

    pub fn start(&mut self) {
        self.start_time = Some(std::time::Instant::now());
        self.current_step = BuildStep::Preparing;
        self.step_progress = 0.0;
        self.overall_progress = 0.0;
        self.warnings.clear();
        self.errors.clear();
    }

    pub fn set_step(&mut self, step: BuildStep, message: impl Into<String>) {
        self.current_step = step;
        self.step_progress = 0.0;
        self.status_message = message.into();
    }

    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// BUILD HISTORY
// ═══════════════════════════════════════════════════════════════════════════════════

/// Build history entry
#[derive(Debug, Clone)]
pub struct BuildHistoryEntry {
    pub timestamp: std::time::SystemTime,
    pub format: DistributionFormat,
    pub platform: TargetPlatform,
    pub profile: BuildProfile,
    pub success: bool,
    pub duration_secs: f32,
    pub output_size_bytes: u64,
    pub output_path: String,
    pub version: String,
}

impl BuildHistoryEntry {
    pub fn age_string(&self) -> String {
        let elapsed = self.timestamp.elapsed().unwrap_or_default();
        let secs = elapsed.as_secs();
        
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    }

    pub fn size_string(&self) -> String {
        let mb = self.output_size_bytes as f64 / 1024.0 / 1024.0;
        if mb < 1.0 {
            format!("{:.0} KB", self.output_size_bytes as f64 / 1024.0)
        } else if mb < 1024.0 {
            format!("{:.1} MB", mb)
        } else {
            format!("{:.2} GB", mb / 1024.0)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// VALIDATION
// ═══════════════════════════════════════════════════════════════════════════════════

/// Pre-build validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl ValidationResult {
    pub fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub category: &'static str,
    pub message: String,
    pub fix_suggestion: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub category: &'static str,
    pub message: String,
}

// ═══════════════════════════════════════════════════════════════════════════════════
// DISTRIBUTION PANEL
// ═══════════════════════════════════════════════════════════════════════════════════

/// Comprehensive distribution panel for creating game packages
pub struct DistributionPanel {
    // Configuration
    config: DistributionConfig,
    selected_format: DistributionFormat,
    build_dir: String,
    output_dir: String,
    
    // Build settings
    profile: BuildProfile,
    platform: TargetPlatform,
    build_options: BuildOptions,
    asset_options: AssetOptions,
    
    // Build state
    is_building: bool,
    progress: BuildProgress,
    last_result: Option<Result<DistributionResult, String>>,
    
    // Validation
    validation: ValidationResult,
    auto_validate: bool,
    
    // History
    build_history: VecDeque<BuildHistoryEntry>,
    max_history: usize,
    
    // UI state
    show_build_options: bool,
    show_asset_options: bool,
    show_validation: bool,
    show_history: bool,
    show_progress: bool,
    
    // File dialog state
    pending_build_dir: Option<PathBuf>,
    pending_output_dir: Option<PathBuf>,
}

impl Default for DistributionPanel {
    fn default() -> Self {
        Self {
            config: DistributionConfig::default(),
            selected_format: DistributionFormat::WindowsPortable,
            build_dir: "target/release".to_string(),
            output_dir: "dist".to_string(),
            profile: BuildProfile::Release,
            platform: TargetPlatform::Native,
            build_options: BuildOptions::default(),
            asset_options: AssetOptions::default(),
            is_building: false,
            progress: BuildProgress::default(),
            last_result: None,
            validation: ValidationResult::default(),
            auto_validate: true,
            build_history: VecDeque::with_capacity(20),
            max_history: 20,
            show_build_options: false,
            show_asset_options: false,
            show_validation: true,
            show_history: true,
            show_progress: true,
            pending_build_dir: None,
            pending_output_dir: None,
        }
    }
}

impl DistributionPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn validate(&mut self) {
        self.validation = ValidationResult::default();
        
        // Check game name
        if self.config.game_name.is_empty() {
            self.validation.add_error(ValidationError {
                category: "Config",
                message: "Game name is required".to_string(),
                fix_suggestion: Some("Enter a game name".to_string()),
            });
        }
        
        // Check version
        if self.config.version.is_empty() {
            self.validation.add_error(ValidationError {
                category: "Config",
                message: "Version is required".to_string(),
                fix_suggestion: Some("Enter a version like '1.0.0'".to_string()),
            });
        }
        
        // Check build directory
        if self.build_dir.is_empty() {
            self.validation.add_error(ValidationError {
                category: "Paths",
                message: "Build directory is required".to_string(),
                fix_suggestion: Some("Set build directory to 'target/release'".to_string()),
            });
        }
        
        // Check Steam settings
        if self.selected_format == DistributionFormat::SteamDepot
            && self.config.steam_app_id.is_none()
        {
            self.validation.add_error(ValidationError {
                category: "Steam",
                message: "Steam App ID is required for depot builds".to_string(),
                fix_suggestion: Some("Enter your Steam App ID".to_string()),
            });
        }
        
        // Warnings
        if !self.build_options.run_tests_before_build {
            self.validation.add_warning(ValidationWarning {
                category: "Build",
                message: "Tests disabled - build may include bugs".to_string(),
            });
        }
        
        if self.profile == BuildProfile::Debug {
            self.validation.add_warning(ValidationWarning {
                category: "Build",
                message: "Debug build - not recommended for distribution".to_string(),
            });
        }
    }

    fn add_history_entry(&mut self, result: &DistributionResult) {
        let entry = BuildHistoryEntry {
            timestamp: std::time::SystemTime::now(),
            format: result.format,
            platform: self.platform,
            profile: self.profile,
            success: true,
            duration_secs: result.duration_secs,
            output_size_bytes: result.size_bytes,
            output_path: result.output_path.display().to_string(),
            version: self.config.version.clone(),
        };
        
        self.build_history.push_front(entry);
        if self.build_history.len() > self.max_history {
            self.build_history.pop_back();
        }
    }

    fn show_summary_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{} {}", 
                self.platform.icon(), 
                self.platform.name())).strong());
            ui.separator();
            ui.label(format!("{} {}", self.profile.icon(), self.profile.name()));
            ui.separator();
            ui.label(format!("📦 {}", self.selected_format.name()));
            
            if self.is_building {
                ui.separator();
                ui.spinner();
                ui.label(format!("{} {}", 
                    self.progress.current_step.icon(), 
                    self.progress.current_step.name()));
            }
        });
    }

    fn show_config_section(&mut self, ui: &mut Ui) {
        ui.heading("📦 Game Configuration");
        ui.separator();

        egui::Grid::new("config_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Game Name:");
                ui.text_edit_singleline(&mut self.config.game_name);
                ui.end_row();

                ui.label("Version:");
                ui.text_edit_singleline(&mut self.config.version);
                ui.end_row();

                ui.label("Publisher:");
                ui.text_edit_singleline(&mut self.config.publisher);
                ui.end_row();

                ui.label("Description:");
                ui.text_edit_singleline(&mut self.config.description);
                ui.end_row();
            });

        ui.add_space(10.0);

        // Steam settings (collapsible)
        ui.collapsing("🎮 Steam Settings", |ui| {
            egui::Grid::new("steam_grid")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label("App ID:");
                    let mut app_id_str = self
                        .config
                        .steam_app_id
                        .map(|id| id.to_string())
                        .unwrap_or_default();
                    if ui.text_edit_singleline(&mut app_id_str).changed() {
                        self.config.steam_app_id = app_id_str.parse().ok();
                    }
                    ui.end_row();

                    ui.label("Depot ID:");
                    let mut depot_id_str = self
                        .config
                        .steam_depot_id
                        .map(|id| id.to_string())
                        .unwrap_or_default();
                    if ui.text_edit_singleline(&mut depot_id_str).changed() {
                        self.config.steam_depot_id = depot_id_str.parse().ok();
                    }
                    ui.end_row();
                });
        });
    }

    fn show_platform_section(&mut self, ui: &mut Ui) {
        ui.add_space(15.0);
        ui.heading("🎯 Target Platform");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            for platform in TargetPlatform::all() {
                let selected = self.platform == *platform;
                let text = format!("{} {}", platform.icon(), platform.name());
                if ui.selectable_label(selected, text).clicked() {
                    self.platform = *platform;
                }
            }
        });
    }

    fn show_profile_section(&mut self, ui: &mut Ui) {
        ui.add_space(15.0);
        ui.heading("⚙️ Build Profile");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            for profile in BuildProfile::all() {
                let selected = self.profile == *profile;
                let text = format!("{} {}", profile.icon(), profile.name());
                if ui.selectable_label(selected, text)
                    .on_hover_text(profile.description())
                    .clicked() 
                {
                    self.profile = *profile;
                }
            }
        });
    }

    fn show_format_section(&mut self, ui: &mut Ui) {
        ui.add_space(15.0);
        ui.heading("🖥️ Distribution Format");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            let formats = [
                (DistributionFormat::WindowsPortable, "📁 Windows ZIP", "Portable ZIP archive"),
                (DistributionFormat::WindowsInstaller, "💿 Windows Installer", "NSIS installer (.exe)"),
                (DistributionFormat::MacOSBundle, "🍎 macOS Bundle", "Application bundle (.app)"),
                (DistributionFormat::MacOSDmg, "💿 macOS DMG", "Disk image (.dmg)"),
                (DistributionFormat::LinuxTarball, "🐧 Linux Tarball", "Compressed archive"),
                (DistributionFormat::LinuxAppImage, "📦 Linux AppImage", "Portable executable"),
                (DistributionFormat::SteamDepot, "🎮 Steam Depot", "Steam content depot"),
            ];

            for (format, label, description) in formats {
                let is_selected = self.selected_format == format;
                let button = egui::Button::new(label).fill(if is_selected {
                    Color32::from_rgb(60, 100, 160)
                } else {
                    Color32::from_rgb(50, 50, 55)
                });

                if ui.add(button).on_hover_text(description).clicked() {
                    self.selected_format = format;
                }
            }
        });
    }

    fn show_paths_section(&mut self, ui: &mut Ui) {
        ui.add_space(15.0);
        ui.heading("📂 Paths");
        ui.separator();

        // Handle pending directory selections from file dialogs
        if let Some(path) = self.pending_build_dir.take() {
            self.build_dir = path.to_string_lossy().to_string();
        }
        if let Some(path) = self.pending_output_dir.take() {
            self.output_dir = path.to_string_lossy().to_string();
        }

        egui::Grid::new("paths_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Build Directory:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.build_dir);
                    if ui.button("📁").on_hover_text("Browse for build directory").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select Build Directory")
                            .set_directory(&self.build_dir)
                            .pick_folder()
                        {
                            self.build_dir = path.to_string_lossy().to_string();
                        }
                    }
                });
                ui.end_row();

                ui.label("Output Directory:");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.output_dir);
                    if ui.button("📁").on_hover_text("Browse for output directory").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select Output Directory")
                            .set_directory(&self.output_dir)
                            .pick_folder()
                        {
                            self.output_dir = path.to_string_lossy().to_string();
                        }
                    }
                });
                ui.end_row();
            });
    }

    fn show_build_options_section(&mut self, ui: &mut Ui) {
        ui.collapsing("🔧 Build Options", |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.build_options.strip_symbols, "Strip symbols");
                ui.checkbox(&mut self.build_options.compress_assets, "Compress assets");
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.build_options.embed_runtime, "Embed runtime");
                ui.checkbox(&mut self.build_options.sign_binary, "Sign binary");
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.build_options.generate_checksums, "Generate checksums");
                ui.checkbox(&mut self.build_options.include_debug_symbols, "Include debug symbols");
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.build_options.run_tests_before_build, "Run tests first");
                ui.checkbox(&mut self.build_options.clean_before_build, "Clean before build");
            });
            
            if self.platform == TargetPlatform::MacOSArm64 || 
               self.platform == TargetPlatform::MacOSx64 ||
               self.platform == TargetPlatform::MacOSUniversal {
                ui.checkbox(&mut self.build_options.notarize_macos, "Notarize for macOS");
            }
        });
    }

    fn show_asset_options_section(&mut self, ui: &mut Ui) {
        ui.collapsing("🎨 Asset Options", |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.asset_options.compress_textures, "Compress textures");
                ui.checkbox(&mut self.asset_options.compress_audio, "Compress audio");
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.asset_options.compress_meshes, "Compress meshes");
                ui.checkbox(&mut self.asset_options.pack_into_archives, "Pack into archives");
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.asset_options.encrypt_assets, "Encrypt assets");
                ui.checkbox(&mut self.asset_options.generate_manifests, "Generate manifests");
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Texture Format:");
                egui::ComboBox::from_id_salt("texture_format")
                    .selected_text(self.asset_options.texture_format.name())
                    .show_ui(ui, |ui| {
                        for fmt in TextureFormat::all() {
                            if ui.selectable_label(self.asset_options.texture_format == *fmt, fmt.name()).clicked() {
                                self.asset_options.texture_format = *fmt;
                            }
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Audio Format:");
                egui::ComboBox::from_id_salt("audio_format")
                    .selected_text(self.asset_options.audio_format.name())
                    .show_ui(ui, |ui| {
                        for fmt in AudioFormat::all() {
                            if ui.selectable_label(self.asset_options.audio_format == *fmt, fmt.name()).clicked() {
                                self.asset_options.audio_format = *fmt;
                            }
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Max Texture Size:");
                ui.add(egui::Slider::new(&mut self.asset_options.max_texture_size, 512..=8192)
                    .suffix(" px"));
            });
        });
    }

    fn show_validation_section(&mut self, ui: &mut Ui) {
        if self.auto_validate {
            self.validate();
        }

        if !self.validation.errors.is_empty() || !self.validation.warnings.is_empty() {
            ui.collapsing(format!("⚠️ Validation ({} errors, {} warnings)", 
                self.validation.errors.len(), 
                self.validation.warnings.len()), |ui| {
                
                for error in &self.validation.errors {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("❌").color(Color32::RED));
                        ui.label(RichText::new(format!("[{}] {}", error.category, error.message))
                            .color(Color32::RED));
                    });
                    if let Some(fix) = &error.fix_suggestion {
                        ui.label(RichText::new(format!("   💡 {}", fix)).weak().small());
                    }
                }
                
                for warning in &self.validation.warnings {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚠️").color(Color32::YELLOW));
                        ui.label(RichText::new(format!("[{}] {}", warning.category, warning.message))
                            .color(Color32::YELLOW));
                    });
                }
            });
        }
    }

    fn show_progress_section(&mut self, ui: &mut Ui) {
        if self.is_building {
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.heading(format!("{} Build Progress", self.progress.current_step.icon()));
                
                // Overall progress bar
                ui.add(egui::ProgressBar::new(self.progress.overall_progress)
                    .text(format!("{:.0}%", self.progress.overall_progress * 100.0)));
                
                // Current step
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(&self.progress.status_message);
                });
                
                // Elapsed time
                ui.label(format!("⏱️ Elapsed: {:.1}s", self.progress.elapsed_secs()));
                
                // Step progress
                if !self.progress.current_step.is_terminal() {
                    ui.add(egui::ProgressBar::new(self.progress.step_progress)
                        .text(self.progress.current_step.name()));
                }
            });
        }
    }

    fn show_build_section(&mut self, ui: &mut Ui) {
        ui.add_space(20.0);
        ui.separator();

        ui.horizontal(|ui| {
            let can_build = !self.is_building && self.validation.valid;
            
            let build_button = egui::Button::new(RichText::new("🚀 Build Distribution").size(16.0))
                .fill(if can_build { Color32::from_rgb(40, 120, 80) } else { Color32::from_rgb(80, 80, 80) })
                .min_size(egui::vec2(200.0, 40.0));

            if ui.add_enabled(can_build, build_button).clicked() {
                self.start_build();
            }

            if self.is_building && ui.button("❌ Cancel").clicked() {
                self.is_building = false;
                self.progress.set_step(BuildStep::Failed, "Build cancelled by user");
            }

            if ui.button("🔄 Validate").clicked() {
                self.validate();
            }
        });

        // Progress section
        if self.show_progress {
            self.show_progress_section(ui);
        }

        // Show last result
        if let Some(result) = &self.last_result {
            ui.add_space(10.0);
            match result {
                Ok(dist) => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("✅ Success!").color(Color32::GREEN));
                        ui.label(format!(
                            "Created {} ({:.2} MB in {:.1}s)",
                            dist.output_path.display(),
                            dist.size_bytes as f64 / 1024.0 / 1024.0,
                            dist.duration_secs
                        ));
                    });
                }
                Err(e) => {
                    ui.label(RichText::new(format!("❌ Error: {}", e)).color(Color32::RED));
                }
            }
        }
    }

    fn show_history_section(&mut self, ui: &mut Ui) {
        if !self.build_history.is_empty() {
            ui.collapsing(format!("📋 Build History ({})", self.build_history.len()), |ui| {
                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    for entry in self.build_history.iter().take(10) {
                        let status_icon = if entry.success { "✅" } else { "❌" };
                        let text = format!(
                            "{} v{} {} {} - {} ({:.1}s)",
                            status_icon,
                            entry.version,
                            entry.platform.icon(),
                            entry.format.name(),
                            entry.size_string(),
                            entry.duration_secs
                        );
                        
                        ui.horizontal(|ui| {
                            ui.label(text);
                            ui.label(RichText::new(entry.age_string()).weak().small());
                        });
                    }
                });
                
                if ui.button("🗑️ Clear History").clicked() {
                    self.build_history.clear();
                }
            });
        }
    }

    fn start_build(&mut self) {
        self.is_building = true;
        self.progress.start();
        self.progress.set_step(BuildStep::CompilingCode, "Building distribution...");

        let builder = DistributionBuilder::new(self.config.clone(), self.selected_format)
            .build_dir(&self.build_dir)
            .output_dir(&self.output_dir);

        match builder.build() {
            Ok(result) => {
                self.add_history_entry(&result);
                self.last_result = Some(Ok(result));
                self.progress.set_step(BuildStep::Complete, "Build successful!");
            }
            Err(e) => {
                self.progress.set_step(BuildStep::Failed, format!("Build failed: {}", e));
                self.last_result = Some(Err(e.to_string()));
            }
        }
        
        self.is_building = false;
    }

    // Accessors
    pub fn config(&self) -> &DistributionConfig {
        &self.config
    }

    pub fn profile(&self) -> BuildProfile {
        self.profile
    }

    pub fn platform(&self) -> TargetPlatform {
        self.platform
    }

    pub fn is_building(&self) -> bool {
        self.is_building
    }

    pub fn validation(&self) -> &ValidationResult {
        &self.validation
    }

    pub fn build_history(&self) -> &VecDeque<BuildHistoryEntry> {
        &self.build_history
    }
}

impl Panel for DistributionPanel {
    fn name(&self) -> &str {
        "Distribution"
    }

    fn show(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_summary_bar(ui);
            ui.separator();
            
            self.show_config_section(ui);
            self.show_platform_section(ui);
            self.show_profile_section(ui);
            self.show_format_section(ui);
            self.show_paths_section(ui);
            self.show_build_options_section(ui);
            self.show_asset_options_section(ui);
            
            if self.show_validation {
                self.show_validation_section(ui);
            }
            
            self.show_build_section(ui);
            
            if self.show_history {
                self.show_history_section(ui);
            }
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════
    // BUILD PROFILE TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_build_profile_name() {
        assert_eq!(BuildProfile::Debug.name(), "Debug");
        assert_eq!(BuildProfile::Release.name(), "Release");
        assert_eq!(BuildProfile::ReleaseOptimized.name(), "Release (Optimized)");
        assert_eq!(BuildProfile::MinSize.name(), "Minimum Size");
    }

    #[test]
    fn test_build_profile_description() {
        assert!(!BuildProfile::Debug.description().is_empty());
        assert!(!BuildProfile::Release.description().is_empty());
    }

    #[test]
    fn test_build_profile_icon() {
        assert!(!BuildProfile::Debug.icon().is_empty());
        assert!(!BuildProfile::Release.icon().is_empty());
    }

    #[test]
    fn test_build_profile_cargo_profile() {
        assert_eq!(BuildProfile::Debug.cargo_profile(), "debug");
        assert_eq!(BuildProfile::Release.cargo_profile(), "release");
    }

    #[test]
    fn test_build_profile_all() {
        let all = BuildProfile::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_build_profile_default() {
        assert_eq!(BuildProfile::default(), BuildProfile::Release);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // TARGET PLATFORM TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_target_platform_name() {
        assert_eq!(TargetPlatform::Native.name(), "Native");
        assert_eq!(TargetPlatform::Windows64.name(), "Windows x64");
        assert_eq!(TargetPlatform::MacOSArm64.name(), "macOS ARM64");
        assert_eq!(TargetPlatform::LinuxX64.name(), "Linux x64");
    }

    #[test]
    fn test_target_platform_icon() {
        assert!(!TargetPlatform::Native.icon().is_empty());
        assert!(!TargetPlatform::Windows64.icon().is_empty());
    }

    #[test]
    fn test_target_platform_rust_target() {
        assert!(TargetPlatform::Native.rust_target().is_none());
        assert_eq!(TargetPlatform::Windows64.rust_target(), Some("x86_64-pc-windows-msvc"));
        assert_eq!(TargetPlatform::LinuxX64.rust_target(), Some("x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn test_target_platform_all() {
        let all = TargetPlatform::all();
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn test_target_platform_default() {
        assert_eq!(TargetPlatform::default(), TargetPlatform::Native);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BUILD OPTIONS TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_build_options_default() {
        let options = BuildOptions::default();
        assert!(options.strip_symbols);
        assert!(options.compress_assets);
        assert!(options.run_tests_before_build);
        assert!(!options.clean_before_build);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ASSET OPTIONS TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_asset_options_default() {
        let options = AssetOptions::default();
        assert!(options.compress_textures);
        assert!(options.compress_audio);
        assert_eq!(options.max_texture_size, 4096);
    }

    #[test]
    fn test_texture_format_name() {
        assert_eq!(TextureFormat::BC7.name(), "BC7 (DX11+)");
        assert_eq!(TextureFormat::ASTC.name(), "ASTC (Mobile)");
    }

    #[test]
    fn test_texture_format_all() {
        let all = TextureFormat::all();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_texture_format_default() {
        assert_eq!(TextureFormat::default(), TextureFormat::BC7);
    }

    #[test]
    fn test_audio_format_name() {
        assert_eq!(AudioFormat::Vorbis.name(), "Vorbis (OGG)");
        assert_eq!(AudioFormat::Opus.name(), "Opus");
    }

    #[test]
    fn test_audio_format_all() {
        let all = AudioFormat::all();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_audio_format_default() {
        assert_eq!(AudioFormat::default(), AudioFormat::Vorbis);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BUILD STEP TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_build_step_name() {
        assert_eq!(BuildStep::Preparing.name(), "Preparing");
        assert_eq!(BuildStep::CompilingCode.name(), "Compiling");
        assert_eq!(BuildStep::Complete.name(), "Complete");
    }

    #[test]
    fn test_build_step_icon() {
        assert!(!BuildStep::Preparing.icon().is_empty());
        assert!(!BuildStep::Complete.icon().is_empty());
    }

    #[test]
    fn test_build_step_is_terminal() {
        assert!(!BuildStep::Preparing.is_terminal());
        assert!(!BuildStep::CompilingCode.is_terminal());
        assert!(BuildStep::Complete.is_terminal());
        assert!(BuildStep::Failed.is_terminal());
    }

    #[test]
    fn test_build_step_all() {
        let all = BuildStep::all_steps();
        assert!(all.len() >= 10);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BUILD PROGRESS TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_build_progress_default() {
        let progress = BuildProgress::default();
        assert_eq!(progress.current_step, BuildStep::Preparing);
        assert_eq!(progress.overall_progress, 0.0);
    }

    #[test]
    fn test_build_progress_start() {
        let mut progress = BuildProgress::default();
        progress.start();
        assert!(progress.start_time.is_some());
        assert!(progress.warnings.is_empty());
        assert!(progress.errors.is_empty());
    }

    #[test]
    fn test_build_progress_set_step() {
        let mut progress = BuildProgress::default();
        progress.set_step(BuildStep::CompilingCode, "Compiling main.rs");
        assert_eq!(progress.current_step, BuildStep::CompilingCode);
        assert_eq!(progress.status_message, "Compiling main.rs");
    }

    #[test]
    fn test_build_progress_add_warning() {
        let mut progress = BuildProgress::default();
        progress.add_warning("Test warning");
        assert_eq!(progress.warnings.len(), 1);
    }

    #[test]
    fn test_build_progress_add_error() {
        let mut progress = BuildProgress::default();
        progress.add_error("Test error");
        assert_eq!(progress.errors.len(), 1);
    }

    #[test]
    fn test_build_progress_elapsed() {
        let mut progress = BuildProgress::default();
        progress.start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(progress.elapsed_secs() >= 0.01);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // BUILD HISTORY TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_build_history_entry_age_string() {
        let entry = BuildHistoryEntry {
            timestamp: std::time::SystemTime::now(),
            format: DistributionFormat::WindowsPortable,
            platform: TargetPlatform::Windows64,
            profile: BuildProfile::Release,
            success: true,
            duration_secs: 10.0,
            output_size_bytes: 1024 * 1024 * 50,
            output_path: "test.zip".to_string(),
            version: "1.0.0".to_string(),
        };
        assert!(entry.age_string().contains("s ago"));
    }

    #[test]
    fn test_build_history_entry_size_string_kb() {
        let entry = BuildHistoryEntry {
            timestamp: std::time::SystemTime::now(),
            format: DistributionFormat::WindowsPortable,
            platform: TargetPlatform::Native,
            profile: BuildProfile::Debug,
            success: true,
            duration_secs: 1.0,
            output_size_bytes: 512 * 1024,
            output_path: "test.zip".to_string(),
            version: "1.0.0".to_string(),
        };
        assert!(entry.size_string().contains("KB"));
    }

    #[test]
    fn test_build_history_entry_size_string_mb() {
        let entry = BuildHistoryEntry {
            timestamp: std::time::SystemTime::now(),
            format: DistributionFormat::WindowsPortable,
            platform: TargetPlatform::Native,
            profile: BuildProfile::Release,
            success: true,
            duration_secs: 5.0,
            output_size_bytes: 50 * 1024 * 1024,
            output_path: "test.zip".to_string(),
            version: "1.0.0".to_string(),
        };
        assert!(entry.size_string().contains("MB"));
    }

    #[test]
    fn test_build_history_entry_size_string_gb() {
        let entry = BuildHistoryEntry {
            timestamp: std::time::SystemTime::now(),
            format: DistributionFormat::WindowsPortable,
            platform: TargetPlatform::Native,
            profile: BuildProfile::Release,
            success: true,
            duration_secs: 60.0,
            output_size_bytes: 2 * 1024 * 1024 * 1024,
            output_path: "test.zip".to_string(),
            version: "1.0.0".to_string(),
        };
        assert!(entry.size_string().contains("GB"));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // VALIDATION TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validation_result_default() {
        let result = ValidationResult::default();
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut result = ValidationResult::default();
        result.add_error(ValidationError {
            category: "Test",
            message: "Test error".to_string(),
            fix_suggestion: None,
        });
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_validation_result_add_warning() {
        let mut result = ValidationResult::default();
        result.add_warning(ValidationWarning {
            category: "Test",
            message: "Test warning".to_string(),
        });
        assert!(result.valid); // Warnings don't invalidate
        assert_eq!(result.warnings.len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // DISTRIBUTION PANEL TESTS
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_distribution_panel_default() {
        let panel = DistributionPanel::new();
        assert_eq!(panel.name(), "Distribution");
        assert!(!panel.is_building);
        assert!(panel.last_result.is_none());
    }

    #[test]
    fn test_distribution_panel_config() {
        let panel = DistributionPanel::new();
        assert!(!panel.config().game_name.is_empty());
    }

    #[test]
    fn test_distribution_panel_profile() {
        let panel = DistributionPanel::new();
        assert_eq!(panel.profile(), BuildProfile::Release);
    }

    #[test]
    fn test_distribution_panel_platform() {
        let panel = DistributionPanel::new();
        assert_eq!(panel.platform(), TargetPlatform::Native);
    }

    #[test]
    fn test_distribution_panel_is_building() {
        let panel = DistributionPanel::new();
        assert!(!panel.is_building());
    }

    #[test]
    fn test_distribution_panel_validation() {
        let panel = DistributionPanel::new();
        let _ = panel.validation();
    }

    #[test]
    fn test_distribution_panel_build_history() {
        let panel = DistributionPanel::new();
        assert!(panel.build_history().is_empty());
    }

    #[test]
    fn test_format_selection() {
        let mut panel = DistributionPanel::new();
        assert_eq!(panel.selected_format, DistributionFormat::WindowsPortable);

        panel.selected_format = DistributionFormat::SteamDepot;
        assert_eq!(panel.selected_format, DistributionFormat::SteamDepot);
    }

    #[test]
    fn test_distribution_panel_validate_empty_name() {
        let mut panel = DistributionPanel::new();
        panel.config.game_name = String::new();
        panel.validate();
        assert!(!panel.validation.valid);
        assert!(panel.validation.errors.iter().any(|e| e.message.contains("Game name")));
    }

    #[test]
    fn test_distribution_panel_validate_empty_version() {
        let mut panel = DistributionPanel::new();
        panel.config.version = String::new();
        panel.validate();
        assert!(!panel.validation.valid);
        assert!(panel.validation.errors.iter().any(|e| e.message.contains("Version")));
    }

    #[test]
    fn test_distribution_panel_validate_steam_no_app_id() {
        let mut panel = DistributionPanel::new();
        panel.selected_format = DistributionFormat::SteamDepot;
        panel.config.steam_app_id = None;
        panel.validate();
        assert!(!panel.validation.valid);
        assert!(panel.validation.errors.iter().any(|e| e.message.contains("Steam App ID")));
    }

    #[test]
    fn test_distribution_panel_validate_debug_warning() {
        let mut panel = DistributionPanel::new();
        panel.profile = BuildProfile::Debug;
        panel.validate();
        assert!(panel.validation.warnings.iter().any(|w| w.message.contains("Debug")));
    }

    #[test]
    fn test_distribution_panel_validate_tests_disabled_warning() {
        let mut panel = DistributionPanel::new();
        panel.build_options.run_tests_before_build = false;
        panel.validate();
        assert!(panel.validation.warnings.iter().any(|w| w.message.contains("Tests disabled")));
    }
}
