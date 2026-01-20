//! Cinematics Panel for the editor UI
//!
//! Provides comprehensive cinematic/cutscene editing:
//! - Timeline editing with multiple tracks
//! - Camera keyframe editor with spline interpolation
//! - Animation track management
//! - Audio clip placement
//! - VFX trigger system
//! - Sequencer playback controls

use egui::{Color32, RichText, Ui, Vec2};

use crate::panels::Panel;

/// Track type for cinematics
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TrackType {
    #[default]
    Camera,
    Animation,
    Audio,
    Fx,
    Dialogue,
    Event,
}

impl TrackType {
    pub fn all() -> &'static [TrackType] {
        &[
            TrackType::Camera,
            TrackType::Animation,
            TrackType::Audio,
            TrackType::Fx,
            TrackType::Dialogue,
            TrackType::Event,
        ]
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TrackType::Camera => "📷",
            TrackType::Animation => "🎬",
            TrackType::Audio => "🔊",
            TrackType::Fx => "✨",
            TrackType::Dialogue => "💬",
            TrackType::Event => "⚡",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            TrackType::Camera => Color32::from_rgb(100, 149, 237),   // Cornflower blue
            TrackType::Animation => Color32::from_rgb(144, 238, 144), // Light green
            TrackType::Audio => Color32::from_rgb(255, 165, 0),       // Orange
            TrackType::Fx => Color32::from_rgb(186, 85, 211),         // Medium orchid
            TrackType::Dialogue => Color32::from_rgb(255, 215, 0),    // Gold
            TrackType::Event => Color32::from_rgb(220, 20, 60),       // Crimson
        }
    }
}

/// Camera interpolation mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CameraInterpolation {
    #[default]
    Linear,
    CatmullRom,
    Bezier,
    Hermite,
    Step,
}

impl CameraInterpolation {
    pub fn all() -> &'static [CameraInterpolation] {
        &[
            CameraInterpolation::Linear,
            CameraInterpolation::CatmullRom,
            CameraInterpolation::Bezier,
            CameraInterpolation::Hermite,
            CameraInterpolation::Step,
        ]
    }
}

/// Playback state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
    Recording,
}

/// Playback speed
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PlaybackSpeed {
    Quarter,
    Half,
    #[default]
    Normal,
    Double,
    Quadruple,
}

impl PlaybackSpeed {
    pub fn all() -> &'static [PlaybackSpeed] {
        &[
            PlaybackSpeed::Quarter,
            PlaybackSpeed::Half,
            PlaybackSpeed::Normal,
            PlaybackSpeed::Double,
            PlaybackSpeed::Quadruple,
        ]
    }

    pub fn multiplier(&self) -> f32 {
        match self {
            PlaybackSpeed::Quarter => 0.25,
            PlaybackSpeed::Half => 0.5,
            PlaybackSpeed::Normal => 1.0,
            PlaybackSpeed::Double => 2.0,
            PlaybackSpeed::Quadruple => 4.0,
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            PlaybackSpeed::Quarter => "0.25×",
            PlaybackSpeed::Half => "0.5×",
            PlaybackSpeed::Normal => "1×",
            PlaybackSpeed::Double => "2×",
            PlaybackSpeed::Quadruple => "4×",
        }
    }
}

/// Camera keyframe
#[derive(Debug, Clone)]
pub struct CameraKeyframe {
    pub time: f32,
    pub position: (f32, f32, f32),
    pub look_at: (f32, f32, f32),
    pub fov: f32,
    pub roll: f32,
}

impl Default for CameraKeyframe {
    fn default() -> Self {
        Self {
            time: 0.0,
            position: (0.0, 5.0, -10.0),
            look_at: (0.0, 0.0, 0.0),
            fov: 60.0,
            roll: 0.0,
        }
    }
}

/// Track entry
#[derive(Debug, Clone)]
pub struct TrackEntry {
    pub id: u32,
    pub name: String,
    pub track_type: TrackType,
    pub start_time: f32,
    pub duration: f32,
    pub muted: bool,
    pub locked: bool,
}

impl Default for TrackEntry {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Track".to_string(),
            track_type: TrackType::Camera,
            start_time: 0.0,
            duration: 5.0,
            muted: false,
            locked: false,
        }
    }
}

