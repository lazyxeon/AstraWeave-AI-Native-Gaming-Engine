// Library exports for aw_editor (enables benchmarks and testing)
#![allow(dead_code)] // Suppress dead code warnings for benchmark-only exports

pub mod clipboard;
pub mod command;
pub mod component_ui;
pub mod editor_mode;
pub mod entity_manager;
pub mod headless;
pub mod interaction;
pub mod prefab;
pub mod scene_serialization;
pub mod scene_state;
pub mod telemetry;
pub mod ui;

pub use command::{
    EditorCommand, MoveEntityCommand, RotateEntityCommand, ScaleEntityCommand, UndoStack,
};
pub use editor_mode::EditorMode;
pub use entity_manager::{EditorEntity, EntityId, EntityManager, SelectionSet};
pub use prefab::{PrefabData, PrefabInstance, PrefabManager};
pub use scene_serialization::{EntityData, SceneData};
pub use scene_state::{EditorSceneState, TransformableScene};
pub use ui::StatusBar;

pub mod gizmo {
    // Export all modules
    pub mod constraints;
    pub mod input;
    pub mod picking;
    pub mod rendering;
    pub mod rotate;
    pub mod scale;
    pub mod scene_viewport;
    pub mod snapping;
    pub mod state;
    pub mod translate;

    // Re-export commonly used types for convenience
    pub use constraints::apply_constraint;
    pub use input::NumericInput;
    pub use picking::{GizmoHandle, GizmoPicker, Ray};
    pub use rendering::{GizmoRenderParams, GizmoRenderer};
    pub use rotate::RotateGizmo;
    pub use scale::ScaleGizmo;
    pub use scene_viewport::{CameraController, SceneViewport, Transform};
    pub use snapping::SnappingConfig;
    pub use state::{AxisConstraint, GizmoMode, GizmoState, TransformSnapshot};
    pub use translate::TranslateGizmo;
}
