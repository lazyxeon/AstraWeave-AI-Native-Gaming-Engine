use egui::{Color32, Pos2, Rect, Sense, Ui, Vec2};

/// A range slider widget for selecting minimum and maximum values
///
/// Supports:
/// - Dual handles (min and max)
/// - Step increments
/// - Value clamping
/// - Visual feedback
/// - Custom ranges
///
/// # Example
/// ```rust,no_run
/// use astract::advanced::RangeSlider;
///
/// let mut slider = RangeSlider::new(0.0, 100.0)
///     .with_min(20.0)
///     .with_max(80.0)
///     .step(5.0);
///
/// // Show returns true if values changed
/// // if slider.show(ui) {
/// //     println!("Range: {} - {}", slider.min_value(), slider.max_value());
/// // }
/// ```
pub struct RangeSlider {
    range_min: f64,
    range_max: f64,
    min_value: f64,
    max_value: f64,
    step: Option<f64>,
    width: f32,
    height: f32,
    show_values: bool,
    prefix: String,
    suffix: String,
}

impl RangeSlider {
    /// Create a new range slider with the specified range
    pub fn new(range_min: f64, range_max: f64) -> Self {
        assert!(
            range_min < range_max,
            "range_min must be less than range_max"
        );

        Self {
            range_min,
            range_max,
            min_value: range_min,
            max_value: range_max,
            step: None,
            width: 200.0,
            height: 30.0,
            show_values: true,
            prefix: String::new(),
            suffix: String::new(),
        }
    }

    /// Set the current minimum value (builder pattern)
    pub fn with_min(mut self, value: f64) -> Self {
        self.min_value = value.clamp(self.range_min, self.max_value);
        self
    }

    /// Set the current maximum value (builder pattern)
    pub fn with_max(mut self, value: f64) -> Self {
        self.max_value = value.clamp(self.min_value, self.range_max);
        self
    }

    /// Set the step increment
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Set the slider width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set the slider height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Show/hide value labels
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Set value prefix (e.g., "$", "€")
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Set value suffix (e.g., "ms", "kg", "%")
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Get the current minimum value (getter)
    pub fn min_value(&self) -> f64 {
        self.min_value
    }

    /// Get the current maximum value
    pub fn max_value(&self) -> f64 {
        self.max_value
    }

    /// Get the selected range size
    pub fn range_size(&self) -> f64 {
        self.max_value - self.min_value
    }

    /// Apply step rounding to a value
    fn apply_step(&self, value: f64) -> f64 {
        if let Some(step) = self.step {
            (value / step).round() * step
        } else {
            value
        }
    }

    /// Convert screen position to value
    fn pos_to_value(&self, pos: f32, rect: Rect) -> f64 {
        let t = ((pos - rect.min.x) / rect.width()).clamp(0.0, 1.0);
        let value = self.range_min + (self.range_max - self.range_min) * t as f64;
        self.apply_step(value)
    }

    /// Convert value to screen position
    fn value_to_pos(&self, value: f64, rect: Rect) -> f32 {
        let t = ((value - self.range_min) / (self.range_max - self.range_min)) as f32;
        rect.min.x + t * rect.width()
    }

    /// Format a value for display
    pub fn format_value(&self, value: f64) -> String {
        if let Some(step) = self.step {
            if step.fract() == 0.0 {
                format!("{}{:.0}{}", self.prefix, value, self.suffix)
            } else {
                format!("{}{:.2}{}", self.prefix, value, self.suffix)
            }
        } else {
            format!("{}{:.2}{}", self.prefix, value, self.suffix)
        }
    }