/// Clip on a track
#[derive(Debug, Clone)]
pub struct TrackClip {
    pub id: u32,
    pub name: String,
    pub start_time: f32,
    pub duration: f32,
    pub color: Color32,
    pub data: ClipData,
}

/// Clip-specific data
#[derive(Debug, Clone)]
pub enum ClipData {
    Camera { keyframes: Vec<CameraKeyframe> },
    Animation { target_id: u32, clip_name: String },
    Audio { file: String, volume: f32, fade_in: f32, fade_out: f32 },
    Fx { effect_name: String, params: String },
    Dialogue { speaker: String, text: String, duration: f32 },
    Event { event_name: String, payload: String },
}

impl Default for ClipData {
    fn default() -> Self {
        ClipData::Camera { keyframes: Vec::new() }
    }
}

/// Timeline settings
#[derive(Debug, Clone)]
pub struct TimelineSettings {
    pub duration: f32,
    pub framerate: f32,
    pub snap_to_frame: bool,
    pub show_markers: bool,
    pub loop_playback: bool,
    pub zoom_level: f32,
    pub scroll_offset: f32,
}

impl Default for TimelineSettings {
    fn default() -> Self {
        Self {
            duration: 30.0,
            framerate: 30.0,
            snap_to_frame: true,
            show_markers: true,
            loop_playback: false,
            zoom_level: 1.0,
            scroll_offset: 0.0,
        }
    }
}

/// Marker on timeline
#[derive(Debug, Clone)]
pub struct TimelineMarker {
    pub time: f32,
    pub name: String,
    pub color: Color32,
}

/// Panel tabs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CinematicsTab {
    #[default]
    Timeline,
    Camera,
    Tracks,
    Clips,
    Preview,
    Export,
}

/// Main Cinematics Panel
pub struct CinematicsPanel {
    // Tab state
    active_tab: CinematicsTab,

    // Timeline data
    current_time: f32,
    playback_state: PlaybackState,
    playback_speed: PlaybackSpeed,
    settings: TimelineSettings,

    // Tracks
    tracks: Vec<TrackEntry>,
    clips: Vec<TrackClip>,
    markers: Vec<TimelineMarker>,

    // Selection
    selected_track: Option<u32>,
    selected_clip: Option<u32>,
    selected_keyframe: Option<usize>,

    // Camera editing
    camera_interpolation: CameraInterpolation,
    camera_keyframes: Vec<CameraKeyframe>,
    preview_camera: bool,

    // Editing state
    editing_clip: Option<u32>,
    dragging_clip: bool,
    drag_offset: f32,

    // Timeline ID counter
    next_id: u32,
}

impl Default for CinematicsPanel {
    fn default() -> Self {
        let mut panel = Self {
            active_tab: CinematicsTab::Timeline,

            current_time: 0.0,
            playback_state: PlaybackState::Stopped,
            playback_speed: PlaybackSpeed::Normal,
            settings: TimelineSettings::default(),

            tracks: Vec::new(),
            clips: Vec::new(),
            markers: Vec::new(),

            selected_track: None,
            selected_clip: None,
            selected_keyframe: None,

            camera_interpolation: CameraInterpolation::CatmullRom,
            camera_keyframes: Vec::new(),
            preview_camera: false,

            editing_clip: None,
            dragging_clip: false,
            drag_offset: 0.0,

            next_id: 1,
        };

        // Add default tracks
        panel.add_default_tracks();

        panel
    }
}

impl CinematicsPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_default_tracks(&mut self) {
        // Camera track
        let camera_id = self.next_id();
        self.tracks.push(TrackEntry {
            id: camera_id,
            name: "Main Camera".to_string(),
            track_type: TrackType::Camera,
            start_time: 0.0,
            duration: 30.0,
            muted: false,
            locked: false,
        });

        // Animation track
        let anim_id = self.next_id();
        self.tracks.push(TrackEntry {
            id: anim_id,
            name: "Character Animation".to_string(),
            track_type: TrackType::Animation,
            start_time: 0.0,
            duration: 30.0,
            muted: false,
            locked: false,
        });

        // Audio track
        let audio_id = self.next_id();
        self.tracks.push(TrackEntry {
            id: audio_id,
            name: "Music".to_string(),
            track_type: TrackType::Audio,
            start_time: 0.0,
            duration: 30.0,
            muted: false,
            locked: false,
        });

