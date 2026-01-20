//! Polish panel for the editor UI
//!
//! Provides a comprehensive GUI for configuring splash screens, loading screens,
//! save/load settings, credits, UI polish, achievements, and game feel.

use egui::{Color32, RichText, Ui};
use std::time::Duration;

use crate::panels::Panel;
use crate::polish::{LoadingScreen, LoadingStyle, SaveConfig, SplashSequence};

/// Panel for configuring game polish features
pub struct PolishPanel {
    splash_sequence: SplashSequence,
    loading_screen: LoadingScreen,
    save_config: SaveConfig,
    credits_config: CreditsConfig,
    ui_polish: UiPolishSettings,
    achievements: AchievementSettings,
    game_feel: GameFeelSettings,
    active_tab: PolishTab,
    new_tip: String,
    new_credit_name: String,
    new_credit_role: String,
    new_achievement_id: String,
    new_achievement_name: String,
    new_achievement_desc: String,
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
enum PolishTab {
    #[default]
    Splash,
    Loading,
    SaveLoad,
    Credits,
    UiPolish,
    Achievements,
    GameFeel,
}

/// Credits configuration
#[derive(Debug, Clone)]
pub struct CreditsConfig {
    /// Game title
    pub game_title: String,
    /// Subtitle/tagline
    pub subtitle: String,
    /// Credits entries (name, role)
    pub entries: Vec<(String, String)>,
    /// Scroll speed (pixels per second)
    pub scroll_speed: f32,
    /// Background color
    pub background_color: [f32; 4],
    /// Text color
    pub text_color: [f32; 4],
    /// Enable music during credits
    pub music_enabled: bool,
    /// Music track path
    pub music_track: String,
}

impl Default for CreditsConfig {
    fn default() -> Self {
        Self {
            game_title: "Your Game".to_string(),
            subtitle: "Made with AstraWeave".to_string(),
            entries: vec![
                ("Studio Name".to_string(), "Developer".to_string()),
                ("Engine Team".to_string(), "AstraWeave Engine".to_string()),
            ],
            scroll_speed: 50.0,
            background_color: [0.0, 0.0, 0.0, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            music_enabled: true,
            music_track: "credits_music.ogg".to_string(),
        }
    }
}

/// UI polish settings
#[derive(Debug, Clone)]
pub struct UiPolishSettings {
    /// Enable UI animations
    pub animations_enabled: bool,
    /// Animation speed multiplier
    pub animation_speed: f32,
    /// Enable UI sounds
    pub sounds_enabled: bool,
    /// UI click sound
    pub click_sound: String,
    /// UI hover sound
    pub hover_sound: String,
    /// Enable screen transitions
    pub transitions_enabled: bool,
    /// Transition duration (seconds)
    pub transition_duration: f32,
    /// Transition style
    pub transition_style: TransitionStyle,
    /// Enable button hover effects
    pub button_hover_enabled: bool,
    /// Enable tooltips
    pub tooltips_enabled: bool,
    /// Tooltip delay (seconds)
    pub tooltip_delay: f32,
    /// Enable notification system
    pub notifications_enabled: bool,
    /// Notification duration (seconds)
    pub notification_duration: f32,
}

impl Default for UiPolishSettings {
    fn default() -> Self {
        Self {
            animations_enabled: true,
            animation_speed: 1.0,
            sounds_enabled: true,
            click_sound: "ui_click.wav".to_string(),
            hover_sound: "ui_hover.wav".to_string(),
            transitions_enabled: true,
            transition_duration: 0.3,
            transition_style: TransitionStyle::Fade,
            button_hover_enabled: true,
            tooltips_enabled: true,
            tooltip_delay: 0.5,
            notifications_enabled: true,
            notification_duration: 3.0,
        }
    }
}

/// Screen transition style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionStyle {
    /// Fade in/out
    Fade,
    /// Slide from right
    SlideRight,
    /// Slide from left
    SlideLeft,
    /// Slide from top
    SlideTop,
    /// Slide from bottom
    SlideBottom,
    /// Dissolve effect
    Dissolve,
    /// Instant (no transition)
    Instant,
}

impl std::fmt::Display for TransitionStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl Default for TransitionStyle {
    fn default() -> Self {
        Self::Fade
    }
}

impl TransitionStyle {
    /// Returns the human-readable name for this transition style
    pub fn name(&self) -> &'static str {
        match self {
            Self::Fade => "Fade",
            Self::SlideRight => "Slide Right",
            Self::SlideLeft => "Slide Left",
            Self::SlideTop => "Slide Top",
            Self::SlideBottom => "Slide Bottom",
            Self::Dissolve => "Dissolve",
            Self::Instant => "Instant",
        }
    }

    /// Returns the icon for this transition style
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Fade => "🌅",
            Self::SlideRight => "➡️",
            Self::SlideLeft => "⬅️",
            Self::SlideTop => "⬆️",
            Self::SlideBottom => "⬇️",
            Self::Dissolve => "✨",
            Self::Instant => "⚡",
        }
    }

    /// Returns all available transition styles
    pub fn all() -> &'static [TransitionStyle] {
        &[
            TransitionStyle::Fade,
            TransitionStyle::SlideRight,
            TransitionStyle::SlideLeft,
            TransitionStyle::SlideTop,
            TransitionStyle::SlideBottom,
            TransitionStyle::Dissolve,
            TransitionStyle::Instant,
        ]
    }

    /// Returns true if this transition is a slide-based transition
    pub fn is_slide(&self) -> bool {
        matches!(
            self,
            Self::SlideRight | Self::SlideLeft | Self::SlideTop | Self::SlideBottom
        )
    }

    /// Returns true if this transition has no visible effect duration
    pub fn is_immediate(&self) -> bool {
        matches!(self, Self::Instant)
    }
}

