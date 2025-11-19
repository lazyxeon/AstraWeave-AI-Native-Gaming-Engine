// Anchor Inspection Modal - UI for examining and repairing anchors
//
// This module provides an egui-based modal window for inspecting anchor status
// and initiating repairs. Triggered by E key when player is in proximity.

use crate::{AbilityType, Anchor, AnchorVfxState};

/// Inspection modal state
#[derive(Debug, Clone)]
pub struct AnchorInspectionModal {
    /// Is modal currently visible?
    pub visible: bool,
    /// Anchor ID being inspected
    pub anchor_id: Option<usize>,
    /// Anchor stability (0.0-1.0)
    pub stability: f32,
    /// Repair cost in Echoes
    pub repair_cost: u32,
    /// Ability unlocked on repair (if any)
    pub unlocks_ability: Option<AbilityType>,
    /// Has this anchor been repaired before?
    pub is_repaired: bool,
    /// Current Echo balance (for affordability check)
    pub player_echo_balance: u32,
    /// Repair button pressed this frame?
    pub repair_requested: bool,
}

impl AnchorInspectionModal {
    /// Create new modal (hidden by default)
    pub fn new() -> Self {
        Self {
            visible: false,
            anchor_id: None,
            stability: 0.0,
            repair_cost: 0,
            unlocks_ability: None,
            is_repaired: false,
            player_echo_balance: 0,
            repair_requested: false,
        }
    }

    /// Open modal with anchor data
    pub fn open(&mut self, anchor_id: usize, anchor: &Anchor, player_echo_balance: u32) {
        self.visible = true;
        self.anchor_id = Some(anchor_id);
        self.stability = anchor.stability();
        self.repair_cost = anchor.repair_cost();
        self.unlocks_ability = anchor.unlocks_ability();
        self.is_repaired = anchor.is_repaired();
        self.player_echo_balance = player_echo_balance;
        self.repair_requested = false;
    }

    /// Close modal
    pub fn close(&mut self) {
        self.visible = false;
        self.anchor_id = None;
        self.repair_requested = false;
    }

    /// Check if player can afford repair
    pub fn can_afford_repair(&self) -> bool {
        self.player_echo_balance >= self.repair_cost
    }

    /// Check if anchor needs repair
    pub fn needs_repair(&self) -> bool {
        self.stability < 1.0
    }

    /// Get VFX state for color coding
    pub fn vfx_state(&self) -> AnchorVfxState {
        AnchorVfxState::from_stability(self.stability)
    }

    /// Get stability percentage string (for display)
    pub fn stability_percentage(&self) -> String {
        format!("{:.0}%", self.stability * 100.0)
    }

    /// Get stability color for UI (RGB 0.0-1.0)
    pub fn stability_color(&self) -> (f32, f32, f32) {
        match self.vfx_state() {
            AnchorVfxState::Perfect => (0.2, 0.8, 0.2),  // Green
            AnchorVfxState::Stable => (0.3, 0.6, 0.9),   // Blue
            AnchorVfxState::Unstable => (0.9, 0.7, 0.2), // Yellow
            AnchorVfxState::Critical => (0.9, 0.3, 0.1), // Red
            AnchorVfxState::Broken => (0.5, 0.5, 0.5),   // Gray
        }
    }

