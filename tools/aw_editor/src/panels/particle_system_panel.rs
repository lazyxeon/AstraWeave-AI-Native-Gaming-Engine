//! Particle System Panel for the editor UI
//!
//! Provides comprehensive VFX and particle editing:
//! - Emitter configuration (shape, rate, lifetime)
//! - Particle properties (size, color, velocity)
//! - Modules (forces, collisions, sub-emitters)
//! - GPU particle system settings
//! - Real-time preview and performance metrics

use egui::{Color32, RichText, Ui, Vec2};

use crate::panels::Panel;

/// Emitter shape type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EmitterShape {
    #[default]
    Point,
    Sphere,
    Hemisphere,
    Cone,
    Box,
    Circle,
    Edge,
    Mesh,
}

impl EmitterShape {
    pub fn all() -> &'static [EmitterShape] {
        &[
            EmitterShape::Point,
            EmitterShape::Sphere,
            EmitterShape::Hemisphere,
            EmitterShape::Cone,
            EmitterShape::Box,
            EmitterShape::Circle,
            EmitterShape::Edge,
            EmitterShape::Mesh,
        ]
    }

    pub fn icon(&self) -> &'static str {
        match self {
            EmitterShape::Point => "•",
            EmitterShape::Sphere => "⚪",
            EmitterShape::Hemisphere => "◗",
            EmitterShape::Cone => "▲",
            EmitterShape::Box => "⬜",
            EmitterShape::Circle => "○",
            EmitterShape::Edge => "―",
            EmitterShape::Mesh => "🔺",
        }
    }
}

/// Simulation space
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SimulationSpace {
    #[default]
    Local,
    World,
}

/// Particle blend mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ParticleBlendMode {
    #[default]
    Alpha,
    Additive,
    Multiply,
    Premultiply,
}

/// Particle render mode
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ParticleRenderMode {
    #[default]
    Billboard,
    StretchedBillboard,
    HorizontalBillboard,
    VerticalBillboard,
    Mesh,
    Trail,
}

/// Value over lifetime curve type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CurveType {
    #[default]
    Constant,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Random,
    Curve,
}

/// Range value (min-max)
#[derive(Debug, Clone, Copy)]
pub struct RangeValue {
    pub min: f32,
    pub max: f32,
}

impl Default for RangeValue {
    fn default() -> Self {
        Self { min: 1.0, max: 1.0 }
    }
}

impl RangeValue {
    pub fn constant(value: f32) -> Self {
        Self { min: value, max: value }
    }

    pub fn range(min: f32, max: f32) -> Self {
        Self { min, max }
    }
}

/// Color gradient stop
#[derive(Debug, Clone)]
pub struct GradientStop {
    pub position: f32,
    pub color: [f32; 4],
}

/// Color over lifetime gradient
#[derive(Debug, Clone)]
pub struct ColorGradient {
    pub stops: Vec<GradientStop>,
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self {
            stops: vec![
                GradientStop { position: 0.0, color: [1.0, 1.0, 1.0, 1.0] },
                GradientStop { position: 1.0, color: [1.0, 1.0, 1.0, 0.0] },
            ],
        }
    }
}

/// Emitter module configuration
#[derive(Debug, Clone)]
pub struct EmitterModule {
    pub enabled: bool,
    pub name: String,
    pub module_type: ModuleType,
}

/// Module types
#[derive(Debug, Clone)]
pub enum ModuleType {
    Velocity { direction: [f32; 3], speed: RangeValue },
    Force { force: [f32; 3], space: SimulationSpace },
    Gravity { multiplier: f32 },
    Noise { strength: f32, frequency: f32, scroll_speed: f32 },
    Collision { bounce: f32, lifetime_loss: f32, radius_scale: f32 },
    SubEmitter { event: SubEmitterEvent, emitter_id: u32 },
    TextureAnimation { tiles_x: u32, tiles_y: u32, fps: f32 },
    Trail { width: RangeValue, lifetime: f32, min_vertex_distance: f32 },
    Light { color: [f32; 3], intensity: RangeValue, range: RangeValue },
    Rotation { speed: RangeValue, random_start: bool },
}

/// Sub-emitter trigger event
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SubEmitterEvent {
    #[default]
    Birth,
    Death,
    Collision,
    Trigger,
}

/// Particle system configuration
#[derive(Debug, Clone)]
pub struct ParticleSystem {
    pub id: u32,
    pub name: String,
    pub enabled: bool,

    // Emission
    pub duration: f32,
    pub looping: bool,
    pub prewarm: bool,
    pub start_delay: RangeValue,
    pub emission_rate: RangeValue,
    pub bursts: Vec<EmissionBurst>,

    // Shape
    pub shape: EmitterShape,
    pub shape_radius: f32,
    pub shape_angle: f32,
    pub shape_arc: f32,
    pub emit_from_edge: bool,

    // Particle properties
    pub start_lifetime: RangeValue,
    pub start_speed: RangeValue,
    pub start_size: RangeValue,
    pub start_rotation: RangeValue,
    pub start_color: [f32; 4],
    pub gravity_modifier: f32,
    pub simulation_space: SimulationSpace,
    pub max_particles: u32,

    // Over lifetime
    pub size_over_lifetime: CurveType,
    pub size_curve_end: f32,
    pub color_over_lifetime: ColorGradient,
    pub velocity_over_lifetime: [f32; 3],
    pub rotation_over_lifetime: f32,

    // Rendering
    pub render_mode: ParticleRenderMode,
    pub blend_mode: ParticleBlendMode,
    pub texture_path: String,
    pub sort_mode: SortMode,
    pub cast_shadows: bool,
    pub receive_shadows: bool,

    // Modules
    pub modules: Vec<EmitterModule>,
}

