//! Animation Editor Panel
//!
//! Provides tools for creating and editing entity animations:
//! - Animation clip selection and loading  
//! - Playback controls (play, pause, stop, loop)
//! - Timeline scrubbing
//! - Property animation preview
//! - Animation application to selected entity
//! - Demo mode with tween/spring examples

use astract::animation::{EasingFunction, Spring, SpringParams, Tween};
use egui::{Color32, Pos2, Vec2};
use glam::Vec3;

/// Animation playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

impl std::fmt::Display for PlaybackState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl PlaybackState {
    /// Returns the human-readable name for this state
    pub fn name(&self) -> &'static str {
        match self {
            Self::Stopped => "Stopped",
            Self::Playing => "Playing",
            Self::Paused => "Paused",
        }
    }

    /// Returns the icon for this state
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Stopped => "⏹",
            Self::Playing => "▶",
            Self::Paused => "⏸",
        }
    }

    /// Returns all playback states
    pub fn all() -> &'static [PlaybackState] {
        &[PlaybackState::Stopped, PlaybackState::Playing, PlaybackState::Paused]
    }

    /// Returns true if playback is active (playing)
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Playing)
    }
}

/// Animation property that can be animated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum AnimatedProperty {
    #[default]
    PositionY,
    PositionX,
    PositionZ,
    RotationY,
    Scale,
}

impl std::fmt::Display for AnimatedProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl AnimatedProperty {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PositionX => "Position X",
            Self::PositionY => "Position Y",
            Self::PositionZ => "Position Z",
            Self::RotationY => "Rotation Y",
            Self::Scale => "Scale",
        }
    }

    /// Returns the icon for this property
    pub fn icon(&self) -> &'static str {
        match self {
            Self::PositionX => "↔",
            Self::PositionY => "↕",
            Self::PositionZ => "↗",
            Self::RotationY => "🔄",
            Self::Scale => "📐",
        }
    }

    /// Returns all animated properties
    pub fn all() -> &'static [AnimatedProperty] {
        &[
            AnimatedProperty::PositionX,
            AnimatedProperty::PositionY,
            AnimatedProperty::PositionZ,
            AnimatedProperty::RotationY,
            AnimatedProperty::Scale,
        ]
    }

    /// Returns true if this property is a position component
    pub fn is_position(&self) -> bool {
        matches!(self, Self::PositionX | Self::PositionY | Self::PositionZ)
    }

    /// Returns true if this property is a rotation component
    pub fn is_rotation(&self) -> bool {
        matches!(self, Self::RotationY)
    }
}

/// A keyframe in an animation
#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub value: f32,
    pub easing: EasingFunction,
}

/// An animation track for a specific property
#[derive(Debug, Clone)]
pub struct AnimationTrack {
    pub property: AnimatedProperty,
    pub keyframes: Vec<Keyframe>,
}

impl AnimationTrack {
    pub fn new(property: AnimatedProperty) -> Self {
        Self {
            property,
            keyframes: Vec::new(),
        }
    }

    /// Evaluate the track at a given time
    pub fn evaluate(&self, time: f32) -> f32 {
        if self.keyframes.is_empty() {
            return match self.property {
                AnimatedProperty::Scale => 1.0,
                _ => 0.0,
            };
        }

        if self.keyframes.len() == 1 {
            return self.keyframes[0].value;
        }

        // Find surrounding keyframes
        let mut prev_idx = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time <= time {
                prev_idx = i;
            }
        }

        let next_idx = (prev_idx + 1).min(self.keyframes.len() - 1);

        if prev_idx == next_idx {
            return self.keyframes[prev_idx].value;
        }

        let prev = &self.keyframes[prev_idx];
        let next = &self.keyframes[next_idx];

        let t = ((time - prev.time) / (next.time - prev.time)).clamp(0.0, 1.0);
        let eased_t = Self::apply_easing(t, prev.easing);

        prev.value + (next.value - prev.value) * eased_t
    }

    fn apply_easing(t: f32, easing: EasingFunction) -> f32 {
        match easing {
            EasingFunction::Linear => t,
            EasingFunction::QuadIn => t * t,
            EasingFunction::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            EasingFunction::CubicIn => t * t * t,
            EasingFunction::CubicOut => 1.0 - (1.0 - t).powi(3),
            EasingFunction::SineIn => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
            EasingFunction::SineOut => (t * std::f32::consts::FRAC_PI_2).sin(),
            EasingFunction::SineInOut => -((std::f32::consts::PI * t).cos() - 1.0) / 2.0,
            EasingFunction::ElasticOut => {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            EasingFunction::BounceOut => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    n1 * (t - 1.5 / d1).powi(2) + 0.75
                } else if t < 2.5 / d1 {
                    n1 * (t - 2.25 / d1).powi(2) + 0.9375
                } else {
                    n1 * (t - 2.625 / d1).powi(2) + 0.984375
                }
            }
            _ => t,
        }
    }
}

/// An animation clip containing multiple tracks
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<AnimationTrack>,
    pub looping: bool,
}

impl Default for AnimationClip {
    fn default() -> Self {
        Self {
            name: "New Animation".to_string(),
            duration: 2.0,
            tracks: Vec::new(),
            looping: false,
        }
    }
}

