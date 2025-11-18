// Ability Unlock Notification - Slide-in notification for new abilities
//
// This module provides an animated notification that appears when the player
// unlocks a new ability by repairing an anchor. Slides in from bottom, holds for 3s, slides out.

use crate::AbilityType;

/// Notification animation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationState {
    /// Hidden (not showing)
    Hidden,
    /// Sliding in from bottom (0.0-0.5s)
    SlideIn,
    /// Holding at center (0.5-3.5s)
    Hold,
    /// Sliding out to bottom (3.5-4.0s)
    SlideOut,
}

/// Ability unlock notification
#[derive(Debug, Clone)]
pub struct AbilityUnlockNotification {
    /// Current state
    pub state: NotificationState,
    /// Ability being unlocked
    pub ability: Option<AbilityType>,
    /// Animation time (0.0-4.0s for full cycle)
    pub animation_time: f32,
    /// Y position (0.0 = center, 1.0 = off-screen bottom)
    pub position_y: f32,
    /// Alpha (0.0-1.0 for fade)
    pub alpha: f32,
}

impl AbilityUnlockNotification {
    /// Create new notification (hidden)
    pub fn new() -> Self {
        Self {
            state: NotificationState::Hidden,
            ability: None,
            animation_time: 0.0,
            position_y: 1.0, // Off-screen (bottom)
            alpha: 0.0,
        }
    }

    /// Show notification for ability
    pub fn show(&mut self, ability: AbilityType) {
        self.state = NotificationState::SlideIn;
        self.ability = Some(ability);
        self.animation_time = 0.0;
        self.position_y = 1.0; // Start off-screen
        self.alpha = 0.0;
    }

    /// Update animation (call every frame)
    pub fn update(&mut self, delta_time: f32) {
        if self.state == NotificationState::Hidden {
            return;
        }

        self.animation_time += delta_time;

        // State transitions
        match self.state {
            NotificationState::SlideIn => {
                // 0.0-0.5s: Slide in
                if self.animation_time >= 0.5 {
                    self.state = NotificationState::Hold;
                }
            }
            NotificationState::Hold => {
                // 0.5-3.5s: Hold at center
                if self.animation_time >= 3.5 {
                    self.state = NotificationState::SlideOut;
                }
            }
            NotificationState::SlideOut => {
                // 3.5-4.0s: Slide out
                if self.animation_time >= 4.0 {
                    self.hide();
                    return;
                }
            }
            NotificationState::Hidden => {}
        }

        // Update position and alpha based on state
        match self.state {
            NotificationState::SlideIn => {
                // Slide in: 1.0 → 0.0 over 0.5s
                let progress = self.animation_time / 0.5;
                self.position_y = 1.0 - progress; // 1.0 → 0.0
                self.alpha = progress; // Fade in
            }
            NotificationState::Hold => {
                // Hold at center
                self.position_y = 0.0;
                self.alpha = 1.0;
            }
            NotificationState::SlideOut => {
                // Slide out: 0.0 → 1.0 over 0.5s
                let slide_start = 3.5;
                let progress = (self.animation_time - slide_start) / 0.5;
                self.position_y = progress; // 0.0 → 1.0
                self.alpha = 1.0 - progress; // Fade out
            }
            NotificationState::Hidden => {}
        }
    }

    /// Hide notification immediately
    pub fn hide(&mut self) {
        self.state = NotificationState::Hidden;
        self.ability = None;
        self.animation_time = 0.0;
        self.position_y = 1.0;
        self.alpha = 0.0;
    }

    /// Check if notification is visible
    pub fn is_visible(&self) -> bool {
        self.state != NotificationState::Hidden
    }

