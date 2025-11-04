use egui::{Color32, Sense, StrokeKind, Ui, Vec2};

/// A professional color picker widget with HSV wheel, RGB sliders, hex input, and presets
///
/// Supports:
/// - HSV color wheel with brightness slider
/// - RGB component sliders
/// - Hex color input (#RRGGBB format)
/// - Alpha channel control
/// - Color presets (common game engine colors)
/// - Live preview with old/new comparison
///
/// # Example
/// ```rust,no_run
/// use astract::advanced::ColorPicker;
/// use egui::Color32;
///
/// let mut picker = ColorPicker::new()
///     .with_color(Color32::from_rgb(100, 150, 200))
///     .show_alpha(true)
///     .show_presets(true);
///
/// // Show returns true if color changed
/// // if picker.show(ui) {
/// //     println!("New color: {:?}", picker.color());
/// // }
/// ```
pub struct ColorPicker {
    color: Color32,
    original_color: Color32,
    show_alpha: bool,
    show_presets: bool,
    show_hex_input: bool,
    hsv: (f32, f32, f32), // Hue [0-360], Saturation [0-1], Value [0-1]
    width: f32,
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPicker {
    /// Create a new color picker with white as the default color
    pub fn new() -> Self {
        Self {
            color: Color32::WHITE,
            original_color: Color32::WHITE,
            show_alpha: true,
            show_presets: true,
            show_hex_input: true,
            hsv: (0.0, 0.0, 1.0),
            width: 280.0,
        }
    }

    /// Set the current color (builder pattern)
    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = color;
        self.original_color = color;
        self.hsv = Self::rgb_to_hsv(color);
        self
    }

    /// Set the picker width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Show/hide alpha channel control
    pub fn show_alpha(mut self, show: bool) -> Self {
        self.show_alpha = show;
        self
    }

    /// Show/hide color presets
    pub fn show_presets(mut self, show: bool) -> Self {
        self.show_presets = show;
        self
    }

    /// Show/hide hex input field
    pub fn show_hex_input(mut self, show: bool) -> Self {
        self.show_hex_input = show;
        self
    }

    /// Get the current color (getter)
    pub fn color(&self) -> Color32 {
        self.color
    }

    /// Get alpha value (0-255)
    pub fn alpha(&self) -> u8 {
        self.color.a()
    }

