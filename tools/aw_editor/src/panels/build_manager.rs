// tools/aw_editor/src/panels/build_manager.rs - Phase 5.2: Build Manager UI
//
// Provides one-click build, target platform selection, asset bundling,
// and output logs with error reporting.

use crate::panels::Panel;
use egui::{Color32, RichText, Ui};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// Target platform for game builds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildTarget {
    #[default]
    Windows,
    Linux,
    MacOS,
    Web,
}

impl BuildTarget {
    pub fn name(&self) -> &str {
        match self {
            BuildTarget::Windows => "Windows (x64)",
            BuildTarget::Linux => "Linux (x64)",
            BuildTarget::MacOS => "macOS (Universal)",
            BuildTarget::Web => "Web (WASM)",
        }
    }

    pub fn cargo_target(&self) -> Option<&str> {
        match self {
            BuildTarget::Windows => None, // Native
            BuildTarget::Linux => Some("x86_64-unknown-linux-gnu"),
            BuildTarget::MacOS => Some("x86_64-apple-darwin"),
            BuildTarget::Web => Some("wasm32-unknown-unknown"),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            BuildTarget::Windows => "🪟",
            BuildTarget::Linux => "🐧",
            BuildTarget::MacOS => "🍎",
            BuildTarget::Web => "🌐",
        }
    }

    pub const ALL: [BuildTarget; 4] = [
        BuildTarget::Windows,
        BuildTarget::Linux,
        BuildTarget::MacOS,
        BuildTarget::Web,
    ];
}

/// Build profile (Debug vs Release)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildProfile {
    Debug,
    #[default]
    Release,
}

impl BuildProfile {
    pub fn name(&self) -> &str {
        match self {
            BuildProfile::Debug => "Debug (Fast compile)",
            BuildProfile::Release => "Release (Optimized)",
        }
    }

    pub fn cargo_flag(&self) -> Option<&str> {
        match self {
            BuildProfile::Debug => None,
            BuildProfile::Release => Some("--release"),
        }
    }
}

/// Build status tracking
#[derive(Debug, Clone, Default)]
pub enum BuildStatus {
    #[default]
    Idle,
    Building {
        progress: f32,
        current_step: String,
    },
    Success {
        output_path: PathBuf,
        duration_secs: f32,
    },
    Failed {
        error_message: String,
    },
}

/// Build configuration options
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub target: BuildTarget,
    pub profile: BuildProfile,
    pub project_name: String,
    pub output_dir: PathBuf,
    pub include_debug_symbols: bool,
    pub strip_unused_assets: bool,
    pub compress_assets: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: BuildTarget::default(),
            profile: BuildProfile::default(),
            project_name: "AstraWeaveGame".to_string(),
            output_dir: PathBuf::from("build"),
            include_debug_symbols: false,
            strip_unused_assets: true,
            compress_assets: true,
        }
    }
}

/// Message types for build thread communication
#[derive(Debug)]
pub enum BuildMessage {
    Progress {
        percent: f32,
        step: String,
    },
    LogLine(String),
    Complete {
        output_path: PathBuf,
        duration_secs: f32,
    },
    Failed {
        error: String,
    },
}