    /// Get ability name
    pub fn ability_name(&self) -> Option<&'static str> {
        self.ability.as_ref().map(|ability| match ability {
            AbilityType::EchoDash => "Echo Dash",
            AbilityType::BarricadeDeploy => "Barricade Deploy",
        })
    }

    /// Get ability description
    pub fn ability_description(&self) -> Option<&'static str> {
        self.ability.as_ref().map(|ability| match ability {
            AbilityType::EchoDash => "Press SHIFT to teleport dash through reality rifts",
            AbilityType::BarricadeDeploy => "Press B to deploy tactical barricades",
        })
    }

    /// Get ability icon (placeholder, can be replaced with image)
    pub fn ability_icon_text(&self) -> &'static str {
        match self.ability {
            Some(AbilityType::EchoDash) => "⚡",        // Lightning bolt
            Some(AbilityType::BarricadeDeploy) => "🛡️", // Shield
            None => "",
        }
    }

    /// Render notification using egui (call from game UI system)
    #[cfg(feature = "egui")]
    pub fn render(&self, ctx: &egui::Context) {
        if !self.is_visible() {
            return;
        }

        let screen_rect = ctx.screen_rect();
        let screen_height = screen_rect.height();

        // Calculate Y position (0.0 = center, 1.0 = off-screen bottom)
        let y_pos = screen_rect.center().y + self.position_y * (screen_height * 0.5 + 100.0);

        // Notification panel
        egui::Area::new("ability_unlock_notification")
            .anchor(
                egui::Align2::CENTER_CENTER,
                egui::Vec2::new(0.0, y_pos - screen_rect.center().y),
            )
            .show(ctx, |ui| {
                let alpha = (self.alpha * 255.0) as u8;

                egui::Frame::window(ui.style())
                    .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 40, alpha))
                    .stroke(egui::Stroke::new(
                        2.0,
                        egui::Color32::from_rgba_unmultiplied(200, 150, 50, alpha),
                    ))
                    .show(ui, |ui| {
                        ui.set_min_width(400.0);
                        ui.vertical_centered(|ui| {
                            // "New Ability Unlocked!" header
                            ui.label(
                                egui::RichText::new("✨ New Ability Unlocked! ✨")
                                    .size(24.0)
                                    .strong()
                                    .color(egui::Color32::from_rgba_unmultiplied(
                                        255, 215, 0, alpha,
                                    )),
                            );
                            ui.add_space(10.0);

                            // Ability icon (large emoji placeholder)
                            ui.label(egui::RichText::new(self.ability_icon_text()).size(64.0));
                            ui.add_space(10.0);

                            // Ability name
                            if let Some(name) = self.ability_name() {
                                ui.label(egui::RichText::new(name).size(28.0).strong().color(
                                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                                ));
                            }
                            ui.add_space(5.0);

                            // Ability description
                            if let Some(desc) = self.ability_description() {
                                ui.label(egui::RichText::new(desc).size(16.0).italics().color(
                                    egui::Color32::from_rgba_unmultiplied(200, 200, 200, alpha),
                                ));
                            }
                            ui.add_space(10.0);
                        });
                    });
            });
    }

    /// Render notification (no-op when egui feature is disabled)
    #[cfg(not(feature = "egui"))]
    pub fn render(&self, _ctx: &()) {
        // No-op: egui not available
    }
}

impl Default for AbilityUnlockNotification {
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
    fn test_notification_creation() {
        let notif = AbilityUnlockNotification::new();

        assert_eq!(notif.state, NotificationState::Hidden);
        assert!(notif.ability.is_none());
        assert_eq!(notif.animation_time, 0.0);
        assert_eq!(notif.position_y, 1.0); // Off-screen
        assert_eq!(notif.alpha, 0.0);
    }

    #[test]
    fn test_show_notification() {
        let mut notif = AbilityUnlockNotification::new();

        notif.show(AbilityType::EchoDash);

        assert_eq!(notif.state, NotificationState::SlideIn);
        assert_eq!(notif.ability, Some(AbilityType::EchoDash));
        assert!(notif.is_visible());
    }

    #[test]
    fn test_hide_notification() {
        let mut notif = AbilityUnlockNotification::new();

        notif.show(AbilityType::EchoDash);
        assert!(notif.is_visible());

        notif.hide();
        assert!(!notif.is_visible());
        assert_eq!(notif.state, NotificationState::Hidden);
        assert!(notif.ability.is_none());
    }

    #[test]
    fn test_slide_in_animation() {
        let mut notif = AbilityUnlockNotification::new();
        notif.show(AbilityType::EchoDash);

        // Update for 0.5s+ (slide in duration)
        for _ in 0..32 {
            notif.update(0.016); // 32 frames @ 60 FPS = 0.512s → past 0.5s
        }

        // Should transition to Hold
        assert_eq!(notif.state, NotificationState::Hold);
        assert!((notif.position_y - 0.0).abs() < 0.01); // Near center
        assert!((notif.alpha - 1.0).abs() < 0.01); // Nearly fully visible
    }

