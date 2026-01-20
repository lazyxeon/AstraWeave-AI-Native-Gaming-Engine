//! Audio panel for the editor UI
//!
//! Provides a comprehensive GUI for configuring and controlling audio:
//! - Master, Music, Voice, and SFX volume controls
//! - Music track selection and crossfade settings
//! - Spatial audio configuration
//! - Voice ducking settings
//! - Audio emitter management
//! - Real-time audio preview and testing

use egui::{Color32, RichText, Ui, Vec2};
use std::collections::HashMap;

use crate::panels::Panel;

/// Represents a music track entry for the playlist
#[derive(Debug, Clone, Default)]
pub struct MusicTrackEntry {
    pub name: String,
    pub path: String,
    pub duration_sec: f32,
    pub bpm: Option<f32>,
    pub mood: MusicMood,
}

/// Music mood categories
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MusicMood {
    #[default]
    Ambient,
    Calm,
    Exploration,
    Combat,
    Tension,
    Victory,
    Defeat,
    Boss,
    Menu,
}

impl MusicMood {
    pub fn all() -> &'static [MusicMood] {
        &[
            MusicMood::Ambient,
            MusicMood::Calm,
            MusicMood::Exploration,
            MusicMood::Combat,
            MusicMood::Tension,
            MusicMood::Victory,
            MusicMood::Defeat,
            MusicMood::Boss,
            MusicMood::Menu,
        ]
    }

    pub fn icon(&self) -> &'static str {
        match self {
            MusicMood::Ambient => "🌿",
            MusicMood::Calm => "☁️",
            MusicMood::Exploration => "🗺️",
            MusicMood::Combat => "⚔️",
            MusicMood::Tension => "⚡",
            MusicMood::Victory => "🏆",
            MusicMood::Defeat => "💀",
            MusicMood::Boss => "👹",
            MusicMood::Menu => "📋",
        }
    }
}

/// Spatial audio preset configurations
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SpatialPreset {
    #[default]
    Standard,
    Headphones,
    Speakers,
    Surround,
    VR,
}

impl SpatialPreset {
    pub fn all() -> &'static [SpatialPreset] {
        &[
            SpatialPreset::Standard,
            SpatialPreset::Headphones,
            SpatialPreset::Speakers,
            SpatialPreset::Surround,
            SpatialPreset::VR,
        ]
    }

    pub fn ear_separation(&self) -> f32 {
        match self {
            SpatialPreset::Standard => 0.2,
            SpatialPreset::Headphones => 0.18,
            SpatialPreset::Speakers => 0.5,
            SpatialPreset::Surround => 0.25,
            SpatialPreset::VR => 0.2,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SpatialPreset::Standard => "Default stereo configuration",
            SpatialPreset::Headphones => "Optimized for headphone listening",
            SpatialPreset::Speakers => "Wide stereo for speaker setups",
            SpatialPreset::Surround => "Multi-channel surround sound",
            SpatialPreset::VR => "VR/AR head-tracked audio",
        }
    }
}

/// Reverb environment presets
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ReverbEnvironment {
    #[default]
    None,
    SmallRoom,
    LargeRoom,
    Hall,
    Cave,
    Forest,
    Underwater,
    Cathedral,
}

impl ReverbEnvironment {
    pub fn all() -> &'static [ReverbEnvironment] {
        &[
            ReverbEnvironment::None,
            ReverbEnvironment::SmallRoom,
            ReverbEnvironment::LargeRoom,
            ReverbEnvironment::Hall,
            ReverbEnvironment::Cave,
            ReverbEnvironment::Forest,
            ReverbEnvironment::Underwater,
            ReverbEnvironment::Cathedral,
        ]
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ReverbEnvironment::None => "🔇",
            ReverbEnvironment::SmallRoom => "🚪",
            ReverbEnvironment::LargeRoom => "🏠",
            ReverbEnvironment::Hall => "🏛️",
            ReverbEnvironment::Cave => "🕳️",
            ReverbEnvironment::Forest => "🌲",
            ReverbEnvironment::Underwater => "🌊",
            ReverbEnvironment::Cathedral => "⛪",
        }
    }

    pub fn decay_time(&self) -> f32 {
        match self {
            ReverbEnvironment::None => 0.0,
            ReverbEnvironment::SmallRoom => 0.5,
            ReverbEnvironment::LargeRoom => 1.2,
            ReverbEnvironment::Hall => 2.5,
            ReverbEnvironment::Cave => 4.0,
            ReverbEnvironment::Forest => 0.8,
            ReverbEnvironment::Underwater => 3.0,
            ReverbEnvironment::Cathedral => 5.0,
        }
    }

    pub fn wet_dry_mix(&self) -> f32 {
        match self {
            ReverbEnvironment::None => 0.0,
            ReverbEnvironment::SmallRoom => 0.2,
            ReverbEnvironment::LargeRoom => 0.3,
            ReverbEnvironment::Hall => 0.4,
            ReverbEnvironment::Cave => 0.6,
            ReverbEnvironment::Forest => 0.25,
            ReverbEnvironment::Underwater => 0.7,
            ReverbEnvironment::Cathedral => 0.5,
        }
    }
}

