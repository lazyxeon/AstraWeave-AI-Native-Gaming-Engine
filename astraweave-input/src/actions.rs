use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputContext {
    Gameplay,
    UI,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Action {
    // Movement / Camera
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Crouch,
    Sprint,
    Interact,
    AttackLight,
    AttackHeavy,
    Ability1,
    Ability2,

    // UI toggles
    OpenInventory,
    OpenMap,
    OpenQuests,
    OpenCrafting,
    OpenMenu,

    // UI navigation (for controller)
    UiAccept,
    UiBack,
    UiUp,
    UiDown,
    UiLeft,
    UiRight,
}

#[derive(Default, Clone, Debug)]
pub struct Axis2 {
    pub x: f32,
    pub y: f32,
}
