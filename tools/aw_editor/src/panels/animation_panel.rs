//! Animation Panel for the editor UI
//!
//! Provides comprehensive animation editing:
//! - Animation clips and timelines
//! - Blend trees and state machines
//! - Animation layers and masking
//! - IK and procedural animation
//! - Animation events and curves

use egui::{Color32, RichText, Ui, Vec2};

use crate::panels::Panel;

/// Animation clip
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub id: u32,
    pub name: String,
    pub duration: f32,
    pub fps: f32,
    pub loop_mode: LoopMode,
    pub events: Vec<AnimationEvent>,
    pub curves: Vec<AnimationCurve>,
}

impl Default for AnimationClip {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Clip".to_string(),
            duration: 1.0,
            fps: 30.0,
            loop_mode: LoopMode::Loop,
            events: Vec::new(),
            curves: Vec::new(),
        }
    }
}

/// Loop mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LoopMode {
    Once,
    #[default]
    Loop,
    PingPong,
    ClampForever,
}

/// Animation event
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    pub id: u32,
    pub time: f32,
    pub name: String,
    pub function: String,
    pub parameter: String,
}

/// Animation curve
#[derive(Debug, Clone)]
pub struct AnimationCurve {
    pub id: u32,
    pub property_path: String,
    pub keyframes: Vec<Keyframe>,
}

/// Keyframe
#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f32,
    pub value: f32,
    pub in_tangent: f32,
    pub out_tangent: f32,
    pub tangent_mode: TangentMode,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TangentMode {
    #[default]
    Auto,
    Linear,
    Constant,
    Free,
    Broken,
}

/// Animation state in a state machine
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub id: u32,
    pub name: String,
    pub clip_id: Option<u32>,
    pub speed: f32,
    pub position: [f32; 2],
    pub is_default: bool,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New State".to_string(),
            clip_id: None,
            speed: 1.0,
            position: [100.0, 100.0],
            is_default: false,
        }
    }
}

/// State transition
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub id: u32,
    pub from_state: u32,
    pub to_state: u32,
    pub duration: f32,
    pub offset: f32,
    pub has_exit_time: bool,
    pub exit_time: f32,
    pub conditions: Vec<TransitionCondition>,
}

impl Default for StateTransition {
    fn default() -> Self {
        Self {
            id: 0,
            from_state: 0,
            to_state: 0,
            duration: 0.25,
            offset: 0.0,
            has_exit_time: false,
            exit_time: 1.0,
            conditions: Vec::new(),
        }
    }
}

/// Transition condition
#[derive(Debug, Clone)]
pub struct TransitionCondition {
    pub parameter: String,
    pub condition_type: ConditionType,
    pub threshold: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConditionType {
    #[default]
    Greater,
    Less,
    Equals,
    NotEquals,
    True,
    False,
}

/// Animation parameter
#[derive(Debug, Clone)]
pub struct AnimationParameter {
    pub id: u32,
    pub name: String,
    pub param_type: ParameterType,
    pub default_value: ParameterValue,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ParameterType {
    #[default]
    Float,
    Int,
    Bool,
    Trigger,
}

#[derive(Debug, Clone)]
pub enum ParameterValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    Trigger,
}

impl Default for ParameterValue {
    fn default() -> Self {
        ParameterValue::Float(0.0)
    }
}

/// Blend tree node type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BlendTreeType {
    #[default]
    Simple1D,
    Simple2D,
    FreeformDirectional,
    FreeformCartesian,
    Direct,
}

/// Blend tree
#[derive(Debug, Clone)]
pub struct BlendTree {
    pub id: u32,
    pub name: String,
    pub blend_type: BlendTreeType,
    pub blend_parameter_x: String,
    pub blend_parameter_y: String,
    pub children: Vec<BlendTreeChild>,
}

impl Default for BlendTree {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Blend Tree".to_string(),
            blend_type: BlendTreeType::Simple1D,
            blend_parameter_x: String::new(),
            blend_parameter_y: String::new(),
            children: Vec::new(),
        }
    }
}

/// Blend tree child
#[derive(Debug, Clone)]
pub struct BlendTreeChild {
    pub clip_id: u32,
    pub threshold: f32,
    pub position: [f32; 2],
    pub time_scale: f32,
}

/// Animation layer
#[derive(Debug, Clone)]
pub struct AnimationLayer {
    pub id: u32,
    pub name: String,
    pub weight: f32,
    pub blending_mode: BlendingMode,
    pub avatar_mask: Option<String>,
    pub ik_pass: bool,
}

impl Default for AnimationLayer {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Layer".to_string(),
            weight: 1.0,
            blending_mode: BlendingMode::Override,
            avatar_mask: None,
            ik_pass: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BlendingMode {
    #[default]
    Override,
    Additive,
}

/// IK settings
#[derive(Debug, Clone)]
pub struct IkSettings {
    pub enabled: bool,

    // Foot IK
    pub foot_ik_enabled: bool,
    pub foot_height_offset: f32,
    pub foot_weight: f32,

    // Look At IK
    pub look_at_enabled: bool,
    pub look_at_weight: f32,
    pub head_weight: f32,
    pub body_weight: f32,
    pub eyes_weight: f32,
    pub clamp_weight: f32,

    // Hand IK
    pub left_hand_ik_enabled: bool,
    pub right_hand_ik_enabled: bool,
    pub hand_ik_weight: f32,
}

impl Default for IkSettings {
    fn default() -> Self {
        Self {
            enabled: false,

            foot_ik_enabled: true,
            foot_height_offset: 0.0,
            foot_weight: 1.0,

            look_at_enabled: false,
            look_at_weight: 1.0,
            head_weight: 1.0,
            body_weight: 0.4,
            eyes_weight: 1.0,
            clamp_weight: 0.5,

            left_hand_ik_enabled: false,
            right_hand_ik_enabled: false,
            hand_ik_weight: 1.0,
        }
    }
}

/// Animation controller (state machine)
#[derive(Debug, Clone)]
pub struct AnimationController {
    pub id: u32,
    pub name: String,
    pub states: Vec<AnimationState>,
    pub transitions: Vec<StateTransition>,
    pub parameters: Vec<AnimationParameter>,
    pub layers: Vec<AnimationLayer>,
    pub blend_trees: Vec<BlendTree>,
    pub ik_settings: IkSettings,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Controller".to_string(),
            states: Vec::new(),
            transitions: Vec::new(),
            parameters: Vec::new(),
            layers: vec![AnimationLayer::default()],
            blend_trees: Vec::new(),
            ik_settings: IkSettings::default(),
        }
    }
}

