// Echo HUD Display - Real-time Echo currency counter with transaction feedback
//
// This module provides a top-right corner HUD element showing the player's
// Echo balance with animated transaction feedback floats (+/- indicators).

use crate::EchoCurrency;

/// Feedback float for transaction animation
#[derive(Debug, Clone)]
pub struct EchoFeedbackFloat {
    /// Amount (+/- Echoes)
    pub amount: i32,
    /// Y position (0.0 = original, 0.2 = top of float range)
    pub position_y: f32,
    /// Alpha (0.0-1.0 for fade in/out)
    pub alpha: f32,
    /// Time alive (0.0-2.0s)
    pub time_alive: f32,
}

impl EchoFeedbackFloat {
    /// Create new feedback float
    pub fn new(amount: i32) -> Self {
        Self {
            amount,
            position_y: 0.0,
            alpha: 1.0,
            time_alive: 0.0,
        }
    }

    /// Update animation (returns true if still alive, false if expired)
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.time_alive += delta_time;

        // Expire after 2s
        if self.time_alive >= 2.0 {
            return false;
        }

        // Animation progress (0.0-1.0)
        let progress = self.time_alive / 2.0;

        // Fade: 0.0-1.0s fade in, 1.0-2.0s fade out
        if progress < 0.5 {
            self.alpha = progress * 2.0; // 0→1 over 0.5
        } else {
            self.alpha = 2.0 - progress * 2.0; // 1→0 over 0.5
        }

        // Float upward (20% of screen height)
        self.position_y = progress * 0.2;

        true // Still alive
    }

    /// Get color (green for +, red for -)
    pub fn color(&self) -> (f32, f32, f32) {
        if self.amount > 0 {
            (0.2, 0.9, 0.2) // Green
        } else {
            (0.9, 0.2, 0.2) // Red
        }
    }

    /// Get text with sign
    pub fn text(&self) -> String {
        if self.amount > 0 {
            format!("+{}", self.amount)
        } else {
            format!("{}", self.amount)
        }
    }
}

/// Echo HUD state
#[derive(Debug, Clone)]
pub struct EchoHud {
    /// Current Echo balance (cached for display)
    pub balance: u32,
    /// Active feedback floats
    pub feedback_floats: Vec<EchoFeedbackFloat>,
    /// Previous balance (for change detection)
    previous_balance: u32,
}

impl EchoHud {
    /// Create new HUD
    pub fn new() -> Self {
        Self {
            balance: 0,
            feedback_floats: Vec::new(),
            previous_balance: 0,
        }
    }

    /// Update HUD with current currency state
    pub fn update(&mut self, currency: &EchoCurrency, delta_time: f32) {
        // Update balance
        let new_balance = currency.count();
        let balance_change = new_balance as i32 - self.previous_balance as i32;

        // Spawn feedback float if balance changed
        if balance_change != 0 {
            self.feedback_floats
                .push(EchoFeedbackFloat::new(balance_change));
        }

        self.balance = new_balance;
        self.previous_balance = new_balance;

        // Update feedback floats (remove expired)
        self.feedback_floats.retain_mut(|f| f.update(delta_time));
    }

    /// Clear all feedback floats (e.g., on scene transition)
    pub fn clear_floats(&mut self) {
        self.feedback_floats.clear();
    }

    /// Get active feedback float count (for debugging)
    pub fn float_count(&self) -> usize {
        self.feedback_floats.len()
    }

    /// Render HUD using egui (call from game UI system)
    #[cfg(any())] // Disabled: egui not in dependencies
    #[allow(dead_code)]
    pub fn render(&self, ctx: &egui::Context) {
        // Top-right corner Echo counter
        egui::Area::new("echo_hud")
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-20.0, 20.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Echo icon (simple circle for now, can be replaced with image)
                    let (rect, _response) =
                        ui.allocate_exact_size(egui::Vec2::new(24.0, 24.0), egui::Sense::hover());
                    ui.painter().circle_filled(
                        rect.center(),
                        12.0,
                        egui::Color32::from_rgb(100, 200, 255), // Cyan
                    );

                    // Echo count
                    ui.label(
                        egui::RichText::new(format!("{}", self.balance))
                            .size(24.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                });
            });

        // Feedback floats (positioned relative to Echo counter)
        let screen_rect = ctx.screen_rect();
        let base_x = screen_rect.max.x - 80.0; // Right side, offset left
        let base_y = screen_rect.min.y + 50.0; // Below Echo counter

        for float in &self.feedback_floats {
            let y_offset = float.position_y * 100.0; // Float upward
            let (r, g, b) = float.color();
            let alpha = (float.alpha * 255.0) as u8;

            egui::Area::new(format!("feedback_float_{:p}", float as *const _))
                .fixed_pos(egui::Pos2::new(base_x, base_y + y_offset))
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(float.text()).size(20.0).strong().color(
                        egui::Color32::from_rgba_unmultiplied(
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8,
                            alpha,
                        ),
                    ));
                });
        }
    }


}

impl Default for EchoHud {
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
    use crate::TransactionReason;

    #[test]
    fn test_feedback_float_creation() {
        let float = EchoFeedbackFloat::new(5);

        assert_eq!(float.amount, 5);
        assert_eq!(float.position_y, 0.0);
        assert_eq!(float.alpha, 1.0);
        assert_eq!(float.time_alive, 0.0);
    }

    #[test]
    fn test_feedback_float_update() {
        let mut float = EchoFeedbackFloat::new(1);

        // Update 1s (halfway through 2s lifetime)
        for _ in 0..60 {
            let still_alive = float.update(0.016); // 60 FPS
            assert!(still_alive);
        }

        assert!(float.position_y > 0.0); // Floated upward
        assert!(float.alpha > 0.0); // Still visible
    }

