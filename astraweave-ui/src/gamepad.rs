//! Gamepad/Controller support for UI navigation
//!
//! Phase 8.1 Week 5: Controller Support
//! PlayStation conventions: X=confirm, O=cancel, D-pad=navigate

use gilrs::{Button, Event, EventType, Gilrs};
use serde::{Deserialize, Serialize};

/// Gamepad input manager for UI navigation
pub struct GamepadManager {
    gilrs: Gilrs,
    /// Currently selected button index in menu
    pub selected_index: usize,
    /// Maximum items in current menu
    pub max_items: usize,
    /// Deadzone for analog sticks (0.0-1.0)
    pub deadzone: f32,
    /// Repeat delay for held buttons (seconds)
    pub repeat_delay: f32,
    /// Time since last repeat
    repeat_timer: f32,
}

/// UI action from gamepad input
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadAction {
    /// Navigate up in menu
    Up,
    /// Navigate down in menu
    Down,
    /// Navigate left (for sliders, tabs)
    Left,
    /// Navigate right (for sliders, tabs)
    Right,
    /// Confirm selection (PlayStation X button)
    Confirm,
    /// Cancel/back (PlayStation O button)
    Cancel,
    /// Open pause menu (Options button)
    Pause,
    /// No action
    None,
}

/// Gamepad button mappings (stored as strings for serialization)
/// Maps to gilrs::Button at runtime
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GamepadBindings {
    /// Button name for confirm action
    pub confirm: String,
    /// Button name for cancel action  
    pub cancel: String,
    /// Button name for pause action
    pub pause: String,
}

impl Default for GamepadBindings {
    fn default() -> Self {
        Self {
            // PlayStation conventions
            confirm: "South".to_string(), // X on PlayStation, A on Xbox
            cancel: "East".to_string(),   // O on PlayStation, B on Xbox
            pause: "Start".to_string(),   // Options on PlayStation
        }
    }
}

impl GamepadBindings {
    /// Convert binding name to gilrs Button
    pub fn to_button(name: &str) -> Option<Button> {
        match name {
            "South" => Some(Button::South),
            "East" => Some(Button::East),
            "North" => Some(Button::North),
            "West" => Some(Button::West),
            "Start" => Some(Button::Start),
            "Select" => Some(Button::Select),
            "DPadUp" => Some(Button::DPadUp),
            "DPadDown" => Some(Button::DPadDown),
            "DPadLeft" => Some(Button::DPadLeft),
            "DPadRight" => Some(Button::DPadRight),
            "LeftTrigger" => Some(Button::LeftTrigger),
            "RightTrigger" => Some(Button::RightTrigger),
            "LeftTrigger2" => Some(Button::LeftTrigger2),
            "RightTrigger2" => Some(Button::RightTrigger2),
            _ => None,
        }
    }

    /// Get confirm button as gilrs Button
    pub fn confirm_button(&self) -> Option<Button> {
        Self::to_button(&self.confirm)
    }

    /// Get cancel button as gilrs Button
    pub fn cancel_button(&self) -> Option<Button> {
        Self::to_button(&self.cancel)
    }

    /// Get pause button as gilrs Button
    pub fn pause_button(&self) -> Option<Button> {
        Self::to_button(&self.pause)
    }
}

impl GamepadManager {
    /// Create a new gamepad manager
    pub fn new() -> Result<Self, gilrs::Error> {
        Ok(Self {
            gilrs: Gilrs::new()?,
            selected_index: 0,
            max_items: 1,
            deadzone: 0.3,
            repeat_delay: 0.2,
            repeat_timer: 0.0,
        })
    }

