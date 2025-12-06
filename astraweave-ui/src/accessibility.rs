//! Accessibility features for UI
//!
//! Phase 8.1 Week 5: Accessibility System
//! - Colorblind modes (Deuteranopia, Protanopia, Tritanopia)
//! - High-contrast mode for low-vision users
//! - UI scaling (80-150%)

use serde::{Deserialize, Serialize};

/// Color as RGB tuple (0.0-1.0 range)
pub type Color = (f32, f32, f32);

/// Colorblind mode for health bar and status colors
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorblindMode {
    /// Normal color vision
    None,
    /// Red-green colorblindness (most common, ~6% of males)
    /// Green → Blue
    Deuteranopia,
    /// Red-green colorblindness
    /// Green → Cyan
    Protanopia,
    /// Blue-yellow colorblindness (rare)
    /// Blue → Magenta
    Tritanopia,
    /// High contrast mode for low-vision users
    /// All colors become black/white with high saturation accents
    HighContrast,
}

impl Default for ColorblindMode {
    fn default() -> Self {
        ColorblindMode::None
    }
}

impl ColorblindMode {
    /// Get all available modes for UI dropdown
    pub fn all() -> &'static [ColorblindMode] {
        &[
            ColorblindMode::None,
            ColorblindMode::Deuteranopia,
            ColorblindMode::Protanopia,
            ColorblindMode::Tritanopia,
            ColorblindMode::HighContrast,
        ]
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            ColorblindMode::None => "None",
            ColorblindMode::Deuteranopia => "Deuteranopia (Red-Green)",
            ColorblindMode::Protanopia => "Protanopia (Red-Green)",
            ColorblindMode::Tritanopia => "Tritanopia (Blue-Yellow)",
            ColorblindMode::HighContrast => "High Contrast",
        }
    }
}

/// Accessibility settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// Colorblind mode
    pub colorblind_mode: ColorblindMode,
    /// UI scale factor (0.8 to 1.5)
    pub ui_scale: f32,
    /// Reduce motion (disable animations)
    pub reduce_motion: bool,
    /// Large text (forces minimum font size)
    pub large_text: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            colorblind_mode: ColorblindMode::None,
            ui_scale: 1.0,
            reduce_motion: false,
            large_text: false,
        }
    }
}

impl AccessibilitySettings {
    /// Clamp UI scale to valid range
    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale = scale.clamp(0.8, 1.5);
    }

    /// Get effective font size multiplier
    pub fn font_scale(&self) -> f32 {
        let base = self.ui_scale;
        if self.large_text {
            base * 1.25
        } else {
            base
        }
    }
}

// ============================================================================
// Color Transformation
// ============================================================================

/// Standard colors for health bar
pub mod colors {
    use super::Color;

    /// Full health - green
    pub const HEALTH_FULL: Color = (0.2, 0.8, 0.2);
    /// Medium health - yellow
    pub const HEALTH_MEDIUM: Color = (0.9, 0.8, 0.1);
    /// Low health - red
    pub const HEALTH_LOW: Color = (0.9, 0.2, 0.2);
    /// Critical health - dark red
    pub const HEALTH_CRITICAL: Color = (0.6, 0.1, 0.1);

    /// Stamina bar - blue
    pub const STAMINA: Color = (0.2, 0.5, 0.9);
    /// Mana bar - purple
    pub const MANA: Color = (0.6, 0.2, 0.8);

    /// Positive effect - green
    pub const POSITIVE: Color = (0.2, 0.9, 0.3);
    /// Negative effect - red
    pub const NEGATIVE: Color = (0.9, 0.2, 0.2);
    /// Neutral - white
    pub const NEUTRAL: Color = (0.9, 0.9, 0.9);
}

/// Transform color based on colorblind mode
pub fn transform_color(color: Color, mode: ColorblindMode) -> Color {
    match mode {
        ColorblindMode::None => color,
        ColorblindMode::Deuteranopia => deuteranopia_transform(color),
        ColorblindMode::Protanopia => protanopia_transform(color),
        ColorblindMode::Tritanopia => tritanopia_transform(color),
        ColorblindMode::HighContrast => high_contrast_transform(color),
    }
}

/// Deuteranopia: Replace green with blue
fn deuteranopia_transform(color: Color) -> Color {
    let (r, g, b) = color;
    // Shift green towards blue channel
    let new_g = g * 0.3;
    let new_b = b + g * 0.7;
    (r, new_g, new_b.min(1.0))
}

/// Protanopia: Replace green with cyan
fn protanopia_transform(color: Color) -> Color {
    let (r, g, b) = color;
    // Shift green and reduce red, add to blue
    let new_r = r * 0.5;
    let new_g = g * 0.8;
    let new_b = b + g * 0.4 + r * 0.2;
    (new_r, new_g, new_b.min(1.0))
}

/// Tritanopia: Replace blue with magenta
fn tritanopia_transform(color: Color) -> Color {
    let (r, g, b) = color;
    // Shift blue towards magenta (red+blue, less green)
    let new_r = r + b * 0.5;
    let new_g = g * 0.5;
    let new_b = b * 0.8;
    (new_r.min(1.0), new_g, new_b)
}