/// Emission burst
#[derive(Debug, Clone)]
pub struct EmissionBurst {
    pub time: f32,
    pub count: RangeValue,
    pub cycles: u32,
    pub interval: f32,
    pub probability: f32,
}

impl Default for EmissionBurst {
    fn default() -> Self {
        Self {
            time: 0.0,
            count: RangeValue::constant(10.0),
            cycles: 1,
            interval: 0.0,
            probability: 1.0,
        }
    }
}

/// Sort mode for particles
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortMode {
    #[default]
    None,
    ByDistance,
    OldestFirst,
    YoungestFirst,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self {
            id: 0,
            name: "New Particle System".to_string(),
            enabled: true,

            duration: 5.0,
            looping: true,
            prewarm: false,
            start_delay: RangeValue::constant(0.0),
            emission_rate: RangeValue::constant(10.0),
            bursts: Vec::new(),

            shape: EmitterShape::Cone,
            shape_radius: 1.0,
            shape_angle: 25.0,
            shape_arc: 360.0,
            emit_from_edge: false,

            start_lifetime: RangeValue::range(3.0, 5.0),
            start_speed: RangeValue::range(1.0, 3.0),
            start_size: RangeValue::range(0.1, 0.3),
            start_rotation: RangeValue::range(0.0, 360.0),
            start_color: [1.0, 1.0, 1.0, 1.0],
            gravity_modifier: 0.0,
            simulation_space: SimulationSpace::Local,
            max_particles: 1000,

            size_over_lifetime: CurveType::Linear,
            size_curve_end: 0.0,
            color_over_lifetime: ColorGradient::default(),
            velocity_over_lifetime: [0.0, 0.0, 0.0],
            rotation_over_lifetime: 0.0,

            render_mode: ParticleRenderMode::Billboard,
            blend_mode: ParticleBlendMode::Additive,
            texture_path: String::new(),
            sort_mode: SortMode::None,
            cast_shadows: false,
            receive_shadows: false,

            modules: Vec::new(),
        }
    }
}

/// Particle system preset
#[derive(Debug, Clone)]
pub struct ParticlePreset {
    pub name: String,
    pub category: String,
    pub description: String,
}

impl ParticlePreset {
    fn presets() -> Vec<ParticlePreset> {
        vec![
            ParticlePreset { name: "Fire".to_string(), category: "Elements".to_string(), description: "Flickering flame effect".to_string() },
            ParticlePreset { name: "Smoke".to_string(), category: "Elements".to_string(), description: "Rising smoke plume".to_string() },
            ParticlePreset { name: "Sparks".to_string(), category: "Elements".to_string(), description: "Flying sparks".to_string() },
            ParticlePreset { name: "Explosion".to_string(), category: "Combat".to_string(), description: "Burst explosion".to_string() },
            ParticlePreset { name: "Muzzle Flash".to_string(), category: "Combat".to_string(), description: "Gun muzzle flash".to_string() },
            ParticlePreset { name: "Blood Splatter".to_string(), category: "Combat".to_string(), description: "Impact blood effect".to_string() },
            ParticlePreset { name: "Magic Sparkle".to_string(), category: "Magic".to_string(), description: "Magical sparkle trail".to_string() },
            ParticlePreset { name: "Heal Aura".to_string(), category: "Magic".to_string(), description: "Healing particle ring".to_string() },
            ParticlePreset { name: "Portal".to_string(), category: "Magic".to_string(), description: "Swirling portal effect".to_string() },
            ParticlePreset { name: "Rain".to_string(), category: "Weather".to_string(), description: "Falling rain drops".to_string() },
            ParticlePreset { name: "Snow".to_string(), category: "Weather".to_string(), description: "Drifting snowflakes".to_string() },
            ParticlePreset { name: "Dust".to_string(), category: "Environment".to_string(), description: "Ambient dust motes".to_string() },
            ParticlePreset { name: "Leaves".to_string(), category: "Environment".to_string(), description: "Falling leaves".to_string() },
            ParticlePreset { name: "Waterfall".to_string(), category: "Environment".to_string(), description: "Waterfall spray".to_string() },
        ]
    }
}

/// Performance stats
#[derive(Debug, Clone, Default)]
pub struct ParticleStats {
    pub active_particles: u32,
    pub total_emitters: u32,
    pub draw_calls: u32,
    pub gpu_memory_mb: f32,
    pub simulation_time_ms: f32,
    pub render_time_ms: f32,
}

/// Panel tabs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ParticleTab {
    #[default]
    Emitter,
    Shape,
    Particles,
    Lifetime,
    Rendering,
    Modules,
    Presets,
    Stats,
}

/// Main Particle System Panel
pub struct ParticleSystemPanel {
    // Tab state
    active_tab: ParticleTab,

    // Systems
    particle_systems: Vec<ParticleSystem>,
    selected_system: Option<u32>,
    current_system: ParticleSystem,

    // Presets
    presets: Vec<ParticlePreset>,
    preset_filter: String,

    // Preview
    preview_playing: bool,
    preview_time: f32,
    preview_speed: f32,

    // Stats
    stats: ParticleStats,

    // ID counter
    next_id: u32,
}

impl Default for ParticleSystemPanel {
    fn default() -> Self {
        let mut panel = Self {
            active_tab: ParticleTab::Emitter,

            particle_systems: Vec::new(),
            selected_system: None,
            current_system: ParticleSystem::default(),

            presets: ParticlePreset::presets(),
            preset_filter: String::new(),

            preview_playing: false,
            preview_time: 0.0,
            preview_speed: 1.0,

            stats: ParticleStats {
                active_particles: 1250,
                total_emitters: 8,
                draw_calls: 12,
                gpu_memory_mb: 24.5,
                simulation_time_ms: 0.45,
                render_time_ms: 0.82,
            },

            next_id: 1,
        };

        panel.create_sample_data();
        panel
    }
}