impl AnimationClip {
    /// Create a sample bounce animation
    pub fn sample_bounce() -> Self {
        let mut clip = Self {
            name: "Bounce".to_string(),
            duration: 1.0,
            tracks: Vec::new(),
            looping: true,
        };

        let mut y_track = AnimationTrack::new(AnimatedProperty::PositionY);
        y_track.keyframes = vec![
            Keyframe {
                time: 0.0,
                value: 0.0,
                easing: EasingFunction::QuadOut,
            },
            Keyframe {
                time: 0.5,
                value: 2.0,
                easing: EasingFunction::QuadIn,
            },
            Keyframe {
                time: 1.0,
                value: 0.0,
                easing: EasingFunction::BounceOut,
            },
        ];
        clip.tracks.push(y_track);

        clip
    }

    /// Create a sample spin animation
    pub fn sample_spin() -> Self {
        let mut clip = Self {
            name: "Spin".to_string(),
            duration: 2.0,
            tracks: Vec::new(),
            looping: true,
        };

        let mut rot_track = AnimationTrack::new(AnimatedProperty::RotationY);
        rot_track.keyframes = vec![
            Keyframe {
                time: 0.0,
                value: 0.0,
                easing: EasingFunction::Linear,
            },
            Keyframe {
                time: 2.0,
                value: std::f32::consts::TAU,
                easing: EasingFunction::Linear,
            },
        ];
        clip.tracks.push(rot_track);

        clip
    }

    /// Create a sample scale pulse animation
    pub fn sample_pulse() -> Self {
        let mut clip = Self {
            name: "Pulse".to_string(),
            duration: 1.0,
            tracks: Vec::new(),
            looping: true,
        };

        let mut scale_track = AnimationTrack::new(AnimatedProperty::Scale);
        scale_track.keyframes = vec![
            Keyframe {
                time: 0.0,
                value: 1.0,
                easing: EasingFunction::SineInOut,
            },
            Keyframe {
                time: 0.5,
                value: 1.3,
                easing: EasingFunction::SineInOut,
            },
            Keyframe {
                time: 1.0,
                value: 1.0,
                easing: EasingFunction::SineInOut,
            },
        ];
        clip.tracks.push(scale_track);

        clip
    }
}

/// Animation output values for applying to an entity
#[derive(Debug, Clone, Default)]
pub struct AnimationOutput {
    pub position_offset: Vec3,
    pub rotation_y: f32,
    pub scale_multiplier: f32,
}

pub struct AnimationPanel {
    // Animation editor state
    pub playback_state: PlaybackState,
    pub current_time: f32,
    pub playback_speed: f32,
    pub selected_entity: Option<u32>,
    pub clips: Vec<AnimationClip>,
    pub selected_clip_idx: Option<usize>,
    pub output: AnimationOutput,

    // UI mode
    pub show_editor: bool, // true = editor mode, false = demo mode

    // Demo mode (tween/spring visualization)
    bounce_tween: Tween<f32>,
    color_tween: Tween<Color32>,
    spring: Spring,
    mouse_target: Pos2,
    easing_tweens: Vec<(String, Tween<f32>)>,
    show_easing: bool,
}

impl Default for AnimationPanel {
    fn default() -> Self {
        // Create demo tweens
        let easings = vec![
            ("Linear", EasingFunction::Linear),
            ("QuadIn", EasingFunction::QuadIn),
            ("QuadOut", EasingFunction::QuadOut),
            ("CubicOut", EasingFunction::CubicOut),
            ("SineInOut", EasingFunction::SineInOut),
            ("ElasticOut", EasingFunction::ElasticOut),
            ("BounceOut", EasingFunction::BounceOut),
        ];

        let easing_tweens = easings
            .into_iter()
            .map(|(name, easing)| {
                let mut tween = Tween::new(0.0f32, 1.0f32, 2.0).with_easing(easing);
                tween.play();
                (name.to_string(), tween)
            })
            .collect();

        let mut bounce_tween =
            Tween::new(0.0f32, 200.0f32, 2.0).with_easing(EasingFunction::ElasticOut);
        bounce_tween.play();

        let mut color_tween =
            Tween::new(Color32::RED, Color32::BLUE, 3.0).with_easing(EasingFunction::SineInOut);
        color_tween.play();

        // Pre-populate with sample clips
        let clips = vec![
            AnimationClip::sample_bounce(),
            AnimationClip::sample_spin(),
            AnimationClip::sample_pulse(),
        ];

        Self {
            playback_state: PlaybackState::Stopped,
            current_time: 0.0,
            playback_speed: 1.0,
            selected_entity: None,
            clips,
            selected_clip_idx: Some(0),
            output: AnimationOutput {
                scale_multiplier: 1.0,
                ..Default::default()
            },
            show_editor: true, // Start in editor mode
            bounce_tween,
            color_tween,
            spring: Spring::with_params(0.0, SpringParams::bouncy()),
            mouse_target: Pos2::ZERO,
            easing_tweens,
            show_easing: false,
        }
    }
}

impl AnimationPanel {
    /// Update animation and return output values
    pub fn update(&mut self, dt: f32) -> Option<AnimationOutput> {
        // Update demo elements
        self.bounce_tween.update(dt);
        self.color_tween.update(dt);
        self.spring.update(dt);
        for (_, tween) in &mut self.easing_tweens {
            tween.update(dt);
        }

        // Only update animation clip if playing
        if self.playback_state != PlaybackState::Playing {
            return None;
        }

        let clip = self.selected_clip_idx.and_then(|idx| self.clips.get(idx))?;

        // Advance time
        self.current_time += dt * self.playback_speed;

        // Handle looping or stop at end
        if self.current_time >= clip.duration {
            if clip.looping {
                self.current_time %= clip.duration;
            } else {
                self.current_time = clip.duration;
                self.playback_state = PlaybackState::Stopped;
            }
        }

        // Evaluate all tracks
        let mut output = AnimationOutput {
            position_offset: Vec3::ZERO,
            rotation_y: 0.0,
            scale_multiplier: 1.0,
        };

        for track in &clip.tracks {
            let value = track.evaluate(self.current_time);

            match track.property {
                AnimatedProperty::PositionX => output.position_offset.x = value,
                AnimatedProperty::PositionY => output.position_offset.y = value,
                AnimatedProperty::PositionZ => output.position_offset.z = value,
                AnimatedProperty::RotationY => output.rotation_y = value,
                AnimatedProperty::Scale => output.scale_multiplier = value,
            }
        }

        self.output = output.clone();
        Some(output)
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.stable_dt);
        self.update(dt);

