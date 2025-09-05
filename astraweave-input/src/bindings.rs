use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use winit::keyboard::KeyCode;
use winit::event::MouseButton;

use crate::{Action};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South, East, West, North, // A/B/X/Y (Xbox)
    L1, R1, L2, R2,
    Select, Start,
    LStick, RStick,
    DPadUp, DPadDown, DPadLeft, DPadRight,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AxisKind { LeftX, LeftY, RightX, RightY, LT, RT }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Binding {
    pub key: Option<KeyCode>,
    pub mouse: Option<MouseButton>,
    pub gamepad: Option<GamepadButton>,
}

impl Default for Binding {
    fn default() -> Self {
        Self { key: None, mouse: None, gamepad: None }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AxisBinding {
    pub axis: AxisKind,
    pub invert: bool,
    pub deadzone: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BindingSet {
    pub actions: HashMap<Action, Binding>,
    pub move_axes: (AxisBinding, AxisBinding),  // (LeftX, LeftY)
    pub look_axes: (AxisBinding, AxisBinding),  // (RightX, RightY)
}

impl Default for BindingSet {
    fn default() -> Self {
        use Action::*;
        let mut actions = HashMap::new();
        // Keyboard defaults
        actions.insert(MoveForward,  Binding { key: Some(KeyCode::KeyW), ..Default::default() });
        actions.insert(MoveBackward, Binding { key: Some(KeyCode::KeyS), ..Default::default() });
        actions.insert(MoveLeft,     Binding { key: Some(KeyCode::KeyA), ..Default::default() });
        actions.insert(MoveRight,    Binding { key: Some(KeyCode::KeyD), ..Default::default() });
        actions.insert(Jump,         Binding { key: Some(KeyCode::Space), ..Default::default() });
        actions.insert(Crouch,       Binding { key: Some(KeyCode::ControlLeft), ..Default::default() });
        actions.insert(Sprint,       Binding { key: Some(KeyCode::ShiftLeft), ..Default::default() });
        actions.insert(Interact,     Binding { key: Some(KeyCode::KeyE), ..Default::default() });
        actions.insert(AttackLight,  Binding { mouse: Some(MouseButton::Left), ..Default::default() });
        actions.insert(AttackHeavy,  Binding { mouse: Some(MouseButton::Right), ..Default::default() });

        actions.insert(OpenInventory, Binding { key: Some(KeyCode::KeyI), ..Default::default() });
        actions.insert(OpenMap,       Binding { key: Some(KeyCode::KeyM), ..Default::default() });
        actions.insert(OpenQuests,    Binding { key: Some(KeyCode::KeyJ), ..Default::default() });
        actions.insert(OpenCrafting,  Binding { key: Some(KeyCode::KeyC), ..Default::default() });
        actions.insert(OpenMenu,      Binding { key: Some(KeyCode::Escape), ..Default::default() });

        // UI nav defaults
        actions.insert(UiAccept, Binding { key: Some(KeyCode::Enter), ..Default::default() });
        actions.insert(UiBack,   Binding { key: Some(KeyCode::Escape), ..Default::default() });
        actions.insert(UiUp,     Binding { key: Some(KeyCode::ArrowUp), ..Default::default() });
        actions.insert(UiDown,   Binding { key: Some(KeyCode::ArrowDown), ..Default::default() });
        actions.insert(UiLeft,   Binding { key: Some(KeyCode::ArrowLeft), ..Default::default() });
        actions.insert(UiRight,  Binding { key: Some(KeyCode::ArrowRight), ..Default::default() });

        Self {
            actions,
            move_axes: (
                AxisBinding{ axis: AxisKind::LeftX, invert:false, deadzone:0.15 },
                AxisBinding{ axis: AxisKind::LeftY, invert:true,  deadzone:0.15 },
            ),
            look_axes: (
                AxisBinding{ axis: AxisKind::RightX, invert:false, deadzone:0.12 },
                AxisBinding{ axis: AxisKind::RightY, invert:true,  deadzone:0.12 },
            ),
        }
    }
}