impl ParticleSystemPanel {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_sample_data(&mut self) {
        // Fire effect
        let id = self.next_id();
        self.particle_systems.push(ParticleSystem {
            id,
            name: "Torch Fire".to_string(),
            shape: EmitterShape::Cone,
            shape_radius: 0.1,
            shape_angle: 15.0,
            emission_rate: RangeValue::constant(50.0),
            start_lifetime: RangeValue::range(0.5, 1.0),
            start_speed: RangeValue::range(1.0, 2.0),
            start_size: RangeValue::range(0.1, 0.2),
            start_color: [1.0, 0.5, 0.1, 1.0],
            gravity_modifier: -0.5,
            blend_mode: ParticleBlendMode::Additive,
            ..Default::default()
        });
        self.next_id += 1;

        // Smoke effect
        let id = self.next_id();
        self.particle_systems.push(ParticleSystem {
            id,
            name: "Campfire Smoke".to_string(),
            shape: EmitterShape::Circle,
            shape_radius: 0.3,
            emission_rate: RangeValue::constant(15.0),
            start_lifetime: RangeValue::range(3.0, 5.0),
            start_speed: RangeValue::range(0.5, 1.0),
            start_size: RangeValue::range(0.3, 0.5),
            start_color: [0.3, 0.3, 0.3, 0.5],
            gravity_modifier: -0.2,
            blend_mode: ParticleBlendMode::Alpha,
            size_over_lifetime: CurveType::Linear,
            size_curve_end: 2.0,
            ..Default::default()
        });
        self.next_id += 1;

        // Magic sparkles
        let id = self.next_id();
        self.particle_systems.push(ParticleSystem {
            id,
            name: "Magic Sparkles".to_string(),
            shape: EmitterShape::Sphere,
            shape_radius: 0.5,
            emission_rate: RangeValue::constant(30.0),
            start_lifetime: RangeValue::range(0.5, 1.5),
            start_speed: RangeValue::range(0.1, 0.5),
            start_size: RangeValue::range(0.02, 0.05),
            start_color: [0.5, 0.8, 1.0, 1.0],
            blend_mode: ParticleBlendMode::Additive,
            ..Default::default()
        });
        self.next_id += 1;

        self.current_system = self.particle_systems[0].clone();
        self.selected_system = Some(self.particle_systems[0].id);
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn show_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let tabs = [
                (ParticleTab::Emitter, "💨 Emitter"),
                (ParticleTab::Shape, "📐 Shape"),
                (ParticleTab::Particles, "✨ Particles"),
                (ParticleTab::Lifetime, "⏱️ Lifetime"),
                (ParticleTab::Rendering, "🖼️ Rendering"),
                (ParticleTab::Modules, "🧩 Modules"),
                (ParticleTab::Presets, "📋 Presets"),
                (ParticleTab::Stats, "📊 Stats"),
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

        // System info and preview controls
        ui.horizontal(|ui| {
            ui.label(format!("✨ {}", self.current_system.name));

            ui.separator();

            if ui.button(if self.preview_playing { "⏸️" } else { "▶️" }).clicked() {
                self.preview_playing = !self.preview_playing;
            }
            if ui.button("⏹️").clicked() {
                self.preview_playing = false;
                self.preview_time = 0.0;
            }
            if ui.button("🔄").clicked() {
                self.preview_time = 0.0;
            }

            ui.label(format!("t: {:.1}s", self.preview_time));
        });

        ui.separator();
    }