/// Audio emitter information for 3D audio
#[derive(Debug, Clone, Default)]
pub struct AudioEmitterInfo {
    pub name: String,
    pub position: [f32; 3],
    pub is_playing: bool,
    pub current_sound: Option<String>,
    pub volume: f32,
    pub attenuation_min: f32,
    pub attenuation_max: f32,
}

/// Panel tab selection
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AudioTab {
    #[default]
    Mixer,
    Music,
    Spatial,
    Emitters,
    Preview,
}

/// Main Audio Panel for editor
pub struct AudioPanel {
    // Tab state
    active_tab: AudioTab,

    // Volume controls
    master_volume: f32,
    music_volume: f32,
    voice_volume: f32,
    sfx_volume: f32,
    master_muted: bool,
    music_muted: bool,
    voice_muted: bool,
    sfx_muted: bool,

    // Music settings
    music_tracks: Vec<MusicTrackEntry>,
    selected_track_index: Option<usize>,
    crossfade_duration: f32,
    playlist_shuffle: bool,
    playlist_loop: bool,
    filter_mood: Option<MusicMood>,

    // Spatial audio settings
    spatial_preset: SpatialPreset,
    ear_separation: f32,
    hrtf_enabled: bool,
    doppler_enabled: bool,
    doppler_factor: f32,
    distance_model: DistanceModel,
    rolloff_factor: f32,
    reference_distance: f32,
    max_distance: f32,

    // Reverb settings
    reverb_environment: ReverbEnvironment,
    reverb_decay_time: f32,
    reverb_wet_dry: f32,
    reverb_enabled: bool,

    // Voice ducking
    duck_enabled: bool,
    duck_factor: f32,
    duck_attack: f32,
    duck_release: f32,

    // Emitters
    emitters: HashMap<u64, AudioEmitterInfo>,
    next_emitter_id: u64,
    selected_emitter_id: Option<u64>,

    // Preview/Testing
    preview_frequency: f32,
    preview_duration: f32,
    preview_position: [f32; 3],
    is_previewing: bool,

    // Status
    audio_stats: AudioStats,
}

/// Distance attenuation model
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DistanceModel {
    #[default]
    Linear,
    Inverse,
    Exponential,
}

impl DistanceModel {
    pub fn all() -> &'static [DistanceModel] {
        &[
            DistanceModel::Linear,
            DistanceModel::Inverse,
            DistanceModel::Exponential,
        ]
    }
}

/// Audio engine statistics
#[derive(Debug, Clone, Default)]
pub struct AudioStats {
    pub active_voices: usize,
    pub active_music_channels: usize,
    pub active_emitters: usize,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: f32,
    pub buffer_underruns: usize,
    pub sample_rate: u32,
    pub latency_ms: f32,
}

impl Default for AudioPanel {
    fn default() -> Self {
        Self {
            active_tab: AudioTab::Mixer,

            // Volume defaults
            master_volume: 1.0,
            music_volume: 0.7,
            voice_volume: 1.0,
            sfx_volume: 0.8,
            master_muted: false,
            music_muted: false,
            voice_muted: false,
            sfx_muted: false,

            // Music defaults
            music_tracks: Vec::new(),
            selected_track_index: None,
            crossfade_duration: 2.0,
            playlist_shuffle: false,
            playlist_loop: true,
            filter_mood: None,

            // Spatial defaults
            spatial_preset: SpatialPreset::Standard,
            ear_separation: 0.2,
            hrtf_enabled: false,
            doppler_enabled: true,
            doppler_factor: 1.0,
            distance_model: DistanceModel::Inverse,
            rolloff_factor: 1.0,
            reference_distance: 1.0,
            max_distance: 100.0,

            // Reverb defaults
            reverb_environment: ReverbEnvironment::None,
            reverb_decay_time: 1.0,
            reverb_wet_dry: 0.3,
            reverb_enabled: false,

            // Voice ducking defaults
            duck_enabled: true,
            duck_factor: 0.3,
            duck_attack: 0.1,
            duck_release: 0.5,

            // Emitters
            emitters: HashMap::new(),
            next_emitter_id: 1,
            selected_emitter_id: None,

            // Preview
            preview_frequency: 440.0,
            preview_duration: 0.5,
            preview_position: [0.0, 0.0, 0.0],
            is_previewing: false,

            // Stats
            audio_stats: AudioStats {
                sample_rate: 44100,
                latency_ms: 10.0,
                ..Default::default()
            },
        }
    }
}