/// Panel tabs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AnimationTab {
    #[default]
    Clips,
    StateMachine,
    BlendTrees,
    Layers,
    Parameters,
    IK,
    Preview,
}

/// Main Animation Panel
pub struct AnimationPanel {
    // Tab state
    active_tab: AnimationTab,

    // Clips
    clips: Vec<AnimationClip>,
    selected_clip: Option<u32>,
    current_clip: AnimationClip,

    // Controller
    controllers: Vec<AnimationController>,
    selected_controller: Option<u32>,
    current_controller: AnimationController,

    // Preview
    preview_playing: bool,
    preview_time: f32,
    preview_speed: f32,

    // Timeline
    timeline_zoom: f32,
    timeline_scroll: f32,

    // State machine editor
    selected_state: Option<u32>,
    selected_transition: Option<u32>,

    // ID counters
    next_clip_id: u32,
    next_controller_id: u32,
    next_state_id: u32,
    next_transition_id: u32,
    next_parameter_id: u32,
}

impl Default for AnimationPanel {
    fn default() -> Self {
        let mut panel = Self {
            active_tab: AnimationTab::Clips,

            clips: Vec::new(),
            selected_clip: None,
            current_clip: AnimationClip::default(),

            controllers: Vec::new(),
            selected_controller: None,
            current_controller: AnimationController::default(),

            preview_playing: false,
            preview_time: 0.0,
            preview_speed: 1.0,

            timeline_zoom: 1.0,
            timeline_scroll: 0.0,

            selected_state: None,
            selected_transition: None,

            next_clip_id: 1,
            next_controller_id: 1,
            next_state_id: 1,
            next_transition_id: 1,
            next_parameter_id: 1,
        };

        panel.create_sample_data();
        panel
    }
}

impl AnimationPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_sample_data(&mut self) {
        // Sample clips
        let clips = [
            ("Idle", 2.0),
            ("Walk", 1.0),
            ("Run", 0.6),
            ("Jump", 0.8),
            ("Attack", 0.5),
            ("Death", 2.0),
        ];

        for (name, duration) in clips {
            let id = self.next_clip_id();
            self.clips.push(AnimationClip {
                id,
                name: name.to_string(),
                duration,
                ..Default::default()
            });
        }

        // Sample controller
        let controller_id = self.next_controller_id();
        let mut controller = AnimationController {
            id: controller_id,
            name: "Character Controller".to_string(),
            ..Default::default()
        };

        // Add states
        let idle_id = self.next_state_id();
        controller.states.push(AnimationState {
            id: idle_id,
            name: "Idle".to_string(),
            clip_id: Some(1),
            position: [100.0, 150.0],
            is_default: true,
            ..Default::default()
        });

        let walk_id = self.next_state_id();
        controller.states.push(AnimationState {
            id: walk_id,
            name: "Walk".to_string(),
            clip_id: Some(2),
            position: [300.0, 100.0],
            ..Default::default()
        });

        let run_id = self.next_state_id();
        controller.states.push(AnimationState {
            id: run_id,
            name: "Run".to_string(),
            clip_id: Some(3),
            position: [300.0, 200.0],
            ..Default::default()
        });

        // Add transitions
        let trans_id = self.next_transition_id();
        controller.transitions.push(StateTransition {
            id: trans_id,
            from_state: idle_id,
            to_state: walk_id,
            duration: 0.2,
            conditions: vec![TransitionCondition {
                parameter: "Speed".to_string(),
                condition_type: ConditionType::Greater,
                threshold: 0.1,
            }],
            ..Default::default()
        });

        let trans_id = self.next_transition_id();
        controller.transitions.push(StateTransition {
            id: trans_id,
            from_state: walk_id,
            to_state: run_id,
            duration: 0.2,
            conditions: vec![TransitionCondition {
                parameter: "Speed".to_string(),
                condition_type: ConditionType::Greater,
                threshold: 0.5,
            }],
            ..Default::default()
        });

        // Add parameters
        let param_id = self.next_parameter_id();
        controller.parameters.push(AnimationParameter {
            id: param_id,
            name: "Speed".to_string(),
            param_type: ParameterType::Float,
            default_value: ParameterValue::Float(0.0),
        });

        let param_id = self.next_parameter_id();
        controller.parameters.push(AnimationParameter {
            id: param_id,
            name: "IsGrounded".to_string(),
            param_type: ParameterType::Bool,
            default_value: ParameterValue::Bool(true),
        });

        let param_id = self.next_parameter_id();
        controller.parameters.push(AnimationParameter {
            id: param_id,
            name: "Jump".to_string(),
            param_type: ParameterType::Trigger,
            default_value: ParameterValue::Trigger,
        });

        self.controllers.push(controller.clone());
        self.current_controller = controller;
        self.selected_controller = Some(controller_id);