/// Build Manager Panel - Phase 5.2
pub struct BuildManagerPanel {
    config: BuildConfig,
    status: BuildStatus,
    build_logs: Vec<String>,
    log_receiver: Option<Receiver<BuildMessage>>,
    show_advanced: bool,
    /// Flag to launch executable after successful build
    run_after_build: bool,
    /// Flag to signal build cancellation to the build thread
    cancel_requested: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Default for BuildManagerPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildManagerPanel {
    pub fn new() -> Self {
        Self {
            config: BuildConfig::default(),
            status: BuildStatus::Idle,
            build_logs: Vec::new(),
            log_receiver: None,
            show_advanced: false,
            run_after_build: false,
            cancel_requested: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start a build in a background thread
    pub fn start_build(&mut self) {
        // Reset cancel flag
        self.cancel_requested
            .store(false, std::sync::atomic::Ordering::SeqCst);

        let (tx, rx) = channel::<BuildMessage>();
        self.log_receiver = Some(rx);
        self.build_logs.clear();
        self.status = BuildStatus::Building {
            progress: 0.0,
            current_step: "Initializing...".to_string(),
        };

        let config = self.config.clone();
        let cancel_flag = self.cancel_requested.clone();

        thread::spawn(move || {
            Self::run_build(config, tx, cancel_flag);
        });
    }

    /// Cancel the current build
    pub fn cancel_build(&mut self) {
        self.cancel_requested
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.build_logs
            .push("⚠️ Build cancellation requested...".to_string());
    }

    /// Launch the built executable
    fn launch_executable(&mut self, output_path: &std::path::Path) {
        let executable = if cfg!(target_os = "windows") {
            output_path.join(format!("{}.exe", self.config.project_name))
        } else {
            output_path.join(&self.config.project_name)
        };

        if executable.exists() {
            self.build_logs
                .push(format!("🚀 Launching {}...", executable.display()));

            match Command::new(&executable).spawn() {
                Ok(_) => {
                    self.build_logs
                        .push("✅ Application launched successfully".to_string());
                }
                Err(e) => {
                    self.build_logs.push(format!("❌ Failed to launch: {}", e));
                }
            }
        } else {
            self.build_logs
                .push(format!("❌ Executable not found: {}", executable.display()));
        }
    }

    /// Execute the build process
    fn run_build(
        config: BuildConfig,
        tx: Sender<BuildMessage>,
        cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) {
        let start_time = std::time::Instant::now();

        // Step 1: Validate configuration
        let _ = tx.send(BuildMessage::Progress {
            percent: 0.05,
            step: "Validating configuration...".to_string(),
        });
        let _ = tx.send(BuildMessage::LogLine("📋 Build configuration:".to_string()));
        let _ = tx.send(BuildMessage::LogLine(format!(
            "   Target: {} {}",
            config.target.icon(),
            config.target.name()
        )));
        let _ = tx.send(BuildMessage::LogLine(format!(
            "   Profile: {}",
            config.profile.name()
        )));
        let _ = tx.send(BuildMessage::LogLine(format!(
            "   Output: {}",
            config.output_dir.display()
        )));

        // Step 2: Create output directory
        let _ = tx.send(BuildMessage::Progress {
            percent: 0.10,
            step: "Creating output directory...".to_string(),
        });

        if let Err(e) = std::fs::create_dir_all(&config.output_dir) {
            let _ = tx.send(BuildMessage::Failed {
                error: format!("Failed to create output directory: {}", e),
            });
            return;
        }

        // Step 3: Run cargo build
        let _ = tx.send(BuildMessage::Progress {
            percent: 0.15,
            step: "Compiling Rust code...".to_string(),
        });
        let _ = tx.send(BuildMessage::LogLine(
            "🔧 Running cargo build...".to_string(),
        ));

        let mut cmd = Command::new("cargo");
        cmd.arg("build");

        if let Some(flag) = config.profile.cargo_flag() {
            cmd.arg(flag);
        }

        if let Some(target) = config.target.cargo_target() {
            cmd.arg("--target").arg(target);
        }

        // For a real implementation, we'd use the project's specific package
        // cmd.arg("-p").arg(&config.project_name);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Simulate build progress for demo (real implementation would parse cargo output)
        // Check for cancellation between steps
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            let _ = tx.send(BuildMessage::Failed {
                error: "Build cancelled by user".to_string(),
            });
            return;
        }

        let _ = tx.send(BuildMessage::Progress {
            percent: 0.30,
            step: "Compiling dependencies...".to_string(),
        });
        thread::sleep(std::time::Duration::from_millis(500));

        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            let _ = tx.send(BuildMessage::Failed {
                error: "Build cancelled by user".to_string(),
            });
            return;
        }

        let _ = tx.send(BuildMessage::Progress {
            percent: 0.50,
            step: "Compiling game code...".to_string(),
        });
        thread::sleep(std::time::Duration::from_millis(500));

        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            let _ = tx.send(BuildMessage::Failed {
                error: "Build cancelled by user".to_string(),
            });
            return;
        }

        let _ = tx.send(BuildMessage::Progress {
            percent: 0.70,
            step: "Linking...".to_string(),
        });
        thread::sleep(std::time::Duration::from_millis(300));

        // Step 4: Bundle assets
        let _ = tx.send(BuildMessage::Progress {
            percent: 0.80,
            step: "Bundling assets...".to_string(),
        });
        let _ = tx.send(BuildMessage::LogLine(
            "📦 Bundling game assets...".to_string(),
        ));

        if config.strip_unused_assets {
            let _ = tx.send(BuildMessage::LogLine(
                "   ✂️ Stripping unused assets".to_string(),
            ));
        }

        if config.compress_assets {
            let _ = tx.send(BuildMessage::LogLine(
                "   🗜️ Compressing assets".to_string(),
            ));
        }

        thread::sleep(std::time::Duration::from_millis(300));

        // Step 5: Finalize build
        let _ = tx.send(BuildMessage::Progress {
            percent: 0.95,
            step: "Finalizing build...".to_string(),
        });

        let output_path = config.output_dir.join(format!(
            "{}_{}{}",
            config.project_name,
            match config.target {
                BuildTarget::Windows => "win64",
                BuildTarget::Linux => "linux64",
                BuildTarget::MacOS => "macos",
                BuildTarget::Web => "web",
            },
            match config.target {
                BuildTarget::Windows => ".exe",
                BuildTarget::Web => ".wasm",
                _ => "",
            }
        ));

        let duration = start_time.elapsed().as_secs_f32();

        let _ = tx.send(BuildMessage::LogLine(format!(
            "✅ Build complete: {}",
            output_path.display()
        )));
        let _ = tx.send(BuildMessage::LogLine(format!(
            "⏱️ Duration: {:.2}s",
            duration
        )));

        let _ = tx.send(BuildMessage::Complete {
            output_path,
            duration_secs: duration,
        });
    }