impl AudioPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (AudioTab::Mixer, "🎚️ Mixer"),
                (AudioTab::Music, "🎵 Music"),
                (AudioTab::Spatial, "🔊 Spatial"),
                (AudioTab::Emitters, "📍 Emitters"),
                (AudioTab::Preview, "🎧 Preview"),
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

    fn show_mixer_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎚️ Audio Mixer");
        ui.add_space(10.0);

        // Master Volume
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Master");
                if ui
                    .button(if self.master_muted { "🔇" } else { "🔊" })
                    .clicked()
                {
                    self.master_muted = !self.master_muted;
                }
            });
            ui.add_space(5.0);

            let effective_master = if self.master_muted {
                0.0
            } else {
                self.master_volume
            };
            Self::show_volume_slider_labeled(ui, "Master Volume", &mut self.master_volume, effective_master);
        });

        ui.add_space(10.0);

        // Channel volumes
        ui.columns(3, |columns| {
            // Music channel
            columns[0].group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🎵 Music").strong());
                    if ui
                        .button(if self.music_muted { "🔇" } else { "🔊" })
                        .clicked()
                    {
                        self.music_muted = !self.music_muted;
                    }
                });
                ui.add_space(5.0);

                let effective = if self.music_muted || self.master_muted {
                    0.0
                } else {
                    self.music_volume * self.master_volume
                };
                Self::show_volume_slider_static(ui, &mut self.music_volume, effective);
            });

            // Voice channel
            columns[1].group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🗣️ Voice").strong());
                    if ui
                        .button(if self.voice_muted { "🔇" } else { "🔊" })
                        .clicked()
                    {
                        self.voice_muted = !self.voice_muted;
                    }
                });
                ui.add_space(5.0);

                let effective = if self.voice_muted || self.master_muted {
                    0.0
                } else {
                    self.voice_volume * self.master_volume
                };
                Self::show_volume_slider_static(ui, &mut self.voice_volume, effective);
            });

            // SFX channel
            columns[2].group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("💥 SFX").strong());
                    if ui
                        .button(if self.sfx_muted { "🔇" } else { "🔊" })
                        .clicked()
                    {
                        self.sfx_muted = !self.sfx_muted;
                    }
                });
                ui.add_space(5.0);

                let effective = if self.sfx_muted || self.master_muted {
                    0.0
                } else {
                    self.sfx_volume * self.master_volume
                };
                Self::show_volume_slider_static(ui, &mut self.sfx_volume, effective);
            });
        });

        ui.add_space(15.0);

        // Voice Ducking section
        ui.collapsing("🔉 Voice Ducking", |ui| {
            ui.checkbox(&mut self.duck_enabled, "Enable voice ducking");
            ui.add_enabled(
                self.duck_enabled,
                egui::Slider::new(&mut self.duck_factor, 0.0..=1.0)
                    .text("Duck amount")
                    .suffix("%")
                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
            );
            ui.add_enabled(
                self.duck_enabled,
                egui::Slider::new(&mut self.duck_attack, 0.01..=1.0)
                    .text("Attack")
                    .suffix("s"),
            );
            ui.add_enabled(
                self.duck_enabled,
                egui::Slider::new(&mut self.duck_release, 0.1..=3.0)
                    .text("Release")
                    .suffix("s"),
            );
        });

        ui.add_space(10.0);

        // Audio Statistics
        self.show_audio_stats(ui);
    }

    fn show_volume_slider_labeled(ui: &mut Ui, label: &str, value: &mut f32, effective: f32) {
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(value, 0.0..=1.0).text(label));
            let db = if effective > 0.0 {
                20.0 * effective.log10()
            } else {
                -60.0
            };
            ui.label(format!("{:.1} dB", db));
        });

        // Visual meter
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 8.0), egui::Sense::hover());
        let filled_width = rect.width() * effective;
        ui.painter().rect_filled(rect, 2.0, Color32::from_rgb(40, 40, 45));
        let filled_rect = egui::Rect::from_min_size(rect.min, Vec2::new(filled_width, rect.height()));

        let color = if effective > 0.9 {
            Color32::from_rgb(220, 80, 80)
        } else if effective > 0.7 {
            Color32::from_rgb(220, 180, 80)
        } else {
            Color32::from_rgb(80, 180, 80)
        };
        ui.painter().rect_filled(filled_rect, 2.0, color);
    }

    fn show_volume_slider_static(ui: &mut Ui, value: &mut f32, effective: f32) {
        ui.add(egui::Slider::new(value, 0.0..=1.0).show_value(true));

        let db = if effective > 0.0 {
            20.0 * effective.log10()
        } else {
            -60.0
        };
        ui.label(format!("{:.1} dB", db.max(-60.0)));

        // Visual meter
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 6.0), egui::Sense::hover());
        let filled_width = rect.width() * effective;
        ui.painter()
            .rect_filled(rect, 2.0, Color32::from_rgb(40, 40, 45));
        let filled_rect = egui::Rect::from_min_size(rect.min, Vec2::new(filled_width, rect.height()));

        let color = if effective > 0.9 {
            Color32::from_rgb(220, 80, 80)
        } else if effective > 0.7 {
            Color32::from_rgb(220, 180, 80)
        } else {
            Color32::from_rgb(80, 180, 80)
        };
        ui.painter().rect_filled(filled_rect, 2.0, color);
    }

    fn show_audio_stats(&self, ui: &mut Ui) {
        ui.collapsing("📊 Audio Statistics", |ui| {
            egui::Grid::new("audio_stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Active Voices:");
                    ui.label(format!("{}", self.audio_stats.active_voices));
                    ui.end_row();

                    ui.label("Music Channels:");
                    ui.label(format!("{}", self.audio_stats.active_music_channels));
                    ui.end_row();

                    ui.label("3D Emitters:");
                    ui.label(format!("{}", self.audio_stats.active_emitters));
                    ui.end_row();

                    ui.label("Sample Rate:");
                    ui.label(format!("{} Hz", self.audio_stats.sample_rate));
                    ui.end_row();

                    ui.label("Latency:");
                    ui.label(format!("{:.1} ms", self.audio_stats.latency_ms));
                    ui.end_row();

                    ui.label("CPU Usage:");
                    ui.label(format!("{:.1}%", self.audio_stats.cpu_usage_percent));
                    ui.end_row();

                    ui.label("Memory:");
                    ui.label(format!("{:.1} MB", self.audio_stats.memory_usage_mb));
                    ui.end_row();

                    if self.audio_stats.buffer_underruns > 0 {
                        ui.label(RichText::new("Buffer Underruns:").color(Color32::RED));
                        ui.label(
                            RichText::new(format!("{}", self.audio_stats.buffer_underruns))
                                .color(Color32::RED),
                        );
                        ui.end_row();
                    }
                });
        });
    }

    fn show_music_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎵 Music Management");
        ui.add_space(10.0);

        // Playback controls
        ui.horizontal(|ui| {
            if ui.button("⏮ Prev").clicked() {
                // Previous track
            }
            if ui.button("▶ Play").clicked() {
                // Play current
            }
            if ui.button("⏸ Pause").clicked() {
                // Pause
            }
            if ui.button("⏹ Stop").clicked() {
                // Stop
            }
            if ui.button("⏭ Next").clicked() {
                // Next track
            }

            ui.separator();

            if ui
                .selectable_label(self.playlist_shuffle, "🔀 Shuffle")
                .clicked()
            {
                self.playlist_shuffle = !self.playlist_shuffle;
            }
            if ui
                .selectable_label(self.playlist_loop, "🔁 Loop")
                .clicked()
            {
                self.playlist_loop = !self.playlist_loop;
            }
        });

        ui.add_space(10.0);

        // Crossfade settings
        ui.group(|ui| {
            ui.label(RichText::new("Crossfade Settings").strong());
            ui.add(
                egui::Slider::new(&mut self.crossfade_duration, 0.0..=10.0)
                    .text("Duration")
                    .suffix("s"),
            );
        });

        ui.add_space(10.0);

        // Mood filter
        ui.horizontal(|ui| {
            ui.label("Filter by mood:");
            if ui
                .selectable_label(self.filter_mood.is_none(), "All")
                .clicked()
            {
                self.filter_mood = None;
            }
            for mood in MusicMood::all() {
                if ui
                    .selectable_label(
                        self.filter_mood == Some(*mood),
                        format!("{} {:?}", mood.icon(), mood),
                    )
                    .clicked()
                {
                    self.filter_mood = Some(*mood);
                }
            }
        });

        ui.add_space(10.0);

        // Track list
        ui.group(|ui| {
            ui.label(RichText::new("Music Tracks").strong());
            ui.separator();

            let filtered_tracks: Vec<(usize, &MusicTrackEntry)> = self
                .music_tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| self.filter_mood.is_none() || self.filter_mood == Some(t.mood))
                .collect();

            let no_tracks = filtered_tracks.is_empty();
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for (idx, track) in &filtered_tracks {
                        let is_selected = self.selected_track_index == Some(*idx);
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_selected, &track.name).clicked() {
                                self.selected_track_index = Some(*idx);
                            }
                            ui.label(track.mood.icon());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("{:.0}s", track.duration_sec));
                            });
                        });
                    }

                    if no_tracks {
                        ui.label("No tracks found. Add tracks to assets/audio/music/");
                    }
                });
        });

        ui.add_space(10.0);

        // Add track button
        if ui.button("➕ Add Music Track").clicked() {
            self.music_tracks.push(MusicTrackEntry {
                name: format!("Track {}", self.music_tracks.len() + 1),
                path: String::new(),
                duration_sec: 180.0,
                bpm: Some(120.0),
                mood: MusicMood::Ambient,
            });
        }
    }

    fn show_spatial_tab(&mut self, ui: &mut Ui) {
        ui.heading("🔊 Spatial Audio");
        ui.add_space(10.0);

        // Spatial preset selection
        ui.group(|ui| {
            ui.label(RichText::new("Audio Preset").strong());
            ui.horizontal_wrapped(|ui| {
                for preset in SpatialPreset::all() {
                    let is_selected = self.spatial_preset == *preset;
                    let button = egui::Button::new(format!("{:?}", preset)).fill(if is_selected {
                        Color32::from_rgb(60, 100, 160)
                    } else {
                        Color32::from_rgb(50, 50, 55)
                    });

                    if ui.add(button).clicked() {
                        self.spatial_preset = *preset;
                        self.apply_spatial_preset(*preset);
                    }
                }
            });
            ui.label(self.spatial_preset.description());
        });

        ui.add_space(10.0);

        // Listener settings
        ui.collapsing("👂 Listener Settings", |ui| {
            ui.add(
                egui::Slider::new(&mut self.ear_separation, 0.1..=1.0)
                    .text("Ear Separation")
                    .suffix("m"),
            );
            ui.checkbox(&mut self.hrtf_enabled, "Enable HRTF (Head-Related Transfer Function)");
        });

        ui.add_space(10.0);

        // Doppler settings
        ui.collapsing("💨 Doppler Effect", |ui| {
            ui.checkbox(&mut self.doppler_enabled, "Enable Doppler Effect");
            ui.add_enabled(
                self.doppler_enabled,
                egui::Slider::new(&mut self.doppler_factor, 0.0..=3.0).text("Doppler Factor"),
            );
        });

        ui.add_space(10.0);

        // Distance attenuation
        ui.collapsing("📏 Distance Attenuation", |ui| {
            ui.horizontal(|ui| {
                ui.label("Model:");
                for model in DistanceModel::all() {
                    if ui
                        .selectable_label(self.distance_model == *model, format!("{:?}", model))
                        .clicked()
                    {
                        self.distance_model = *model;
                    }
                }
            });

            ui.add(
                egui::Slider::new(&mut self.rolloff_factor, 0.1..=5.0).text("Rolloff Factor"),
            );
            ui.add(
                egui::Slider::new(&mut self.reference_distance, 0.1..=20.0)
                    .text("Reference Distance")
                    .suffix("m"),
            );
            ui.add(
                egui::Slider::new(&mut self.max_distance, 10.0..=500.0)
                    .text("Max Distance")
                    .suffix("m"),
            );
        });

        ui.add_space(10.0);

        // Reverb settings
        ui.collapsing("🔊 Reverb / Environment", |ui| {
            ui.checkbox(&mut self.reverb_enabled, "Enable Reverb");

            ui.add_space(5.0);
            ui.label("Environment Preset:");
            ui.horizontal_wrapped(|ui| {
                for env in ReverbEnvironment::all() {
                    let is_selected = self.reverb_environment == *env;
                    if ui
                        .selectable_label(is_selected, format!("{} {:?}", env.icon(), env))
                        .clicked()
                    {
                        self.reverb_environment = *env;
                        self.apply_reverb_preset(*env);
                    }
                }
            });

            ui.add_space(5.0);
            ui.add_enabled(
                self.reverb_enabled,
                egui::Slider::new(&mut self.reverb_decay_time, 0.1..=10.0)
                    .text("Decay Time")
                    .suffix("s"),
            );
            ui.add_enabled(
                self.reverb_enabled,
                egui::Slider::new(&mut self.reverb_wet_dry, 0.0..=1.0)
                    .text("Wet/Dry Mix")
                    .suffix("%")
                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
            );
        });
    }

    fn show_emitters_tab(&mut self, ui: &mut Ui) {
        ui.heading("📍 Audio Emitters");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("➕ Add Emitter").clicked() {
                let id = self.next_emitter_id;
                self.next_emitter_id += 1;
                self.emitters.insert(
                    id,
                    AudioEmitterInfo {
                        name: format!("Emitter {}", id),
                        position: [0.0, 0.0, 0.0],
                        is_playing: false,
                        current_sound: None,
                        volume: 1.0,
                        attenuation_min: 1.0,
                        attenuation_max: 50.0,
                    },
                );
                self.selected_emitter_id = Some(id);
            }

            ui.label(format!("Total: {}", self.emitters.len()));
        });

        ui.add_space(10.0);

        // Emitter list
        ui.group(|ui| {
            ui.label(RichText::new("Emitters").strong());
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    let emitter_ids: Vec<u64> = self.emitters.keys().cloned().collect();
                    for id in emitter_ids {
                        if let Some(emitter) = self.emitters.get(&id) {
                            let is_selected = self.selected_emitter_id == Some(id);
                            ui.horizontal(|ui| {
                                let status_icon = if emitter.is_playing { "🔊" } else { "🔇" };
                                if ui
                                    .selectable_label(
                                        is_selected,
                                        format!("{} {}", status_icon, emitter.name),
                                    )
                                    .clicked()
                                {
                                    self.selected_emitter_id = Some(id);
                                }
                            });
                        }
                    }

                    if self.emitters.is_empty() {
                        ui.label("No emitters. Click 'Add Emitter' to create one.");
                    }
                });
        });

        ui.add_space(10.0);

        // Selected emitter details
        let mut should_delete_emitter = false;
        if let Some(id) = self.selected_emitter_id {
            if let Some(emitter) = self.emitters.get_mut(&id) {
                ui.group(|ui| {
                    ui.label(RichText::new("Selected Emitter").strong());
                    ui.separator();

                    egui::Grid::new("emitter_grid")
                        .num_columns(2)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut emitter.name);
                            ui.end_row();

                            ui.label("Position X:");
                            ui.add(egui::DragValue::new(&mut emitter.position[0]).speed(0.1));
                            ui.end_row();

                            ui.label("Position Y:");
                            ui.add(egui::DragValue::new(&mut emitter.position[1]).speed(0.1));
                            ui.end_row();

                            ui.label("Position Z:");
                            ui.add(egui::DragValue::new(&mut emitter.position[2]).speed(0.1));
                            ui.end_row();

                            ui.label("Volume:");
                            ui.add(egui::Slider::new(&mut emitter.volume, 0.0..=2.0));
                            ui.end_row();

                            ui.label("Min Distance:");
                            ui.add(
                                egui::DragValue::new(&mut emitter.attenuation_min)
                                    .speed(0.1)
                                    .suffix("m"),
                            );
                            ui.end_row();

                            ui.label("Max Distance:");
                            ui.add(
                                egui::DragValue::new(&mut emitter.attenuation_max)
                                    .speed(0.1)
                                    .suffix("m"),
                            );
                            ui.end_row();
                        });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        if ui.button("▶ Play Test").clicked() {
                            emitter.is_playing = true;
                        }
                        if ui.button("⏹ Stop").clicked() {
                            emitter.is_playing = false;
                        }
                    });
                });
            }

            // Delete button outside the borrow scope
            if ui.button("🗑 Delete Selected Emitter").clicked() {
                should_delete_emitter = true;
            }
        }

        // Handle deletion after borrow is released
        if should_delete_emitter {
            if let Some(id) = self.selected_emitter_id {
                self.emitters.remove(&id);
                self.selected_emitter_id = None;
            }
        }
    }

    fn show_preview_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎧 Audio Preview & Testing");
        ui.add_space(10.0);

        // Test tone generator
        ui.group(|ui| {
            ui.label(RichText::new("🎵 Test Tone Generator").strong());
            ui.separator();

            ui.add(
                egui::Slider::new(&mut self.preview_frequency, 20.0..=20000.0)
                    .text("Frequency")
                    .suffix(" Hz")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.preview_duration, 0.1..=5.0)
                    .text("Duration")
                    .suffix("s"),
            );

            ui.horizontal(|ui| {
                if ui.button("▶ Play Tone").clicked() {
                    self.is_previewing = true;
                }
                if ui.button("⏹ Stop").clicked() {
                    self.is_previewing = false;
                }
            });

            // Frequency presets
            ui.add_space(5.0);
            ui.label("Common frequencies:");
            ui.horizontal_wrapped(|ui| {
                let presets = [
                    ("A4", 440.0),
                    ("C5", 523.25),
                    ("E5", 659.25),
                    ("Beep", 1000.0),
                    ("Low", 100.0),
                    ("High", 8000.0),
                ];
                for (name, freq) in presets {
                    if ui.button(name).clicked() {
                        self.preview_frequency = freq;
                    }
                }
            });
        });

        ui.add_space(10.0);

        // 3D Position test
        ui.group(|ui| {
            ui.label(RichText::new("📍 3D Position Test").strong());
            ui.separator();

            egui::Grid::new("preview_pos_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("X:");
                    ui.add(egui::DragValue::new(&mut self.preview_position[0]).speed(0.1));
                    ui.end_row();

                    ui.label("Y:");
                    ui.add(egui::DragValue::new(&mut self.preview_position[1]).speed(0.1));
                    ui.end_row();

                    ui.label("Z:");
                    ui.add(egui::DragValue::new(&mut self.preview_position[2]).speed(0.1));
                    ui.end_row();
                });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.button("▶ Play at Position").clicked() {
                    // Play 3D positioned test sound
                }

                // Quick position presets
                if ui.button("Front").clicked() {
                    self.preview_position = [0.0, 0.0, -5.0];
                }
                if ui.button("Left").clicked() {
                    self.preview_position = [-5.0, 0.0, 0.0];
                }
                if ui.button("Right").clicked() {
                    self.preview_position = [5.0, 0.0, 0.0];
                }
                if ui.button("Behind").clicked() {
                    self.preview_position = [0.0, 0.0, 5.0];
                }
            });
        });

        ui.add_space(10.0);

        // Quick audio tests
        ui.group(|ui| {
            ui.label(RichText::new("🧪 Quick Tests").strong());
            ui.separator();

            ui.horizontal_wrapped(|ui| {
                if ui.button("🎵 Music Fade").clicked() {
                    // Test music crossfade
                }
                if ui.button("💥 SFX Burst").clicked() {
                    // Play rapid SFX
                }
                if ui.button("🗣️ Voice Test").clicked() {
                    // Test voice with ducking
                }
                if ui.button("🔊 Surround Test").clicked() {
                    // Circle around listener
                }
                if ui.button("🌊 Reverb Test").clicked() {
                    // Test reverb settings
                }
            });
        });
    }

    fn apply_spatial_preset(&mut self, preset: SpatialPreset) {
        self.ear_separation = preset.ear_separation();
        match preset {
            SpatialPreset::Headphones => {
                self.hrtf_enabled = true;
                self.doppler_enabled = true;
            }
            SpatialPreset::VR => {
                self.hrtf_enabled = true;
                self.doppler_enabled = true;
            }
            SpatialPreset::Surround => {
                self.hrtf_enabled = false;
                self.doppler_enabled = true;
            }
            _ => {
                self.hrtf_enabled = false;
                self.doppler_enabled = true;
            }
        }
    }

    fn apply_reverb_preset(&mut self, env: ReverbEnvironment) {
        self.reverb_decay_time = env.decay_time();
        self.reverb_wet_dry = env.wet_dry_mix();
        self.reverb_enabled = env != ReverbEnvironment::None;
    }

    // Getters for testing
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn music_volume(&self) -> f32 {
        self.music_volume
    }

    pub fn voice_volume(&self) -> f32 {
        self.voice_volume
    }

    pub fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }

    pub fn spatial_preset(&self) -> SpatialPreset {
        self.spatial_preset
    }

    pub fn reverb_environment(&self) -> ReverbEnvironment {
        self.reverb_environment
    }

    pub fn emitter_count(&self) -> usize {
        self.emitters.len()
    }

    pub fn add_emitter(&mut self, name: &str, position: [f32; 3]) -> u64 {
        let id = self.next_emitter_id;
        self.next_emitter_id += 1;
        self.emitters.insert(
            id,
            AudioEmitterInfo {
                name: name.to_string(),
                position,
                is_playing: false,
                current_sound: None,
                volume: 1.0,
                attenuation_min: 1.0,
                attenuation_max: 50.0,
            },
        );
        id
    }

    pub fn set_volumes(&mut self, master: f32, music: f32, voice: f32, sfx: f32) {
        self.master_volume = master.clamp(0.0, 1.0);
        self.music_volume = music.clamp(0.0, 1.0);
        self.voice_volume = voice.clamp(0.0, 1.0);
        self.sfx_volume = sfx.clamp(0.0, 1.0);
    }

    pub fn music_tracks(&self) -> &[MusicTrackEntry] {
        &self.music_tracks
    }

    pub fn add_music_track(&mut self, track: MusicTrackEntry) {
        self.music_tracks.push(track);
    }
}