        // Add sample keyframes
        self.camera_keyframes.push(CameraKeyframe {
            time: 0.0,
            position: (0.0, 5.0, -15.0),
            look_at: (0.0, 0.0, 0.0),
            fov: 60.0,
            roll: 0.0,
        });

        self.camera_keyframes.push(CameraKeyframe {
            time: 5.0,
            position: (10.0, 8.0, -10.0),
            look_at: (0.0, 2.0, 0.0),
            fov: 50.0,
            roll: 0.0,
        });
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (CinematicsTab::Timeline, "📐 Timeline"),
                (CinematicsTab::Camera, "📷 Camera"),
                (CinematicsTab::Tracks, "🎚️ Tracks"),
                (CinematicsTab::Clips, "🎬 Clips"),
                (CinematicsTab::Preview, "👁️ Preview"),
                (CinematicsTab::Export, "📦 Export"),
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

        // Playback controls
        ui.horizontal(|ui| {
            let state_icon = match self.playback_state {
                PlaybackState::Stopped => "⏹",
                PlaybackState::Playing => "▶️",
                PlaybackState::Paused => "⏸️",
                PlaybackState::Recording => "🔴",
            };
            ui.label(state_icon);

            // Transport controls
            if ui.button("⏮").clicked() {
                self.current_time = 0.0;
            }
            if ui.button("◀").clicked() {
                self.current_time = (self.current_time - 1.0).max(0.0);
            }
            if ui.button(if self.playback_state == PlaybackState::Playing { "⏸️" } else { "▶️" }).clicked() {
                self.playback_state = if self.playback_state == PlaybackState::Playing {
                    PlaybackState::Paused
                } else {
                    PlaybackState::Playing
                };
            }
            if ui.button("▶").clicked() {
                self.current_time = (self.current_time + 1.0).min(self.settings.duration);
            }
            if ui.button("⏭").clicked() {
                self.current_time = self.settings.duration;
            }
            if ui.button("⏹").clicked() {
                self.playback_state = PlaybackState::Stopped;
                self.current_time = 0.0;
            }

            ui.separator();

            // Time display
            let frame = (self.current_time * self.settings.framerate) as u32;
            let total_frames = (self.settings.duration * self.settings.framerate) as u32;
            ui.label(format!(
                "{:.2}s / {:.2}s (F{}/{})",
                self.current_time, self.settings.duration, frame, total_frames
            ));

            ui.separator();

            // Speed selector
            ui.label("Speed:");
            egui::ComboBox::from_id_salt("playback_speed")
                .selected_text(self.playback_speed.display())
                .show_ui(ui, |ui| {
                    for speed in PlaybackSpeed::all() {
                        ui.selectable_value(&mut self.playback_speed, *speed, speed.display());
                    }
                });

            // Loop toggle
            ui.checkbox(&mut self.settings.loop_playback, "🔁 Loop");
        });

