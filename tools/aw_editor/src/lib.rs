// Library exports for aw_editor (enables benchmarks and testing)
#![allow(dead_code)] // Suppress dead code warnings for benchmark-only exports

pub mod gizmo {
    // Export all modules
    pub mod constraints;
    pub mod input;
    pub mod picking;
    pub mod rendering;
    pub mod rotate;
    pub mod scale;
    pub mod scene_viewport;
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
    pub use state::{AxisConstraint, GizmoMode, GizmoState, TransformSnapshot};
    pub use translate::TranslateGizmo;
}