    fn show_emitter_tab(&mut self, ui: &mut Ui) {
        ui.heading("💨 Emitter Settings");
        ui.add_space(10.0);

        // System selector
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("system_select")
                .selected_text(&self.current_system.name)
                .show_ui(ui, |ui| {
                    for sys in &self.particle_systems.clone() {
                        if ui.selectable_value(&mut self.selected_system, Some(sys.id), &sys.name).clicked() {
                            self.current_system = sys.clone();
                        }
                    }
                });

            if ui.button("+ New").clicked() {
                let id = self.next_id();
                let new_sys = ParticleSystem {
                    id,
                    name: format!("Particle System {}", id),
                    ..Default::default()
                };
                self.particle_systems.push(new_sys.clone());
                self.current_system = new_sys;
                self.selected_system = Some(id);
            }

            if ui.button("📋 Duplicate").clicked() {
                let id = self.next_id();
                let mut dup = self.current_system.clone();
                dup.id = id;
                dup.name = format!("{} (Copy)", dup.name);
                self.particle_systems.push(dup);
            }
        });

        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                // Basic settings
                ui.group(|ui| {
                    ui.label(RichText::new("📝 Basic").strong());

                    egui::Grid::new("emitter_basic")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.current_system.name);
                            ui.end_row();

                            ui.label("Enabled:");
                            ui.checkbox(&mut self.current_system.enabled, "");
                            ui.end_row();

                            ui.label("Duration:");
                            ui.add(egui::DragValue::new(&mut self.current_system.duration).speed(0.1).range(0.1..=60.0).suffix("s"));
                            ui.end_row();

                            ui.label("Looping:");
                            ui.checkbox(&mut self.current_system.looping, "");
                            ui.end_row();

                            ui.label("Prewarm:");
                            ui.checkbox(&mut self.current_system.prewarm, "");
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // Emission settings
                ui.group(|ui| {
                    ui.label(RichText::new("📤 Emission").strong());

                    egui::Grid::new("emitter_emission")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Rate:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_system.emission_rate.min).prefix("Min:").speed(1.0).range(0.0..=1000.0));
                                ui.add(egui::DragValue::new(&mut self.current_system.emission_rate.max).prefix("Max:").speed(1.0).range(0.0..=1000.0));
                            });
                            ui.end_row();

                            ui.label("Start Delay:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_system.start_delay.min).prefix("Min:").speed(0.1).range(0.0..=10.0));
                                ui.add(egui::DragValue::new(&mut self.current_system.start_delay.max).prefix("Max:").speed(0.1).range(0.0..=10.0));
                            });
                            ui.end_row();

                            ui.label("Max Particles:");
                            ui.add(egui::DragValue::new(&mut self.current_system.max_particles).range(1..=100000));
                            ui.end_row();
                        });

                    ui.add_space(5.0);

                    // Bursts
                    ui.collapsing("💥 Bursts", |ui| {
                        if ui.button("+ Add Burst").clicked() {
                            self.current_system.bursts.push(EmissionBurst::default());
                        }

                        for (idx, burst) in self.current_system.bursts.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("#{}", idx + 1));
                                ui.add(egui::DragValue::new(&mut burst.time).prefix("t:").speed(0.1));
                                ui.add(egui::DragValue::new(&mut burst.count.min).prefix("n:").speed(1.0).range(1.0..=1000.0));
                            });
                        }
                    });
                });
            });
    }

    fn show_shape_tab(&mut self, ui: &mut Ui) {
        ui.heading("📐 Emitter Shape");
        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                // Shape selector
                ui.group(|ui| {
                    ui.label(RichText::new("Shape Type").strong());

                    ui.horizontal_wrapped(|ui| {
                        for shape in EmitterShape::all() {
                            let is_selected = self.current_system.shape == *shape;
                            let button = egui::Button::new(format!("{} {:?}", shape.icon(), shape))
                                .fill(if is_selected { Color32::from_rgb(60, 100, 160) } else { Color32::from_rgb(50, 50, 55) });

                            if ui.add(button).clicked() {
                                self.current_system.shape = *shape;
                            }
                        }
                    });
                });

                ui.add_space(10.0);

                // Shape parameters
                ui.group(|ui| {
                    ui.label(RichText::new("Shape Parameters").strong());

                    egui::Grid::new("shape_params")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            match self.current_system.shape {
                                EmitterShape::Sphere | EmitterShape::Hemisphere | EmitterShape::Circle => {
                                    ui.label("Radius:");
                                    ui.add(egui::DragValue::new(&mut self.current_system.shape_radius).speed(0.1).range(0.01..=100.0));
                                    ui.end_row();
                                }
                                EmitterShape::Cone => {
                                    ui.label("Radius:");
                                    ui.add(egui::DragValue::new(&mut self.current_system.shape_radius).speed(0.1).range(0.01..=100.0));
                                    ui.end_row();

                                    ui.label("Angle:");
                                    ui.add(egui::Slider::new(&mut self.current_system.shape_angle, 0.0..=90.0).suffix("°"));
                                    ui.end_row();
                                }
                                _ => {}
                            }

                            if matches!(self.current_system.shape, EmitterShape::Circle | EmitterShape::Cone) {
                                ui.label("Arc:");
                                ui.add(egui::Slider::new(&mut self.current_system.shape_arc, 0.0..=360.0).suffix("°"));
                                ui.end_row();
                            }

                            ui.label("Emit From Edge:");
                            ui.checkbox(&mut self.current_system.emit_from_edge, "");
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // Shape preview (simple visual)
                let preview_size = Vec2::new(ui.available_width().min(200.0), 100.0);
                let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());

                let painter = ui.painter();
                painter.rect_filled(rect, 5.0, Color32::from_rgb(30, 30, 35));

                let center = rect.center();
                let shape_color = Color32::from_rgb(100, 150, 255);

                match self.current_system.shape {
                    EmitterShape::Point => {
                        painter.circle_filled(center, 4.0, shape_color);
                    }
                    EmitterShape::Sphere | EmitterShape::Circle => {
                        let r = 30.0 * self.current_system.shape_radius.min(2.0);
                        painter.circle_stroke(center, r, egui::Stroke::new(2.0, shape_color));
                    }
                    EmitterShape::Cone => {
                        let angle = self.current_system.shape_angle.to_radians();
                        let length = 40.0;
                        let width = length * angle.tan();
                        painter.line_segment([center, egui::Pos2::new(center.x - width, center.y - length)], egui::Stroke::new(2.0, shape_color));
                        painter.line_segment([center, egui::Pos2::new(center.x + width, center.y - length)], egui::Stroke::new(2.0, shape_color));
                    }
                    EmitterShape::Box => {
                        let size = 40.0;
                        painter.rect_stroke(
                            egui::Rect::from_center_size(center, Vec2::splat(size)),
                            0.0,
                            egui::Stroke::new(2.0, shape_color),
                            egui::StrokeKind::Outside,
                        );
                    }
                    _ => {
                        painter.circle_filled(center, 4.0, shape_color);
                    }
                }
            });
    }

    fn show_particles_tab(&mut self, ui: &mut Ui) {
        ui.heading("✨ Particle Properties");
        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                // Start properties
                ui.group(|ui| {
                    ui.label(RichText::new("🎬 Start Properties").strong());

                    egui::Grid::new("start_props")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Lifetime:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_system.start_lifetime.min).prefix("Min:").speed(0.1).range(0.01..=60.0));
                                ui.add(egui::DragValue::new(&mut self.current_system.start_lifetime.max).prefix("Max:").speed(0.1).range(0.01..=60.0));
                            });
                            ui.end_row();

                            ui.label("Speed:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_system.start_speed.min).prefix("Min:").speed(0.1).range(0.0..=100.0));
                                ui.add(egui::DragValue::new(&mut self.current_system.start_speed.max).prefix("Max:").speed(0.1).range(0.0..=100.0));
                            });
                            ui.end_row();

                            ui.label("Size:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_system.start_size.min).prefix("Min:").speed(0.01).range(0.001..=10.0));
                                ui.add(egui::DragValue::new(&mut self.current_system.start_size.max).prefix("Max:").speed(0.01).range(0.001..=10.0));
                            });
                            ui.end_row();

                            ui.label("Rotation:");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.current_system.start_rotation.min).prefix("Min:").speed(1.0).suffix("°"));
                                ui.add(egui::DragValue::new(&mut self.current_system.start_rotation.max).prefix("Max:").speed(1.0).suffix("°"));
                            });
                            ui.end_row();

                            ui.label("Color:");
                            let mut color = Color32::from_rgba_unmultiplied(
                                (self.current_system.start_color[0] * 255.0) as u8,
                                (self.current_system.start_color[1] * 255.0) as u8,
                                (self.current_system.start_color[2] * 255.0) as u8,
                                (self.current_system.start_color[3] * 255.0) as u8,
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                self.current_system.start_color = [
                                    color.r() as f32 / 255.0,
                                    color.g() as f32 / 255.0,
                                    color.b() as f32 / 255.0,
                                    color.a() as f32 / 255.0,
                                ];
                            }
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // Physics
                ui.group(|ui| {
                    ui.label(RichText::new("🌍 Physics").strong());

                    egui::Grid::new("particle_physics")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Gravity Modifier:");
                            ui.add(egui::Slider::new(&mut self.current_system.gravity_modifier, -2.0..=2.0));
                            ui.end_row();

                            ui.label("Simulation Space:");
                            egui::ComboBox::from_id_salt("sim_space")
                                .selected_text(format!("{:?}", self.current_system.simulation_space))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.current_system.simulation_space, SimulationSpace::Local, "Local");
                                    ui.selectable_value(&mut self.current_system.simulation_space, SimulationSpace::World, "World");
                                });
                            ui.end_row();
                        });
                });
            });
    }

    fn show_lifetime_tab(&mut self, ui: &mut Ui) {
        ui.heading("⏱️ Over Lifetime");
        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                // Size over lifetime
                ui.group(|ui| {
                    ui.label(RichText::new("📏 Size Over Lifetime").strong());

                    egui::Grid::new("size_lifetime")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Curve:");
                            egui::ComboBox::from_id_salt("size_curve")
                                .selected_text(format!("{:?}", self.current_system.size_over_lifetime))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.current_system.size_over_lifetime, CurveType::Constant, "Constant");
                                    ui.selectable_value(&mut self.current_system.size_over_lifetime, CurveType::Linear, "Linear");
                                    ui.selectable_value(&mut self.current_system.size_over_lifetime, CurveType::EaseIn, "Ease In");
                                    ui.selectable_value(&mut self.current_system.size_over_lifetime, CurveType::EaseOut, "Ease Out");
                                    ui.selectable_value(&mut self.current_system.size_over_lifetime, CurveType::EaseInOut, "Ease In/Out");
                                });
                            ui.end_row();

                            ui.label("End Scale:");
                            ui.add(egui::Slider::new(&mut self.current_system.size_curve_end, 0.0..=5.0));
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // Color over lifetime
                ui.group(|ui| {
                    ui.label(RichText::new("🎨 Color Over Lifetime").strong());

                    // Display gradient stops
                    let gradient_height = 20.0;
                    let gradient_width = ui.available_width().min(300.0);
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(gradient_width, gradient_height), egui::Sense::hover());

                    let painter = ui.painter();

                    // Draw gradient preview
                    for i in 0..100 {
                        let t = i as f32 / 99.0;
                        let x = rect.min.x + t * gradient_width;
                        let color = self.sample_gradient(t);
                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                egui::Pos2::new(x, rect.min.y),
                                Vec2::new(gradient_width / 99.0 + 1.0, gradient_height),
                            ),
                            0.0,
                            color,
                        );
                    }

                    ui.add_space(5.0);

                    // Gradient stops
                    for (idx, stop) in self.current_system.color_over_lifetime.stops.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Stop {}:", idx + 1));
                            ui.add(egui::Slider::new(&mut stop.position, 0.0..=1.0).show_value(true));

                            let mut color = Color32::from_rgba_unmultiplied(
                                (stop.color[0] * 255.0) as u8,
                                (stop.color[1] * 255.0) as u8,
                                (stop.color[2] * 255.0) as u8,
                                (stop.color[3] * 255.0) as u8,
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                stop.color = [
                                    color.r() as f32 / 255.0,
                                    color.g() as f32 / 255.0,
                                    color.b() as f32 / 255.0,
                                    color.a() as f32 / 255.0,
                                ];
                            }
                        });
                    }

                    if ui.button("+ Add Stop").clicked() {
                        self.current_system.color_over_lifetime.stops.push(GradientStop {
                            position: 0.5,
                            color: [1.0, 1.0, 1.0, 1.0],
                        });
                    }
                });

                ui.add_space(10.0);

                // Velocity over lifetime
                ui.group(|ui| {
                    ui.label(RichText::new("🚀 Velocity Over Lifetime").strong());

                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::DragValue::new(&mut self.current_system.velocity_over_lifetime[0]).speed(0.1));
                        ui.label("Y:");
                        ui.add(egui::DragValue::new(&mut self.current_system.velocity_over_lifetime[1]).speed(0.1));
                        ui.label("Z:");
                        ui.add(egui::DragValue::new(&mut self.current_system.velocity_over_lifetime[2]).speed(0.1));
                    });
                });

                ui.add_space(10.0);

                // Rotation over lifetime
                ui.group(|ui| {
                    ui.label(RichText::new("🔄 Rotation Over Lifetime").strong());

                    ui.horizontal(|ui| {
                        ui.label("Angular Velocity:");
                        ui.add(egui::Slider::new(&mut self.current_system.rotation_over_lifetime, -360.0..=360.0).suffix("°/s"));
                    });
                });
            });
    }

    fn show_rendering_tab(&mut self, ui: &mut Ui) {
        ui.heading("🖼️ Rendering");
        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                // Render mode
                ui.group(|ui| {
                    ui.label(RichText::new("📺 Render Mode").strong());

                    egui::Grid::new("render_mode")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Mode:");
                            egui::ComboBox::from_id_salt("render_mode")
                                .selected_text(format!("{:?}", self.current_system.render_mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.current_system.render_mode, ParticleRenderMode::Billboard, "Billboard");
                                    ui.selectable_value(&mut self.current_system.render_mode, ParticleRenderMode::StretchedBillboard, "Stretched Billboard");
                                    ui.selectable_value(&mut self.current_system.render_mode, ParticleRenderMode::HorizontalBillboard, "Horizontal Billboard");
                                    ui.selectable_value(&mut self.current_system.render_mode, ParticleRenderMode::VerticalBillboard, "Vertical Billboard");
                                    ui.selectable_value(&mut self.current_system.render_mode, ParticleRenderMode::Mesh, "Mesh");
                                    ui.selectable_value(&mut self.current_system.render_mode, ParticleRenderMode::Trail, "Trail");
                                });
                            ui.end_row();

                            ui.label("Blend Mode:");
                            egui::ComboBox::from_id_salt("blend_mode")
                                .selected_text(format!("{:?}", self.current_system.blend_mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.current_system.blend_mode, ParticleBlendMode::Alpha, "Alpha");
                                    ui.selectable_value(&mut self.current_system.blend_mode, ParticleBlendMode::Additive, "Additive");
                                    ui.selectable_value(&mut self.current_system.blend_mode, ParticleBlendMode::Multiply, "Multiply");
                                    ui.selectable_value(&mut self.current_system.blend_mode, ParticleBlendMode::Premultiply, "Premultiply");
                                });
                            ui.end_row();

                            ui.label("Sort Mode:");
                            egui::ComboBox::from_id_salt("sort_mode")
                                .selected_text(format!("{:?}", self.current_system.sort_mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.current_system.sort_mode, SortMode::None, "None");
                                    ui.selectable_value(&mut self.current_system.sort_mode, SortMode::ByDistance, "By Distance");
                                    ui.selectable_value(&mut self.current_system.sort_mode, SortMode::OldestFirst, "Oldest First");
                                    ui.selectable_value(&mut self.current_system.sort_mode, SortMode::YoungestFirst, "Youngest First");
                                });
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                // Texture
                ui.group(|ui| {
                    ui.label(RichText::new("🖼️ Texture").strong());

                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.text_edit_singleline(&mut self.current_system.texture_path);
                        if ui.button("📂").clicked() {
                            // Open file dialog
                        }
                    });
                });

                ui.add_space(10.0);

                // Shadows
                ui.group(|ui| {
                    ui.label(RichText::new("🌑 Shadows").strong());

                    ui.checkbox(&mut self.current_system.cast_shadows, "Cast Shadows");
                    ui.checkbox(&mut self.current_system.receive_shadows, "Receive Shadows");
                });
            });
    }

    fn show_modules_tab(&mut self, ui: &mut Ui) {
        ui.heading("🧩 Modules");
        ui.add_space(10.0);

        // Add module button
        ui.horizontal(|ui| {
            if ui.button("+ Velocity").clicked() {
                self.current_system.modules.push(EmitterModule {
                    enabled: true,
                    name: "Velocity".to_string(),
                    module_type: ModuleType::Velocity { direction: [0.0, 1.0, 0.0], speed: RangeValue::constant(1.0) },
                });
            }
            if ui.button("+ Force").clicked() {
                self.current_system.modules.push(EmitterModule {
                    enabled: true,
                    name: "Force".to_string(),
                    module_type: ModuleType::Force { force: [0.0, 0.0, 0.0], space: SimulationSpace::World },
                });
            }
            if ui.button("+ Noise").clicked() {
                self.current_system.modules.push(EmitterModule {
                    enabled: true,
                    name: "Noise".to_string(),
                    module_type: ModuleType::Noise { strength: 1.0, frequency: 1.0, scroll_speed: 0.5 },
                });
            }
            if ui.button("+ Collision").clicked() {
                self.current_system.modules.push(EmitterModule {
                    enabled: true,
                    name: "Collision".to_string(),
                    module_type: ModuleType::Collision { bounce: 0.5, lifetime_loss: 0.0, radius_scale: 1.0 },
                });
            }
        });

        ui.add_space(10.0);

        if self.current_system.modules.is_empty() {
            ui.label("No modules added. Click a button above to add a module.");
        } else {
            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    let mut to_remove = None;

                    for (idx, module) in self.current_system.modules.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut module.enabled, "");
                                ui.label(RichText::new(&module.name).strong());

                                if ui.button("🗑️").clicked() {
                                    to_remove = Some(idx);
                                }
                            });

                            if module.enabled {
                                match &mut module.module_type {
                                    ModuleType::Velocity { direction, speed } => {
                                        ui.horizontal(|ui| {
                                            ui.label("Dir:");
                                            ui.add(egui::DragValue::new(&mut direction[0]).prefix("X:").speed(0.1));
                                            ui.add(egui::DragValue::new(&mut direction[1]).prefix("Y:").speed(0.1));
                                            ui.add(egui::DragValue::new(&mut direction[2]).prefix("Z:").speed(0.1));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Speed:");
                                            ui.add(egui::DragValue::new(&mut speed.min).prefix("Min:").speed(0.1));
                                            ui.add(egui::DragValue::new(&mut speed.max).prefix("Max:").speed(0.1));
                                        });
                                    }
                                    ModuleType::Force { force, space } => {
                                        ui.horizontal(|ui| {
                                            ui.label("Force:");
                                            ui.add(egui::DragValue::new(&mut force[0]).prefix("X:").speed(0.1));
                                            ui.add(egui::DragValue::new(&mut force[1]).prefix("Y:").speed(0.1));
                                            ui.add(egui::DragValue::new(&mut force[2]).prefix("Z:").speed(0.1));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Space:");
                                            egui::ComboBox::from_id_salt(format!("force_space_{}", idx))
                                                .selected_text(format!("{:?}", space))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(space, SimulationSpace::Local, "Local");
                                                    ui.selectable_value(space, SimulationSpace::World, "World");
                                                });
                                        });
                                    }
                                    ModuleType::Noise { strength, frequency, scroll_speed } => {
                                        ui.horizontal(|ui| {
                                            ui.label("Strength:");
                                            ui.add(egui::Slider::new(strength, 0.0..=5.0));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Frequency:");
                                            ui.add(egui::Slider::new(frequency, 0.1..=10.0));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Scroll:");
                                            ui.add(egui::Slider::new(scroll_speed, 0.0..=5.0));
                                        });
                                    }
                                    ModuleType::Collision { bounce, lifetime_loss, radius_scale } => {
                                        ui.horizontal(|ui| {
                                            ui.label("Bounce:");
                                            ui.add(egui::Slider::new(bounce, 0.0..=1.0));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Lifetime Loss:");
                                            ui.add(egui::Slider::new(lifetime_loss, 0.0..=1.0));
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("Radius Scale:");
                                            ui.add(egui::Slider::new(radius_scale, 0.1..=2.0));
                                        });
                                    }
                                    _ => {
                                        ui.label("(Module parameters)");
                                    }
                                }
                            }
                        });
                    }

                    if let Some(idx) = to_remove {
                        self.current_system.modules.remove(idx);
                    }
                });
        }
    }

    fn show_presets_tab(&mut self, ui: &mut Ui) {
        ui.heading("📋 Presets");
        ui.add_space(10.0);

        // Filter
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.preset_filter);
        });

        ui.add_space(10.0);

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                let mut current_category = String::new();

                for preset in &self.presets {
                    if !self.preset_filter.is_empty() &&
                       !preset.name.to_lowercase().contains(&self.preset_filter.to_lowercase()) {
                        continue;
                    }

                    if preset.category != current_category {
                        current_category = preset.category.clone();
                        ui.add_space(5.0);
                        ui.label(RichText::new(&current_category).strong().color(Color32::from_rgb(150, 150, 200)));
                    }

                    ui.horizontal(|ui| {
                        ui.label(&preset.name);
                        ui.label(RichText::new(&preset.description).small().color(Color32::GRAY));

                        if ui.button("Apply").clicked() {
                            // Apply preset configuration
                        }
                    });
                }
            });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("💾 Save as Preset").clicked() {
                // Save current system as preset
            }
        });
    }

    fn show_stats_tab(&mut self, ui: &mut Ui) {
        ui.heading("📊 Performance Stats");
        ui.add_space(10.0);

        // Live stats
        ui.group(|ui| {
            ui.label(RichText::new("📈 Live Statistics").strong());

            egui::Grid::new("live_stats")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Active Particles:");
                    ui.label(RichText::new(format!("{}", self.stats.active_particles)).monospace());
                    ui.end_row();

                    ui.label("Total Emitters:");
                    ui.label(RichText::new(format!("{}", self.stats.total_emitters)).monospace());
                    ui.end_row();

                    ui.label("Draw Calls:");
                    ui.label(RichText::new(format!("{}", self.stats.draw_calls)).monospace());
                    ui.end_row();

                    ui.label("GPU Memory:");
                    ui.label(RichText::new(format!("{:.1} MB", self.stats.gpu_memory_mb)).monospace());
                    ui.end_row();
                });
        });

        ui.add_space(10.0);

        // Timing
        ui.group(|ui| {
            ui.label(RichText::new("⏱️ Timing").strong());

            egui::Grid::new("timing_stats")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Simulation:");
                    let sim_color = if self.stats.simulation_time_ms < 1.0 { Color32::GREEN } else { Color32::YELLOW };
                    ui.label(RichText::new(format!("{:.2} ms", self.stats.simulation_time_ms)).monospace().color(sim_color));
                    ui.end_row();

                    ui.label("Rendering:");
                    let render_color = if self.stats.render_time_ms < 2.0 { Color32::GREEN } else { Color32::YELLOW };
                    ui.label(RichText::new(format!("{:.2} ms", self.stats.render_time_ms)).monospace().color(render_color));
                    ui.end_row();

                    ui.label("Total:");
                    let total = self.stats.simulation_time_ms + self.stats.render_time_ms;
                    let total_color = if total < 3.0 { Color32::GREEN } else if total < 5.0 { Color32::YELLOW } else { Color32::RED };
                    ui.label(RichText::new(format!("{:.2} ms", total)).monospace().color(total_color));
                    ui.end_row();
                });
        });

        ui.add_space(10.0);

        // Current system stats
        ui.group(|ui| {
            ui.label(RichText::new("📊 Current System").strong());

            ui.label(format!("Name: {}", self.current_system.name));
            ui.label(format!("Max Particles: {}", self.current_system.max_particles));
            ui.label(format!("Emission Rate: {:.0}-{:.0}/s", self.current_system.emission_rate.min, self.current_system.emission_rate.max));
            ui.label(format!("Modules: {}", self.current_system.modules.len()));
            ui.label(format!("Bursts: {}", self.current_system.bursts.len()));
        });
    }

    fn sample_gradient(&self, t: f32) -> Color32 {
        let stops = &self.current_system.color_over_lifetime.stops;
        if stops.is_empty() {
            return Color32::WHITE;
        }
        if stops.len() == 1 {
            let c = &stops[0].color;
            return Color32::from_rgba_unmultiplied(
                (c[0] * 255.0) as u8,
                (c[1] * 255.0) as u8,
                (c[2] * 255.0) as u8,
                (c[3] * 255.0) as u8,
            );
        }

        // Find surrounding stops
        let mut left = &stops[0];
        let mut right = &stops[stops.len() - 1];

        for i in 0..stops.len() - 1 {
            if stops[i].position <= t && stops[i + 1].position >= t {
                left = &stops[i];
                right = &stops[i + 1];
                break;
            }
        }

        // Interpolate
        let range = right.position - left.position;
        let local_t = if range > 0.001 { (t - left.position) / range } else { 0.0 };

        let r = left.color[0] + (right.color[0] - left.color[0]) * local_t;
        let g = left.color[1] + (right.color[1] - left.color[1]) * local_t;
        let b = left.color[2] + (right.color[2] - left.color[2]) * local_t;
        let a = left.color[3] + (right.color[3] - left.color[3]) * local_t;

        Color32::from_rgba_unmultiplied(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        )
    }

    // Getters for testing
    pub fn system_count(&self) -> usize {
        self.particle_systems.len()
    }

    pub fn preset_count(&self) -> usize {
        self.presets.len()
    }

    pub fn current_system_name(&self) -> &str {
        &self.current_system.name
    }

    pub fn module_count(&self) -> usize {
        self.current_system.modules.len()
    }

    pub fn add_system(&mut self, name: &str) -> u32 {
        let id = self.next_id();
        self.particle_systems.push(ParticleSystem {
            id,
            name: name.to_string(),
            ..Default::default()
        });
        id
    }

    pub fn set_emission_rate(&mut self, min: f32, max: f32) {
        self.current_system.emission_rate = RangeValue::range(min, max);
    }

    pub fn set_max_particles(&mut self, count: u32) {
        self.current_system.max_particles = count;
    }
}