/// Achievement system settings
#[derive(Debug, Clone)]
pub struct AchievementSettings {
    /// Enable achievement system
    pub enabled: bool,
    /// Achievement definitions
    pub achievements: Vec<Achievement>,
    /// Show notification on unlock
    pub show_notifications: bool,
    /// Notification duration
    pub notification_duration: f32,
    /// Play sound on unlock
    pub unlock_sound_enabled: bool,
    /// Unlock sound path
    pub unlock_sound: String,
    /// Steam integration enabled
    pub steam_integration: bool,
}

impl Default for AchievementSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            achievements: vec![],
            show_notifications: true,
            notification_duration: 5.0,
            unlock_sound_enabled: true,
            unlock_sound: "achievement_unlock.wav".to_string(),
            steam_integration: false,
        }
    }
}

/// Achievement definition
#[derive(Debug, Clone)]
pub struct Achievement {
    /// Unique achievement ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Icon path
    pub icon: String,
    /// Hidden until unlocked
    pub hidden: bool,
    /// Achievement points/score
    pub points: u32,
}

impl Achievement {
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            icon: "achievement_icon.png".to_string(),
            hidden: false,
            points: 10,
        }
    }
}

/// Game feel settings (juice, feedback, polish)
#[derive(Debug, Clone)]
pub struct GameFeelSettings {
    /// Enable screen shake
    pub screen_shake_enabled: bool,
    /// Screen shake intensity
    pub screen_shake_intensity: f32,
    /// Enable hit stop/freeze frames
    pub hit_stop_enabled: bool,
    /// Hit stop duration (milliseconds)
    pub hit_stop_duration: u32,
    /// Enable camera zoom on events
    pub camera_zoom_enabled: bool,
    /// Zoom intensity
    pub camera_zoom_intensity: f32,
    /// Enable particle effects
    pub particles_enabled: bool,
    /// Particle density
    pub particle_density: f32,
    /// Enable chromatic aberration
    pub chromatic_aberration: bool,
    /// Chromatic aberration intensity
    pub chromatic_intensity: f32,
    /// Enable motion blur
    pub motion_blur: bool,
    /// Motion blur samples
    pub motion_blur_samples: u32,
    /// Enable vignette
    pub vignette_enabled: bool,
    /// Vignette intensity
    pub vignette_intensity: f32,
}

impl Default for GameFeelSettings {
    fn default() -> Self {
        Self {
            screen_shake_enabled: true,
            screen_shake_intensity: 1.0,
            hit_stop_enabled: true,
            hit_stop_duration: 50,
            camera_zoom_enabled: true,
            camera_zoom_intensity: 1.0,
            particles_enabled: true,
            particle_density: 1.0,
            chromatic_aberration: false,
            chromatic_intensity: 0.5,
            motion_blur: false,
            motion_blur_samples: 8,
            vignette_enabled: true,
            vignette_intensity: 0.3,
        }
    }
}


impl Default for PolishPanel {
    fn default() -> Self {
        Self {
            splash_sequence: SplashSequence::new().with_engine_logo(),
            loading_screen: LoadingScreen::default(),
            save_config: SaveConfig::default(),
            credits_config: CreditsConfig::default(),
            ui_polish: UiPolishSettings::default(),
            achievements: AchievementSettings::default(),
            game_feel: GameFeelSettings::default(),
            active_tab: PolishTab::Splash,
            new_tip: String::new(),
            new_credit_name: String::new(),
            new_credit_role: String::new(),
            new_achievement_id: String::new(),
            new_achievement_name: String::new(),
            new_achievement_desc: String::new(),
        }
    }
}