    /// Poll for gamepad events and return UI action
    pub fn poll(&mut self, dt: f32) -> GamepadAction {
        self.repeat_timer += dt;

        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::ButtonPressed(button, _) => {
                    return self.map_button(button);
                }
                EventType::ButtonRepeated(button, _) => {
                    if self.repeat_timer >= self.repeat_delay {
                        self.repeat_timer = 0.0;
                        return self.map_button(button);
                    }
                }
                EventType::AxisChanged(axis, value, _) => {
                    if value.abs() > self.deadzone {
                        return self.map_axis(axis, value);
                    }
                }
                _ => {}
            }
        }

        GamepadAction::None
    }

    /// Map button press to UI action (PlayStation conventions)
    fn map_button(&self, button: Button) -> GamepadAction {
        match button {
            // D-pad navigation
            Button::DPadUp => GamepadAction::Up,
            Button::DPadDown => GamepadAction::Down,
            Button::DPadLeft => GamepadAction::Left,
            Button::DPadRight => GamepadAction::Right,
            // PlayStation button mapping
            Button::South => GamepadAction::Confirm, // X button
            Button::East => GamepadAction::Cancel,   // O button
            Button::Start => GamepadAction::Pause,   // Options button
            _ => GamepadAction::None,
        }
    }

    /// Map analog stick to D-pad equivalent
    fn map_axis(&mut self, axis: gilrs::Axis, value: f32) -> GamepadAction {
        use gilrs::Axis;
        match axis {
            Axis::LeftStickY if value > self.deadzone => GamepadAction::Up,
            Axis::LeftStickY if value < -self.deadzone => GamepadAction::Down,
            Axis::LeftStickX if value < -self.deadzone => GamepadAction::Left,
            Axis::LeftStickX if value > self.deadzone => GamepadAction::Right,
            _ => GamepadAction::None,
        }
    }

    /// Navigate selection up
    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.max_items.saturating_sub(1); // Wrap to bottom
        }
    }

    /// Navigate selection down
    pub fn navigate_down(&mut self) {
        if self.selected_index < self.max_items.saturating_sub(1) {
            self.selected_index += 1;
        } else {
            self.selected_index = 0; // Wrap to top
        }
    }

    /// Set menu item count
    pub fn set_menu_size(&mut self, count: usize) {
        self.max_items = count;
        if self.selected_index >= count {
            self.selected_index = count.saturating_sub(1);
        }
    }

    /// Check if a gamepad is connected
    pub fn is_connected(&self) -> bool {
        self.gilrs.gamepads().count() > 0
    }

    /// Get connected gamepad count
    pub fn gamepad_count(&self) -> usize {
        self.gilrs.gamepads().count()
    }
}

impl Default for GamepadManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            gilrs: Gilrs::new().expect("Failed to initialize gilrs"),
            selected_index: 0,
            max_items: 1,
            deadzone: 0.3,
            repeat_delay: 0.2,
            repeat_timer: 0.0,
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamepad_bindings_default() {
        let bindings = GamepadBindings::default();
        assert_eq!(bindings.confirm, "South");
        assert_eq!(bindings.cancel, "East");
        assert_eq!(bindings.pause, "Start");
    }

    #[test]
    fn test_button_conversion() {
        assert_eq!(GamepadBindings::to_button("South"), Some(Button::South));
        assert_eq!(GamepadBindings::to_button("East"), Some(Button::East));
        assert_eq!(GamepadBindings::to_button("Start"), Some(Button::Start));
        assert_eq!(GamepadBindings::to_button("Invalid"), None);
    }

    #[test]
    fn test_navigate_up() {
        let mut manager = GamepadManager {
            gilrs: Gilrs::new().expect("gilrs init"),
            selected_index: 2,
            max_items: 4,
            deadzone: 0.3,
            repeat_delay: 0.2,
            repeat_timer: 0.0,
        };

        manager.navigate_up();
        assert_eq!(manager.selected_index, 1);

        manager.navigate_up();
        assert_eq!(manager.selected_index, 0);

        // Wrap to bottom
        manager.navigate_up();
        assert_eq!(manager.selected_index, 3);
    }

    #[test]
    fn test_navigate_down() {
        let mut manager = GamepadManager {
            gilrs: Gilrs::new().expect("gilrs init"),
            selected_index: 2,
            max_items: 4,
            deadzone: 0.3,
            repeat_delay: 0.2,
            repeat_timer: 0.0,
        };

        manager.navigate_down();
        assert_eq!(manager.selected_index, 3);

        // Wrap to top
        manager.navigate_down();
        assert_eq!(manager.selected_index, 0);
    }

    #[test]
    fn test_set_menu_size() {
        let mut manager = GamepadManager {
            gilrs: Gilrs::new().expect("gilrs init"),
            selected_index: 5,
            max_items: 10,
            deadzone: 0.3,
            repeat_delay: 0.2,
            repeat_timer: 0.0,
        };

        // Shrink menu - should clamp selected index
        manager.set_menu_size(3);
        assert_eq!(manager.max_items, 3);
        assert_eq!(manager.selected_index, 2);
    }

    #[test]
    fn test_gamepad_action_enum() {
        assert_ne!(GamepadAction::Confirm, GamepadAction::Cancel);
        assert_eq!(GamepadAction::None, GamepadAction::None);
    }

    #[test]
    fn test_button_mapping() {
        let manager = GamepadManager {
            gilrs: Gilrs::new().expect("gilrs init"),
            selected_index: 0,
            max_items: 4,
            deadzone: 0.3,
            repeat_delay: 0.2,
            repeat_timer: 0.0,
        };

        assert_eq!(manager.map_button(Button::South), GamepadAction::Confirm);
        assert_eq!(manager.map_button(Button::East), GamepadAction::Cancel);
        assert_eq!(manager.map_button(Button::DPadUp), GamepadAction::Up);
        assert_eq!(manager.map_button(Button::DPadDown), GamepadAction::Down);
        assert_eq!(manager.map_button(Button::Start), GamepadAction::Pause);
    }
}