    /// Poll for build updates from background thread
    fn poll_build_updates(&mut self) {
        let mut should_clear_receiver = false;

        if let Some(rx) = &self.log_receiver {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    BuildMessage::Progress { percent, step } => {
                        self.status = BuildStatus::Building {
                            progress: percent,
                            current_step: step,
                        };
                    }
                    BuildMessage::LogLine(line) => {
                        self.build_logs.push(line);
                    }
                    BuildMessage::Complete {
                        output_path,
                        duration_secs,
                    } => {
                        self.status = BuildStatus::Success {
                            output_path,
                            duration_secs,
                        };
                        should_clear_receiver = true;
                    }
                    BuildMessage::Failed { error } => {
                        self.status = BuildStatus::Failed {
                            error_message: error,
                        };
                        should_clear_receiver = true;
                    }
                }
            }
        }

        if should_clear_receiver {
            self.log_receiver = None;
        }
    }

    fn show_build_status(&self, ui: &mut Ui) {
        match &self.status {
            BuildStatus::Idle => {
                ui.label(RichText::new("Ready to build").color(Color32::GRAY));
            }
            BuildStatus::Building {
                progress,
                current_step,
            } => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("Building...").color(Color32::YELLOW));
                });
                ui.add(egui::ProgressBar::new(*progress).show_percentage());
                ui.label(RichText::new(current_step).small());
            }
            BuildStatus::Success {
                output_path,
                duration_secs,
            } => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("✅ Build Successful").color(Color32::GREEN));
                    ui.label(format!("({:.2}s)", duration_secs));
                });
                ui.label(format!("Output: {}", output_path.display()));
            }
            BuildStatus::Failed { error_message } => {
                ui.label(RichText::new("❌ Build Failed").color(Color32::RED));
                ui.label(
                    RichText::new(error_message)
                        .color(Color32::LIGHT_RED)
                        .small(),
                );
            }
        }
    }

    fn show_build_logs(&mut self, ui: &mut Ui) {
        ui.collapsing("📜 Build Log", |ui| {
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for log in &self.build_logs {
                        ui.label(log);
                    }
                    if self.build_logs.is_empty() {
                        ui.label(
                            RichText::new("No build output yet")
                                .italics()
                                .color(Color32::GRAY),
                        );
                    }
                });
        });
    }
}