    /// Show the range slider UI
    /// Returns true if values changed
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.vertical(|ui| {
            // Value labels (if enabled)
            if self.show_values {
                ui.horizontal(|ui| {
                    ui.label(format!("Min: {}", self.format_value(self.min_value)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("Max: {}", self.format_value(self.max_value)));
                    });
                });
            }

            // Allocate space for the slider
            let desired_size = Vec2::new(self.width, self.height);
            let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

            // Track area (the bar)
            let track_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, rect.center().y - 3.0),
                Vec2::new(rect.width(), 6.0),
            );

            // Draw track background
            ui.painter()
                .rect_filled(track_rect, 3.0, Color32::DARK_GRAY);

            // Draw selected range
            let min_x = self.value_to_pos(self.min_value, rect);
            let max_x = self.value_to_pos(self.max_value, rect);
            let selected_rect = Rect::from_min_max(
                Pos2::new(min_x, track_rect.min.y),
                Pos2::new(max_x, track_rect.max.y),
            );
            ui.painter()
                .rect_filled(selected_rect, 3.0, Color32::from_rgb(100, 180, 255));

            // Draw min handle
            let min_handle_center = Pos2::new(min_x, rect.center().y);

            let min_handle_color = if response.hovered() {
                Color32::from_rgb(120, 200, 255)
            } else {
                Color32::from_rgb(100, 180, 255)
            };

            ui.painter()
                .circle_filled(min_handle_center, 8.0, min_handle_color);
            ui.painter()
                .circle_stroke(min_handle_center, 8.0, (2.0, Color32::WHITE));

            // Draw max handle
            let max_handle_center = Pos2::new(max_x, rect.center().y);

            ui.painter()
                .circle_filled(max_handle_center, 8.0, min_handle_color);
            ui.painter()
                .circle_stroke(max_handle_center, 8.0, (2.0, Color32::WHITE));

            // Handle interaction
            if response.dragged() || response.clicked() {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let new_value = self.pos_to_value(pointer_pos.x, rect);

                    // Determine which handle to move (closest one)
                    let dist_to_min = (pointer_pos.x - min_x).abs();
                    let dist_to_max = (pointer_pos.x - max_x).abs();

                    if dist_to_min < dist_to_max {
                        // Move min handle
                        let old_min = self.min_value;
                        self.min_value = new_value.clamp(self.range_min, self.max_value);
                        if (self.min_value - old_min).abs() > 0.001 {
                            changed = true;
                        }
                    } else {
                        // Move max handle
                        let old_max = self.max_value;
                        self.max_value = new_value.clamp(self.min_value, self.range_max);
                        if (self.max_value - old_max).abs() > 0.001 {
                            changed = true;
                        }
                    }
                }
            }

            // Range size label
            if self.show_values {
                ui.label(format!("Range: {}", self.format_value(self.range_size())));
            }
        });

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_slider_creation() {
        let slider = RangeSlider::new(0.0, 100.0);
        assert_eq!(slider.min_value(), 0.0);
        assert_eq!(slider.max_value(), 100.0);
        assert_eq!(slider.range_size(), 100.0);
    }

    #[test]
    fn test_set_values() {
        let slider = RangeSlider::new(0.0, 100.0).with_min(20.0).with_max(80.0);

        assert_eq!(slider.min_value(), 20.0);
        assert_eq!(slider.max_value(), 80.0);
        assert_eq!(slider.range_size(), 60.0);
    }

    #[test]
    fn test_clamping() {
        let slider = RangeSlider::new(0.0, 100.0)
            .with_min(-10.0) // Should clamp to 0.0
            .with_max(150.0); // Should clamp to 100.0

        assert_eq!(slider.min_value(), 0.0);
        assert_eq!(slider.max_value(), 100.0);
    }

    #[test]
    fn test_min_max_constraint() {
        let slider = RangeSlider::new(0.0, 100.0).with_max(30.0).with_min(50.0); // Should clamp to max_value

        assert_eq!(slider.min_value(), 30.0);
        assert_eq!(slider.max_value(), 30.0);
    }

    #[test]
    fn test_step_rounding() {
        let slider = RangeSlider::new(0.0, 100.0).step(5.0);

        assert_eq!(slider.apply_step(12.3), 10.0);
        assert_eq!(slider.apply_step(17.6), 20.0);
        assert_eq!(slider.apply_step(25.0), 25.0);
    }

    #[test]
    fn test_format_value_integer_step() {
        let slider = RangeSlider::new(0.0, 100.0)
            .step(1.0)
            .prefix("$")
            .suffix(" USD");

        assert_eq!(slider.format_value(42.0), "$42 USD");
    }

    #[test]
    fn test_format_value_decimal_step() {
        let slider = RangeSlider::new(0.0, 1.0).step(0.1).suffix("%");

        assert_eq!(slider.format_value(0.75), "0.75%");
    }

    #[test]
    fn test_range_size() {
        let slider = RangeSlider::new(0.0, 100.0).with_min(25.0).with_max(75.0);

        assert_eq!(slider.range_size(), 50.0);
    }

    #[test]
    #[should_panic(expected = "range_min must be less than range_max")]
    fn test_invalid_range() {
        RangeSlider::new(100.0, 0.0);
    }
}