        if !self.clips.is_empty() {
            self.current_clip = self.clips[0].clone();
            self.selected_clip = Some(self.clips[0].id);
        }
    }

    fn next_clip_id(&mut self) -> u32 {
        let id = self.next_clip_id;
        self.next_clip_id += 1;
        id
    }

    fn next_controller_id(&mut self) -> u32 {
        let id = self.next_controller_id;
        self.next_controller_id += 1;
        id
    }

    fn next_state_id(&mut self) -> u32 {
        let id = self.next_state_id;
        self.next_state_id += 1;
        id
    }

    fn next_transition_id(&mut self) -> u32 {
        let id = self.next_transition_id;
        self.next_transition_id += 1;
        id
    }

    fn next_parameter_id(&mut self) -> u32 {
        let id = self.next_parameter_id;
        self.next_parameter_id += 1;
        id
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (AnimationTab::Clips, "🎬 Clips"),
                (AnimationTab::StateMachine, "📊 States"),
                (AnimationTab::BlendTrees, "🌳 Blend Trees"),
                (AnimationTab::Layers, "📚 Layers"),
                (AnimationTab::Parameters, "🔧 Parameters"),
                (AnimationTab::IK, "🦴 IK"),
                (AnimationTab::Preview, "▶️ Preview"),
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

        // Info bar
        ui.horizontal(|ui| {
            ui.label(format!("🎬 {} clips", self.clips.len()));
            ui.separator();
            ui.label(format!("📊 {} states", self.current_controller.states.len()));
            ui.separator();
            ui.label(format!("🔧 {} params", self.current_controller.parameters.len()));
        });

        ui.separator();
    }

    fn show_clips_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎬 Animation Clips");
        ui.add_space(10.0);

        // Clip selector
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("clip_select")
                .selected_text(&self.current_clip.name)
                .show_ui(ui, |ui| {
                    for clip in &self.clips.clone() {
                        if ui.selectable_value(&mut self.selected_clip, Some(clip.id), &clip.name).clicked() {
                            self.current_clip = clip.clone();
                        }
                    }
                });

            if ui.button("+ New").clicked() {
                let id = self.next_clip_id();
                let new_clip = AnimationClip {
                    id,
                    name: format!("Clip {}", id),
                    ..Default::default()
                };
                self.clips.push(new_clip.clone());
                self.current_clip = new_clip;
                self.selected_clip = Some(id);
            }

            if ui.button("📋 Duplicate").clicked() {
                let id = self.next_clip_id();
                let mut dup = self.current_clip.clone();
                dup.id = id;
                dup.name = format!("{} (Copy)", dup.name);
                self.clips.push(dup);
            }
        });

        ui.add_space(10.0);

        // Clip properties
        ui.group(|ui| {
            ui.label(RichText::new("📝 Properties").strong());

            egui::Grid::new("clip_props")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.current_clip.name);
                    ui.end_row();

                    ui.label("Duration:");
                    ui.add(egui::DragValue::new(&mut self.current_clip.duration).speed(0.1).suffix("s"));
                    ui.end_row();

                    ui.label("FPS:");
                    ui.add(egui::DragValue::new(&mut self.current_clip.fps).speed(1.0).range(1.0..=120.0));
                    ui.end_row();

                    ui.label("Loop Mode:");
                    egui::ComboBox::from_id_salt("loop_mode")
                        .selected_text(format!("{:?}", self.current_clip.loop_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.current_clip.loop_mode, LoopMode::Once, "Once");
                            ui.selectable_value(&mut self.current_clip.loop_mode, LoopMode::Loop, "Loop");
                            ui.selectable_value(&mut self.current_clip.loop_mode, LoopMode::PingPong, "Ping Pong");
                            ui.selectable_value(&mut self.current_clip.loop_mode, LoopMode::ClampForever, "Clamp Forever");
                        });
                    ui.end_row();
                });
        });

        ui.add_space(10.0);

        // Simple timeline
        ui.group(|ui| {
            ui.label(RichText::new("⏱️ Timeline").strong());

            // Playback controls
            ui.horizontal(|ui| {
                if ui.button(if self.preview_playing { "⏸️" } else { "▶️" }).clicked() {
                    self.preview_playing = !self.preview_playing;
                }
                if ui.button("⏹️").clicked() {
                    self.preview_playing = false;
                    self.preview_time = 0.0;
                }
                if ui.button("⏮️").clicked() {
                    self.preview_time = 0.0;
                }
                if ui.button("⏭️").clicked() {
                    self.preview_time = self.current_clip.duration;
                }

                ui.add(egui::Slider::new(&mut self.preview_time, 0.0..=self.current_clip.duration).suffix("s"));

                ui.label(format!("Speed: {:.1}x", self.preview_speed));
                ui.add(egui::DragValue::new(&mut self.preview_speed).speed(0.1).range(0.1..=3.0));
            });

            // Timeline visualization
            let timeline_height = 60.0;
            let timeline_width = ui.available_width();
            let (rect, _) = ui.allocate_exact_size(Vec2::new(timeline_width, timeline_height), egui::Sense::click_and_drag());

            let painter = ui.painter();
            painter.rect_filled(rect, 4.0, Color32::from_rgb(40, 40, 45));

            // Time markers
            let duration = self.current_clip.duration;
            if duration > 0.0 {
                let steps = 10;
                for i in 0..=steps {
                    let t = i as f32 / steps as f32;
                    let x = rect.min.x + t * timeline_width;
                    painter.line_segment(
                        [egui::Pos2::new(x, rect.min.y), egui::Pos2::new(x, rect.min.y + 10.0)],
                        egui::Stroke::new(1.0, Color32::GRAY),
                    );
                }

                // Playhead
                let playhead_x = rect.min.x + (self.preview_time / duration) * timeline_width;
                painter.line_segment(
                    [egui::Pos2::new(playhead_x, rect.min.y), egui::Pos2::new(playhead_x, rect.max.y)],
                    egui::Stroke::new(2.0, Color32::RED),
                );
                painter.circle_filled(egui::Pos2::new(playhead_x, rect.min.y + 5.0), 5.0, Color32::RED);
            }
        });

        ui.add_space(10.0);

        // Events
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚡ Events").strong());
                if ui.button("+ Add Event").clicked() {
                    self.current_clip.events.push(AnimationEvent {
                        id: self.current_clip.events.len() as u32 + 1,
                        time: self.preview_time,
                        name: "NewEvent".to_string(),
                        function: "OnAnimationEvent".to_string(),
                        parameter: String::new(),
                    });
                }
            });

            if self.current_clip.events.is_empty() {
                ui.label("No events. Click '+ Add Event' to add one.");
            } else {
                for event in &self.current_clip.events {
                    ui.horizontal(|ui| {
                        ui.label(format!("@{:.2}s:", event.time));
                        ui.label(&event.name);
                        ui.label(RichText::new(&event.function).small().color(Color32::GRAY));
                    });
                }
            }
        });
    }

    fn show_state_machine_tab(&mut self, ui: &mut Ui) {
        ui.heading("📊 State Machine");
        ui.add_space(10.0);

        // Controller selector
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("controller_select")
                .selected_text(&self.current_controller.name)
                .show_ui(ui, |ui| {
                    for ctrl in &self.controllers.clone() {
                        if ui.selectable_value(&mut self.selected_controller, Some(ctrl.id), &ctrl.name).clicked() {
                            self.current_controller = ctrl.clone();
                        }
                    }
                });

            if ui.button("+ New").clicked() {
                let id = self.next_controller_id();
                let new_ctrl = AnimationController {
                    id,
                    name: format!("Controller {}", id),
                    ..Default::default()
                };
                self.controllers.push(new_ctrl.clone());
                self.current_controller = new_ctrl;
                self.selected_controller = Some(id);
            }
        });

        ui.add_space(10.0);

        // State list
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📋 States").strong());
                if ui.button("+ Add State").clicked() {
                    let id = self.next_state_id();
                    self.current_controller.states.push(AnimationState {
                        id,
                        name: format!("State {}", id),
                        position: [200.0, 100.0 + (self.current_controller.states.len() as f32) * 50.0],
                        ..Default::default()
                    });
                }
            });

            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for state in &mut self.current_controller.states {
                        ui.horizontal(|ui| {
                            let is_selected = self.selected_state == Some(state.id);
                            let label = if state.is_default {
                                format!("🟢 {}", state.name)
                            } else {
                                format!("   {}", state.name)
                            };

                            if ui.selectable_label(is_selected, label).clicked() {
                                self.selected_state = Some(state.id);
                            }

                            // Clip assignment
                            egui::ComboBox::from_id_salt(format!("state_clip_{}", state.id))
                                .selected_text(state.clip_id.map_or("(None)".to_string(), |id| {
                                    self.clips.iter().find(|c| c.id == id).map_or("(None)".to_string(), |c| c.name.clone())
                                }))
                                .width(100.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut state.clip_id, None, "(None)").clicked();
                                    for clip in &self.clips {
                                        ui.selectable_value(&mut state.clip_id, Some(clip.id), &clip.name);
                                    }
                                });
                        });
                    }
                });
        });

        ui.add_space(10.0);

        // Transition list
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔀 Transitions").strong());
                if ui.button("+ Add").clicked() && self.current_controller.states.len() >= 2 {
                    let from = self.current_controller.states[0].id;
                    let to = self.current_controller.states[1].id;
                    let id = self.next_transition_id();
                    self.current_controller.transitions.push(StateTransition {
                        id,
                        from_state: from,
                        to_state: to,
                        ..Default::default()
                    });
                }
            });

            for trans in &self.current_controller.transitions {
                let from_name = self.current_controller.states.iter()
                    .find(|s| s.id == trans.from_state)
                    .map_or("?", |s| &s.name);
                let to_name = self.current_controller.states.iter()
                    .find(|s| s.id == trans.to_state)
                    .map_or("?", |s| &s.name);

                ui.horizontal(|ui| {
                    ui.label(format!("{} → {}", from_name, to_name));
                    ui.label(RichText::new(format!("{:.2}s", trans.duration)).small().color(Color32::GRAY));

                    if !trans.conditions.is_empty() {
                        ui.label(RichText::new(format!("({} conditions)", trans.conditions.len())).small());
                    }
                });
            }
        });
    }

    fn show_blend_trees_tab(&mut self, ui: &mut Ui) {
        ui.heading("🌳 Blend Trees");
        ui.add_space(10.0);

        // Blend tree list
        ui.horizontal(|ui| {
            if ui.button("+ New Blend Tree").clicked() {
                let id = self.current_controller.blend_trees.len() as u32 + 1;
                self.current_controller.blend_trees.push(BlendTree {
                    id,
                    name: format!("Blend Tree {}", id),
                    ..Default::default()
                });
            }
        });

        ui.add_space(10.0);

        if self.current_controller.blend_trees.is_empty() {
            ui.label("No blend trees. Click '+ New Blend Tree' to create one.");
        } else {
            for tree in &mut self.current_controller.blend_trees {
                ui.group(|ui| {
                    ui.label(RichText::new(&tree.name).strong());

                    egui::Grid::new(format!("blend_tree_{}", tree.id))
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Type:");
                            egui::ComboBox::from_id_salt(format!("blend_type_{}", tree.id))
                                .selected_text(format!("{:?}", tree.blend_type))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut tree.blend_type, BlendTreeType::Simple1D, "Simple 1D");
                                    ui.selectable_value(&mut tree.blend_type, BlendTreeType::Simple2D, "Simple 2D");
                                    ui.selectable_value(&mut tree.blend_type, BlendTreeType::FreeformDirectional, "Freeform Directional");
                                    ui.selectable_value(&mut tree.blend_type, BlendTreeType::FreeformCartesian, "Freeform Cartesian");
                                    ui.selectable_value(&mut tree.blend_type, BlendTreeType::Direct, "Direct");
                                });
                            ui.end_row();

                            ui.label("Parameter X:");
                            ui.text_edit_singleline(&mut tree.blend_parameter_x);
                            ui.end_row();

                            if matches!(tree.blend_type, BlendTreeType::Simple2D | BlendTreeType::FreeformDirectional | BlendTreeType::FreeformCartesian) {
                                ui.label("Parameter Y:");
                                ui.text_edit_singleline(&mut tree.blend_parameter_y);
                                ui.end_row();
                            }
                        });

                    ui.label(format!("Children: {}", tree.children.len()));
                });
            }
        }
    }

    fn show_layers_tab(&mut self, ui: &mut Ui) {
        ui.heading("📚 Animation Layers");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("+ Add Layer").clicked() {
                let id = self.current_controller.layers.len() as u32 + 1;
                self.current_controller.layers.push(AnimationLayer {
                    id,
                    name: format!("Layer {}", id),
                    ..Default::default()
                });
            }
        });

        ui.add_space(10.0);

        for (idx, layer) in self.current_controller.layers.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("#{} {}", idx, layer.name)).strong());

                    if idx == 0 {
                        ui.label(RichText::new("(Base)").small().color(Color32::GRAY));
                    }
                });

                egui::Grid::new(format!("layer_{}", layer.id))
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut layer.name);
                        ui.end_row();

                        ui.label("Weight:");
                        ui.add(egui::Slider::new(&mut layer.weight, 0.0..=1.0));
                        ui.end_row();

                        ui.label("Blending:");
                        egui::ComboBox::from_id_salt(format!("layer_blend_{}", layer.id))
                            .selected_text(format!("{:?}", layer.blending_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut layer.blending_mode, BlendingMode::Override, "Override");
                                ui.selectable_value(&mut layer.blending_mode, BlendingMode::Additive, "Additive");
                            });
                        ui.end_row();

                        ui.label("IK Pass:");
                        ui.checkbox(&mut layer.ik_pass, "");
                        ui.end_row();
                    });
            });
        }
    }

    fn show_parameters_tab(&mut self, ui: &mut Ui) {
        ui.heading("🔧 Parameters");
        ui.add_space(10.0);

        // Add parameter buttons
        ui.horizontal(|ui| {
            if ui.button("+ Float").clicked() {
                let id = self.next_parameter_id();
                self.current_controller.parameters.push(AnimationParameter {
                    id,
                    name: format!("Float{}", id),
                    param_type: ParameterType::Float,
                    default_value: ParameterValue::Float(0.0),
                });
            }
            if ui.button("+ Int").clicked() {
                let id = self.next_parameter_id();
                self.current_controller.parameters.push(AnimationParameter {
                    id,
                    name: format!("Int{}", id),
                    param_type: ParameterType::Int,
                    default_value: ParameterValue::Int(0),
                });
            }
            if ui.button("+ Bool").clicked() {
                let id = self.next_parameter_id();
                self.current_controller.parameters.push(AnimationParameter {
                    id,
                    name: format!("Bool{}", id),
                    param_type: ParameterType::Bool,
                    default_value: ParameterValue::Bool(false),
                });
            }
            if ui.button("+ Trigger").clicked() {
                let id = self.next_parameter_id();
                self.current_controller.parameters.push(AnimationParameter {
                    id,
                    name: format!("Trigger{}", id),
                    param_type: ParameterType::Trigger,
                    default_value: ParameterValue::Trigger,
                });
            }
        });

        ui.add_space(10.0);

        // Parameter list
        egui::ScrollArea::vertical()
            .max_height(250.0)
            .show(ui, |ui| {
                for param in &mut self.current_controller.parameters {
                    ui.horizontal(|ui| {
                        let type_icon = match param.param_type {
                            ParameterType::Float => "🔢",
                            ParameterType::Int => "🔢",
                            ParameterType::Bool => "☑️",
                            ParameterType::Trigger => "⚡",
                        };

                        ui.label(type_icon);
                        ui.text_edit_singleline(&mut param.name);

                        match &mut param.default_value {
                            ParameterValue::Float(v) => {
                                ui.add(egui::DragValue::new(v).speed(0.1));
                            }
                            ParameterValue::Int(v) => {
                                ui.add(egui::DragValue::new(v).speed(1));
                            }
                            ParameterValue::Bool(v) => {
                                ui.checkbox(v, "");
                            }
                            ParameterValue::Trigger => {
                                if ui.button("Fire").clicked() {
                                    // Trigger the parameter
                                }
                            }
                        }
                    });
                }
            });
    }

    fn show_ik_tab(&mut self, ui: &mut Ui) {
        ui.heading("🦴 Inverse Kinematics");
        ui.add_space(10.0);

        ui.checkbox(&mut self.current_controller.ik_settings.enabled, RichText::new("Enable IK").strong());

        if !self.current_controller.ik_settings.enabled {
            return;
        }

        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                // Foot IK
                ui.group(|ui| {
                    ui.checkbox(&mut self.current_controller.ik_settings.foot_ik_enabled, RichText::new("🦶 Foot IK").strong());

                    if self.current_controller.ik_settings.foot_ik_enabled {
                        egui::Grid::new("foot_ik")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Height Offset:");
                                ui.add(egui::Slider::new(&mut self.current_controller.ik_settings.foot_height_offset, -0.5..=0.5));
                                ui.end_row();

                                ui.label("Weight:");
                                ui.add(egui::Slider::new(&mut self.current_controller.ik_settings.foot_weight, 0.0..=1.0));
                                ui.end_row();
                            });
                    }
                });

                ui.add_space(10.0);

                // Look At IK
                ui.group(|ui| {
                    ui.checkbox(&mut self.current_controller.ik_settings.look_at_enabled, RichText::new("👀 Look At IK").strong());

                    if self.current_controller.ik_settings.look_at_enabled {
                        egui::Grid::new("look_at_ik")
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Weight:");
                                ui.add(egui::Slider::new(&mut self.current_controller.ik_settings.look_at_weight, 0.0..=1.0));
                                ui.end_row();

                                ui.label("Head Weight:");
                                ui.add(egui::Slider::new(&mut self.current_controller.ik_settings.head_weight, 0.0..=1.0));
                                ui.end_row();

                                ui.label("Body Weight:");
                                ui.add(egui::Slider::new(&mut self.current_controller.ik_settings.body_weight, 0.0..=1.0));
                                ui.end_row();

                                ui.label("Eyes Weight:");
                                ui.add(egui::Slider::new(&mut self.current_controller.ik_settings.eyes_weight, 0.0..=1.0));
                                ui.end_row();

                                ui.label("Clamp Weight:");
                                ui.add(egui::Slider::new(&mut self.current_controller.ik_settings.clamp_weight, 0.0..=1.0));
                                ui.end_row();
                            });
                    }
                });

                ui.add_space(10.0);

                // Hand IK
                ui.group(|ui| {
                    ui.label(RichText::new("✋ Hand IK").strong());

                    ui.checkbox(&mut self.current_controller.ik_settings.left_hand_ik_enabled, "Left Hand");
                    ui.checkbox(&mut self.current_controller.ik_settings.right_hand_ik_enabled, "Right Hand");

                    if self.current_controller.ik_settings.left_hand_ik_enabled || self.current_controller.ik_settings.right_hand_ik_enabled {
                        ui.horizontal(|ui| {
                            ui.label("Weight:");
                            ui.add(egui::Slider::new(&mut self.current_controller.ik_settings.hand_ik_weight, 0.0..=1.0));
                        });
                    }
                });
            });
    }

    fn show_preview_tab(&mut self, ui: &mut Ui) {
        ui.heading("▶️ Preview");
        ui.add_space(10.0);

        // Playback controls
        ui.group(|ui| {
            ui.label(RichText::new("🎮 Playback").strong());

            ui.horizontal(|ui| {
                if ui.button(if self.preview_playing { "⏸️ Pause" } else { "▶️ Play" }).clicked() {
                    self.preview_playing = !self.preview_playing;
                }
                if ui.button("⏹️ Stop").clicked() {
                    self.preview_playing = false;
                    self.preview_time = 0.0;
                }
                if ui.button("🔄 Reset").clicked() {
                    self.preview_time = 0.0;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut self.preview_speed, 0.1..=3.0));
            });

            ui.horizontal(|ui| {
                ui.label("Time:");
                ui.add(egui::DragValue::new(&mut self.preview_time).speed(0.01).suffix("s"));
            });
        });

        ui.add_space(10.0);

        // Current state
        ui.group(|ui| {
            ui.label(RichText::new("📊 Current State").strong());

            let default_state = self.current_controller.states.iter()
                .find(|s| s.is_default)
                .map_or("(none)", |s| &s.name);

            ui.label(format!("Active State: {}", default_state));
            ui.label(format!("Time: {:.2}s", self.preview_time));

            if let Some(clip) = self.selected_clip.and_then(|id| self.clips.iter().find(|c| c.id == id)) {
                ui.label(format!("Clip: {} ({:.2}s)", clip.name, clip.duration));
                let progress = if clip.duration > 0.0 {
                    (self.preview_time % clip.duration) / clip.duration
                } else {
                    0.0
                };
                ui.add(egui::ProgressBar::new(progress).show_percentage());
            }
        });

        ui.add_space(10.0);

        // Parameter values
        ui.group(|ui| {
            ui.label(RichText::new("🔧 Live Parameters").strong());

            for param in &self.current_controller.parameters {
                ui.horizontal(|ui| {
                    ui.label(&param.name);
                    match &param.default_value {
                        ParameterValue::Float(v) => { ui.label(format!("{:.2}", v)); }
                        ParameterValue::Int(v) => { ui.label(format!("{}", v)); }
                        ParameterValue::Bool(v) => { ui.label(if *v { "true" } else { "false" }); }
                        ParameterValue::Trigger => { ui.label("(trigger)"); }
                    }
                });
            }
        });
    }

    // Getters for testing
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    pub fn controller_count(&self) -> usize {
        self.controllers.len()
    }

    pub fn state_count(&self) -> usize {
        self.current_controller.states.len()
    }

    pub fn parameter_count(&self) -> usize {
        self.current_controller.parameters.len()
    }

    pub fn layer_count(&self) -> usize {
        self.current_controller.layers.len()
    }

    pub fn add_clip(&mut self, name: &str, duration: f32) -> u32 {
        let id = self.next_clip_id();
        self.clips.push(AnimationClip {
            id,
            name: name.to_string(),
            duration,
            ..Default::default()
        });
        id
    }

    pub fn add_state(&mut self, name: &str) -> u32 {
        let id = self.next_state_id();
        self.current_controller.states.push(AnimationState {
            id,
            name: name.to_string(),
            ..Default::default()
        });
        id
    }
}