impl PolishPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (PolishTab::Splash, "🌟 Splash"),
                (PolishTab::Loading, "⏳ Loading"),
                (PolishTab::SaveLoad, "💾 Save/Load"),
                (PolishTab::Credits, "🎬 Credits"),
                (PolishTab::UiPolish, "✨ UI Polish"),
                (PolishTab::Achievements, "🏆 Achievements"),
                (PolishTab::GameFeel, "🎮 Game Feel"),
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
        ui.separator();
    }

    fn show_splash_tab(&mut self, ui: &mut Ui) {
        ui.heading("Splash Screen Sequence");
        ui.add_space(5.0);

        // Sequence settings
        ui.checkbox(
            &mut self.splash_sequence.skip_all_on_input,
            "Skip all on any input",
        );

        ui.add_space(10.0);
        ui.label(format!(
            "Total duration: {:.1}s",
            self.splash_sequence.total_duration().as_secs_f32()
        ));
        ui.add_space(10.0);

        // List current screens
        ui.group(|ui| {
            ui.label(RichText::new("Screens").strong());

            for (i, screen) in self.splash_sequence.screens.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", i + 1));
                    ui.label(screen.image_path.display().to_string());
                    ui.label(format!(
                        "({:.1}s)",
                        screen.duration.unwrap_or(Duration::ZERO).as_secs_f32()
                    ));
                    if screen.skippable {
                        ui.label("⏭️ Skippable");
                    }
                });
            }

            if self.splash_sequence.screens.is_empty() {
                ui.label("No splash screens configured");
            }
        });

        ui.add_space(10.0);

        // Add screen buttons
        ui.horizontal(|ui| {
            if ui.button("➕ Add Engine Logo").clicked() {
                self.splash_sequence = std::mem::take(&mut self.splash_sequence).with_engine_logo();
            }
            if ui.button("➕ Add Publisher Logo").clicked() {
                self.splash_sequence = std::mem::take(&mut self.splash_sequence)
                    .with_publisher_logo("publisher_logo.png");
            }
        });
    }

    fn show_loading_tab(&mut self, ui: &mut Ui) {
        ui.heading("Loading Screen Configuration");
        ui.add_space(5.0);

        // Style selection
        ui.label("Style:");
        ui.horizontal(|ui| {
            let styles = [
                (LoadingStyle::ProgressBar, "Progress Bar"),
                (LoadingStyle::Spinner, "Spinner"),
                (LoadingStyle::FullScreen, "Full Screen"),
                (LoadingStyle::Dots, "Dots"),
                (LoadingStyle::ArtworkWithTips, "Artwork + Tips"),
            ];

            for (style, label) in styles {
                if ui
                    .selectable_label(self.loading_screen.style == style, label)
                    .clicked()
                {
                    self.loading_screen.style = style;
                }
            }
        });

        ui.add_space(10.0);

        // Options
        ui.checkbox(&mut self.loading_screen.show_percentage, "Show percentage");
        ui.checkbox(
            &mut self.loading_screen.show_task_description,
            "Show task description",
        );

        ui.add_space(10.0);

        // Tips section
        ui.collapsing("💡 Loading Tips", |ui| {
            for (i, tip) in self.loading_screen.tips.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}: {}", i + 1, tip));
                });
            }

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new_tip);
                if ui.button("Add Tip").clicked() && !self.new_tip.is_empty() {
                    self.loading_screen
                        .tips
                        .push(std::mem::take(&mut self.new_tip));
                }
            });
        });

        // Color pickers
        ui.add_space(10.0);
        ui.collapsing("🎨 Colors", |ui| {
            ui.horizontal(|ui| {
                ui.label("Background:");
                let mut bg = Color32::from_rgba_unmultiplied(
                    (self.loading_screen.background_color[0] * 255.0) as u8,
                    (self.loading_screen.background_color[1] * 255.0) as u8,
                    (self.loading_screen.background_color[2] * 255.0) as u8,
                    (self.loading_screen.background_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut bg).changed() {
                    self.loading_screen.background_color = [
                        bg.r() as f32 / 255.0,
                        bg.g() as f32 / 255.0,
                        bg.b() as f32 / 255.0,
                        bg.a() as f32 / 255.0,
                    ];
                }
            });

            ui.horizontal(|ui| {
                ui.label("Progress Bar:");
                let mut pc = Color32::from_rgba_unmultiplied(
                    (self.loading_screen.progress_color[0] * 255.0) as u8,
                    (self.loading_screen.progress_color[1] * 255.0) as u8,
                    (self.loading_screen.progress_color[2] * 255.0) as u8,
                    (self.loading_screen.progress_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut pc).changed() {
                    self.loading_screen.progress_color = [
                        pc.r() as f32 / 255.0,
                        pc.g() as f32 / 255.0,
                        pc.b() as f32 / 255.0,
                        pc.a() as f32 / 255.0,
                    ];
                }
            });
        });
    }


    fn show_save_load_tab(&mut self, ui: &mut Ui) {
        ui.heading("Save/Load Configuration");
        ui.add_space(5.0);

        egui::Grid::new("save_config_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Save Extension:");
                ui.text_edit_singleline(&mut self.save_config.extension);
                ui.end_row();

                ui.label("Save Directory:");
                ui.text_edit_singleline(&mut self.save_config.directory);
                ui.end_row();

                ui.label("Max Autosaves:");
                ui.add(egui::Slider::new(
                    &mut self.save_config.max_autosaves,
                    1..=10,
                ));
                ui.end_row();
            });

        ui.add_space(10.0);

        // Checkboxes
        ui.checkbox(&mut self.save_config.compress, "Compress save files (zstd)");
        ui.checkbox(
            &mut self.save_config.include_screenshot,
            "Include screenshot in saves",
        );

        // Autosave interval
        ui.add_space(10.0);
        let mut autosave_enabled = self.save_config.autosave_interval.is_some();
        if ui
            .checkbox(&mut autosave_enabled, "Enable autosave")
            .changed()
        {
            if autosave_enabled {
                self.save_config.autosave_interval = Some(Duration::from_secs(300));
            } else {
                self.save_config.autosave_interval = None;
            }
        }

        if let Some(interval) = &mut self.save_config.autosave_interval {
            ui.horizontal(|ui| {
                ui.label("Autosave interval:");
                let mut minutes = interval.as_secs() / 60;
                if ui
                    .add(egui::Slider::new(&mut minutes, 1..=30).suffix(" min"))
                    .changed()
                {
                    *interval = Duration::from_secs(minutes * 60);
                }
            });
        }

        // Preview
        ui.add_space(15.0);
        ui.separator();
        ui.label(RichText::new("📋 Configuration Preview").strong());
        ui.label(format!(
            "Save path: <game_data>/{}/slot.{}",
            self.save_config.directory, self.save_config.extension
        ));
    }

    fn show_credits_tab(&mut self, ui: &mut Ui) {
        ui.heading("Credits Configuration");
        ui.add_space(5.0);

        egui::Grid::new("credits_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label("Game Title:");
                ui.text_edit_singleline(&mut self.credits_config.game_title);
                ui.end_row();

                ui.label("Subtitle:");
                ui.text_edit_singleline(&mut self.credits_config.subtitle);
                ui.end_row();

                ui.label("Scroll Speed:");
                ui.add(egui::Slider::new(&mut self.credits_config.scroll_speed, 10.0..=200.0).suffix(" px/s"));
                ui.end_row();

                ui.label("Music Track:");
                ui.text_edit_singleline(&mut self.credits_config.music_track);
                ui.end_row();
            });

        ui.add_space(10.0);
        ui.checkbox(&mut self.credits_config.music_enabled, "Enable music during credits");

        // Credits entries
        ui.add_space(10.0);
        ui.separator();
        ui.label(RichText::new("Credits Entries").strong());

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                let mut to_remove = None;
                for (i, (name, role)) in self.credits_config.entries.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", i + 1));
                        ui.label(format!("{} - {}", name, role));
                        if ui.button("❌").clicked() {
                            to_remove = Some(i);
                        }
                    });
                }
                if let Some(i) = to_remove {
                    self.credits_config.entries.remove(i);
                }
            });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.new_credit_name);
            ui.label("Role:");
            ui.text_edit_singleline(&mut self.new_credit_role);
            if ui.button("Add Entry").clicked() && !self.new_credit_name.is_empty() && !self.new_credit_role.is_empty() {
                self.credits_config.entries.push((
                    std::mem::take(&mut self.new_credit_name),
                    std::mem::take(&mut self.new_credit_role),
                ));
            }
        });

        // Color pickers
        ui.add_space(10.0);
        ui.collapsing("🎨 Colors", |ui| {
            ui.horizontal(|ui| {
                ui.label("Background:");
                let mut bg = Color32::from_rgba_unmultiplied(
                    (self.credits_config.background_color[0] * 255.0) as u8,
                    (self.credits_config.background_color[1] * 255.0) as u8,
                    (self.credits_config.background_color[2] * 255.0) as u8,
                    (self.credits_config.background_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut bg).changed() {
                    self.credits_config.background_color = [
                        bg.r() as f32 / 255.0,
                        bg.g() as f32 / 255.0,
                        bg.b() as f32 / 255.0,
                        bg.a() as f32 / 255.0,
                    ];
                }
            });

            ui.horizontal(|ui| {
                ui.label("Text:");
                let mut tc = Color32::from_rgba_unmultiplied(
                    (self.credits_config.text_color[0] * 255.0) as u8,
                    (self.credits_config.text_color[1] * 255.0) as u8,
                    (self.credits_config.text_color[2] * 255.0) as u8,
                    (self.credits_config.text_color[3] * 255.0) as u8,
                );
                if ui.color_edit_button_srgba(&mut tc).changed() {
                    self.credits_config.text_color = [
                        tc.r() as f32 / 255.0,
                        tc.g() as f32 / 255.0,
                        tc.b() as f32 / 255.0,
                        tc.a() as f32 / 255.0,
                    ];
                }
            });
        });
    }

    fn show_ui_polish_tab(&mut self, ui: &mut Ui) {
        ui.heading("UI Polish Settings");
        ui.add_space(5.0);

        // Animations
        ui.group(|ui| {
            ui.label(RichText::new("Animations").strong());
            ui.checkbox(&mut self.ui_polish.animations_enabled, "Enable UI animations");
            
            if self.ui_polish.animations_enabled {
                ui.horizontal(|ui| {
                    ui.label("Speed:");
                    ui.add(egui::Slider::new(&mut self.ui_polish.animation_speed, 0.1..=3.0).suffix("×"));
                });
            }
        });

        ui.add_space(10.0);

        // Sounds
        ui.group(|ui| {
            ui.label(RichText::new("UI Sounds").strong());
            ui.checkbox(&mut self.ui_polish.sounds_enabled, "Enable UI sounds");
            
            if self.ui_polish.sounds_enabled {
                egui::Grid::new("ui_sounds_grid")
                    .num_columns(2)
                    .spacing([10.0, 5.0])
                    .show(ui, |ui| {
                        ui.label("Click Sound:");
                        ui.text_edit_singleline(&mut self.ui_polish.click_sound);
                        ui.end_row();

                        ui.label("Hover Sound:");
                        ui.text_edit_singleline(&mut self.ui_polish.hover_sound);
                        ui.end_row();
                    });
            }
        });

        ui.add_space(10.0);

        // Transitions
        ui.group(|ui| {
            ui.label(RichText::new("Screen Transitions").strong());
            ui.checkbox(&mut self.ui_polish.transitions_enabled, "Enable screen transitions");
            
            if self.ui_polish.transitions_enabled {
                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(egui::Slider::new(&mut self.ui_polish.transition_duration, 0.1..=2.0).suffix("s"));
                });

                ui.horizontal(|ui| {
                    ui.label("Style:");
                    egui::ComboBox::from_id_salt("transition_style")
                        .selected_text(format!("{:?}", self.ui_polish.transition_style))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.ui_polish.transition_style, TransitionStyle::Fade, "Fade");
                            ui.selectable_value(&mut self.ui_polish.transition_style, TransitionStyle::SlideRight, "Slide Right");
                            ui.selectable_value(&mut self.ui_polish.transition_style, TransitionStyle::SlideLeft, "Slide Left");
                            ui.selectable_value(&mut self.ui_polish.transition_style, TransitionStyle::SlideTop, "Slide Top");
                            ui.selectable_value(&mut self.ui_polish.transition_style, TransitionStyle::SlideBottom, "Slide Bottom");
                            ui.selectable_value(&mut self.ui_polish.transition_style, TransitionStyle::Dissolve, "Dissolve");
                            ui.selectable_value(&mut self.ui_polish.transition_style, TransitionStyle::Instant, "Instant");
                        });
                });
            }
        });

        ui.add_space(10.0);

        // Other UI features
        ui.group(|ui| {
            ui.label(RichText::new("Other Features").strong());
            ui.checkbox(&mut self.ui_polish.button_hover_enabled, "Button hover effects");
            ui.checkbox(&mut self.ui_polish.tooltips_enabled, "Tooltips");
            
            if self.ui_polish.tooltips_enabled {
                ui.horizontal(|ui| {
                    ui.label("Delay:");
                    ui.add(egui::Slider::new(&mut self.ui_polish.tooltip_delay, 0.0..=2.0).suffix("s"));
                });
            }

            ui.checkbox(&mut self.ui_polish.notifications_enabled, "Notifications");
            
            if self.ui_polish.notifications_enabled {
                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(egui::Slider::new(&mut self.ui_polish.notification_duration, 1.0..=10.0).suffix("s"));
                });
            }
        });
    }

    fn show_achievements_tab(&mut self, ui: &mut Ui) {
        ui.heading("Achievement System");
        ui.add_space(5.0);

        ui.checkbox(&mut self.achievements.enabled, "Enable achievement system");
        
        if !self.achievements.enabled {
            ui.label("Achievement system is disabled");
            return;
        }

        ui.add_space(10.0);

        // Settings
        ui.group(|ui| {
            ui.label(RichText::new("Settings").strong());
            ui.checkbox(&mut self.achievements.show_notifications, "Show unlock notifications");
            
            if self.achievements.show_notifications {
                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(egui::Slider::new(&mut self.achievements.notification_duration, 2.0..=10.0).suffix("s"));
                });
            }

            ui.checkbox(&mut self.achievements.unlock_sound_enabled, "Play unlock sound");
            
            if self.achievements.unlock_sound_enabled {
                ui.horizontal(|ui| {
                    ui.label("Sound:");
                    ui.text_edit_singleline(&mut self.achievements.unlock_sound);
                });
            }

            ui.checkbox(&mut self.achievements.steam_integration, "Steam integration");
        });

        ui.add_space(10.0);

        // Achievements list
        ui.separator();
        ui.label(RichText::new(format!("Achievements ({})", self.achievements.achievements.len())).strong());

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                let mut to_remove = None;
                for (i, achievement) in self.achievements.achievements.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("🏆 {}", achievement.name));
                        ui.label(format!("({} pts)", achievement.points));
                        if achievement.hidden {
                            ui.label("🔒 Hidden");
                        }
                        if ui.button("❌").clicked() {
                            to_remove = Some(i);
                        }
                    });
                    ui.label(format!("   {}", achievement.description));
                    ui.add_space(5.0);
                }
                if let Some(i) = to_remove {
                    self.achievements.achievements.remove(i);
                }
            });

        ui.add_space(10.0);
        ui.separator();
        ui.label("Add New Achievement");

        egui::Grid::new("new_achievement_grid")
            .num_columns(2)
            .spacing([10.0, 5.0])
            .show(ui, |ui| {
                ui.label("ID:");
                ui.text_edit_singleline(&mut self.new_achievement_id);
                ui.end_row();

                ui.label("Name:");
                ui.text_edit_singleline(&mut self.new_achievement_name);
                ui.end_row();

                ui.label("Description:");
                ui.text_edit_singleline(&mut self.new_achievement_desc);
                ui.end_row();
            });

        if ui.button("➕ Add Achievement").clicked() 
            && !self.new_achievement_id.is_empty() 
            && !self.new_achievement_name.is_empty() 
        {
            self.achievements.achievements.push(Achievement::new(
                std::mem::take(&mut self.new_achievement_id),
                std::mem::take(&mut self.new_achievement_name),
                std::mem::take(&mut self.new_achievement_desc),
            ));
        }
    }

    fn show_game_feel_tab(&mut self, ui: &mut Ui) {
        ui.heading("Game Feel Settings (Juice & Polish)");
        ui.add_space(5.0);

        // Camera effects
        ui.group(|ui| {
            ui.label(RichText::new("Camera Effects").strong());
            
            ui.checkbox(&mut self.game_feel.screen_shake_enabled, "Screen shake");
            if self.game_feel.screen_shake_enabled {
                ui.horizontal(|ui| {
                    ui.label("Intensity:");
                    ui.add(egui::Slider::new(&mut self.game_feel.screen_shake_intensity, 0.0..=2.0));
                });
            }

            ui.checkbox(&mut self.game_feel.camera_zoom_enabled, "Camera zoom on events");
            if self.game_feel.camera_zoom_enabled {
                ui.horizontal(|ui| {
                    ui.label("Intensity:");
                    ui.add(egui::Slider::new(&mut self.game_feel.camera_zoom_intensity, 0.0..=2.0));
                });
            }
        });

        ui.add_space(10.0);

        // Hit effects
        ui.group(|ui| {
            ui.label(RichText::new("Hit Effects").strong());
            
            ui.checkbox(&mut self.game_feel.hit_stop_enabled, "Hit stop (freeze frames)");
            if self.game_feel.hit_stop_enabled {
                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(egui::Slider::new(&mut self.game_feel.hit_stop_duration, 10..=200).suffix(" ms"));
                });
            }
        });

        ui.add_space(10.0);

        // Particles
        ui.group(|ui| {
            ui.label(RichText::new("Particles").strong());
            
            ui.checkbox(&mut self.game_feel.particles_enabled, "Particle effects");
            if self.game_feel.particles_enabled {
                ui.horizontal(|ui| {
                    ui.label("Density:");
                    ui.add(egui::Slider::new(&mut self.game_feel.particle_density, 0.1..=2.0).suffix("×"));
                });
            }
        });

        ui.add_space(10.0);

        // Post-processing effects
        ui.group(|ui| {
            ui.label(RichText::new("Post-Processing Effects").strong());
            
            ui.checkbox(&mut self.game_feel.chromatic_aberration, "Chromatic aberration");
            if self.game_feel.chromatic_aberration {
                ui.horizontal(|ui| {
                    ui.label("Intensity:");
                    ui.add(egui::Slider::new(&mut self.game_feel.chromatic_intensity, 0.0..=1.0));
                });
            }

            ui.checkbox(&mut self.game_feel.motion_blur, "Motion blur");
            if self.game_feel.motion_blur {
                ui.horizontal(|ui| {
                    ui.label("Samples:");
                    ui.add(egui::Slider::new(&mut self.game_feel.motion_blur_samples, 2..=16));
                });
            }

            ui.checkbox(&mut self.game_feel.vignette_enabled, "Vignette");
            if self.game_feel.vignette_enabled {
                ui.horizontal(|ui| {
                    ui.label("Intensity:");
                    ui.add(egui::Slider::new(&mut self.game_feel.vignette_intensity, 0.0..=1.0));
                });
            }
        });
    }
}