    /// Set color from RGB components
    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.color = Color32::from_rgb(r, g, b);
        self.hsv = Self::rgb_to_hsv(self.color);
    }

    /// Set color from hex string (#RRGGBB or RRGGBB)
    pub fn set_hex(&mut self, hex: &str) -> Result<(), String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err("Hex must be 6 characters (RRGGBB)".to_string());
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;

        self.set_rgb(r, g, b);
        Ok(())
    }

    /// Get hex string representation (#RRGGBB)
    pub fn hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            self.color.r(),
            self.color.g(),
            self.color.b()
        )
    }

    /// Convert RGB to HSV (Hue, Saturation, Value)
    fn rgb_to_hsv(color: Color32) -> (f32, f32, f32) {
        let r = color.r() as f32 / 255.0;
        let g = color.g() as f32 / 255.0;
        let b = color.b() as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        // Value
        let v = max;

        // Saturation
        let s = if max > 0.0 { delta / max } else { 0.0 };

        // Hue
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };

        (h, s, v)
    }

    /// Convert HSV to RGB
    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color32 {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r_prime, g_prime, b_prime) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r_prime + m) * 255.0) as u8;
        let g = ((g_prime + m) * 255.0) as u8;
        let b = ((b_prime + m) * 255.0) as u8;

        Color32::from_rgb(r, g, b)
    }

    /// Show the color picker UI
    /// Returns true if the color changed
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.vertical(|ui| {
            ui.set_width(self.width);

            // Preview (original vs current)
            ui.horizontal(|ui| {
                ui.label("Preview:");

                // Original color
                let preview_size = Vec2::new(self.width / 2.0 - 10.0, 40.0);
                let (rect, _) = ui.allocate_exact_size(preview_size, Sense::hover());
                ui.painter().rect_filled(rect, 2.0, self.original_color);
                ui.painter()
                    .rect_stroke(rect, 2.0, (1.0, Color32::GRAY), StrokeKind::Middle);

                // Current color
                let (rect, _) = ui.allocate_exact_size(preview_size, Sense::hover());
                ui.painter().rect_filled(rect, 2.0, self.color);
                ui.painter()
                    .rect_stroke(rect, 2.0, (1.0, Color32::GRAY), StrokeKind::Middle);
            });

            ui.add_space(8.0);

            // HSV Color Wheel (simplified to saturation-value square for Day 7)
            ui.label("Hue:");
            let hue_response = ui.add(egui::Slider::new(&mut self.hsv.0, 0.0..=360.0).text("°"));
            if hue_response.changed() {
                self.color = Self::hsv_to_rgb(self.hsv.0, self.hsv.1, self.hsv.2);
                changed = true;
            }

            ui.label("Saturation:");
            let sat_response = ui.add(egui::Slider::new(&mut self.hsv.1, 0.0..=1.0));
            if sat_response.changed() {
                self.color = Self::hsv_to_rgb(self.hsv.0, self.hsv.1, self.hsv.2);
                changed = true;
            }

            ui.label("Value (Brightness):");
            let val_response = ui.add(egui::Slider::new(&mut self.hsv.2, 0.0..=1.0));
            if val_response.changed() {
                self.color = Self::hsv_to_rgb(self.hsv.0, self.hsv.1, self.hsv.2);
                changed = true;
            }

            ui.separator();

            // RGB Sliders
            let mut r = self.color.r();
            let mut g = self.color.g();
            let mut b = self.color.b();

            ui.label("Red:");
            let r_response = ui.add(egui::Slider::new(&mut r, 0..=255));

            ui.label("Green:");
            let g_response = ui.add(egui::Slider::new(&mut g, 0..=255));

            ui.label("Blue:");
            let b_response = ui.add(egui::Slider::new(&mut b, 0..=255));

            if r_response.changed() || g_response.changed() || b_response.changed() {
                self.set_rgb(r, g, b);
                changed = true;
            }

            // Alpha slider
            if self.show_alpha {
                ui.separator();
                let mut a = self.color.a();
                ui.label("Alpha:");
                let a_response = ui.add(egui::Slider::new(&mut a, 0..=255));
                if a_response.changed() {
                    self.color = Color32::from_rgba_premultiplied(r, g, b, a);
                    changed = true;
                }
            }

            // Hex input
            if self.show_hex_input {
                ui.separator();
                ui.label("Hex:");
                let mut hex_string = self.hex();
                let hex_response = ui.text_edit_singleline(&mut hex_string);
                if hex_response.lost_focus() {
                    if let Ok(()) = self.set_hex(&hex_string) {
                        changed = true;
                    }
                }
            }

            // Color presets
            if self.show_presets {
                ui.separator();
                ui.label("Presets:");
                ui.horizontal_wrapped(|ui| {
                    let presets = [
                        ("White", Color32::WHITE),
                        ("Black", Color32::BLACK),
                        ("Red", Color32::from_rgb(220, 50, 50)),
                        ("Green", Color32::from_rgb(50, 220, 50)),
                        ("Blue", Color32::from_rgb(50, 50, 220)),
                        ("Yellow", Color32::from_rgb(220, 220, 50)),
                        ("Cyan", Color32::from_rgb(50, 220, 220)),
                        ("Magenta", Color32::from_rgb(220, 50, 220)),
                        ("Orange", Color32::from_rgb(255, 165, 0)),
                        ("Purple", Color32::from_rgb(128, 0, 128)),
                        ("Gray", Color32::GRAY),
                        ("Dark Gray", Color32::DARK_GRAY),
                    ];

                    for (_name, preset_color) in &presets {
                        let button_size = Vec2::splat(24.0);
                        let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());

                        ui.painter().rect_filled(rect, 2.0, *preset_color);
                        ui.painter().rect_stroke(
                            rect,
                            2.0,
                            (1.0, Color32::GRAY),
                            StrokeKind::Middle,
                        );

                        if response.clicked() {
                            self.color = *preset_color;
                            self.hsv = Self::rgb_to_hsv(*preset_color);
                            changed = true;
                        }
                    }
                });
            }

            // Reset button
            ui.separator();
            if ui.button("Reset to Original").clicked() {
                self.color = self.original_color;
                self.hsv = Self::rgb_to_hsv(self.original_color);
                changed = true;
            }
        });

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_picker_default() {
        let picker = ColorPicker::new();
        assert_eq!(picker.color(), Color32::WHITE);
        assert_eq!(picker.alpha(), 255);
        assert_eq!(picker.hex(), "#FFFFFF");
    }

    #[test]
    fn test_color_picker_set_color() {
        let picker = ColorPicker::new().with_color(Color32::RED);
        assert_eq!(picker.color(), Color32::RED);
    }

    #[test]
    fn test_rgb_to_hsv_red() {
        let (h, s, v) = ColorPicker::rgb_to_hsv(Color32::RED);
        assert!((h - 0.0).abs() < 0.1);
        assert!((s - 1.0).abs() < 0.1);
        assert!((v - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_rgb_to_hsv_green() {
        let (h, s, v) = ColorPicker::rgb_to_hsv(Color32::GREEN);
        assert!((h - 120.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.1);
        assert!((v - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_rgb_to_hsv_blue() {
        let (h, s, v) = ColorPicker::rgb_to_hsv(Color32::BLUE);
        assert!((h - 240.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.1);
        assert!((v - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_hsv_to_rgb_red() {
        let color = ColorPicker::hsv_to_rgb(0.0, 1.0, 1.0);
        assert_eq!(color.r(), 255);
        assert_eq!(color.g(), 0);
        assert_eq!(color.b(), 0);
    }

    #[test]
    fn test_hsv_to_rgb_green() {
        let color = ColorPicker::hsv_to_rgb(120.0, 1.0, 1.0);
        assert_eq!(color.r(), 0);
        assert_eq!(color.g(), 255);
        assert_eq!(color.b(), 0);
    }

    #[test]
    fn test_hsv_to_rgb_blue() {
        let color = ColorPicker::hsv_to_rgb(240.0, 1.0, 1.0);
        assert_eq!(color.r(), 0);
        assert_eq!(color.g(), 0);
        assert_eq!(color.b(), 255);
    }

    #[test]
    fn test_hex_conversion() {
        let mut picker = ColorPicker::new();
        assert_eq!(picker.hex(), "#FFFFFF");

        picker.set_rgb(255, 0, 0);
        assert_eq!(picker.hex(), "#FF0000");

        picker.set_rgb(0, 255, 0);
        assert_eq!(picker.hex(), "#00FF00");

        picker.set_rgb(0, 0, 255);
        assert_eq!(picker.hex(), "#0000FF");
    }

    #[test]
    fn test_hex_parsing() {
        let mut picker = ColorPicker::new();

        assert!(picker.set_hex("#FF0000").is_ok());
        assert_eq!(picker.color().r(), 255);
        assert_eq!(picker.color().g(), 0);
        assert_eq!(picker.color().b(), 0);

        assert!(picker.set_hex("00FF00").is_ok()); // Without #
        assert_eq!(picker.color().r(), 0);
        assert_eq!(picker.color().g(), 255);
        assert_eq!(picker.color().b(), 0);
    }

    #[test]
    fn test_hex_parsing_errors() {
        let mut picker = ColorPicker::new();

        assert!(picker.set_hex("#FFF").is_err()); // Too short
        assert!(picker.set_hex("#FFFFFFF").is_err()); // Too long
        assert!(picker.set_hex("#GGGGGG").is_err()); // Invalid hex
    }
}
