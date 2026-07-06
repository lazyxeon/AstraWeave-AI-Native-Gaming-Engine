// tools/aw_editor/src/ui/palette.rs - AstraWeave branded palette
//
// Single source of truth for the AstraWeave branded theme colors. Every branded
// color in the editor chrome is defined here and ONLY here — call sites must
// reference these constants (or the theme-aware helpers below) instead of
// constructing `Color32` values inline.
//
// Routing conventions (invariants — see docs/architecture/aw_editor.md §8):
// - The editor accent rides `Visuals::hyperlink_color` for ALL themes; call
//   sites read it via `palette::accent(visuals)` and never hard-code the accent.
// - Severity text uses `visuals.error_fg_color` / `visuals.warn_fg_color`
//   (theme-aware); SUCCESS has no `Visuals` field, so it is the one severity
//   constant referenced directly.
// - `astraweave_visuals()` must NEVER set `override_text_color`: doing so
//   flattens the weak/faint text tiers and the severity colors.

use egui::{Color32, CornerRadius, Stroke, Visuals};

// ── Black metallic-flake base (backgrounds, darkest → lightest) ──────────────
// Near-black surfaces with a subtle cool-blue flake sheen.
pub const BG_DEEP: Color32 = Color32::from_rgb(5, 7, 11); // extreme_bg, tab bar, text edits
pub const BG_PANEL: Color32 = Color32::from_rgb(9, 12, 18); // panel_fill (black w/ blue flake)
pub const BG_WINDOW: Color32 = Color32::from_rgb(13, 17, 25); // window/popup/menu fill, active tab body
pub const BG_FAINT: Color32 = Color32::from_rgb(15, 20, 29); // faint_bg (striped rows)
pub const BG_WIDGET: Color32 = Color32::from_rgb(21, 27, 38); // inactive widget fill
pub const BG_HOVER: Color32 = Color32::from_rgb(29, 38, 54); // hovered widget fill
pub const BG_ACTIVE: Color32 = Color32::from_rgb(14, 42, 64); // pressed widget fill (blue-tinted)
pub const STROKE_DIM: Color32 = Color32::from_rgb(42, 52, 74); // borders, hlines, separators

// ── Neon sky-blue accent ──────────────────────────────────────────────────────
pub const ACCENT: Color32 = Color32::from_rgb(0, 180, 255);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(80, 205, 255);
pub const ACCENT_ACTIVE: Color32 = Color32::from_rgb(0, 145, 215);
pub const SELECTION_BG: Color32 = Color32::from_rgb(12, 62, 100); // selection.bg_fill (dim blue)
pub const ON_ACCENT: Color32 = Color32::from_rgb(3, 12, 22); // text on accent-filled buttons

// ── Text tiers ────────────────────────────────────────────────────────────────
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(224, 232, 240);
pub const TEXT_WEAK: Color32 = Color32::from_rgb(148, 160, 180);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(96, 106, 128);

// ── Severity ─────────────────────────────────────────────────────────────────
pub const ERROR: Color32 = Color32::from_rgb(255, 95, 87);
pub const WARNING: Color32 = Color32::from_rgb(255, 165, 60); // orange (concept)
pub const SUCCESS: Color32 = Color32::from_rgb(80, 210, 120);

// ── Dialog action-button fills (dark, for light button text) ─────────────────
pub const BTN_SAFE_FILL: Color32 = Color32::from_rgb(24, 96, 60); // save / confirm
pub const BTN_DANGER_FILL: Color32 = Color32::from_rgb(126, 40, 46); // discard / destructive

// ── Entity catalog category hues (harmonized for navy bg) ────────────────────
pub const CAT_CHARACTER: Color32 = Color32::from_rgb(92, 176, 255);
pub const CAT_BUILDING: Color32 = Color32::from_rgb(255, 158, 92);
pub const CAT_VEHICLE: Color32 = Color32::from_rgb(96, 214, 160);
pub const CAT_NATURE: Color32 = Color32::from_rgb(112, 200, 96);
pub const CAT_FURNITURE: Color32 = Color32::from_rgb(206, 168, 116);
pub const CAT_FOOD: Color32 = Color32::from_rgb(255, 204, 96);
pub const CAT_WEAPON: Color32 = Color32::from_rgb(240, 98, 98);
pub const CAT_INFRASTRUCTURE: Color32 = Color32::from_rgb(160, 170, 220);
pub const CAT_PROP: Color32 = Color32::from_rgb(198, 150, 255);

/// Corner radius for widgets (buttons, text edits, checkboxes).
pub const WIDGET_RADIUS: u8 = 6;
/// Corner radius for windows, menus, and overlay cards.
pub const CARD_RADIUS: u8 = 8;