impl Panel for PolishPanel {
    fn name(&self) -> &str {
        "Polish"
    }

    fn show(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.show_tab_bar(ui);

            match self.active_tab {
                PolishTab::Splash => self.show_splash_tab(ui),
                PolishTab::Loading => self.show_loading_tab(ui),
                PolishTab::SaveLoad => self.show_save_load_tab(ui),
                PolishTab::Credits => self.show_credits_tab(ui),
                PolishTab::UiPolish => self.show_ui_polish_tab(ui),
                PolishTab::Achievements => self.show_achievements_tab(ui),
                PolishTab::GameFeel => self.show_game_feel_tab(ui),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polish_panel_default() {
        let panel = PolishPanel::new();
        assert_eq!(panel.name(), "Polish");
        assert_eq!(panel.active_tab, PolishTab::Splash);
    }

    #[test]
    fn test_tab_switching() {
        let mut panel = PolishPanel::new();
        panel.active_tab = PolishTab::Loading;
        assert_eq!(panel.active_tab, PolishTab::Loading);

        panel.active_tab = PolishTab::SaveLoad;
        assert_eq!(panel.active_tab, PolishTab::SaveLoad);
        
        panel.active_tab = PolishTab::Credits;
        assert_eq!(panel.active_tab, PolishTab::Credits);
    }

    #[test]
    fn test_initial_splash_sequence() {
        let panel = PolishPanel::new();
        // Default has engine logo
        assert_eq!(panel.splash_sequence.screens.len(), 1);
    }

    #[test]
    fn test_credits_config_default() {
        let config = CreditsConfig::default();
        assert_eq!(config.game_title, "Your Game");
        assert_eq!(config.scroll_speed, 50.0);
        assert!(config.music_enabled);
        assert_eq!(config.entries.len(), 2);
    }

    #[test]
    fn test_credits_add_entry() {
        let mut config = CreditsConfig::default();
        config.entries.push(("John Doe".to_string(), "Lead Developer".to_string()));
        assert_eq!(config.entries.len(), 3);
    }

    #[test]
    fn test_ui_polish_settings_default() {
        let settings = UiPolishSettings::default();
        assert!(settings.animations_enabled);
        assert_eq!(settings.animation_speed, 1.0);
        assert!(settings.sounds_enabled);
        assert!(settings.transitions_enabled);
        assert_eq!(settings.transition_duration, 0.3);
        assert_eq!(settings.transition_style, TransitionStyle::Fade);
    }

    #[test]
    fn test_transition_styles() {
        let mut settings = UiPolishSettings {
            transition_style: TransitionStyle::SlideRight,
            ..Default::default()
        };
        assert_eq!(settings.transition_style, TransitionStyle::SlideRight);
        
        settings.transition_style = TransitionStyle::Dissolve;
        assert_eq!(settings.transition_style, TransitionStyle::Dissolve);
    }

    #[test]
    fn test_achievement_settings_default() {
        let settings = AchievementSettings::default();
        assert!(settings.enabled);
        assert!(settings.show_notifications);
        assert_eq!(settings.notification_duration, 5.0);
        assert!(settings.unlock_sound_enabled);
        assert!(!settings.steam_integration);
        assert_eq!(settings.achievements.len(), 0);
    }

    #[test]
    fn test_achievement_creation() {
        let achievement = Achievement::new("first_blood", "First Blood", "Defeat your first enemy");
        assert_eq!(achievement.id, "first_blood");
        assert_eq!(achievement.name, "First Blood");
        assert_eq!(achievement.points, 10);
        assert!(!achievement.hidden);
    }

    #[test]
    fn test_add_achievement() {
        let mut settings = AchievementSettings::default();
        settings.achievements.push(Achievement::new("test_id", "Test Achievement", "Test description"));
        assert_eq!(settings.achievements.len(), 1);
        assert_eq!(settings.achievements[0].id, "test_id");
    }

    #[test]
    fn test_game_feel_settings_default() {
        let settings = GameFeelSettings::default();
        assert!(settings.screen_shake_enabled);
        assert_eq!(settings.screen_shake_intensity, 1.0);
        assert!(settings.hit_stop_enabled);
        assert_eq!(settings.hit_stop_duration, 50);
        assert!(settings.particles_enabled);
        assert_eq!(settings.particle_density, 1.0);
        assert!(!settings.chromatic_aberration);
        assert!(!settings.motion_blur);
        assert!(settings.vignette_enabled);
    }

    #[test]
    fn test_screen_shake_intensity() {
        let settings = GameFeelSettings {
            screen_shake_intensity: 1.5,
            ..Default::default()
        };
        assert_eq!(settings.screen_shake_intensity, 1.5);
    }

    #[test]
    fn test_hit_stop_duration() {
        let settings = GameFeelSettings {
            hit_stop_duration: 100,
            ..Default::default()
        };
        assert_eq!(settings.hit_stop_duration, 100);
    }

    #[test]
    fn test_particle_density() {
        let settings = GameFeelSettings {
            particle_density: 2.0,
            ..Default::default()
        };
        assert_eq!(settings.particle_density, 2.0);
    }

    #[test]
    fn test_chromatic_aberration() {
        let settings = GameFeelSettings {
            chromatic_aberration: true,
            chromatic_intensity: 0.8,
            ..Default::default()
        };
        assert!(settings.chromatic_aberration);
        assert_eq!(settings.chromatic_intensity, 0.8);
    }

    #[test]
    fn test_motion_blur_settings() {
        let settings = GameFeelSettings {
            motion_blur: true,
            motion_blur_samples: 12,
            ..Default::default()
        };
        assert!(settings.motion_blur);
        assert_eq!(settings.motion_blur_samples, 12);
    }

    #[test]
    fn test_vignette_intensity() {
        let settings = GameFeelSettings {
            vignette_intensity: 0.6,
            ..Default::default()
        };
        assert_eq!(settings.vignette_intensity, 0.6);
    }

    #[test]
    fn test_ui_polish_tooltip_delay() {
        let settings = UiPolishSettings {
            tooltip_delay: 1.0,
            ..Default::default()
        };
        assert_eq!(settings.tooltip_delay, 1.0);
    }

    #[test]
    fn test_ui_polish_notification_duration() {
        let settings = UiPolishSettings {
            notification_duration: 5.0,
            ..Default::default()
        };
        assert_eq!(settings.notification_duration, 5.0);
    }

    #[test]
    fn test_credits_scroll_speed_range() {
        let config = CreditsConfig {
            scroll_speed: 100.0,
            ..Default::default()
        };
        assert_eq!(config.scroll_speed, 100.0);
    }

    #[test]
    fn test_credits_color_configuration() {
        let config = CreditsConfig {
            background_color: [0.1, 0.1, 0.1, 1.0],
            text_color: [0.9, 0.9, 0.9, 1.0],
            ..Default::default()
        };
        assert_eq!(config.background_color[0], 0.1);
        assert_eq!(config.text_color[0], 0.9);
    }

    #[test]
    fn test_achievement_hidden_flag() {
        let mut achievement = Achievement::new("secret", "Secret Achievement", "Discover the secret");
        achievement.hidden = true;
        assert!(achievement.hidden);
    }

    #[test]
    fn test_achievement_points() {
        let mut achievement = Achievement::new("master", "Master", "Complete the game");
        achievement.points = 100;
        assert_eq!(achievement.points, 100);
    }

    #[test]
    fn test_transition_style_instant() {
        let settings = UiPolishSettings {
            transition_style: TransitionStyle::Instant,
            ..Default::default()
        };
        assert_eq!(settings.transition_style, TransitionStyle::Instant);
    }

    #[test]
    fn test_ui_sounds_configuration() {
        let settings = UiPolishSettings {
            click_sound: "custom_click.wav".to_string(),
            hover_sound: "custom_hover.wav".to_string(),
            ..Default::default()
        };
        assert_eq!(settings.click_sound, "custom_click.wav");
        assert_eq!(settings.hover_sound, "custom_hover.wav");
    }

    #[test]
    fn test_camera_zoom_settings() {
        let settings = GameFeelSettings {
            camera_zoom_enabled: true,
            camera_zoom_intensity: 1.5,
            ..Default::default()
        };
        assert!(settings.camera_zoom_enabled);
        assert_eq!(settings.camera_zoom_intensity, 1.5);
    }

    #[test]
    fn test_achievement_unlock_sound() {
        let settings = AchievementSettings {
            unlock_sound: "custom_unlock.wav".to_string(),
            ..Default::default()
        };
        assert_eq!(settings.unlock_sound, "custom_unlock.wav");
    }

    #[test]
    fn test_all_tabs_exist() {
        let panel = PolishPanel::new();
        // Test that we can switch to all tabs
        assert_eq!(panel.active_tab, PolishTab::Splash);
        
        let mut panel = panel;
        panel.active_tab = PolishTab::Loading;
        panel.active_tab = PolishTab::SaveLoad;
        panel.active_tab = PolishTab::Credits;
        panel.active_tab = PolishTab::UiPolish;
        panel.active_tab = PolishTab::Achievements;
        panel.active_tab = PolishTab::GameFeel;
        
        assert_eq!(panel.active_tab, PolishTab::GameFeel);
    }

    #[test]
    fn test_credits_multiple_entries() {
        let mut config = CreditsConfig::default();
        for i in 0..10 {
            config.entries.push((format!("Person {}", i), format!("Role {}", i)));
        }
        assert_eq!(config.entries.len(), 12); // 2 default + 10 new
    }

    #[test]
    fn test_achievements_multiple() {
        let mut settings = AchievementSettings::default();
        for i in 0..20 {
            settings.achievements.push(Achievement::new(
                format!("achievement_{}", i),
                format!("Achievement {}", i),
                format!("Description {}", i),
            ));
        }
        assert_eq!(settings.achievements.len(), 20);
    }

    #[test]
    fn test_game_feel_all_effects_disabled() {
        let settings = GameFeelSettings {
            screen_shake_enabled: false,
            hit_stop_enabled: false,
            camera_zoom_enabled: false,
            particles_enabled: false,
            vignette_enabled: false,
            ..Default::default()
        };
        
        assert!(!settings.screen_shake_enabled);
        assert!(!settings.hit_stop_enabled);
        assert!(!settings.camera_zoom_enabled);
        assert!(!settings.particles_enabled);
        assert!(!settings.vignette_enabled);
    }

    #[test]
    fn test_ui_polish_all_features_disabled() {
        let settings = UiPolishSettings {
            animations_enabled: false,
            sounds_enabled: false,
            transitions_enabled: false,
            button_hover_enabled: false,
            tooltips_enabled: false,
            notifications_enabled: false,
            ..Default::default()
        };
        
        assert!(!settings.animations_enabled);
        assert!(!settings.sounds_enabled);
        assert!(!settings.transitions_enabled);
        assert!(!settings.button_hover_enabled);
        assert!(!settings.tooltips_enabled);
        assert!(!settings.notifications_enabled);
    }

    #[test]
    fn test_panel_name() {
        let panel = PolishPanel::new();
        assert_eq!(panel.name(), "Polish");
    }

    #[test]
    fn test_default_new_strings_empty() {
        let panel = PolishPanel::new();
        assert!(panel.new_tip.is_empty());
        assert!(panel.new_credit_name.is_empty());
        assert!(panel.new_credit_role.is_empty());
        assert!(panel.new_achievement_id.is_empty());
        assert!(panel.new_achievement_name.is_empty());
        assert!(panel.new_achievement_desc.is_empty());
    }

    // ============================================================================
    // ENHANCED ENUM TESTS (Display, Hash, helpers)
    // ============================================================================

    #[test]
    fn test_transition_style_display() {
        for style in TransitionStyle::all() {
            let display = format!("{}", style);
            assert!(display.contains(style.name()));
            assert!(display.contains(style.icon()));
        }
    }

    #[test]
    fn test_transition_style_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for style in TransitionStyle::all() {
            set.insert(*style);
        }
        assert_eq!(set.len(), 7);
    }

    #[test]
    fn test_transition_style_all() {
        let all = TransitionStyle::all();
        assert_eq!(all.len(), 7);
        assert!(all.contains(&TransitionStyle::Fade));
        assert!(all.contains(&TransitionStyle::SlideRight));
        assert!(all.contains(&TransitionStyle::SlideLeft));
        assert!(all.contains(&TransitionStyle::SlideTop));
        assert!(all.contains(&TransitionStyle::SlideBottom));
        assert!(all.contains(&TransitionStyle::Dissolve));
        assert!(all.contains(&TransitionStyle::Instant));
    }

    #[test]
    fn test_transition_style_name() {
        assert_eq!(TransitionStyle::Fade.name(), "Fade");
        assert_eq!(TransitionStyle::SlideRight.name(), "Slide Right");
        assert_eq!(TransitionStyle::SlideLeft.name(), "Slide Left");
        assert_eq!(TransitionStyle::SlideTop.name(), "Slide Top");
        assert_eq!(TransitionStyle::SlideBottom.name(), "Slide Bottom");
        assert_eq!(TransitionStyle::Dissolve.name(), "Dissolve");
        assert_eq!(TransitionStyle::Instant.name(), "Instant");
    }

    #[test]
    fn test_transition_style_icon() {
        assert_eq!(TransitionStyle::Fade.icon(), "🌅");
        assert_eq!(TransitionStyle::SlideRight.icon(), "➡️");
        assert_eq!(TransitionStyle::SlideLeft.icon(), "⬅️");
        assert_eq!(TransitionStyle::SlideTop.icon(), "⬆️");
        assert_eq!(TransitionStyle::SlideBottom.icon(), "⬇️");
        assert_eq!(TransitionStyle::Dissolve.icon(), "✨");
        assert_eq!(TransitionStyle::Instant.icon(), "⚡");
    }

    #[test]
    fn test_transition_style_is_slide() {
        assert!(!TransitionStyle::Fade.is_slide());
        assert!(TransitionStyle::SlideRight.is_slide());
        assert!(TransitionStyle::SlideLeft.is_slide());
        assert!(TransitionStyle::SlideTop.is_slide());
        assert!(TransitionStyle::SlideBottom.is_slide());
        assert!(!TransitionStyle::Dissolve.is_slide());
        assert!(!TransitionStyle::Instant.is_slide());
    }

    #[test]
    fn test_transition_style_is_immediate() {
        assert!(!TransitionStyle::Fade.is_immediate());
        assert!(!TransitionStyle::SlideRight.is_immediate());
        assert!(!TransitionStyle::Dissolve.is_immediate());
        assert!(TransitionStyle::Instant.is_immediate());
    }

    #[test]
    fn test_transition_style_default() {
        let style: TransitionStyle = Default::default();
        assert_eq!(style, TransitionStyle::Fade);
    }
}