    /// Get status text for display
    pub fn status_text(&self) -> &'static str {
        match self.vfx_state() {
            AnchorVfxState::Perfect => "Perfect",
            AnchorVfxState::Stable => "Stable",
            AnchorVfxState::Unstable => "Unstable",
            AnchorVfxState::Critical => "Critical",
            AnchorVfxState::Broken => "Broken",
        }
    }

    /// Get ability name string (for display)
    pub fn ability_name(&self) -> Option<&'static str> {
        self.unlocks_ability.as_ref().map(|ability| match ability {
            AbilityType::EchoDash => "Echo Dash",
            AbilityType::BarricadeDeploy => "Barricade Deploy",
        })
    }

    /// Get ability description (for display)
    pub fn ability_description(&self) -> Option<&'static str> {
        self.unlocks_ability.as_ref().map(|ability| match ability {
            AbilityType::EchoDash => "Teleport dash through reality rifts",
            AbilityType::BarricadeDeploy => "Deploy tactical barricades",
        })
    }

    /// Render modal using egui (call from game UI system)
    #[cfg(any())] // Disabled: egui not in dependencies
    #[allow(dead_code)]
    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        // Center modal window
        egui::Window::new("Anchor Inspection")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.set_min_width(400.0);

                // Anchor ID
                if let Some(anchor_id) = self.anchor_id {
                    ui.label(format!("Anchor #{}", anchor_id));
                }
                ui.separator();

                // Stability bar
                ui.label("Stability:");
                let (r, g, b) = self.stability_color();
                let color = egui::Color32::from_rgb(
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                );

                let progress_bar = egui::ProgressBar::new(self.stability)
                    .text(self.stability_percentage())
                    .fill(color);
                ui.add(progress_bar);

                ui.label(format!("Status: {}", self.status_text()));
                ui.add_space(10.0);

                // Repair cost
                ui.horizontal(|ui| {
                    ui.label("Repair Cost:");
                    ui.label(format!("{} Echoes", self.repair_cost));
                });

                ui.horizontal(|ui| {
                    ui.label("Your Echoes:");
                    ui.label(format!("{}", self.player_echo_balance));
                });
                ui.add_space(10.0);

                // Ability unlock (if any)
                if let Some(ability_name) = self.ability_name() {
                    ui.separator();
                    ui.label(egui::RichText::new("Unlocks Ability:").strong());
                    ui.label(egui::RichText::new(ability_name).color(egui::Color32::GOLD));
                    if let Some(desc) = self.ability_description() {
                        ui.label(egui::RichText::new(desc).italics().small());
                    }
                    ui.add_space(10.0);
                }

                // Already repaired notice
                if self.is_repaired {
                    ui.separator();
                    ui.label(
                        egui::RichText::new("✓ Previously repaired").color(egui::Color32::GREEN),
                    );
                    ui.add_space(5.0);
                }

                ui.separator();

                // Buttons
                ui.horizontal(|ui| {
                    // Repair button
                    let can_repair = self.needs_repair() && self.can_afford_repair();
                    let button_text = if !self.needs_repair() {
                        "Already at Max Stability"
                    } else if !self.can_afford_repair() {
                        "Insufficient Echoes"
                    } else {
                        "Repair (R)"
                    };

                    let repair_button = egui::Button::new(button_text).fill(if can_repair {
                        egui::Color32::from_rgb(40, 120, 40)
                    } else {
                        egui::Color32::DARK_GRAY
                    });

                    if ui.add_enabled(can_repair, repair_button).clicked() {
                        self.repair_requested = true;
                    }

                    // Close button
                    if ui.button("Close (ESC)").clicked() {
                        self.close();
                    }
                });
            });

        // Handle ESC key to close modal
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.close();
        }

        // Handle R key for repair (if affordable)
        if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            if self.needs_repair() && self.can_afford_repair() {
                self.repair_requested = true;
            }
        }
    }


}

impl Default for AnchorInspectionModal {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_creation() {
        let modal = AnchorInspectionModal::new();

        assert!(!modal.visible);
        assert!(modal.anchor_id.is_none());
        assert_eq!(modal.stability, 0.0);
        assert!(!modal.repair_requested);
    }

    #[test]
    fn test_open_modal() {
        let mut modal = AnchorInspectionModal::new();
        let anchor = Anchor::new(0.7, 2, Some(AbilityType::EchoDash));

        modal.open(1, &anchor, 5);

        assert!(modal.visible);
        assert_eq!(modal.anchor_id, Some(1));
        assert_eq!(modal.stability, 0.7);
        assert_eq!(modal.repair_cost, 2);
        assert_eq!(modal.unlocks_ability, Some(AbilityType::EchoDash));
        assert_eq!(modal.player_echo_balance, 5);
    }