impl Panel for AudioPanel {
    fn name(&self) -> &'static str {
        "Audio"
    }

    fn show(&mut self, ui: &mut Ui) {
        self.show_tab_bar(ui);

        match self.active_tab {
            AudioTab::Mixer => self.show_mixer_tab(ui),
            AudioTab::Music => self.show_music_tab(ui),
            AudioTab::Spatial => self.show_spatial_tab(ui),
            AudioTab::Emitters => self.show_emitters_tab(ui),
            AudioTab::Preview => self.show_preview_tab(ui),
        }
    }

    fn update(&mut self) {
        // Update audio stats periodically
        // This would connect to the actual AudioEngine in production
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_panel_creation() {
        let panel = AudioPanel::new();
        assert_eq!(panel.master_volume(), 1.0);
        assert_eq!(panel.music_volume(), 0.7);
        assert_eq!(panel.voice_volume(), 1.0);
        assert_eq!(panel.sfx_volume(), 0.8);
    }

    #[test]
    fn test_volume_clamping() {
        let mut panel = AudioPanel::new();
        panel.set_volumes(1.5, -0.5, 0.5, 2.0);
        assert_eq!(panel.master_volume(), 1.0); // Clamped
        assert_eq!(panel.music_volume(), 0.0);  // Clamped
        assert_eq!(panel.voice_volume(), 0.5);  // Normal
        assert_eq!(panel.sfx_volume(), 1.0);    // Clamped
    }

    #[test]
    fn test_spatial_presets() {
        let mut panel = AudioPanel::new();
        
        panel.apply_spatial_preset(SpatialPreset::Headphones);
        assert!(panel.hrtf_enabled);
        assert_eq!(panel.ear_separation, 0.18);
        
        panel.apply_spatial_preset(SpatialPreset::Speakers);
        assert!(!panel.hrtf_enabled);
        assert_eq!(panel.ear_separation, 0.5);
    }

    #[test]
    fn test_reverb_presets() {
        let mut panel = AudioPanel::new();
        
        panel.apply_reverb_preset(ReverbEnvironment::Cathedral);
        assert!(panel.reverb_enabled);
        assert_eq!(panel.reverb_decay_time, 5.0);
        assert_eq!(panel.reverb_wet_dry, 0.5);
        
        panel.apply_reverb_preset(ReverbEnvironment::None);
        assert!(!panel.reverb_enabled);
    }

    #[test]
    fn test_emitter_management() {
        let mut panel = AudioPanel::new();
        assert_eq!(panel.emitter_count(), 0);
        
        let id1 = panel.add_emitter("Test1", [1.0, 2.0, 3.0]);
        assert_eq!(panel.emitter_count(), 1);
        
        let id2 = panel.add_emitter("Test2", [4.0, 5.0, 6.0]);
        assert_eq!(panel.emitter_count(), 2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_music_track_management() {
        let mut panel = AudioPanel::new();
        assert!(panel.music_tracks().is_empty());
        
        panel.add_music_track(MusicTrackEntry {
            name: "Test Track".to_string(),
            path: "/test/path.ogg".to_string(),
            duration_sec: 180.0,
            bpm: Some(120.0),
            mood: MusicMood::Combat,
        });
        
        assert_eq!(panel.music_tracks().len(), 1);
        assert_eq!(panel.music_tracks()[0].name, "Test Track");
        assert_eq!(panel.music_tracks()[0].mood, MusicMood::Combat);
    }

    #[test]
    fn test_music_mood_icons() {
        assert_eq!(MusicMood::Combat.icon(), "⚔️");
        assert_eq!(MusicMood::Ambient.icon(), "🌿");
        assert_eq!(MusicMood::Boss.icon(), "👹");
    }

    #[test]
    fn test_reverb_environment_properties() {
        let cave = ReverbEnvironment::Cave;
        assert_eq!(cave.icon(), "🕳️");
        assert_eq!(cave.decay_time(), 4.0);
        assert_eq!(cave.wet_dry_mix(), 0.6);
    }

    #[test]
    fn test_distance_model_options() {
        let models = DistanceModel::all();
        assert_eq!(models.len(), 3);
        assert!(models.contains(&DistanceModel::Linear));
        assert!(models.contains(&DistanceModel::Inverse));
        assert!(models.contains(&DistanceModel::Exponential));
    }

    #[test]
    fn test_panel_trait_implementation() {
        let panel = AudioPanel::new();
        assert_eq!(panel.name(), "Audio");
    }
}