/// Build the `Visuals` for the AstraWeave branded theme.
///
/// INVARIANT: must never set `override_text_color` — that would flatten the
/// weak/faint text tiers and severity colors (guarded by a unit test below).
pub fn astraweave_visuals() -> Visuals {
    let mut v = Visuals::dark();

    // Surfaces
    v.panel_fill = BG_PANEL;
    v.window_fill = BG_WINDOW;
    v.extreme_bg_color = BG_DEEP;
    v.faint_bg_color = BG_FAINT;
    v.code_bg_color = BG_DEEP;
    v.text_edit_bg_color = Some(BG_DEEP);
    v.window_stroke = Stroke::new(1.0, STROKE_DIM);
    v.window_corner_radius = CornerRadius::same(CARD_RADIUS);
    v.menu_corner_radius = CornerRadius::same(CARD_RADIUS);

    // Text
    v.weak_text_color = Some(TEXT_WEAK);
    v.hyperlink_color = ACCENT; // editor accent rides hyperlink_color
    v.warn_fg_color = WARNING;
    v.error_fg_color = ERROR;

    // Selection
    v.selection.bg_fill = SELECTION_BG;
    v.selection.stroke = Stroke::new(1.0, ACCENT);

    // Widget states.
    // Text-tier wiring (egui derives these from fg_stroke colors):
    //   text_color()        = noninteractive.fg_stroke → TEXT_PRIMARY
    //   weak_text_color()   = the Some(TEXT_WEAK) override above (must stay
    //                         dimmer than noninteractive fg — tier separation)
    //   strong_text_color() = active.fg_stroke → WHITE (neutral emphasis; the
    //                         accent must NOT leak into .strong() text)
    v.widgets.noninteractive.bg_fill = BG_PANEL;
    v.widgets.noninteractive.weak_bg_fill = BG_PANEL;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, STROKE_DIM);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);

    v.widgets.inactive.bg_fill = BG_WIDGET;
    v.widgets.inactive.weak_bg_fill = BG_WIDGET;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);

    v.widgets.hovered.bg_fill = BG_HOVER;
    v.widgets.hovered.weak_bg_fill = BG_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_ACTIVE);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);

    v.widgets.active.bg_fill = BG_ACTIVE;
    v.widgets.active.weak_bg_fill = BG_ACTIVE;
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);

    v.widgets.open.bg_fill = BG_HOVER;
    v.widgets.open.weak_bg_fill = BG_HOVER;
    v.widgets.open.bg_stroke = Stroke::new(1.0, STROKE_DIM);

    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(WIDGET_RADIUS);
    }

    v.slider_trailing_fill = true;

    v
}

/// Overlay card fill derived from the active theme (viewport toolbar,
/// performance/camera overlay cards). Theme-aware so non-branded themes keep
/// legible overlays.
pub fn overlay_fill(visuals: &Visuals) -> Color32 {
    let p = visuals.panel_fill;
    Color32::from_rgba_unmultiplied(p.r(), p.g(), p.b(), 235)
}

/// The editor accent color for the active theme.
///
/// Convention: the accent rides `Visuals::hyperlink_color` for all themes;
/// call sites must use this instead of hard-coding teal.
pub fn accent(visuals: &Visuals) -> Color32 {
    visuals.hyperlink_color
}

/// Add an accent-filled primary action button (empty-state actions, Reset Camera) with
/// real hover/pressed state fills. `Button::fill` would override the
/// per-state fills and make the button feel dead, so this scopes the widget
/// visuals instead. `size: None` uses natural sizing.
pub fn accent_button_ui(ui: &mut egui::Ui, size: Option<egui::Vec2>, text: &str) -> egui::Response {
    ui.scope(|ui| {
        let widgets = &mut ui.style_mut().visuals.widgets;
        let on_accent = Stroke::new(1.5, ON_ACCENT);
        widgets.inactive.bg_fill = ACCENT;
        widgets.inactive.weak_bg_fill = ACCENT;
        widgets.inactive.fg_stroke = on_accent;
        widgets.hovered.bg_fill = ACCENT_HOVER;
        widgets.hovered.weak_bg_fill = ACCENT_HOVER;
        widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_HOVER);
        widgets.hovered.fg_stroke = on_accent;
        widgets.active.bg_fill = ACCENT_ACTIVE;
        widgets.active.weak_bg_fill = ACCENT_ACTIVE;
        widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_ACTIVE);
        widgets.active.fg_stroke = on_accent;
        let button = egui::Button::new(egui::RichText::new(text).color(ON_ACCENT).strong())
            .corner_radius(CornerRadius::same(WIDGET_RADIUS));
        match size {
            Some(s) => ui.add_sized(s, button),
            None => ui.add(button),
        }
    })
    .inner
}

