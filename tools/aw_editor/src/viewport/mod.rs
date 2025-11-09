//! 3D Viewport Module
//!
//! Professional 3D scene viewport for the AstraWeave editor. Provides:
//! - wgpu rendering integration with egui
//! - Orbit camera controls (Unity/Blender-style)
//! - Entity rendering with selection
//! - Visual gizmo manipulation
//! - Grid overlay and debug visualization
//!
//! # Architecture
//!
//! ```text
//! ViewportWidget (egui integration)
//!     ↓
//! ViewportRenderer (rendering coordinator)
//!     ↓
//! ├─ GridRenderer (floor grid + axes)
//! ├─ EntityRenderer (world entities)
//! └─ GizmoRenderer (transform handles)
//! ```
//!
//! # Usage
//!
//! ```no_run
//! use aw_editor_lib::viewport::ViewportWidget;
//!
//! // In eframe::App::new()
//! let viewport = ViewportWidget::new(cc)?;
//!
//! // In eframe::App::update()
//! viewport.ui(ui, &world)?;
//! ```

mod camera;
mod entity_renderer;
mod gizmo_renderer;
mod grid_renderer;
mod renderer;
mod skybox_renderer;
mod toolbar;
mod widget;

pub use widget::ViewportWidget;