        ui.separator();
    }

    fn show_timeline_tab(&mut self, ui: &mut Ui) {
        ui.heading("📐 Timeline Editor");
        ui.add_space(5.0);

        // Timeline settings bar
        ui.horizontal(|ui| {
            ui.label("Duration:");
            ui.add(
                egui::DragValue::new(&mut self.settings.duration)
                    .speed(0.1)
                    .range(1.0..=600.0)
                    .suffix("s"),
            );

            ui.separator();

            ui.label("FPS:");
            ui.add(
                egui::DragValue::new(&mut self.settings.framerate)
                    .speed(1.0)
                    .range(1.0..=120.0),
            );

            ui.separator();

            ui.label("Zoom:");
            ui.add(
                egui::Slider::new(&mut self.settings.zoom_level, 0.1..=5.0)
                    .show_value(false),
            );

            ui.checkbox(&mut self.settings.snap_to_frame, "Snap");
            ui.checkbox(&mut self.settings.show_markers, "Markers");
        });

        ui.add_space(10.0);

        // Timeline view
        self.draw_timeline(ui);

        ui.add_space(10.0);

        // Track list
        ui.group(|ui| {
            ui.label(RichText::new("Tracks").strong());

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let track_ids: Vec<u32> = self.tracks.iter().map(|t| t.id).collect();

                    for track_id in track_ids {
                        if let Some(track) = self.tracks.iter_mut().find(|t| t.id == track_id) {
                            let is_selected = self.selected_track == Some(track.id);

                            ui.horizontal(|ui| {
                                // Track type icon
                                ui.label(track.track_type.icon());

                                // Track name
                                if ui
                                    .selectable_label(is_selected, &track.name)
                                    .clicked()
                                {
                                    self.selected_track = Some(track.id);
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Mute/Solo/Lock buttons
                                    if ui.small_button(if track.locked { "🔒" } else { "🔓" }).clicked() {
                                        track.locked = !track.locked;
                                    }
                                    if ui.small_button(if track.muted { "🔇" } else { "🔊" }).clicked() {
                                        track.muted = !track.muted;
                                    }
                                });
                            });
                        }
                    }
                });
        });
    }

    fn draw_timeline(&mut self, ui: &mut Ui) {
        let available_width = ui.available_width();
        let timeline_height = 80.0;

        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(available_width, timeline_height),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, 5.0, Color32::from_rgb(30, 30, 35));

        // Time ruler
        let ruler_height = 20.0;
        let ruler_rect = egui::Rect::from_min_size(
            rect.min,
            Vec2::new(rect.width(), ruler_height),
        );
        painter.rect_filled(ruler_rect, 0.0, Color32::from_rgb(40, 40, 45));

        // Draw time markers
        let pixels_per_second = (rect.width() / self.settings.duration) * self.settings.zoom_level;
        let marker_interval = if pixels_per_second > 100.0 {
            0.5
        } else if pixels_per_second > 50.0 {
            1.0
        } else if pixels_per_second > 20.0 {
            5.0
        } else {
            10.0
        };

        let mut t = 0.0;
        while t <= self.settings.duration {
            let x = rect.min.x + (t / self.settings.duration) * rect.width();

            // Tick mark
            let tick_height = if (t % 5.0).abs() < 0.01 { 15.0 } else { 8.0 };
            painter.line_segment(
                [egui::Pos2::new(x, rect.min.y), egui::Pos2::new(x, rect.min.y + tick_height)],
                egui::Stroke::new(1.0, Color32::GRAY),
            );

            // Time label (every 5 seconds)
            if (t % 5.0).abs() < 0.01 {
                painter.text(
                    egui::Pos2::new(x + 2.0, rect.min.y + 5.0),
                    egui::Align2::LEFT_TOP,
                    format!("{:.0}s", t),
                    egui::FontId::proportional(10.0),
                    Color32::GRAY,
                );
            }

            t += marker_interval;
        }

        // Draw playhead
        let playhead_x = rect.min.x + (self.current_time / self.settings.duration) * rect.width();
        painter.line_segment(
            [egui::Pos2::new(playhead_x, rect.min.y), egui::Pos2::new(playhead_x, rect.max.y)],
            egui::Stroke::new(2.0, Color32::from_rgb(255, 100, 100)),
        );

        // Playhead handle
        let handle_points = [
            egui::Pos2::new(playhead_x - 6.0, rect.min.y),
            egui::Pos2::new(playhead_x + 6.0, rect.min.y),
            egui::Pos2::new(playhead_x, rect.min.y + 10.0),
        ];
        painter.add(egui::Shape::convex_polygon(
            handle_points.to_vec(),
            Color32::from_rgb(255, 100, 100),
            egui::Stroke::NONE,
        ));

        // Handle click/drag on timeline
        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let normalized_x = (pos.x - rect.min.x) / rect.width();
                self.current_time = (normalized_x * self.settings.duration).clamp(0.0, self.settings.duration);

                if self.settings.snap_to_frame {
                    let frame = (self.current_time * self.settings.framerate).round();
                    self.current_time = frame / self.settings.framerate;
                }
            }
        }

        // Draw markers
        if self.settings.show_markers {
            for marker in &self.markers {
                let x = rect.min.x + (marker.time / self.settings.duration) * rect.width();
                painter.line_segment(
                    [egui::Pos2::new(x, rect.min.y + ruler_height), egui::Pos2::new(x, rect.max.y)],
                    egui::Stroke::new(1.0, marker.color),
                );
            }
        }
    }

    fn show_camera_tab(&mut self, ui: &mut Ui) {
        ui.heading("📷 Camera Editor");
        ui.add_space(10.0);

        // Interpolation mode
        ui.horizontal(|ui| {
            ui.label("Interpolation:");
            for interp in CameraInterpolation::all() {
                if ui
                    .selectable_label(self.camera_interpolation == *interp, format!("{:?}", interp))
                    .clicked()
                {
                    self.camera_interpolation = *interp;
                }
            }
        });

        ui.checkbox(&mut self.preview_camera, "Preview camera in viewport");

        ui.add_space(10.0);

        // Keyframes
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Keyframes").strong());
                if ui.button("+ Add Keyframe").clicked() {
                    self.camera_keyframes.push(CameraKeyframe {
                        time: self.current_time,
                        ..Default::default()
                    });
                    self.camera_keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
                }
            });

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let mut to_remove: Option<usize> = None;

                    for (idx, keyframe) in self.camera_keyframes.iter_mut().enumerate() {
                        let is_selected = self.selected_keyframe == Some(idx);

                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(is_selected, format!("🔑 {:.2}s", keyframe.time))
                                .clicked()
                            {
                                self.selected_keyframe = Some(idx);
                                self.current_time = keyframe.time;
                            }

                            ui.label(format!(
                                "Pos: ({:.1}, {:.1}, {:.1})",
                                keyframe.position.0, keyframe.position.1, keyframe.position.2
                            ));

                            if ui.small_button("🗑").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    }

                    if let Some(idx) = to_remove {
                        self.camera_keyframes.remove(idx);
                        self.selected_keyframe = None;
                    }
                });
        });

        // Selected keyframe editor
        if let Some(idx) = self.selected_keyframe {
            if idx < self.camera_keyframes.len() {
                ui.add_space(10.0);
                ui.group(|ui| {
                    ui.label(RichText::new("Edit Keyframe").strong());

                    let keyframe = &mut self.camera_keyframes[idx];

                    ui.horizontal(|ui| {
                        ui.label("Time:");
                        ui.add(
                            egui::DragValue::new(&mut keyframe.time)
                                .speed(0.01)
                                .range(0.0..=self.settings.duration)
                                .suffix("s"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.add(egui::DragValue::new(&mut keyframe.position.0).speed(0.1).prefix("X: "));
                        ui.add(egui::DragValue::new(&mut keyframe.position.1).speed(0.1).prefix("Y: "));
                        ui.add(egui::DragValue::new(&mut keyframe.position.2).speed(0.1).prefix("Z: "));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Look At:");
                        ui.add(egui::DragValue::new(&mut keyframe.look_at.0).speed(0.1).prefix("X: "));
                        ui.add(egui::DragValue::new(&mut keyframe.look_at.1).speed(0.1).prefix("Y: "));
                        ui.add(egui::DragValue::new(&mut keyframe.look_at.2).speed(0.1).prefix("Z: "));
                    });

                    ui.horizontal(|ui| {
                        ui.label("FOV:");
                        ui.add(
                            egui::Slider::new(&mut keyframe.fov, 10.0..=120.0)
                                .suffix("°"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Roll:");
                        ui.add(
                            egui::Slider::new(&mut keyframe.roll, -180.0..=180.0)
                                .suffix("°"),
                        );
                    });
                });
            }
        }
    }

    fn show_tracks_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎚️ Track Management");
        ui.add_space(10.0);

        // Add track buttons
        let mut new_track: Option<(TrackType, f32)> = None;
        ui.horizontal(|ui| {
            ui.label("Add Track:");
            for track_type in TrackType::all() {
                if ui.button(format!("{} {:?}", track_type.icon(), track_type)).clicked() {
                    new_track = Some((*track_type, self.settings.duration));
                }
            }
        });

        if let Some((track_type, duration)) = new_track {
            let id = self.next_id();
            self.tracks.push(TrackEntry {
                id,
                name: format!("New {:?}", track_type),
                track_type,
                start_time: 0.0,
                duration,
                muted: false,
                locked: false,
            });
        }

        ui.add_space(10.0);

        // Track list with details
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                let track_ids: Vec<u32> = self.tracks.iter().map(|t| t.id).collect();

                for track_id in track_ids {
                    if let Some(track) = self.tracks.iter_mut().find(|t| t.id == track_id) {
                        let is_selected = self.selected_track == Some(track.id);

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                // Color indicator
                                let color_rect = egui::Rect::from_min_size(
                                    ui.cursor().min,
                                    Vec2::new(4.0, 20.0),
                                );
                                ui.painter().rect_filled(color_rect, 2.0, track.track_type.color());
                                ui.add_space(8.0);

                                // Track icon
                                ui.label(track.track_type.icon());

                                // Track name (editable)
                                if ui
                                    .selectable_label(is_selected, &track.name)
                                    .clicked()
                                {
                                    self.selected_track = Some(track.id);
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Delete button
                                    if ui.small_button("🗑").clicked() {
                                        // Mark for deletion
                                    }

                                    // Lock button
                                    if ui.small_button(if track.locked { "🔒" } else { "🔓" }).clicked() {
                                        track.locked = !track.locked;
                                    }

                                    // Mute button
                                    if ui.small_button(if track.muted { "🔇" } else { "🔊" }).clicked() {
                                        track.muted = !track.muted;
                                    }
                                });
                            });

                            // Track details when selected
                            if is_selected {
                                ui.add_space(5.0);
                                egui::Grid::new(format!("track_details_{}", track.id))
                                    .num_columns(2)
                                    .spacing([10.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Name:");
                                        ui.text_edit_singleline(&mut track.name);
                                        ui.end_row();

                                        ui.label("Start:");
                                        ui.add(
                                            egui::DragValue::new(&mut track.start_time)
                                                .speed(0.1)
                                                .range(0.0..=self.settings.duration)
                                                .suffix("s"),
                                        );
                                        ui.end_row();

                                        ui.label("Duration:");
                                        ui.add(
                                            egui::DragValue::new(&mut track.duration)
                                                .speed(0.1)
                                                .range(0.1..=600.0)
                                                .suffix("s"),
                                        );
                                        ui.end_row();
                                    });
                            }
                        });
                    }
                }
            });
    }

    fn show_clips_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎬 Clip Editor");
        ui.add_space(10.0);

        if self.clips.is_empty() {
            ui.label("No clips yet. Add clips from the track editor.");

            if ui.button("+ Create Sample Clip").clicked() {
                let id = self.next_id();
                let keyframes = self.camera_keyframes.clone();
                self.clips.push(TrackClip {
                    id,
                    name: "Sample Clip".to_string(),
                    start_time: 0.0,
                    duration: 5.0,
                    color: Color32::from_rgb(100, 149, 237),
                    data: ClipData::Camera {
                        keyframes,
                    },
                });
            }
        } else {
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for clip in &self.clips {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(&clip.name);
                                ui.label(format!("{:.1}s - {:.1}s", clip.start_time, clip.start_time + clip.duration));
                            });
                        });
                    }
                });
        }
    }

    fn show_preview_tab(&mut self, ui: &mut Ui) {
        ui.heading("👁️ Preview");
        ui.add_space(10.0);

        // Preview viewport placeholder
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 200.0),
            egui::Sense::hover(),
        );

        ui.painter().rect_filled(rect, 5.0, Color32::from_rgb(20, 20, 25));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Camera Preview",
            egui::FontId::proportional(16.0),
            Color32::GRAY,
        );

        ui.add_space(10.0);

        // Current camera info
        ui.group(|ui| {
            ui.label(RichText::new("Current Camera State").strong());

            if let Some(keyframe) = self.get_interpolated_camera() {
                egui::Grid::new("camera_preview_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Position:");
                        ui.label(format!(
                            "({:.2}, {:.2}, {:.2})",
                            keyframe.position.0, keyframe.position.1, keyframe.position.2
                        ));
                        ui.end_row();

                        ui.label("Look At:");
                        ui.label(format!(
                            "({:.2}, {:.2}, {:.2})",
                            keyframe.look_at.0, keyframe.look_at.1, keyframe.look_at.2
                        ));
                        ui.end_row();

                        ui.label("FOV:");
                        ui.label(format!("{:.1}°", keyframe.fov));
                        ui.end_row();

                        ui.label("Roll:");
                        ui.label(format!("{:.1}°", keyframe.roll));
                        ui.end_row();
                    });
            } else {
                ui.label("No keyframes to interpolate");
            }
        });
    }

    fn show_export_tab(&mut self, ui: &mut Ui) {
        ui.heading("📦 Export");
        ui.add_space(10.0);

        // Export format
        ui.group(|ui| {
            ui.label(RichText::new("Export Format").strong());

            ui.horizontal(|ui| {
                if ui.button("JSON Timeline").clicked() {
                    // Export as JSON
                }
                if ui.button("RON Format").clicked() {
                    // Export as RON
                }
                if ui.button("Binary Pack").clicked() {
                    // Export as binary
                }
            });
        });

        ui.add_space(10.0);

        // Export options
        ui.group(|ui| {
            ui.label(RichText::new("Export Options").strong());

            ui.checkbox(&mut self.settings.loop_playback, "Include loop markers");
            ui.checkbox(&mut self.preview_camera, "Include preview data");
        });

        ui.add_space(10.0);

        // Statistics
        ui.group(|ui| {
            ui.label(RichText::new("Timeline Statistics").strong());

            egui::Grid::new("export_stats_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Duration:");
                    ui.label(format!("{:.2}s", self.settings.duration));
                    ui.end_row();

                    ui.label("Tracks:");
                    ui.label(format!("{}", self.tracks.len()));
                    ui.end_row();

                    ui.label("Clips:");
                    ui.label(format!("{}", self.clips.len()));
                    ui.end_row();

                    ui.label("Camera Keyframes:");
                    ui.label(format!("{}", self.camera_keyframes.len()));
                    ui.end_row();

                    ui.label("Markers:");
                    ui.label(format!("{}", self.markers.len()));
                    ui.end_row();

                    ui.label("Total Frames:");
                    ui.label(format!("{}", (self.settings.duration * self.settings.framerate) as u32));
                    ui.end_row();
                });
        });
    }

    fn get_interpolated_camera(&self) -> Option<CameraKeyframe> {
        if self.camera_keyframes.is_empty() {
            return None;
        }

        if self.camera_keyframes.len() == 1 {
            return Some(self.camera_keyframes[0].clone());
        }

        // Find surrounding keyframes
        let mut before: Option<&CameraKeyframe> = None;
        let mut after: Option<&CameraKeyframe> = None;

        for kf in &self.camera_keyframes {
            if kf.time <= self.current_time {
                before = Some(kf);
            }
            if kf.time >= self.current_time && after.is_none() {
                after = Some(kf);
            }
        }

        match (before, after) {
            (Some(b), Some(a)) if b.time != a.time => {
                let t = (self.current_time - b.time) / (a.time - b.time);
                Some(CameraKeyframe {
                    time: self.current_time,
                    position: (
                        b.position.0 + (a.position.0 - b.position.0) * t,
                        b.position.1 + (a.position.1 - b.position.1) * t,
                        b.position.2 + (a.position.2 - b.position.2) * t,
                    ),
                    look_at: (
                        b.look_at.0 + (a.look_at.0 - b.look_at.0) * t,
                        b.look_at.1 + (a.look_at.1 - b.look_at.1) * t,
                        b.look_at.2 + (a.look_at.2 - b.look_at.2) * t,
                    ),
                    fov: b.fov + (a.fov - b.fov) * t,
                    roll: b.roll + (a.roll - b.roll) * t,
                })
            }
            (Some(kf), _) | (_, Some(kf)) => Some(kf.clone()),
            _ => None,
        }
    }

    // Getters for testing
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    pub fn playback_state(&self) -> PlaybackState {
        self.playback_state
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    pub fn keyframe_count(&self) -> usize {
        self.camera_keyframes.len()
    }

    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    pub fn duration(&self) -> f32 {
        self.settings.duration
    }

    pub fn framerate(&self) -> f32 {
        self.settings.framerate
    }

    pub fn set_current_time(&mut self, time: f32) {
        self.current_time = time.clamp(0.0, self.settings.duration);
    }

    pub fn set_playback_state(&mut self, state: PlaybackState) {
        self.playback_state = state;
    }

    pub fn add_track(&mut self, track_type: TrackType, name: &str) -> u32 {
        let id = self.next_id();
        self.tracks.push(TrackEntry {
            id,
            name: name.to_string(),
            track_type,
            start_time: 0.0,
            duration: self.settings.duration,
            muted: false,
            locked: false,
        });
        id
    }

    pub fn add_camera_keyframe(&mut self, keyframe: CameraKeyframe) {
        self.camera_keyframes.push(keyframe);
        self.camera_keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    }

    pub fn add_marker(&mut self, time: f32, name: &str, color: Color32) {
        self.markers.push(TimelineMarker {
            time,
            name: name.to_string(),
            color,
        });
    }

    pub fn marker_count(&self) -> usize {
        self.markers.len()
    }
}