    #[test]
    fn test_hold_animation() {
        let mut notif = AbilityUnlockNotification::new();
        notif.show(AbilityType::EchoDash);

        // Update to Hold state
        for _ in 0..32 {
            notif.update(0.016); // 0.512s → past 0.5s
        }
        assert_eq!(notif.state, NotificationState::Hold);

        // Update for 3s more (hold duration) - need to reach 3.5s+ total
        for _ in 0..188 {
            notif.update(0.016); // 188 * 0.016 = 3.008s → 3.52s total (past 3.5s)
        }

        // Should transition to SlideOut
        assert_eq!(notif.state, NotificationState::SlideOut);
    }

    #[test]
    fn test_slide_out_animation() {
        let mut notif = AbilityUnlockNotification::new();
        notif.show(AbilityType::EchoDash);

        // Update to SlideOut state (3.5s)
        for _ in 0..220 {
            notif.update(0.016); // 3.52s
        }
        assert_eq!(notif.state, NotificationState::SlideOut);

        // Update for 0.5s more (slide out duration)
        for _ in 0..30 {
            notif.update(0.016); // 0.48s
        }

        // Should be hidden
        assert_eq!(notif.state, NotificationState::Hidden);
        assert!(!notif.is_visible());
    }

    #[test]
    fn test_full_animation_cycle() {
        let mut notif = AbilityUnlockNotification::new();
        notif.show(AbilityType::EchoDash);

        // Update for full 4s cycle
        for _ in 0..250 {
            notif.update(0.016); // 4.0s
        }

        // Should complete and hide
        assert_eq!(notif.state, NotificationState::Hidden);
        assert!(!notif.is_visible());
    }

    #[test]
    fn test_ability_name() {
        let mut notif = AbilityUnlockNotification::new();

        notif.show(AbilityType::EchoDash);
        assert_eq!(notif.ability_name(), Some("Echo Dash"));

        notif.show(AbilityType::BarricadeDeploy);
        assert_eq!(notif.ability_name(), Some("Barricade Deploy"));
    }

    #[test]
    fn test_ability_description() {
        let mut notif = AbilityUnlockNotification::new();

        notif.show(AbilityType::EchoDash);
        assert!(notif.ability_description().unwrap().contains("SHIFT"));

        notif.show(AbilityType::BarricadeDeploy);
        assert!(notif.ability_description().unwrap().contains("B"));
    }

    #[test]
    fn test_ability_icon() {
        let mut notif = AbilityUnlockNotification::new();

        notif.show(AbilityType::EchoDash);
        assert_eq!(notif.ability_icon_text(), "⚡");

        notif.show(AbilityType::BarricadeDeploy);
        assert_eq!(notif.ability_icon_text(), "🛡️");
    }

    #[test]
    fn test_position_y_progression() {
        let mut notif = AbilityUnlockNotification::new();
        notif.show(AbilityType::EchoDash);

        // Slide in: 1.0 → 0.0
        assert_eq!(notif.position_y, 1.0);

        for _ in 0..31 {
            notif.update(0.016);
        }
        assert!((notif.position_y - 0.0).abs() < 0.01); // Near center

        // Hold: 0.0
        for _ in 0..188 {
            notif.update(0.016);
        }
        assert!((notif.position_y - 0.0).abs() < 0.01); // Still at center

        // Slide out: 0.0 → 1.0
        for _ in 0..31 {
            notif.update(0.016);
        }
        assert!(notif.position_y > 0.9); // Near off-screen
    }

    #[test]
    fn test_alpha_progression() {
        let mut notif = AbilityUnlockNotification::new();
        notif.show(AbilityType::EchoDash);

        // Slide in: 0.0 → 1.0
        assert_eq!(notif.alpha, 0.0);

        for _ in 0..31 {
            notif.update(0.016);
        }
        assert!((notif.alpha - 1.0).abs() < 0.01); // Nearly fully visible

        // Hold: 1.0
        for _ in 0..188 {
            notif.update(0.016);
        }
        assert!((notif.alpha - 1.0).abs() < 0.01); // Still fully visible

        // Slide out: 1.0 → 0.0
        for _ in 0..31 {
            notif.update(0.016);
        }
        assert!(notif.alpha < 0.1); // Faded out
    }
}