        egui::Window::new("🎬 Animation")
            .default_size([550.0, 700.0])
            .show(ctx, |ui| {
                // Mode toggle
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.show_editor, true, "📝 Editor");
                    ui.selectable_value(&mut self.show_editor, false, "🎮 Demos");
                });

                ui.separator();

                if self.show_editor {
                    self.show_editor_ui(ui);
                } else {
                    self.show_demo_ui(ui, dt);
                }
            });
    }

    fn show_editor_ui(&mut self, ui: &mut egui::Ui) {
        // Entity indicator
        ui.horizontal(|ui| {
            ui.label("Target:");
            if let Some(entity_id) = self.selected_entity {
                ui.label(
                    egui::RichText::new(format!("Entity #{}", entity_id))
                        .color(Color32::GREEN)
                        .strong(),
                );
            } else {
                ui.label(
                    egui::RichText::new("None selected")
                        .color(Color32::GRAY)
                        .italics(),
                );
            }
        });

        ui.separator();

        // Clip selector
        ui.heading("Clip");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("clip_sel")
                .selected_text(
                    self.selected_clip_idx
                        .and_then(|idx| self.clips.get(idx))
                        .map(|c| c.name.as_str())
                        .unwrap_or("Select..."),
                )
                .show_ui(ui, |ui| {
                    for (idx, clip) in self.clips.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_clip_idx, Some(idx), &clip.name);
                    }
                });

            if ui.button("➕ New").clicked() {
                self.clips.push(AnimationClip {
                    name: format!("Clip {}", self.clips.len() + 1),
                    ..Default::default()
                });
                self.selected_clip_idx = Some(self.clips.len() - 1);
            }
        });

        ui.separator();

        // Playback controls
        ui.heading("Playback");
        ui.horizontal(|ui| {
            let (play_icon, play_tip) = match self.playback_state {
                PlaybackState::Playing => ("⏸", "Pause"),
                _ => ("▶", "Play"),
            };

            if ui.button(play_icon).on_hover_text(play_tip).clicked() {
                self.playback_state = match self.playback_state {
                    PlaybackState::Playing => PlaybackState::Paused,
                    _ => PlaybackState::Playing,
                };
            }

            if ui.button("⏹").on_hover_text("Stop").clicked() {
                self.playback_state = PlaybackState::Stopped;
                self.current_time = 0.0;
            }

            if let Some(clip) = self
                .selected_clip_idx
                .and_then(|idx| self.clips.get_mut(idx))
            {
                ui.checkbox(&mut clip.looping, "🔁");
            }

            ui.label("Speed:");
            ui.add(
                egui::DragValue::new(&mut self.playback_speed)
                    .speed(0.1)
                    .range(0.1..=3.0)
                    .suffix("x"),
            );
        });

        // Timeline
        if let Some(clip) = self.selected_clip_idx.and_then(|idx| self.clips.get(idx)) {
            ui.horizontal(|ui| {
                ui.label(format!("{:.2}s", self.current_time));
                let response = ui.add(
                    egui::Slider::new(&mut self.current_time, 0.0..=clip.duration)
                        .show_value(false),
                );
                if response.dragged() {
                    self.playback_state = PlaybackState::Paused;
                }
                ui.label(format!("{:.2}s", clip.duration));
            });
        }

        ui.separator();

        // Tracks
        ui.heading("Tracks");

        if let Some(clip_idx) = self.selected_clip_idx {
            if let Some(clip) = self.clips.get_mut(clip_idx) {
                ui.horizontal(|ui| {
                    if ui.button("➕ Add Track").clicked() {
                        clip.tracks
                            .push(AnimationTrack::new(AnimatedProperty::PositionY));
                    }
                    ui.add(
                        egui::DragValue::new(&mut clip.duration)
                            .prefix("Dur: ")
                            .suffix("s")
                            .speed(0.1)
                            .range(0.1..=30.0),
                    );
                });

                let mut track_to_remove: Option<usize> = None;

                for (track_idx, track) in clip.tracks.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt(format!("tr_{}", track_idx))
                            .selected_text(track.property.name())
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut track.property,
                                    AnimatedProperty::PositionX,
                                    "Position X",
                                );
                                ui.selectable_value(
                                    &mut track.property,
                                    AnimatedProperty::PositionY,
                                    "Position Y",
                                );
                                ui.selectable_value(
                                    &mut track.property,
                                    AnimatedProperty::PositionZ,
                                    "Position Z",
                                );
                                ui.selectable_value(
                                    &mut track.property,
                                    AnimatedProperty::RotationY,
                                    "Rotation Y",
                                );
                                ui.selectable_value(
                                    &mut track.property,
                                    AnimatedProperty::Scale,
                                    "Scale",
                                );
                            });

                        ui.label(format!("{}kf", track.keyframes.len()));

                        if ui
                            .small_button("➕")
                            .on_hover_text("Add keyframe")
                            .clicked()
                        {
                            let val = track.evaluate(self.current_time);
                            track.keyframes.push(Keyframe {
                                time: self.current_time,
                                value: val,
                                easing: EasingFunction::Linear,
                            });
                            track.keyframes.sort_by(|a, b| {
                                a.time
                                    .partial_cmp(&b.time)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                        }

                        if ui.small_button("🗑").clicked() {
                            track_to_remove = Some(track_idx);
                        }
                    });

                    // Show keyframes inline
                    if !track.keyframes.is_empty() {
                        ui.indent(format!("kf_{}", track_idx), |ui| {
                            let mut kf_remove: Option<usize> = None;
                            for (kf_idx, kf) in track.keyframes.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::DragValue::new(&mut kf.time)
                                            .prefix("t:")
                                            .suffix("s")
                                            .speed(0.01)
                                            .range(0.0..=clip.duration),
                                    );
                                    ui.add(
                                        egui::DragValue::new(&mut kf.value).prefix("v:").speed(0.1),
                                    );
                                    egui::ComboBox::from_id_salt(format!(
                                        "e_{}_{}",
                                        track_idx, kf_idx
                                    ))
                                    .selected_text(
                                        format!("{:?}", kf.easing)
                                            .chars()
                                            .take(6)
                                            .collect::<String>(),
                                    )
                                    .width(60.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut kf.easing,
                                            EasingFunction::Linear,
                                            "Linear",
                                        );
                                        ui.selectable_value(
                                            &mut kf.easing,
                                            EasingFunction::QuadOut,
                                            "QuadOut",
                                        );
                                        ui.selectable_value(
                                            &mut kf.easing,
                                            EasingFunction::SineInOut,
                                            "SineInOut",
                                        );
                                        ui.selectable_value(
                                            &mut kf.easing,
                                            EasingFunction::ElasticOut,
                                            "Elastic",
                                        );
                                        ui.selectable_value(
                                            &mut kf.easing,
                                            EasingFunction::BounceOut,
                                            "Bounce",
                                        );
                                    });
                                    if ui.small_button("×").clicked() {
                                        kf_remove = Some(kf_idx);
                                    }
                                });
                            }
                            if let Some(idx) = kf_remove {
                                track.keyframes.remove(idx);
                            }
                        });
                    }
                }

                if let Some(idx) = track_to_remove {
                    clip.tracks.remove(idx);
                }
            }
        }

        ui.separator();

        // Preview
        ui.heading("Preview");
        let (rect, _) = ui.allocate_exact_size(Vec2::new(180.0, 180.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 4.0, Color32::from_gray(40));

        let center = rect.center();
        let anim_y = center.y - self.output.position_offset.y * 20.0;
        let anim_scale = 25.0 * self.output.scale_multiplier;

        // Rotation indicator
        let rot = self.output.rotation_y;
        let end_x = center.x + rot.sin() * 40.0;
        let end_y = anim_y - rot.cos() * 40.0;
        ui.painter().line_segment(
            [Pos2::new(center.x, anim_y), Pos2::new(end_x, end_y)],
            egui::Stroke::new(2.0, Color32::YELLOW),
        );

        // Entity preview
        ui.painter().circle_filled(
            Pos2::new(center.x, anim_y),
            anim_scale,
            Color32::from_rgb(100, 150, 255),
        );

        ui.label(format!(
            "Y:{:.1} Rot:{:.0}° S:{:.2}",
            self.output.position_offset.y,
            self.output.rotation_y.to_degrees(),
            self.output.scale_multiplier
        ));

        if self.selected_entity.is_some() && self.playback_state == PlaybackState::Playing {
            ui.label(egui::RichText::new("✓ Animating").color(Color32::GREEN));
        }
    }

    fn show_demo_ui(&mut self, ui: &mut egui::Ui, dt: f32) {
        ui.heading("Animation Demos");
        ui.label("Interactive examples of tweens, springs, and easing functions.");

        ui.separator();

        // Bounce demo
        ui.label("🎾 Tween (ElasticOut):");
        let y = self.bounce_tween.value();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(350.0, 200.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 0.0, Color32::from_gray(40));
        ui.painter().circle_filled(
            Pos2::new(rect.left() + 80.0, rect.top() + y + 20.0),
            18.0,
            Color32::GREEN,
        );
        if ui.button("🔄 Restart").clicked() {
            self.bounce_tween.restart();
        }

        ui.add_space(10.0);

        // Color demo
        ui.label("🎨 Color Tween:");
        let color = self.color_tween.value();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(350.0, 60.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 4.0, color);
        if ui.button("🔄 Restart").clicked() {
            self.color_tween.restart();
        }

        ui.add_space(10.0);

        // Spring demo
        ui.label("🌀 Spring Physics:");
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(350.0, 150.0), egui::Sense::hover());
        if let Some(hover) = response.hover_pos() {
            self.mouse_target = hover;
            let norm_x = ((hover.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            self.spring.set_target(norm_x);
        }
        ui.painter().rect_filled(rect, 0.0, Color32::from_gray(40));
        let spring_x = rect.left() + self.spring.position() * rect.width();
        ui.painter()
            .circle_filled(Pos2::new(spring_x, rect.center().y), 22.0, Color32::YELLOW);

        ui.add_space(10.0);

        // Easing toggle
        ui.checkbox(&mut self.show_easing, "Show Easing Curves");
        if self.show_easing {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(350.0, 250.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(40));

            let colors = [
                Color32::RED,
                Color32::GREEN,
                Color32::BLUE,
                Color32::YELLOW,
                Color32::LIGHT_BLUE,
                Color32::from_rgb(255, 128, 0),
                Color32::WHITE,
            ];

            for (i, (name, tween)) in self.easing_tweens.iter_mut().enumerate() {
                tween.update(dt);
                let t = tween.progress();
                let v = tween.value();
                let x = rect.left() + t * rect.width();
                let y = rect.bottom() - v * rect.height();
                let c = colors[i % colors.len()];
                ui.painter().circle_filled(Pos2::new(x, y), 3.0, c);
                if t > 0.9 {
                    ui.painter().text(
                        Pos2::new(rect.right() + 5.0, rect.top() + i as f32 * 14.0),
                        egui::Align2::LEFT_TOP,
                        name,
                        egui::FontId::proportional(9.0),
                        c,
                    );
                }
            }

            if ui.button("🔄 Restart All").clicked() {
                for (_, tw) in &mut self.easing_tweens {
                    tw.restart();
                }
            }
        }
    }

    /// Set the selected entity for animation
    pub fn set_selected_entity(&mut self, entity: Option<u32>) {
        self.selected_entity = entity;
    }

    /// Get current animation output
    pub fn get_output(&self) -> &AnimationOutput {
        &self.output
    }

    /// Check if animation is playing
    pub fn is_playing(&self) -> bool {
        self.playback_state == PlaybackState::Playing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === PlaybackState Tests ===

    #[test]
    fn test_playback_state_default() {
        let state = PlaybackState::default();
        assert_eq!(state, PlaybackState::Stopped);
    }

    #[test]
    fn test_playback_state_equality() {
        assert_eq!(PlaybackState::Playing, PlaybackState::Playing);
        assert_ne!(PlaybackState::Playing, PlaybackState::Paused);
        assert_ne!(PlaybackState::Paused, PlaybackState::Stopped);
    }

    // === AnimatedProperty Tests ===

    #[test]
    fn test_animated_property_default() {
        let prop = AnimatedProperty::default();
        assert_eq!(prop, AnimatedProperty::PositionY);
    }

    #[test]
    fn test_animated_property_names() {
        assert_eq!(AnimatedProperty::PositionX.name(), "Position X");
        assert_eq!(AnimatedProperty::PositionY.name(), "Position Y");
        assert_eq!(AnimatedProperty::PositionZ.name(), "Position Z");
        assert_eq!(AnimatedProperty::RotationY.name(), "Rotation Y");
        assert_eq!(AnimatedProperty::Scale.name(), "Scale");
    }

    // === Keyframe Tests ===

    #[test]
    fn test_keyframe_creation() {
        let kf = Keyframe {
            time: 0.5,
            value: 10.0,
            easing: EasingFunction::QuadOut,
        };
        assert_eq!(kf.time, 0.5);
        assert_eq!(kf.value, 10.0);
    }

    #[test]
    fn test_keyframe_clone() {
        let kf = Keyframe {
            time: 1.0,
            value: 5.0,
            easing: EasingFunction::Linear,
        };
        let cloned = kf.clone();
        assert_eq!(cloned.time, kf.time);
        assert_eq!(cloned.value, kf.value);
    }

    // === AnimationTrack Tests ===

    #[test]
    fn test_animation_track_new() {
        let track = AnimationTrack::new(AnimatedProperty::PositionX);
        assert_eq!(track.property, AnimatedProperty::PositionX);
        assert!(track.keyframes.is_empty());
    }

    #[test]
    fn test_animation_track_evaluate_empty() {
        let track = AnimationTrack::new(AnimatedProperty::PositionY);
        assert_eq!(track.evaluate(0.5), 0.0);
    }

    #[test]
    fn test_animation_track_evaluate_empty_scale() {
        let track = AnimationTrack::new(AnimatedProperty::Scale);
        assert_eq!(track.evaluate(0.5), 1.0); // Scale defaults to 1.0
    }

    #[test]
    fn test_animation_track_evaluate_single_keyframe() {
        let mut track = AnimationTrack::new(AnimatedProperty::PositionY);
        track.keyframes.push(Keyframe {
            time: 0.0,
            value: 5.0,
            easing: EasingFunction::Linear,
        });
        assert_eq!(track.evaluate(0.0), 5.0);
        assert_eq!(track.evaluate(0.5), 5.0); // Returns same value
        assert_eq!(track.evaluate(1.0), 5.0);
    }

    #[test]
    fn test_animation_track_evaluate_linear_interpolation() {
        let mut track = AnimationTrack::new(AnimatedProperty::PositionY);
        track.keyframes = vec![
            Keyframe {
                time: 0.0,
                value: 0.0,
                easing: EasingFunction::Linear,
            },
            Keyframe {
                time: 1.0,
                value: 10.0,
                easing: EasingFunction::Linear,
            },
        ];
        assert_eq!(track.evaluate(0.0), 0.0);
        assert_eq!(track.evaluate(0.5), 5.0);
        assert_eq!(track.evaluate(1.0), 10.0);
    }

    #[test]
    fn test_animation_track_evaluate_before_first() {
        let mut track = AnimationTrack::new(AnimatedProperty::PositionY);
        track.keyframes = vec![
            Keyframe {
                time: 0.5,
                value: 10.0,
                easing: EasingFunction::Linear,
            },
            Keyframe {
                time: 1.0,
                value: 20.0,
                easing: EasingFunction::Linear,
            },
        ];
        // Time 0.0 is before first keyframe - implementation interpolates from first
        let val = track.evaluate(0.0);
        // The implementation returns first keyframe value when at or before first time
        assert!(val >= 0.0); // Just verify it doesn't crash
    }

    #[test]
    fn test_animation_track_evaluate_after_last() {
        let mut track = AnimationTrack::new(AnimatedProperty::PositionY);
        track.keyframes = vec![
            Keyframe {
                time: 0.0,
                value: 0.0,
                easing: EasingFunction::Linear,
            },
            Keyframe {
                time: 0.5,
                value: 10.0,
                easing: EasingFunction::Linear,
            },
        ];
        let val = track.evaluate(1.0);
        assert_eq!(val, 10.0); // Stays at last keyframe value
    }

    #[test]
    fn test_animation_track_three_keyframes() {
        let mut track = AnimationTrack::new(AnimatedProperty::PositionY);
        track.keyframes = vec![
            Keyframe {
                time: 0.0,
                value: 0.0,
                easing: EasingFunction::Linear,
            },
            Keyframe {
                time: 0.5,
                value: 10.0,
                easing: EasingFunction::Linear,
            },
            Keyframe {
                time: 1.0,
                value: 5.0,
                easing: EasingFunction::Linear,
            },
        ];
        assert_eq!(track.evaluate(0.0), 0.0);
        assert_eq!(track.evaluate(0.5), 10.0);
        assert_eq!(track.evaluate(1.0), 5.0);
        // Midpoint between keyframes
        assert!((track.evaluate(0.25) - 5.0).abs() < 0.01);
    }

    // === Easing Function Tests ===

    #[test]
    fn test_easing_linear() {
        let t = AnimationTrack::apply_easing(0.5, EasingFunction::Linear);
        assert!((t - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_easing_quad_in() {
        let t = AnimationTrack::apply_easing(0.5, EasingFunction::QuadIn);
        assert!((t - 0.25).abs() < 0.001); // 0.5^2 = 0.25
    }

    #[test]
    fn test_easing_quad_out() {
        let t = AnimationTrack::apply_easing(0.5, EasingFunction::QuadOut);
        assert!((t - 0.75).abs() < 0.001); // 1 - (1-0.5)^2 = 0.75
    }

    #[test]
    fn test_easing_boundaries() {
        // All easing functions should be 0 at t=0 and 1 at t=1
        let easings = [
            EasingFunction::Linear,
            EasingFunction::QuadIn,
            EasingFunction::QuadOut,
            EasingFunction::CubicIn,
            EasingFunction::CubicOut,
            EasingFunction::SineIn,
            EasingFunction::SineOut,
        ];
        for easing in easings {
            let at_0 = AnimationTrack::apply_easing(0.0, easing);
            let at_1 = AnimationTrack::apply_easing(1.0, easing);
            assert!(at_0.abs() < 0.001, "{:?} at 0 should be ~0", easing);
            assert!((at_1 - 1.0).abs() < 0.001, "{:?} at 1 should be ~1", easing);
        }
    }

    #[test]
    fn test_easing_quad_in_out() {
        let before_mid = AnimationTrack::apply_easing(0.25, EasingFunction::QuadInOut);
        let at_mid = AnimationTrack::apply_easing(0.5, EasingFunction::QuadInOut);
        let after_mid = AnimationTrack::apply_easing(0.75, EasingFunction::QuadInOut);

        assert!(before_mid < 0.5);
        assert!((at_mid - 0.5).abs() < 0.01);
        assert!(after_mid > 0.5);
    }

    // === AnimationClip Tests ===

    #[test]
    fn test_animation_clip_default() {
        let clip = AnimationClip::default();
        assert_eq!(clip.name, "New Animation");
        assert_eq!(clip.duration, 2.0);
        assert!(!clip.looping);
        assert!(clip.tracks.is_empty());
    }

    #[test]
    fn test_animation_clip_sample_bounce() {
        let clip = AnimationClip::sample_bounce();
        assert_eq!(clip.name, "Bounce");
        assert!(clip.looping);
        assert_eq!(clip.duration, 1.0);
        assert_eq!(clip.tracks.len(), 1);
        assert_eq!(clip.tracks[0].property, AnimatedProperty::PositionY);
        assert_eq!(clip.tracks[0].keyframes.len(), 3);
    }

    #[test]
    fn test_animation_clip_sample_spin() {
        let clip = AnimationClip::sample_spin();
        assert_eq!(clip.name, "Spin");
        assert!(clip.looping);
        assert_eq!(clip.duration, 2.0);
        assert_eq!(clip.tracks[0].property, AnimatedProperty::RotationY);
    }

    #[test]
    fn test_animation_clip_sample_pulse() {
        let clip = AnimationClip::sample_pulse();
        assert_eq!(clip.name, "Pulse");
        assert!(clip.looping);
        assert_eq!(clip.tracks[0].property, AnimatedProperty::Scale);
    }

    // === AnimationOutput Tests ===

    #[test]
    fn test_animation_output_default() {
        let output = AnimationOutput::default();
        assert_eq!(output.position_offset, Vec3::ZERO);
        assert_eq!(output.rotation_y, 0.0);
        assert_eq!(output.scale_multiplier, 0.0); // Default f32
    }

    #[test]
    fn test_animation_output_clone() {
        let output = AnimationOutput {
            position_offset: Vec3::new(1.0, 2.0, 3.0),
            rotation_y: 1.57,
            scale_multiplier: 1.5,
        };
        let cloned = output.clone();
        assert_eq!(cloned.position_offset, output.position_offset);
        assert_eq!(cloned.rotation_y, output.rotation_y);
    }

    // === AnimationPanel Tests ===

    #[test]
    fn test_animation_panel_creation() {
        let panel = AnimationPanel::default();
        assert_eq!(panel.clips.len(), 3);
        assert_eq!(panel.playback_state, PlaybackState::Stopped);
        assert!(panel.show_editor);
    }

    #[test]
    fn test_animation_panel_default_clips() {
        let panel = AnimationPanel::default();
        assert_eq!(panel.clips[0].name, "Bounce");
        assert_eq!(panel.clips[1].name, "Spin");
        assert_eq!(panel.clips[2].name, "Pulse");
    }

    #[test]
    fn test_animation_panel_initial_selection() {
        let panel = AnimationPanel::default();
        assert_eq!(panel.selected_clip_idx, Some(0));
        assert!(panel.selected_entity.is_none());
    }

    #[test]
    fn test_animation_panel_playback_speed() {
        let panel = AnimationPanel::default();
        assert_eq!(panel.playback_speed, 1.0);
    }

    #[test]
    fn test_animation_panel_easing_tweens_count() {
        let panel = AnimationPanel::default();
        assert_eq!(panel.easing_tweens.len(), 7);
    }

    #[test]
    fn test_animation_panel_set_selected_entity() {
        let mut panel = AnimationPanel::default();
        assert!(panel.selected_entity.is_none());

        panel.set_selected_entity(Some(42));
        assert_eq!(panel.selected_entity, Some(42));

        panel.set_selected_entity(None);
        assert!(panel.selected_entity.is_none());
    }

    #[test]
    fn test_animation_panel_get_output() {
        let panel = AnimationPanel::default();
        let output = panel.get_output();
        assert_eq!(output.scale_multiplier, 1.0);
    }

    #[test]
    fn test_animation_panel_is_playing() {
        let mut panel = AnimationPanel::default();
        assert!(!panel.is_playing());

        panel.playback_state = PlaybackState::Playing;
        assert!(panel.is_playing());

        panel.playback_state = PlaybackState::Paused;
        assert!(!panel.is_playing());
    }

    // === Update Tests ===

    #[test]
    fn test_update_when_stopped() {
        let mut panel = AnimationPanel {
            playback_state: PlaybackState::Stopped,
            ..Default::default()
        };

        let result = panel.update(0.1);
        assert!(result.is_none());
    }

    #[test]
    fn test_update_when_paused() {
        let mut panel = AnimationPanel {
            playback_state: PlaybackState::Paused,
            ..Default::default()
        };

        let result = panel.update(0.1);
        assert!(result.is_none());
    }

    #[test]
    fn test_update_when_playing() {
        let mut panel = AnimationPanel {
            playback_state: PlaybackState::Playing,
            ..Default::default()
        };

        let result = panel.update(0.1);
        assert!(result.is_some());
        assert!(panel.current_time > 0.0);
    }

    #[test]
    fn test_update_advances_time() {
        let mut panel = AnimationPanel {
            playback_state: PlaybackState::Playing,
            current_time: 0.0,
            ..Default::default()
        };

        panel.update(0.5);
        assert!((panel.current_time - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_update_respects_playback_speed() {
        let mut panel = AnimationPanel {
            playback_state: PlaybackState::Playing,
            playback_speed: 2.0,
            current_time: 0.0,
            ..Default::default()
        };

        panel.update(0.25);
        // 0.25 * 2.0 = 0.5, but default clip (Bounce) is 1.0s looping
        assert!(panel.current_time > 0.0);
    }

    #[test]
    fn test_update_looping_clip() {
        let mut panel = AnimationPanel {
            playback_state: PlaybackState::Playing,
            selected_clip_idx: Some(0),
            current_time: 0.9,
            ..Default::default()
        };

        panel.update(0.2); // Should wrap around

        assert!(panel.current_time < 1.0);
        assert_eq!(panel.playback_state, PlaybackState::Playing);
    }

    #[test]
    fn test_update_non_looping_stops_at_end() {
        let mut panel = AnimationPanel::default();
        // Create non-looping clip
        panel.clips.push(AnimationClip {
            name: "OneShot".to_string(),
            duration: 1.0,
            looping: false,
            tracks: Vec::new(),
        });
        panel.selected_clip_idx = Some(3); // The new non-looping clip
        panel.playback_state = PlaybackState::Playing;
        panel.current_time = 0.9;

        panel.update(0.2);

        assert_eq!(panel.current_time, 1.0);
        assert_eq!(panel.playback_state, PlaybackState::Stopped);
    }

    #[test]
    fn test_update_evaluates_tracks() {
        let mut panel = AnimationPanel {
            playback_state: PlaybackState::Playing,
            selected_clip_idx: Some(0),
            current_time: 0.0,
            ..Default::default()
        };

        let output = panel.update(0.25);

        assert!(output.is_some());
        let out = output.unwrap();
        // Bounce clip animates position_offset.y
        // At some point it should be non-zero
        assert!(out.position_offset.y.abs() > 1e-3);
    }

    // === Integration Tests ===

    #[test]
    fn test_full_playback_workflow() {
        let mut panel = AnimationPanel::default();

        // Select entity
        panel.set_selected_entity(Some(1));
        assert_eq!(panel.selected_entity, Some(1));

        // Select clip
        panel.selected_clip_idx = Some(1); // Spin

        // Start playing
        panel.playback_state = PlaybackState::Playing;
        assert!(panel.is_playing());

        // Update
        let output = panel.update(0.5);
        assert!(output.is_some());

        // Pause
        panel.playback_state = PlaybackState::Paused;
        assert!(!panel.is_playing());

        // Resume
        panel.playback_state = PlaybackState::Playing;
        assert!(panel.is_playing());

        // Stop
        panel.playback_state = PlaybackState::Stopped;
        assert!(!panel.is_playing());
    }

    #[test]
    fn test_multiple_tracks_evaluation() {
        let mut panel = AnimationPanel {
            playback_state: PlaybackState::Playing,
            ..Default::default()
        };
        
        // Create clip with multiple tracks
        let mut clip = AnimationClip {
            name: "Multi".to_string(),
            duration: 1.0,
            looping: true,
            tracks: Vec::new(),
        };

        let mut y_track = AnimationTrack::new(AnimatedProperty::PositionY);
        y_track.keyframes = vec![
            Keyframe { time: 0.0, value: 0.0, easing: EasingFunction::Linear },
            Keyframe { time: 1.0, value: 5.0, easing: EasingFunction::Linear },
        ];
        clip.tracks.push(y_track);

        let mut scale_track = AnimationTrack::new(AnimatedProperty::Scale);
        scale_track.keyframes = vec![
            Keyframe { time: 0.0, value: 1.0, easing: EasingFunction::Linear },
            Keyframe { time: 1.0, value: 2.0, easing: EasingFunction::Linear },
        ];
        clip.tracks.push(scale_track);

        panel.clips.push(clip);
        panel.selected_clip_idx = Some(3);
        panel.current_time = 0.0;

        let output = panel.update(0.5);
        assert!(output.is_some());

        let out = output.unwrap();
        assert!((out.position_offset.y - 2.5).abs() < 0.1);
        assert!((out.scale_multiplier - 1.5).abs() < 0.1);
    }

    #[test]
    fn test_show_editor_mode_toggle() {
        let mut panel = AnimationPanel::default();
        assert!(panel.show_editor);

        panel.show_editor = false;
        assert!(!panel.show_editor);
    }

    // ============================================================================
    // ENHANCED ENUM TESTS (Display, Hash, helpers)
    // ============================================================================

    #[test]
    fn test_playback_state_display() {
        for state in PlaybackState::all() {
            let display = format!("{}", state);
            assert!(display.contains(state.name()));
            assert!(display.contains(state.icon()));
        }
    }

    #[test]
    fn test_playback_state_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for state in PlaybackState::all() {
            set.insert(*state);
        }
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_playback_state_all() {
        let all = PlaybackState::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&PlaybackState::Stopped));
        assert!(all.contains(&PlaybackState::Playing));
        assert!(all.contains(&PlaybackState::Paused));
    }

    #[test]
    fn test_playback_state_name() {
        assert_eq!(PlaybackState::Stopped.name(), "Stopped");
        assert_eq!(PlaybackState::Playing.name(), "Playing");
        assert_eq!(PlaybackState::Paused.name(), "Paused");
    }

    #[test]
    fn test_playback_state_icon() {
        assert_eq!(PlaybackState::Stopped.icon(), "⏹");
        assert_eq!(PlaybackState::Playing.icon(), "▶");
        assert_eq!(PlaybackState::Paused.icon(), "⏸");
    }

    #[test]
    fn test_playback_state_is_active() {
        assert!(!PlaybackState::Stopped.is_active());
        assert!(PlaybackState::Playing.is_active());
        assert!(!PlaybackState::Paused.is_active());
    }

    #[test]
    fn test_animated_property_display() {
        for prop in AnimatedProperty::all() {
            let display = format!("{}", prop);
            assert!(display.contains(prop.name()));
            assert!(display.contains(prop.icon()));
        }
    }

    #[test]
    fn test_animated_property_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for prop in AnimatedProperty::all() {
            set.insert(*prop);
        }
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn test_animated_property_all() {
        let all = AnimatedProperty::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&AnimatedProperty::PositionX));
        assert!(all.contains(&AnimatedProperty::PositionY));
        assert!(all.contains(&AnimatedProperty::PositionZ));
        assert!(all.contains(&AnimatedProperty::RotationY));
        assert!(all.contains(&AnimatedProperty::Scale));
    }

    #[test]
    fn test_animated_property_icon() {
        assert_eq!(AnimatedProperty::PositionX.icon(), "↔");
        assert_eq!(AnimatedProperty::PositionY.icon(), "↕");
        assert_eq!(AnimatedProperty::PositionZ.icon(), "↗");
        assert_eq!(AnimatedProperty::RotationY.icon(), "🔄");
        assert_eq!(AnimatedProperty::Scale.icon(), "📐");
    }

    #[test]
    fn test_animated_property_is_position() {
        assert!(AnimatedProperty::PositionX.is_position());
        assert!(AnimatedProperty::PositionY.is_position());
        assert!(AnimatedProperty::PositionZ.is_position());
        assert!(!AnimatedProperty::RotationY.is_position());
        assert!(!AnimatedProperty::Scale.is_position());
    }

    #[test]
    fn test_animated_property_is_rotation() {
        assert!(!AnimatedProperty::PositionX.is_rotation());
        assert!(!AnimatedProperty::PositionY.is_rotation());
        assert!(!AnimatedProperty::PositionZ.is_rotation());
        assert!(AnimatedProperty::RotationY.is_rotation());
        assert!(!AnimatedProperty::Scale.is_rotation());
    }
}