    #[test]
    fn test_close_modal() {
        let mut modal = AnchorInspectionModal::new();
        let anchor = Anchor::new(0.5, 1, None);

        modal.open(1, &anchor, 3);
        assert!(modal.visible);

        modal.close();
        assert!(!modal.visible);
        assert!(modal.anchor_id.is_none());
        assert!(!modal.repair_requested);
    }

    #[test]
    fn test_can_afford_repair() {
        let mut modal = AnchorInspectionModal::new();
        let anchor = Anchor::new(0.5, 2, None);

        modal.open(1, &anchor, 3);
        assert!(modal.can_afford_repair()); // 3 >= 2

        modal.player_echo_balance = 1;
        assert!(!modal.can_afford_repair()); // 1 < 2
    }

    #[test]
    fn test_needs_repair() {
        let mut modal = AnchorInspectionModal::new();
        let anchor = Anchor::new(1.0, 0, None);

        modal.open(1, &anchor, 0);
        assert!(!modal.needs_repair()); // Perfect (1.0)

        modal.stability = 0.7;
        assert!(modal.needs_repair()); // Needs repair
    }

    #[test]
    fn test_stability_color() {
        let mut modal = AnchorInspectionModal::new();

        modal.stability = 1.0; // Perfect
        let color = modal.stability_color();
        assert_eq!(color, (0.2, 0.8, 0.2)); // Green

        modal.stability = 0.7; // Stable
        let color = modal.stability_color();
        assert_eq!(color, (0.3, 0.6, 0.9)); // Blue

        modal.stability = 0.5; // Unstable
        let color = modal.stability_color();
        assert_eq!(color, (0.9, 0.7, 0.2)); // Yellow

        modal.stability = 0.2; // Critical
        let color = modal.stability_color();
        assert_eq!(color, (0.9, 0.3, 0.1)); // Red

        modal.stability = 0.0; // Broken
        let color = modal.stability_color();
        assert_eq!(color, (0.5, 0.5, 0.5)); // Gray
    }

    #[test]
    fn test_status_text() {
        let mut modal = AnchorInspectionModal::new();

        modal.stability = 1.0;
        assert_eq!(modal.status_text(), "Perfect");

        modal.stability = 0.7;
        assert_eq!(modal.status_text(), "Stable");

        modal.stability = 0.5;
        assert_eq!(modal.status_text(), "Unstable");

        modal.stability = 0.2;
        assert_eq!(modal.status_text(), "Critical");

        modal.stability = 0.0;
        assert_eq!(modal.status_text(), "Broken");
    }

    #[test]
    fn test_ability_name() {
        let mut modal = AnchorInspectionModal::new();

        modal.unlocks_ability = Some(AbilityType::EchoDash);
        assert_eq!(modal.ability_name(), Some("Echo Dash"));

        modal.unlocks_ability = Some(AbilityType::BarricadeDeploy);
        assert_eq!(modal.ability_name(), Some("Barricade Deploy"));

        modal.unlocks_ability = None;
        assert_eq!(modal.ability_name(), None);
    }

    #[test]
    fn test_stability_percentage() {
        let mut modal = AnchorInspectionModal::new();

        modal.stability = 1.0;
        assert_eq!(modal.stability_percentage(), "100%");

        modal.stability = 0.73;
        assert_eq!(modal.stability_percentage(), "73%");

        modal.stability = 0.0;
        assert_eq!(modal.stability_percentage(), "0%");
    }

    #[test]
    fn test_repair_request() {
        let mut modal = AnchorInspectionModal::new();
        let anchor = Anchor::new(0.5, 1, None);

        modal.open(1, &anchor, 3);
        assert!(!modal.repair_requested);

        // Simulate repair button press
        modal.repair_requested = true;
        assert!(modal.repair_requested);

        // Close should reset repair_requested
        modal.close();
        assert!(!modal.repair_requested);
    }
}
