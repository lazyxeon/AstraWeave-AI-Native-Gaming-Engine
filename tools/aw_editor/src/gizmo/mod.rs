//! Blender-style 3D transform gizmos for AstraWeave editor.
//!
//! This module provides modal transform tools with keyboard shortcuts:
//! - **G**: Translate (move)
//! - **R**: Rotate
//! - **S**: Scale
//! - **X/Y/Z**: Constrain to axis
//! - **Shift+X/Y/Z**: Constrain to plane
//! - **Escape**: Cancel transform
//! - **Enter**: Confirm transform
//! - **Numeric input**: Type values (e.g., "5.2" → move 5.2 units)
//!
//! Inspired by Blender's workflow for fast, keyboard-driven 3D editing.

pub mod constraints;
pub mod input;
pub mod picking;
pub mod rendering;
pub mod rotate;
pub mod scale;
pub mod scene_viewport;
pub mod state;
pub mod translate;

pub use constraints::apply_constraint;
pub use input::NumericInput;
pub use picking::{GizmoHandle, GizmoPicker, Ray};
pub use rendering::{GizmoRenderParams, GizmoRenderer};
pub use rotate::RotateGizmo;
pub use scale::ScaleGizmo;
pub use scene_viewport::{CameraController, SceneViewport, Transform};
pub use state::{AxisConstraint, GizmoMode, GizmoState, TransformSnapshot};
pub use translate::TranslateGizmo;