    #[test]
    fn test_feedback_float_expiry() {
        let mut float = EchoFeedbackFloat::new(1);

        // Update beyond 2s lifetime
        for _ in 0..130 {
            float.update(0.016); // 130 frames @ 60 FPS = 2.08s
        }

        // Should expire
        let still_alive = float.update(0.016);
        assert!(!still_alive);
    }

    #[test]
    fn test_feedback_float_color() {
        let float_positive = EchoFeedbackFloat::new(5);
        assert_eq!(float_positive.color(), (0.2, 0.9, 0.2)); // Green

        let float_negative = EchoFeedbackFloat::new(-3);
        assert_eq!(float_negative.color(), (0.9, 0.2, 0.2)); // Red
    }

    #[test]
    fn test_feedback_float_text() {
        let float_positive = EchoFeedbackFloat::new(7);
        assert_eq!(float_positive.text(), "+7");

        let float_negative = EchoFeedbackFloat::new(-2);
        assert_eq!(float_negative.text(), "-2");
    }

    #[test]
    fn test_hud_creation() {
        let hud = EchoHud::new();

        assert_eq!(hud.balance, 0);
        assert_eq!(hud.feedback_floats.len(), 0);
        assert_eq!(hud.previous_balance, 0);
    }

    #[test]
    fn test_hud_balance_update() {
        let mut hud = EchoHud::new();
        let mut currency = EchoCurrency::new();

        currency.add(5, TransactionReason::TutorialReward);

        hud.update(&currency, 0.016);

        assert_eq!(hud.balance, 5);
        assert_eq!(hud.feedback_floats.len(), 1); // Spawned +5 float
    }

    #[test]
    fn test_hud_feedback_float_spawn() {
        let mut hud = EchoHud::new();
        let mut currency = EchoCurrency::new();

        currency.add(3, TransactionReason::KillRiftStalker);
        hud.update(&currency, 0.016);

        assert_eq!(hud.feedback_floats.len(), 1);
        assert_eq!(hud.feedback_floats[0].amount, 3);

        currency.spend(1, TransactionReason::UseEchoDash);
        hud.update(&currency, 0.016);

        assert_eq!(hud.feedback_floats.len(), 2); // +3 and -1
        assert_eq!(hud.feedback_floats[1].amount, -1);
    }

    #[test]
    fn test_hud_float_expiry() {
        let mut hud = EchoHud::new();
        let mut currency = EchoCurrency::new();

        currency.add(1, TransactionReason::FoundShard);
        hud.update(&currency, 0.016);

        assert_eq!(hud.feedback_floats.len(), 1);

        // Update for 2.1s (beyond 2s lifetime)
        for _ in 0..130 {
            hud.update(&currency, 0.016);
        }

        // Float should be expired and removed
        assert_eq!(hud.feedback_floats.len(), 0);
    }

    #[test]
    fn test_hud_clear_floats() {
        let mut hud = EchoHud::new();
        let mut currency = EchoCurrency::new();

        currency.add(5, TransactionReason::TutorialReward);
        hud.update(&currency, 0.016);

        assert_eq!(hud.feedback_floats.len(), 1);

        hud.clear_floats();

        assert_eq!(hud.feedback_floats.len(), 0);
    }

    #[test]
    fn test_hud_multiple_transactions() {
        let mut hud = EchoHud::new();
        let mut currency = EchoCurrency::new();

        // Multiple transactions in rapid succession
        currency.add(1, TransactionReason::KillRiftStalker);
        hud.update(&currency, 0.016);

        currency.add(2, TransactionReason::KillSentinel);
        hud.update(&currency, 0.016);

        currency.spend(1, TransactionReason::RepairAnchor("test".to_string()));
        hud.update(&currency, 0.016);

        // Should have 3 active floats
        assert_eq!(hud.feedback_floats.len(), 3);
        assert_eq!(hud.balance, 2); // 1 + 2 - 1 = 2
    }

    #[test]
    fn test_hud_fade_animation() {
        let mut float = EchoFeedbackFloat::new(1);

        // At 0.0s: alpha = 1.0 (initial, but NOT after first update)
        assert_eq!(float.alpha, 1.0);

        // After first update (0.016s): alpha should be ~0.0 (progress = 0.008, alpha = 0.016)
        float.update(0.016);
        assert!(float.alpha < 0.05); // Near 0

        // At 0.5s: alpha = 0.5 (mid fade in)
        for _ in 0..30 {
            float.update(0.016); // 30 more frames = 0.496s total
        }
        assert!(float.alpha > 0.4 && float.alpha < 0.6);

        // At 1.0s: alpha = 1.0 (peak)
        for _ in 0..31 {
            float.update(0.016); // 31 more frames = 0.992s total
        }
        assert!(float.alpha > 0.95);

        // At 1.5s: alpha = 0.5 (mid fade out)
        for _ in 0..31 {
            float.update(0.016); // 31 more frames = 1.488s total
        }
        assert!(float.alpha > 0.4 && float.alpha < 0.6);
    }

    #[test]
    fn test_hud_no_spawn_on_no_change() {
        let mut hud = EchoHud::new();
        let currency = EchoCurrency::new(); // No transactions

        hud.update(&currency, 0.016);
        assert_eq!(hud.feedback_floats.len(), 0); // No floats spawned

        hud.update(&currency, 0.016);
        assert_eq!(hud.feedback_floats.len(), 0); // Still no floats
    }
}
