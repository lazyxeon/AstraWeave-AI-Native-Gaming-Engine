pub mod actions;
pub mod bindings;
pub mod manager;
pub mod save;

pub use actions::*;
pub use bindings::*;
pub use manager::*;
pub use save::*;

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::MouseButton;
    use winit::keyboard::KeyCode;

    #[test]
    fn test_keycode_serde() {
        let key = KeyCode::KeyW;
        let serialized = serde_json::to_string(&key).unwrap();
        let deserialized: KeyCode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(key, deserialized);
    }

    #[test]
    fn test_mousebutton_serde() {
        let mouse = MouseButton::Left;
        let serialized = serde_json::to_string(&mouse).unwrap();
        let deserialized: MouseButton = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mouse, deserialized);
    }

    #[test]
    fn test_binding_serde() {
        let binding = Binding {
            key: Some(KeyCode::KeyW),
            mouse: Some(MouseButton::Left),
            gamepad: Some(GamepadButton::South),
        };

        let serialized = serde_json::to_string(&binding).unwrap();
        let deserialized: Binding = serde_json::from_str(&serialized).unwrap();

        assert_eq!(binding.key, deserialized.key);
        assert_eq!(binding.mouse, deserialized.mouse);
        assert_eq!(binding.gamepad, deserialized.gamepad);
    }

    #[test]
    fn test_binding_set_serde() {
        let binding_set = BindingSet::default();
        let serialized = serde_json::to_string(&binding_set).unwrap();
        let _deserialized: BindingSet = serde_json::from_str(&serialized).unwrap();
        // If we get here without panicking, serialization works
    }
}