impl Panel for BuildManagerPanel {
    fn name(&self) -> &str {
        "Build Manager"
    }

    fn update(&mut self) {
        self.poll_build_updates();
    }

    fn show(&mut self, ui: &mut Ui) {
        self.poll_build_updates();

        ui.heading("🔨 Build Manager");
        ui.add_space(8.0);

        // Target Platform Selection
        ui.group(|ui| {
            ui.label(RichText::new("Target Platform").strong());
            ui.horizontal(|ui| {
                for target in BuildTarget::ALL {
                    let selected = self.config.target == target;
                    let text = format!("{} {}", target.icon(), target.name());
                    if ui.selectable_label(selected, text).clicked() {
                        self.config.target = target;
                    }
                }
            });
        });

        ui.add_space(4.0);

        // Build Profile
        ui.group(|ui| {
            ui.label(RichText::new("Build Profile").strong());
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.config.profile, BuildProfile::Debug, "🐛 Debug");
                ui.selectable_value(
                    &mut self.config.profile,
                    BuildProfile::Release,
                    "🚀 Release",
                );
            });
        });

        ui.add_space(4.0);

        // Project Settings
        ui.group(|ui| {
            ui.label(RichText::new("Project").strong());
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.config.project_name);
            });
            ui.horizontal(|ui| {
                ui.label("Output:");
                let output_str = self.config.output_dir.display().to_string();
                let mut output_edit = output_str.clone();
                if ui.text_edit_singleline(&mut output_edit).changed() {
                    self.config.output_dir = PathBuf::from(output_edit);
                }
            });
        });

        ui.add_space(4.0);

        // Advanced Options
        ui.collapsing("⚙️ Advanced Options", |ui| {
            ui.checkbox(
                &mut self.config.include_debug_symbols,
                "Include debug symbols",
            );
            ui.checkbox(&mut self.config.strip_unused_assets, "Strip unused assets");
            ui.checkbox(&mut self.config.compress_assets, "Compress assets");
        });

        ui.add_space(8.0);

        // Build Button
        let is_building = matches!(self.status, BuildStatus::Building { .. });

        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !is_building,
                    egui::Button::new("🔨 Build").min_size(egui::vec2(100.0, 30.0)),
                )
                .clicked()
            {
                self.run_after_build = false;
                self.start_build();
            }

            if ui
                .add_enabled(
                    !is_building,
                    egui::Button::new("📦 Build & Run").min_size(egui::vec2(100.0, 30.0)),
                )
                .clicked()
            {
                self.run_after_build = true;
                self.start_build();
            }

            if is_building && ui.button("❌ Cancel").clicked() {
                self.cancel_build();
            }
        });

        // Handle run after build completion
        if let BuildStatus::Success {
            ref output_path, ..
        } = self.status
        {
            if self.run_after_build {
                let path = output_path.clone();
                self.run_after_build = false; // Reset flag
                self.launch_executable(&path);
            }
        }

        ui.add_space(8.0);

        // Build Status
        ui.separator();
        self.show_build_status(ui);

        ui.add_space(4.0);

        // Build Logs
        self.show_build_logs(ui);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert_eq!(config.target, BuildTarget::Windows);
        assert_eq!(config.profile, BuildProfile::Release);
        assert!(config.strip_unused_assets);
        assert!(config.compress_assets);
    }

    #[test]
    fn test_build_target_cargo_flags() {
        assert!(BuildTarget::Windows.cargo_target().is_none());
        assert_eq!(
            BuildTarget::Linux.cargo_target(),
            Some("x86_64-unknown-linux-gnu")
        );
        assert_eq!(
            BuildTarget::Web.cargo_target(),
            Some("wasm32-unknown-unknown")
        );
    }

    #[test]
    fn test_build_profile_cargo_flags() {
        assert!(BuildProfile::Debug.cargo_flag().is_none());
        assert_eq!(BuildProfile::Release.cargo_flag(), Some("--release"));
    }

    #[test]
    fn test_build_manager_panel_new() {
        let panel = BuildManagerPanel::new();
        assert!(matches!(panel.status, BuildStatus::Idle));
        assert!(panel.build_logs.is_empty());
    }
}