impl Panel for CinematicsPanel {
    fn name(&self) -> &'static str {
        "Cinematics"
    }

    fn show(&mut self, ui: &mut Ui) {
        self.show_tab_bar(ui);

        match self.active_tab {
            CinematicsTab::Timeline => self.show_timeline_tab(ui),
            CinematicsTab::Camera => self.show_camera_tab(ui),
            CinematicsTab::Tracks => self.show_tracks_tab(ui),
            CinematicsTab::Clips => self.show_clips_tab(ui),
            CinematicsTab::Preview => self.show_preview_tab(ui),
            CinematicsTab::Export => self.show_export_tab(ui),
        }
    }

    fn update(&mut self) {
        // Update playback
        if self.playback_state == PlaybackState::Playing {
            let dt = 1.0 / 60.0; // Assume 60 FPS update rate
            self.current_time += dt * self.playback_speed.multiplier();

            if self.current_time >= self.settings.duration {
                if self.settings.loop_playback {
                    self.current_time = 0.0;
                } else {
                    self.current_time = self.settings.duration;
                    self.playback_state = PlaybackState::Stopped;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cinematics_panel_creation() {
        let panel = CinematicsPanel::new();
        assert_eq!(panel.playback_state(), PlaybackState::Stopped);
        assert_eq!(panel.current_time(), 0.0);
    }

    #[test]
    fn test_default_tracks() {
        let panel = CinematicsPanel::new();
        assert_eq!(panel.track_count(), 3); // Camera, Animation, Audio
    }

    #[test]
    fn test_default_keyframes() {
        let panel = CinematicsPanel::new();
        assert_eq!(panel.keyframe_count(), 2); // Two sample keyframes
    }

    #[test]
    fn test_playback_state() {
        let mut panel = CinematicsPanel::new();
        panel.set_playback_state(PlaybackState::Playing);
        assert_eq!(panel.playback_state(), PlaybackState::Playing);

        panel.set_playback_state(PlaybackState::Paused);
        assert_eq!(panel.playback_state(), PlaybackState::Paused);
    }

    #[test]
    fn test_current_time() {
        let mut panel = CinematicsPanel::new();
        panel.set_current_time(10.0);
        assert_eq!(panel.current_time(), 10.0);

        // Test clamping
        panel.set_current_time(100.0);
        assert_eq!(panel.current_time(), panel.duration());
    }

    #[test]
    fn test_add_track() {
        let mut panel = CinematicsPanel::new();
        let initial_count = panel.track_count();

        let id = panel.add_track(TrackType::Fx, "Explosion FX");
        assert!(id > 0);
        assert_eq!(panel.track_count(), initial_count + 1);
    }

    #[test]
    fn test_add_camera_keyframe() {
        let mut panel = CinematicsPanel::new();
        let initial_count = panel.keyframe_count();

        panel.add_camera_keyframe(CameraKeyframe {
            time: 15.0,
            ..Default::default()
        });

        assert_eq!(panel.keyframe_count(), initial_count + 1);
    }

    #[test]
    fn test_add_marker() {
        let mut panel = CinematicsPanel::new();
        assert_eq!(panel.marker_count(), 0);

        panel.add_marker(5.0, "Start Scene", Color32::RED);
        assert_eq!(panel.marker_count(), 1);
    }

    #[test]
    fn test_playback_speed() {
        assert_eq!(PlaybackSpeed::Normal.multiplier(), 1.0);
        assert_eq!(PlaybackSpeed::Double.multiplier(), 2.0);
        assert_eq!(PlaybackSpeed::Half.multiplier(), 0.5);
    }

    #[test]
    fn test_track_type_properties() {
        assert_eq!(TrackType::Camera.icon(), "📷");
        assert_eq!(TrackType::Audio.icon(), "🔊");
        assert_eq!(TrackType::Fx.color(), Color32::from_rgb(186, 85, 211));
    }

    #[test]
    fn test_timeline_settings() {
        let panel = CinematicsPanel::new();
        assert_eq!(panel.duration(), 30.0);
        assert_eq!(panel.framerate(), 30.0);
    }

    #[test]
    fn test_panel_trait_implementation() {
        let panel = CinematicsPanel::new();
        assert_eq!(panel.name(), "Cinematics");
    }
}