/// Paint an isometric cube glyph (Inspector empty state). Painter-drawn since
/// the default egui fonts have no reliable cube glyph.
pub fn paint_cube_glyph(painter: &egui::Painter, center: egui::Pos2, size: f32, color: Color32) {
    let w = size * 0.5; // half-width of the cube silhouette
    let top = egui::pos2(center.x, center.y - size * 0.5);
    let upper_left = egui::pos2(center.x - w, center.y - size * 0.25);
    let upper_right = egui::pos2(center.x + w, center.y - size * 0.25);
    let mid = egui::pos2(center.x, center.y); // front vertical edge, top
    let lower_left = egui::pos2(center.x - w, center.y + size * 0.25);
    let lower_right = egui::pos2(center.x + w, center.y + size * 0.25);
    let bottom = egui::pos2(center.x, center.y + size * 0.5);

    // Top face fill (low alpha)
    let face = color.gamma_multiply(0.25);
    painter.add(egui::Shape::convex_polygon(
        vec![top, upper_right, mid, upper_left],
        face,
        Stroke::NONE,
    ));

    let stroke = Stroke::new(1.5, color);
    for (a, b) in [
        (top, upper_left),
        (top, upper_right),
        (upper_left, mid),
        (upper_right, mid),
        (upper_left, lower_left),
        (upper_right, lower_right),
        (mid, bottom),
        (lower_left, bottom),
        (lower_right, bottom),
    ] {
        painter.line_segment([a, b], stroke);
    }
}

/// Centered "no entity selected" empty state: accent cube glyph + label +
/// weak hint (Inspector / Transform panels).
pub fn empty_selection_state(ui: &mut egui::Ui, hint: &str) {
    let accent = accent(ui.visuals());
    let avail = ui.available_size();
    ui.add_space((avail.y * 0.5 - 60.0).max(12.0));
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(56.0, 56.0), egui::Sense::hover());
        paint_cube_glyph(ui.painter(), rect.center(), 44.0, accent);
        ui.add_space(10.0);
        ui.label(egui::RichText::new("No entity selected").strong());
        ui.add_space(4.0);
        ui.weak(hint);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn astraweave_visuals_never_overrides_text_color() {
        // Setting override_text_color would flatten text tiers + severity colors.
        let v = astraweave_visuals();
        assert!(v.override_text_color.is_none());
    }

    #[test]
    fn accent_rides_hyperlink_color() {
        let v = astraweave_visuals();
        assert_eq!(v.hyperlink_color, ACCENT);
        assert_eq!(accent(&v), ACCENT);
    }

    #[test]
    fn text_tiers_stay_distinct() {
        let v = astraweave_visuals();
        // Normal label text is the primary tier.
        assert_eq!(v.text_color(), TEXT_PRIMARY);
        // Weak text must stay dimmer than / distinct from normal labels.
        assert_ne!(v.weak_text_color(), v.text_color());
        // Strong (.strong()) text must be neutral emphasis, not the accent.
        assert_ne!(v.strong_text_color(), v.hyperlink_color);
    }

    #[test]
    fn astraweave_visuals_uses_palette_surfaces() {
        let v = astraweave_visuals();
        assert!(v.dark_mode);
        assert_eq!(v.panel_fill, BG_PANEL);
        assert_eq!(v.window_fill, BG_WINDOW);
        assert_eq!(v.extreme_bg_color, BG_DEEP);
        assert_eq!(v.warn_fg_color, WARNING);
        assert_eq!(v.error_fg_color, ERROR);
        assert_eq!(v.selection.bg_fill, SELECTION_BG);
    }

    #[test]
    fn astraweave_visuals_rounds_corners() {
        let v = astraweave_visuals();
        assert_eq!(v.window_corner_radius, CornerRadius::same(CARD_RADIUS));
        assert_eq!(
            v.widgets.inactive.corner_radius,
            CornerRadius::same(WIDGET_RADIUS)
        );
        assert_eq!(
            v.widgets.hovered.corner_radius,
            CornerRadius::same(WIDGET_RADIUS)
        );
    }

    #[test]
    fn overlay_fill_derives_from_panel_fill() {
        // Color32 stores premultiplied alpha, so compare the unmultiplied
        // components (with ±1 rounding slack) rather than raw channels.
        let v = astraweave_visuals();
        let fill = overlay_fill(&v);
        let [r, g, b, a] = fill.to_srgba_unmultiplied();
        assert_eq!(a, 235);
        assert!((r as i16 - BG_PANEL.r() as i16).abs() <= 1);
        assert!((g as i16 - BG_PANEL.g() as i16).abs() <= 1);
        assert!((b as i16 - BG_PANEL.b() as i16).abs() <= 1);
    }
}
