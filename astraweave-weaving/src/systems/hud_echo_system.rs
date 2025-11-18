/// HUD Echo System
///
/// Displays Echo currency HUD:
/// - Current balance (top-right corner)
/// - Transaction feedback floats (+3 Echoes, -2 Echoes)
/// - Fade animations (0.0 → 1.0 → 0.0 over 2s)
/// - Color coding (green for gains, red for spends)
///
/// Integration:
/// - Input: Query<&EchoCurrency>, Res<TransactionFeedbackEvents>
/// - Output: HUD render data (for UI system)
use crate::echo_currency::EchoCurrency;

/// HUD state for Echo display
#[derive(Debug, Clone)]
pub struct EchoHudState {
    pub balance: u32,
    pub feedback_floats: Vec<FeedbackFloat>,
}

/// Transaction feedback float (UI element)
#[derive(Debug, Clone)]
pub struct FeedbackFloat {
    pub amount: i32,
    pub position_y: f32, // Vertical position (0.0 = top, 1.0 = bottom)
    pub alpha: f32,      // Opacity (0.0 = invisible, 1.0 = opaque)
    pub time_alive: f32, // Seconds since creation
}

impl FeedbackFloat {
    pub fn new(amount: i32) -> Self {
        Self {
            amount,
            position_y: 0.0,
            alpha: 1.0,
            time_alive: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time_alive += delta_time;

        // Fade animation: 0.0 → 1.0 → 0.0 over 2s
        const LIFETIME: f32 = 2.0;
        let progress = self.time_alive / LIFETIME;

        if progress < 0.5 {
            // Fade in (0.0 → 1.0 over 0-1s)
            self.alpha = progress * 2.0;
        } else {
            // Fade out (1.0 → 0.0 over 1-2s)
            self.alpha = 2.0 - progress * 2.0;
        }

        // Float upward
        self.position_y = progress * 0.2; // Move 20% of screen height
    }

    pub fn is_expired(&self) -> bool {
        self.time_alive >= 2.0
    }

    pub fn color(&self) -> (f32, f32, f32) {
        if self.amount > 0 {
            (0.0, 1.0, 0.0) // Green for gains
        } else {
            (1.0, 0.0, 0.0) // Red for spends
        }
    }
}

/// HUD Echo system (ECS system function)
pub fn hud_echo_system(
    currency: &EchoCurrency,
    hud_state: &mut EchoHudState,
    new_transaction_amount: Option<i32>,
    delta_time: f32,
) {
    // Update balance
    hud_state.balance = currency.count();

    // Add new feedback float if transaction occurred
    if let Some(amount) = new_transaction_amount {
        hud_state.feedback_floats.push(FeedbackFloat::new(amount));
    }

    // Update existing feedback floats
    for float in hud_state.feedback_floats.iter_mut() {
        float.update(delta_time);
    }

    // Remove expired floats
    hud_state.feedback_floats.retain(|f| !f.is_expired());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_displays_balance() {
        let currency = EchoCurrency::with_balance(10);
        let mut hud_state = EchoHudState {
            balance: 0,
            feedback_floats: Vec::new(),
        };

        hud_echo_system(&currency, &mut hud_state, None, 0.0);

        assert_eq!(hud_state.balance, 10);
    }

    #[test]
    fn test_feedback_float_creation() {
        let currency = EchoCurrency::new();
        let mut hud_state = EchoHudState {
            balance: 0,
            feedback_floats: Vec::new(),
        };

        hud_echo_system(&currency, &mut hud_state, Some(5), 0.0);

        assert_eq!(hud_state.feedback_floats.len(), 1);
        assert_eq!(hud_state.feedback_floats[0].amount, 5);
    }

    #[test]
    fn test_feedback_float_fade_in() {
        let mut float = FeedbackFloat::new(3);

        // At 0.0s: alpha = 1.0 (initial state)
        assert_eq!(float.alpha, 1.0);

        // At 0.5s: alpha = 0.5 (fade in progress: 0.5 / 2.0 * 2 = 0.5)
        // Actually: progress = 0.5 / 2.0 = 0.25, which is < 0.5, so alpha = 0.25 * 2 = 0.5
        float.update(0.5);
        assert!((float.alpha - 0.5).abs() < 0.01);

        // At 1.0s: alpha = 1.0 (peak, progress = 0.5)
        float.update(0.5);
        assert!((float.alpha - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_feedback_float_fade_out() {
        let mut float = FeedbackFloat::new(3);

        // At 1.5s: alpha = 0.5
        float.update(1.5);
        assert!((float.alpha - 0.5).abs() < 0.1);

        // At 2.0s: alpha = 0.0 (expired)
        float.update(0.5);
        assert!(float.alpha < 0.1);
        assert!(float.is_expired());
    }

    #[test]
    fn test_feedback_float_position() {
        let mut float = FeedbackFloat::new(3);

        // At 0.0s: position_y = 0.0
        assert_eq!(float.position_y, 0.0);

        // At 1.0s: position_y = 0.1 (10% of screen)
        float.update(1.0);
        assert!((float.position_y - 0.1).abs() < 0.01);

        // At 2.0s: position_y = 0.2 (20% of screen)
        float.update(1.0);
        assert!((float.position_y - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_feedback_float_color_gain() {
        let float = FeedbackFloat::new(5); // Positive amount
        let (r, g, b) = float.color();

        assert_eq!(r, 0.0);
        assert_eq!(g, 1.0); // Green
        assert_eq!(b, 0.0);
    }

    #[test]
    fn test_feedback_float_color_spend() {
        let float = FeedbackFloat::new(-3); // Negative amount
        let (r, g, b) = float.color();

        assert_eq!(r, 1.0); // Red
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
    }

    #[test]
    fn test_expired_floats_removed() {
        let currency = EchoCurrency::new();
        let mut hud_state = EchoHudState {
            balance: 0,
            feedback_floats: Vec::new(),
        };

        // Add 2 floats
        hud_echo_system(&currency, &mut hud_state, Some(3), 0.0);
        hud_echo_system(&currency, &mut hud_state, Some(2), 0.0);
        assert_eq!(hud_state.feedback_floats.len(), 2);

        // Update for 2.5s (should expire both)
        hud_echo_system(&currency, &mut hud_state, None, 2.5);
        assert_eq!(hud_state.feedback_floats.len(), 0);
    }

    #[test]
    fn test_multiple_active_floats() {
        let currency = EchoCurrency::new();
        let mut hud_state = EchoHudState {
            balance: 0,
            feedback_floats: Vec::new(),
        };

        // Add float 1
        hud_echo_system(&currency, &mut hud_state, Some(3), 0.0);

        // Update 0.5s
        hud_echo_system(&currency, &mut hud_state, None, 0.5);

        // Add float 2
        hud_echo_system(&currency, &mut hud_state, Some(-2), 0.0);

        // Both should be active
        assert_eq!(hud_state.feedback_floats.len(), 2);
        assert_eq!(hud_state.feedback_floats[0].amount, 3);
        assert_eq!(hud_state.feedback_floats[1].amount, -2);
    }
}