/// High contrast: Boost saturation, use pure colors
fn high_contrast_transform(color: Color) -> Color {
    let (r, g, b) = color;

    // Calculate luminance
    let lum = 0.299 * r + 0.587 * g + 0.114 * b;

    // Find dominant channel
    let max = r.max(g).max(b);

    if max < 0.3 {
        // Dark colors become black
        (0.0, 0.0, 0.0)
    } else if lum > 0.7 {
        // Light colors become white
        (1.0, 1.0, 1.0)
    } else {
        // Boost saturation for mid-tones
        let boost = 1.5;
        (
            ((r - lum) * boost + lum).clamp(0.0, 1.0),
            ((g - lum) * boost + lum).clamp(0.0, 1.0),
            ((b - lum) * boost + lum).clamp(0.0, 1.0),
        )
    }
}

/// Get transformed health colors for current colorblind mode
pub fn get_health_colors(mode: ColorblindMode) -> (Color, Color, Color, Color) {
    (
        transform_color(colors::HEALTH_FULL, mode),
        transform_color(colors::HEALTH_MEDIUM, mode),
        transform_color(colors::HEALTH_LOW, mode),
        transform_color(colors::HEALTH_CRITICAL, mode),
    )
}

/// Convert Color tuple to egui Color32
pub fn to_egui_color(color: Color) -> egui::Color32 {
    egui::Color32::from_rgb(
        (color.0 * 255.0) as u8,
        (color.1 * 255.0) as u8,
        (color.2 * 255.0) as u8,
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorblind_mode_default() {
        let mode = ColorblindMode::default();
        assert_eq!(mode, ColorblindMode::None);
    }

    #[test]
    fn test_colorblind_mode_all() {
        let modes = ColorblindMode::all();
        assert_eq!(modes.len(), 5);
        assert!(modes.contains(&ColorblindMode::None));
        assert!(modes.contains(&ColorblindMode::HighContrast));
    }

    #[test]
    fn test_accessibility_settings_default() {
        let settings = AccessibilitySettings::default();
        assert_eq!(settings.ui_scale, 1.0);
        assert_eq!(settings.colorblind_mode, ColorblindMode::None);
        assert!(!settings.reduce_motion);
        assert!(!settings.large_text);
    }

    #[test]
    fn test_ui_scale_clamping() {
        let mut settings = AccessibilitySettings::default();

        settings.set_ui_scale(0.5); // Below min
        assert_eq!(settings.ui_scale, 0.8);

        settings.set_ui_scale(2.0); // Above max
        assert_eq!(settings.ui_scale, 1.5);

        settings.set_ui_scale(1.25); // Valid
        assert_eq!(settings.ui_scale, 1.25);
    }

    #[test]
    fn test_font_scale() {
        let mut settings = AccessibilitySettings::default();
        settings.ui_scale = 1.0;
        assert_eq!(settings.font_scale(), 1.0);

        settings.large_text = true;
        assert_eq!(settings.font_scale(), 1.25);

        settings.ui_scale = 1.2;
        assert_eq!(settings.font_scale(), 1.5); // 1.2 * 1.25
    }

    #[test]
    fn test_color_transform_none() {
        let color = colors::HEALTH_FULL;
        let transformed = transform_color(color, ColorblindMode::None);
        assert_eq!(color, transformed);
    }

    #[test]
    fn test_deuteranopia_shifts_green_to_blue() {
        let green = (0.0, 1.0, 0.0);
        let transformed = transform_color(green, ColorblindMode::Deuteranopia);
        // Green should shift to blue
        assert!(transformed.2 > green.2, "Blue should increase");
        assert!(transformed.1 < green.1, "Green should decrease");
    }

    #[test]
    fn test_high_contrast_extremes() {
        // Dark color becomes black
        let dark = (0.1, 0.1, 0.1);
        let transformed = transform_color(dark, ColorblindMode::HighContrast);
        assert_eq!(transformed, (0.0, 0.0, 0.0));

        // Light color becomes white
        let light = (0.9, 0.9, 0.9);
        let transformed = transform_color(light, ColorblindMode::HighContrast);
        assert_eq!(transformed, (1.0, 1.0, 1.0));
    }

    #[test]
    fn test_get_health_colors() {
        let (full, medium, low, critical) = get_health_colors(ColorblindMode::None);
        assert_eq!(full, colors::HEALTH_FULL);
        assert_eq!(medium, colors::HEALTH_MEDIUM);
        assert_eq!(low, colors::HEALTH_LOW);
        assert_eq!(critical, colors::HEALTH_CRITICAL);
    }

    #[test]
    fn test_to_egui_color() {
        let color = (1.0, 0.5, 0.0);
        let egui_color = to_egui_color(color);
        assert_eq!(egui_color.r(), 255);
        assert_eq!(egui_color.g(), 127);
        assert_eq!(egui_color.b(), 0);
    }

    #[test]
    fn test_display_names() {
        assert_eq!(ColorblindMode::None.display_name(), "None");
        assert_eq!(
            ColorblindMode::Deuteranopia.display_name(),
            "Deuteranopia (Red-Green)"
        );
        assert_eq!(ColorblindMode::HighContrast.display_name(), "High Contrast");
    }
}
