//! Animation panel showing tween, spring, and easing demos

use astract::animation::{EasingFunction, Spring, SpringParams, Tween};
use egui::{Color32, Pos2, Vec2};

pub struct AnimationPanel {
    // Tween demos
    bounce_tween: Tween<f32>,
    color_tween: Tween<Color32>,

    // Spring demo
    spring: Spring,
    mouse_target: Pos2,

    // Easing comparison
    easing_tweens: Vec<(String, Tween<f32>)>,
    show_easing: bool,

    // Time tracking
    time: f32,
}

impl Default for AnimationPanel {
    fn default() -> Self {
        // Create easing comparison tweens
        let easings = vec![
            ("Linear", EasingFunction::Linear),
            ("QuadIn", EasingFunction::QuadIn),
            ("QuadOut", EasingFunction::QuadOut),
            ("CubicIn", EasingFunction::CubicIn),
            ("CubicOut", EasingFunction::CubicOut),
            ("SineIn", EasingFunction::SineIn),
            ("SineOut", EasingFunction::SineOut),
            ("ExpoIn", EasingFunction::ExpoIn),
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

        let mut color_tween =
            Tween::new(Color32::RED, Color32::BLUE, 3.0).with_easing(EasingFunction::SineInOut);
        color_tween.play();

        Self {
            bounce_tween,
            color_tween,
            spring: Spring::with_params(0.0, SpringParams::bouncy()),
            mouse_target: Pos2::ZERO,
            easing_tweens,
            show_easing: false,
            time: 0.0,
        }
    }
}

impl AnimationPanel {
    pub fn show(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.stable_dt);
        self.time += dt;

        egui::Window::new("🎬 Animation")
            .default_size([600.0, 800.0])
            .show(ctx, |ui| {
                ui.heading("Animation Demos");

                ui.separator();

                // Tween demo: Bouncing ball
                ui.label("Tween Demo (ElasticOut):");

                self.bounce_tween.update(dt);
                let y = self.bounce_tween.value();

                let (rect, _response) =
                    ui.allocate_exact_size(Vec2::new(400.0, 250.0), egui::Sense::hover());

                // Draw bouncing ball
                let ball_pos = Pos2::new(rect.left() + 100.0, rect.top() + y + 25.0);
                ui.painter().rect_filled(rect, 0.0, Color32::from_gray(40));
                ui.painter().circle_filled(ball_pos, 20.0, Color32::GREEN);

                // Draw target line
                ui.painter().line_segment(
                    [
                        Pos2::new(rect.left(), rect.top() + 225.0),
                        Pos2::new(rect.right(), rect.top() + 225.0),
                    ],
                    egui::Stroke::new(1.0, Color32::DARK_GRAY),
                );

                // Restart button
                if ui.button("🔄 Restart Bounce").clicked() {
                    self.bounce_tween.restart();
                }

                ui.add_space(10.0);
                ui.separator();

                // Color tween demo
                ui.label("Color Tween (SineInOut):");

                self.color_tween.update(dt);
                let color = self.color_tween.value();

                let (rect, _response) =
                    ui.allocate_exact_size(Vec2::new(400.0, 80.0), egui::Sense::hover());

                ui.painter().rect_filled(rect, 4.0, color);

                if ui.button("🔄 Restart Color").clicked() {
                    self.color_tween.restart();
                }

                ui.add_space(10.0);
                ui.separator();

                // Spring demo
                ui.label("Spring Physics (Bouncy):");
                ui.label("Move your mouse in the area below:");

                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(400.0, 300.0), egui::Sense::hover());

                if let Some(hover_pos) = response.hover_pos() {
                    self.mouse_target = hover_pos;

                    // Convert to normalized 0-1 range relative to rect
                    let normalized_x = ((hover_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                    self.spring.set_target(normalized_x);
                }

                self.spring.update(dt);
                let spring_value = self.spring.position();

                ui.painter().rect_filled(rect, 0.0, Color32::from_gray(40));

                // Draw spring position
                let circle_x = rect.left() + spring_value * rect.width();
                let circle_pos = Pos2::new(circle_x, rect.center().y);
                ui.painter()
                    .circle_filled(circle_pos, 25.0, Color32::YELLOW);

                // Draw target
                let target_x = rect.left()
                    + ((self.mouse_target.x - rect.left()) / rect.width()).clamp(0.0, 1.0)
                        * rect.width();
                ui.painter().circle_stroke(
                    Pos2::new(target_x, rect.center().y),
                    25.0,
                    egui::Stroke::new(2.0, Color32::WHITE),
                );

                ui.add_space(10.0);
                ui.separator();

                // Easing comparison
                ui.checkbox(&mut self.show_easing, "Show Easing Comparison");

                if self.show_easing {
                    ui.label("Easing Functions (0 → 1 over 2 seconds):");

                    let (rect, _response) =
                        ui.allocate_exact_size(Vec2::new(400.0, 400.0), egui::Sense::hover());

                    ui.painter().rect_filled(rect, 0.0, Color32::from_gray(40));

                    // Draw grid
                    for i in 0..=4 {
                        let y = rect.top() + (i as f32 / 4.0) * rect.height();
                        ui.painter().line_segment(
                            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                            egui::Stroke::new(1.0, Color32::from_gray(60)),
                        );
                    }

                    for i in 0..=4 {
                        let x = rect.left() + (i as f32 / 4.0) * rect.width();
                        ui.painter().line_segment(
                            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                            egui::Stroke::new(1.0, Color32::from_gray(60)),
                        );
                    }

                    // Colors for each easing function
                    let colors = [
                        Color32::RED,
                        Color32::GREEN,
                        Color32::BLUE,
                        Color32::YELLOW,
                        Color32::LIGHT_BLUE,
                        Color32::LIGHT_GREEN,
                        Color32::from_rgb(255, 128, 0),
                        Color32::from_rgb(255, 0, 255),
                        Color32::from_rgb(0, 255, 255),
                        Color32::from_rgb(128, 128, 255),
                        Color32::WHITE,
                    ];

                    // Update and draw each easing function
                    for (i, (name, tween)) in self.easing_tweens.iter_mut().enumerate() {
                        tween.update(dt);
                        let value = tween.value();

                        // Position on curve (time on X, value on Y)
                        let t = tween.progress();
                        let x = rect.left() + t * rect.width();
                        let y = rect.bottom() - value * rect.height();

                        let color = colors[i % colors.len()];
                        ui.painter().circle_filled(Pos2::new(x, y), 4.0, color);

                        // Draw label at the end
                        if t > 0.9 {
                            ui.painter().text(
                                Pos2::new(rect.right() + 10.0, rect.top() + (i as f32 * 15.0)),
                                egui::Align2::LEFT_TOP,
                                name,
                                egui::FontId::proportional(10.0),
                                color,
                            );
                        }
                    }

                    if ui.button("🔄 Restart All Easings").clicked() {
                        for (_name, tween) in &mut self.easing_tweens {
                            tween.restart();
                        }
                    }
                }
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_panel_creation() {
        let panel = AnimationPanel::default();
        assert_eq!(panel.easing_tweens.len(), 11);
        assert!(!panel.show_easing);
    }

    #[test]
    fn test_bounce_tween_running() {
        let panel = AnimationPanel::default();
        // Should be running after default initialization
        assert!(panel.bounce_tween.is_running());
    }

    #[test]
    fn test_color_tween_running() {
        let panel = AnimationPanel::default();
        assert!(panel.color_tween.is_running());
    }

    #[test]
    fn test_spring_initial_position() {
        let panel = AnimationPanel::default();
        assert_eq!(panel.spring.position(), 0.0);
    }
}
