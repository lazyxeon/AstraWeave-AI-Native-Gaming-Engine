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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

/// Animation property that can be animated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimatedProperty {
    #[default]
    PositionY,
    PositionX,
    PositionZ,
    RotationY,
    Scale,
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
                self.current_time = self.current_time % clip.duration;
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
                            track
                                .keyframes
                                .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
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

    #[test]
    fn test_animation_panel_creation() {
        let panel = AnimationPanel::default();
        assert_eq!(panel.clips.len(), 3);
        assert_eq!(panel.playback_state, PlaybackState::Stopped);
        assert!(panel.show_editor);
    }

    #[test]
    fn test_keyframe_evaluation() {
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
    fn test_sample_clips() {
        let bounce = AnimationClip::sample_bounce();
        assert!(bounce.looping);
        assert_eq!(bounce.tracks.len(), 1);

        let spin = AnimationClip::sample_spin();
        assert!(spin.looping);
    }

    #[test]
    fn test_playback_update() {
        let mut panel = AnimationPanel::default();
        panel.playback_state = PlaybackState::Playing;
        let output = panel.update(0.1);
        assert!(output.is_some());
        assert!(panel.current_time > 0.0);
    }

    #[test]
    fn test_easing_tweens_count() {
        let panel = AnimationPanel::default();
        assert_eq!(panel.easing_tweens.len(), 7);
    }
}
