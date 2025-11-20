//! Animation showcase tab

use astract::animation::{EasingFunction, Spring, SpringParams, Tween};
use astract::prelude::{egui::*, *};

pub struct AnimationTab {
    // Tween demo
    bounce_tween: Tween<f32>,
    #[allow(dead_code)]
    fade_tween: Tween<f32>,
    color_tween: Tween<Color32>,

    // Spring demo
    spring: Spring,
    spring_target: f32,
    spring_params_type: SpringParamsType,

    // Easing comparison
    easing_tweens: Vec<(String, Tween<f32>)>,
    show_easing_comparison: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpringParamsType {
    Smooth,
    Bouncy,
    Sluggish,
}

impl Default for AnimationTab {
    fn default() -> Self {
        // Create easing comparison tweens
        let easings = vec![
            ("Linear", EasingFunction::Linear),
            ("QuadOut", EasingFunction::QuadOut),
            ("CubicOut", EasingFunction::CubicOut),
            ("SineOut", EasingFunction::SineOut),
            ("ExpoOut", EasingFunction::ExpoOut),
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

        let mut fade_tween = Tween::new(0.0f32, 1.0f32, 1.5).with_easing(EasingFunction::CubicOut);
        fade_tween.play();

        let mut color_tween =
            Tween::new(Color32::RED, Color32::BLUE, 3.0).with_easing(EasingFunction::SineInOut);
        color_tween.play();

        Self {
            bounce_tween,
            fade_tween,
            color_tween,
            spring: Spring::with_params(0.5, SpringParams::smooth()),
            spring_target: 0.5,
            spring_params_type: SpringParamsType::Smooth,
            easing_tweens,
            show_easing_comparison: false,
        }
    }
}

impl AnimationTab {
    pub fn show(&mut self, ctx: &Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let dt = ui.input(|i| i.stable_dt);

            ui.heading("🎬 Animation Showcase");
            ui.add_space(10.0);

            // Bounce demo
            ui.group(|ui| {
                ui.heading("Tween Animation (ElasticOut)");
                ui.label("Smooth interpolation with easing functions");

                self.bounce_tween.update(dt);
                let y = self.bounce_tween.value();

                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(500.0, 250.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, Color32::from_gray(40));

                // Ball
                let ball_pos = Pos2::new(rect.left() + 100.0, rect.top() + y + 25.0);
                ui.painter().circle_filled(ball_pos, 20.0, Color32::GREEN);

                // Target line
                ui.painter().line_segment(
                    [
                        Pos2::new(rect.left(), rect.top() + 225.0),
                        Pos2::new(rect.right(), rect.top() + 225.0),
                    ],
                    egui::Stroke::new(2.0, Color32::DARK_GRAY),
                );

                ui.label(format!(
                    "Progress: {:.1}%",
                    self.bounce_tween.progress() * 100.0
                ));

                if ui.button("🔄 Restart").clicked() {
                    self.bounce_tween.restart();
                }
            });

            ui.add_space(20.0);

            // Spring demo
            ui.group(|ui| {
                ui.heading("Spring Physics");
                ui.label("Physics-based smooth motion with damping");

                ui.horizontal(|ui| {
                    ui.label("Spring type:");
                    if ui
                        .selectable_value(
                            &mut self.spring_params_type,
                            SpringParamsType::Smooth,
                            "Smooth",
                        )
                        .clicked()
                    {
                        self.spring.set_params(SpringParams::smooth());
                    }
                    if ui
                        .selectable_value(
                            &mut self.spring_params_type,
                            SpringParamsType::Bouncy,
                            "Bouncy",
                        )
                        .clicked()
                    {
                        self.spring.set_params(SpringParams::bouncy());
                    }
                    if ui
                        .selectable_value(
                            &mut self.spring_params_type,
                            SpringParamsType::Sluggish,
                            "Sluggish",
                        )
                        .clicked()
                    {
                        self.spring.set_params(SpringParams::sluggish());
                    }
                });

                ui.add(egui::Slider::new(&mut self.spring_target, 0.0..=1.0).text("Target"));
                self.spring.set_target(self.spring_target);

                self.spring.update(dt);
                let spring_value = self.spring.position();

                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(500.0, 100.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, Color32::from_gray(40));

                // Spring position
                let circle_x = rect.left() + spring_value * rect.width();
                ui.painter().circle_filled(
                    Pos2::new(circle_x, rect.center().y),
                    20.0,
                    Color32::YELLOW,
                );

                // Target
                let target_x = rect.left() + self.spring_target * rect.width();
                ui.painter().circle_stroke(
                    Pos2::new(target_x, rect.center().y),
                    20.0,
                    egui::Stroke::new(2.0, Color32::WHITE),
                );

                ui.label(format!("Position: {:.3}", spring_value));
                ui.label(format!("Velocity: {:.3}", self.spring.velocity()));
                ui.label(format!("Settled: {}", self.spring.is_settled(0.01)));
            });

            ui.add_space(20.0);

            // Color tween
            ui.group(|ui| {
                ui.heading("Color Animation (SineInOut)");

                self.color_tween.update(dt);
                let color = self.color_tween.value();

                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(500.0, 80.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 4.0, color);

                ui.label(format!(
                    "RGB: ({}, {}, {})",
                    color.r(),
                    color.g(),
                    color.b()
                ));

                if ui.button("🔄 Restart").clicked() {
                    self.color_tween.restart();
                }
            });

            ui.add_space(20.0);

            // Easing comparison
            ui.checkbox(
                &mut self.show_easing_comparison,
                "Show Easing Function Comparison",
            );

            if self.show_easing_comparison {
                ui.group(|ui| {
                    ui.heading("Easing Functions Comparison");
                    ui.label("7 common easing functions animating 0 → 1");

                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(500.0, 300.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 4.0, Color32::from_gray(40));

                    // Grid
                    for i in 0..=4 {
                        let y = rect.top() + (i as f32 / 4.0) * rect.height();
                        ui.painter().line_segment(
                            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                            egui::Stroke::new(1.0, Color32::from_gray(60)),
                        );
                    }

                    let colors = [
                        Color32::RED,
                        Color32::GREEN,
                        Color32::BLUE,
                        Color32::YELLOW,
                        Color32::from_rgb(255, 128, 0),
                        Color32::from_rgb(255, 0, 255),
                        Color32::WHITE,
                    ];

                    // Draw curves
                    for (i, (name, tween)) in self.easing_tweens.iter_mut().enumerate() {
                        tween.update(dt);
                        let value = tween.value();
                        let t = tween.progress();

                        let x = rect.left() + t * rect.width();
                        let y = rect.bottom() - value * rect.height();

                        let color = colors[i % colors.len()];
                        ui.painter().circle_filled(Pos2::new(x, y), 5.0, color);

                        // Label
                        ui.painter().text(
                            Pos2::new(rect.right() + 10.0, rect.top() + (i as f32 * 20.0)),
                            egui::Align2::LEFT_TOP,
                            name,
                            egui::FontId::proportional(12.0),
                            color,
                        );
                    }

                    if ui.button("🔄 Restart All").clicked() {
                        for (_name, tween) in &mut self.easing_tweens {
                            tween.restart();
                        }
                    }
                });
            }

            ui.add_space(20.0);

            // Code example
            ui.collapsing("📝 Code Example", |ui| {
                ui.label("Tween Usage:");
                ui.code(
                    r#"let mut tween = Tween::new(0.0f32, 100.0f32, 2.0)
    .with_easing(EasingFunction::ElasticOut);

tween.play();

// In update loop:
tween.update(dt);
let current_value = tween.value();"#,
                );

                ui.add_space(5.0);
                ui.label("Spring Usage:");
                ui.code(
                    r#"let mut spring = Spring::new(0.0)
    .with_params(SpringParams::bouncy());

spring.set_target(100.0);

// In update loop:
spring.update(dt);
let position = spring.position();"#,
                );
            });
        });
    }
}