impl Panel for AnimationPanel {
    fn name(&self) -> &'static str {
        "Animation"
    }

    fn show(&mut self, ui: &mut Ui) {
        self.show_tab_bar(ui);

        match self.active_tab {
            AnimationTab::Clips => self.show_clips_tab(ui),
            AnimationTab::StateMachine => self.show_state_machine_tab(ui),
            AnimationTab::BlendTrees => self.show_blend_trees_tab(ui),
            AnimationTab::Layers => self.show_layers_tab(ui),
            AnimationTab::Parameters => self.show_parameters_tab(ui),
            AnimationTab::IK => self.show_ik_tab(ui),
            AnimationTab::Preview => self.show_preview_tab(ui),
        }
    }

    fn update(&mut self) {
        if self.preview_playing {
            self.preview_time += 0.016 * self.preview_speed;
            if let Some(clip) = self.clips.iter().find(|c| self.selected_clip == Some(c.id)) {
                match clip.loop_mode {
                    LoopMode::Loop => {
                        if self.preview_time > clip.duration {
                            self.preview_time = 0.0;
                        }
                    }
                    LoopMode::PingPong => {
                        // Handle ping pong
                    }
                    LoopMode::Once | LoopMode::ClampForever => {
                        if self.preview_time > clip.duration {
                            self.preview_time = clip.duration;
                            self.preview_playing = false;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // LOOP MODE TESTS
    // ============================================================

    #[test]
    fn test_loop_mode_default() {
        let mode = LoopMode::default();
        assert_eq!(mode, LoopMode::Loop);
    }

    #[test]
    fn test_loop_mode_all_variants() {
        let variants = [
            LoopMode::Once,
            LoopMode::Loop,
            LoopMode::PingPong,
            LoopMode::ClampForever,
        ];
        assert_eq!(variants.len(), 4);
    }

    #[test]
    fn test_loop_mode_clone() {
        let mode = LoopMode::PingPong;
        let cloned = mode;
        assert_eq!(cloned, LoopMode::PingPong);
    }

    // ============================================================
    // TANGENT MODE TESTS
    // ============================================================

    #[test]
    fn test_tangent_mode_default() {
        let mode = TangentMode::default();
        assert_eq!(mode, TangentMode::Auto);
    }

    #[test]
    fn test_tangent_mode_all_variants() {
        let variants = [
            TangentMode::Auto,
            TangentMode::Linear,
            TangentMode::Constant,
            TangentMode::Free,
            TangentMode::Broken,
        ];
        assert_eq!(variants.len(), 5);
    }

    // ============================================================
    // CONDITION TYPE TESTS
    // ============================================================

    #[test]
    fn test_condition_type_default() {
        let ct = ConditionType::default();
        assert_eq!(ct, ConditionType::Greater);
    }

    #[test]
    fn test_condition_type_all_variants() {
        let variants = [
            ConditionType::Greater,
            ConditionType::Less,
            ConditionType::Equals,
            ConditionType::NotEquals,
            ConditionType::True,
            ConditionType::False,
        ];
        assert_eq!(variants.len(), 6);
    }

    // ============================================================
    // PARAMETER TYPE TESTS
    // ============================================================

    #[test]
    fn test_parameter_type_default() {
        let pt = ParameterType::default();
        assert_eq!(pt, ParameterType::Float);
    }

    #[test]
    fn test_parameter_type_all_variants() {
        let variants = [
            ParameterType::Float,
            ParameterType::Int,
            ParameterType::Bool,
            ParameterType::Trigger,
        ];
        assert_eq!(variants.len(), 4);
    }

    // ============================================================
    // PARAMETER VALUE TESTS
    // ============================================================

    #[test]
    fn test_parameter_value_default() {
        let pv = ParameterValue::default();
        match pv {
            ParameterValue::Float(v) => assert_eq!(v, 0.0),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_parameter_value_float() {
        let pv = ParameterValue::Float(1.5);
        match pv {
            ParameterValue::Float(v) => assert!((v - 1.5).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_parameter_value_int() {
        let pv = ParameterValue::Int(42);
        match pv {
            ParameterValue::Int(v) => assert_eq!(v, 42),
            _ => panic!("Expected Int"),
        }
    }

    #[test]
    fn test_parameter_value_bool() {
        let pv = ParameterValue::Bool(true);
        match pv {
            ParameterValue::Bool(v) => assert!(v),
            _ => panic!("Expected Bool"),
        }
    }

    #[test]
    fn test_parameter_value_trigger() {
        let pv = ParameterValue::Trigger;
        assert!(matches!(pv, ParameterValue::Trigger));
    }

    // ============================================================
    // BLEND TREE TYPE TESTS
    // ============================================================

    #[test]
    fn test_blend_tree_type_default() {
        let btt = BlendTreeType::default();
        assert_eq!(btt, BlendTreeType::Simple1D);
    }

    #[test]
    fn test_blend_tree_type_all_variants() {
        let variants = [
            BlendTreeType::Simple1D,
            BlendTreeType::Simple2D,
            BlendTreeType::FreeformDirectional,
            BlendTreeType::FreeformCartesian,
            BlendTreeType::Direct,
        ];
        assert_eq!(variants.len(), 5);
    }

    // ============================================================
    // BLENDING MODE TESTS
    // ============================================================

    #[test]
    fn test_blending_mode_default() {
        let bm = BlendingMode::default();
        assert_eq!(bm, BlendingMode::Override);
    }

    #[test]
    fn test_blending_mode_all_variants() {
        let variants = [BlendingMode::Override, BlendingMode::Additive];
        assert_eq!(variants.len(), 2);
    }

    // ============================================================
    // ANIMATION CLIP TESTS
    // ============================================================

    #[test]
    fn test_default_clip() {
        let clip = AnimationClip::default();
        assert_eq!(clip.loop_mode, LoopMode::Loop);
        assert!((clip.fps - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_animation_clip_default_full() {
        let clip = AnimationClip::default();
        assert_eq!(clip.id, 0);
        assert_eq!(clip.name, "New Clip");
        assert!((clip.duration - 1.0).abs() < 0.001);
        assert!((clip.fps - 30.0).abs() < 0.001);
        assert_eq!(clip.loop_mode, LoopMode::Loop);
        assert!(clip.events.is_empty());
        assert!(clip.curves.is_empty());
    }

    #[test]
    fn test_animation_clip_custom() {
        let clip = AnimationClip {
            id: 1,
            name: "Walk".to_string(),
            duration: 2.5,
            fps: 60.0,
            loop_mode: LoopMode::Once,
            events: vec![],
            curves: vec![],
        };
        assert_eq!(clip.name, "Walk");
        assert!((clip.duration - 2.5).abs() < 0.001);
        assert_eq!(clip.loop_mode, LoopMode::Once);
    }

    #[test]
    fn test_animation_clip_clone() {
        let clip = AnimationClip::default();
        let cloned = clip.clone();
        assert_eq!(cloned.name, "New Clip");
    }

    // ============================================================
    // ANIMATION STATE TESTS
    // ============================================================

    #[test]
    fn test_animation_state_default() {
        let state = AnimationState::default();
        assert_eq!(state.id, 0);
        assert_eq!(state.name, "New State");
        assert!(state.clip_id.is_none());
        assert!((state.speed - 1.0).abs() < 0.001);
        assert_eq!(state.position, [100.0, 100.0]);
        assert!(!state.is_default);
    }

    #[test]
    fn test_animation_state_clone() {
        let state = AnimationState::default();
        let cloned = state.clone();
        assert_eq!(cloned.name, "New State");
    }

    // ============================================================
    // STATE TRANSITION TESTS
    // ============================================================

    #[test]
    fn test_state_transition_default() {
        let transition = StateTransition::default();
        assert_eq!(transition.id, 0);
        assert_eq!(transition.from_state, 0);
        assert_eq!(transition.to_state, 0);
        assert!((transition.duration - 0.25).abs() < 0.001);
        assert!((transition.offset - 0.0).abs() < 0.001);
        assert!(!transition.has_exit_time);
        assert!((transition.exit_time - 1.0).abs() < 0.001);
        assert!(transition.conditions.is_empty());
    }

    #[test]
    fn test_state_transition_clone() {
        let transition = StateTransition::default();
        let cloned = transition.clone();
        assert!((cloned.duration - 0.25).abs() < 0.001);
    }

    // ============================================================
    // BLEND TREE TESTS
    // ============================================================

    #[test]
    fn test_blend_tree_default() {
        let tree = BlendTree::default();
        assert_eq!(tree.blend_type, BlendTreeType::Simple1D);
        assert!(tree.children.is_empty());
    }

    #[test]
    fn test_blend_tree_default_full() {
        let tree = BlendTree::default();
        assert_eq!(tree.id, 0);
        assert_eq!(tree.name, "New Blend Tree");
        assert_eq!(tree.blend_type, BlendTreeType::Simple1D);
        assert!(tree.blend_parameter_x.is_empty());
        assert!(tree.blend_parameter_y.is_empty());
        assert!(tree.children.is_empty());
    }

    #[test]
    fn test_blend_tree_clone() {
        let tree = BlendTree::default();
        let cloned = tree.clone();
        assert_eq!(cloned.name, "New Blend Tree");
    }

    // ============================================================
    // ANIMATION LAYER TESTS
    // ============================================================

    #[test]
    fn test_default_layer() {
        let layer = AnimationLayer::default();
        assert!((layer.weight - 1.0).abs() < 0.001);
        assert_eq!(layer.blending_mode, BlendingMode::Override);
    }

    #[test]
    fn test_animation_layer_default_full() {
        let layer = AnimationLayer::default();
        assert_eq!(layer.id, 0);
        assert_eq!(layer.name, "New Layer");
        assert!((layer.weight - 1.0).abs() < 0.001);
        assert_eq!(layer.blending_mode, BlendingMode::Override);
        assert!(layer.avatar_mask.is_none());
        assert!(!layer.ik_pass);
    }

    #[test]
    fn test_animation_layer_clone() {
        let layer = AnimationLayer::default();
        let cloned = layer.clone();
        assert_eq!(cloned.name, "New Layer");
    }

    // ============================================================
    // IK SETTINGS TESTS
    // ============================================================

    #[test]
    fn test_ik_settings_default() {
        let ik = IkSettings::default();
        assert!(!ik.enabled);
        assert!(ik.foot_ik_enabled);
        assert!(!ik.look_at_enabled);
    }

    #[test]
    fn test_ik_settings_default_full() {
        let ik = IkSettings::default();
        assert!(!ik.enabled);
        assert!(ik.foot_ik_enabled);
        assert!((ik.foot_height_offset - 0.0).abs() < 0.001);
        assert!((ik.foot_weight - 1.0).abs() < 0.001);
        assert!(!ik.look_at_enabled);
        assert!((ik.look_at_weight - 1.0).abs() < 0.001);
        assert!((ik.head_weight - 1.0).abs() < 0.001);
        assert!((ik.body_weight - 0.4).abs() < 0.001);
        assert!(!ik.left_hand_ik_enabled);
        assert!(!ik.right_hand_ik_enabled);
    }

    #[test]
    fn test_ik_settings_clone() {
        let ik = IkSettings::default();
        let cloned = ik.clone();
        assert!(!cloned.enabled);
        assert!(cloned.foot_ik_enabled);
    }

    // ============================================================
    // ANIMATION CONTROLLER TESTS
    // ============================================================

    #[test]
    fn test_animation_controller_default() {
        let ctrl = AnimationController::default();
        assert_eq!(ctrl.id, 0);
        assert_eq!(ctrl.name, "New Controller");
        assert!(ctrl.states.is_empty());
        assert!(ctrl.transitions.is_empty());
        assert!(ctrl.parameters.is_empty());
        assert_eq!(ctrl.layers.len(), 1);
        assert!(ctrl.blend_trees.is_empty());
    }

    #[test]
    fn test_animation_controller_clone() {
        let ctrl = AnimationController::default();
        let cloned = ctrl.clone();
        assert_eq!(cloned.name, "New Controller");
    }

    // ============================================================
    // ANIMATION PANEL TESTS
    // ============================================================

    #[test]
    fn test_animation_panel_creation() {
        let panel = AnimationPanel::new();
        assert!(panel.clip_count() >= 6);
    }

    #[test]
    fn test_default_sample_data() {
        let panel = AnimationPanel::new();
        assert!(panel.controller_count() >= 1);
        assert!(panel.state_count() >= 3);
        assert!(panel.parameter_count() >= 3);
    }

    #[test]
    fn test_add_clip() {
        let mut panel = AnimationPanel::new();
        let initial_count = panel.clip_count();

        let id = panel.add_clip("Test Clip", 2.5);
        assert!(id > 0);
        assert_eq!(panel.clip_count(), initial_count + 1);
    }

    #[test]
    fn test_add_state() {
        let mut panel = AnimationPanel::new();
        let initial_count = panel.state_count();

        let id = panel.add_state("Test State");
        assert!(id > 0);
        assert_eq!(panel.state_count(), initial_count + 1);
    }

    #[test]
    fn test_panel_trait_implementation() {
        let panel = AnimationPanel::new();
        assert_eq!(panel.name(), "Animation");
    }

    // ============================================================
    // ANIMATION TAB TESTS
    // ============================================================

    #[test]
    fn test_animation_tab_default() {
        let tab = AnimationTab::default();
        assert_eq!(tab, AnimationTab::Clips);
    }

    #[test]
    fn test_animation_tab_all_variants() {
        let variants = [
            AnimationTab::Clips,
            AnimationTab::StateMachine,
            AnimationTab::BlendTrees,
            AnimationTab::Layers,
            AnimationTab::Parameters,
            AnimationTab::IK,
        ];
        assert_eq!(variants.len(), 6);
    }

    // ============================================================
    // KEYFRAME TESTS
    // ============================================================

    #[test]
    fn test_keyframe_creation() {
        let kf = Keyframe {
            time: 0.5,
            value: 1.0,
            in_tangent: 0.0,
            out_tangent: 0.0,
            tangent_mode: TangentMode::Auto,
        };
        assert!((kf.time - 0.5).abs() < 0.001);
        assert!((kf.value - 1.0).abs() < 0.001);
        assert_eq!(kf.tangent_mode, TangentMode::Auto);
    }

    #[test]
    fn test_keyframe_clone() {
        let kf = Keyframe {
            time: 1.0,
            value: 2.0,
            in_tangent: 0.5,
            out_tangent: 0.5,
            tangent_mode: TangentMode::Linear,
        };
        let cloned = kf.clone();
        assert!((cloned.time - 1.0).abs() < 0.001);
        assert_eq!(cloned.tangent_mode, TangentMode::Linear);
    }

    // ============================================================
    // BLEND TREE CHILD TESTS
    // ============================================================

    #[test]
    fn test_blend_tree_child_creation() {
        let child = BlendTreeChild {
            clip_id: 1,
            threshold: 0.5,
            position: [0.0, 0.0],
            time_scale: 1.0,
        };
        assert_eq!(child.clip_id, 1);
        assert!((child.threshold - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_blend_tree_child_clone() {
        let child = BlendTreeChild {
            clip_id: 2,
            threshold: 0.75,
            position: [1.0, 1.0],
            time_scale: 1.5,
        };
        let cloned = child.clone();
        assert_eq!(cloned.clip_id, 2);
        assert!((cloned.time_scale - 1.5).abs() < 0.001);
    }

    // ============================================================
    // TRANSITION CONDITION TESTS
    // ============================================================

    #[test]
    fn test_transition_condition_creation() {
        let cond = TransitionCondition {
            parameter: "Speed".to_string(),
            condition_type: ConditionType::Greater,
            threshold: 0.5,
        };
        assert_eq!(cond.parameter, "Speed");
        assert_eq!(cond.condition_type, ConditionType::Greater);
    }

    #[test]
    fn test_transition_condition_clone() {
        let cond = TransitionCondition {
            parameter: "IsJumping".to_string(),
            condition_type: ConditionType::True,
            threshold: 0.0,
        };
        let cloned = cond.clone();
        assert_eq!(cloned.parameter, "IsJumping");
    }
}