impl Panel for ParticleSystemPanel {
    fn name(&self) -> &'static str {
        "Particle System"
    }

    fn show(&mut self, ui: &mut Ui) {
        self.show_tab_bar(ui);

        match self.active_tab {
            ParticleTab::Emitter => self.show_emitter_tab(ui),
            ParticleTab::Shape => self.show_shape_tab(ui),
            ParticleTab::Particles => self.show_particles_tab(ui),
            ParticleTab::Lifetime => self.show_lifetime_tab(ui),
            ParticleTab::Rendering => self.show_rendering_tab(ui),
            ParticleTab::Modules => self.show_modules_tab(ui),
            ParticleTab::Presets => self.show_presets_tab(ui),
            ParticleTab::Stats => self.show_stats_tab(ui),
        }
    }

    fn update(&mut self) {
        if self.preview_playing {
            self.preview_time += 0.016 * self.preview_speed; // ~60 FPS
            if self.current_system.looping && self.preview_time > self.current_system.duration {
                self.preview_time = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_system_panel_creation() {
        let panel = ParticleSystemPanel::new();
        assert!(!panel.current_system_name().is_empty());
    }

    #[test]
    fn test_default_sample_data() {
        let panel = ParticleSystemPanel::new();
        assert!(panel.system_count() >= 3);
        assert!(panel.preset_count() >= 10);
    }

    #[test]
    fn test_add_system() {
        let mut panel = ParticleSystemPanel::new();
        let initial_count = panel.system_count();

        let id = panel.add_system("Test VFX");
        assert!(id > 0);
        assert_eq!(panel.system_count(), initial_count + 1);
    }

    #[test]
    fn test_set_emission_rate() {
        let mut panel = ParticleSystemPanel::new();
        panel.set_emission_rate(50.0, 100.0);
        assert!((panel.current_system.emission_rate.min - 50.0).abs() < 0.001);
        assert!((panel.current_system.emission_rate.max - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_set_max_particles() {
        let mut panel = ParticleSystemPanel::new();
        panel.set_max_particles(5000);
        assert_eq!(panel.current_system.max_particles, 5000);
    }

    #[test]
    fn test_emitter_shape_icons() {
        assert_eq!(EmitterShape::Point.icon(), "•");
        assert_eq!(EmitterShape::Sphere.icon(), "⚪");
        assert_eq!(EmitterShape::Cone.icon(), "▲");
    }

    #[test]
    fn test_range_value() {
        let constant = RangeValue::constant(5.0);
        assert!((constant.min - 5.0).abs() < 0.001);
        assert!((constant.max - 5.0).abs() < 0.001);

        let range = RangeValue::range(1.0, 10.0);
        assert!((range.min - 1.0).abs() < 0.001);
        assert!((range.max - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_color_gradient_default() {
        let gradient = ColorGradient::default();
        assert_eq!(gradient.stops.len(), 2);
        assert!((gradient.stops[0].position - 0.0).abs() < 0.001);
        assert!((gradient.stops[1].position - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_gradient_sampling() {
        let panel = ParticleSystemPanel::new();
        let color_start = panel.sample_gradient(0.0);
        let color_end = panel.sample_gradient(1.0);

        // Start should be opaque white
        assert_eq!(color_start.a(), 255);
        // End should be transparent
        assert_eq!(color_end.a(), 0);
    }

    #[test]
    fn test_panel_trait_implementation() {
        let panel = ParticleSystemPanel::new();
        assert_eq!(panel.name(), "Particle System");
    }
}
