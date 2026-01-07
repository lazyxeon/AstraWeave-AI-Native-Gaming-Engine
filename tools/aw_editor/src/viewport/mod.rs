//! 3D Viewport Module
//!
//! Professional 3D scene viewport for the AstraWeave editor. Provides:
//! - wgpu rendering integration with egui
//! - Orbit camera controls (Unity/Blender-style)
//! - Entity rendering with selection
//! - Visual gizmo manipulation
//! - Grid overlay and debug visualization
//! - Physics debug rendering (collider wireframes)
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
//! ├─ GizmoRenderer (transform handles)
//! └─ PhysicsDebugRenderer (collider wireframes)
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

pub mod camera;
#[cfg(feature = "astraweave-render")]
mod engine_adapter;
mod entity_renderer;
mod gizmo_renderer;
mod grid_renderer;
mod physics_renderer;
mod renderer;
mod skybox_renderer;
pub mod terrain_renderer;
mod toolbar;
mod widget;

// Physics debug types are exported for external configuration
#[allow(unused_imports)]
pub use physics_renderer::{PhysicsDebugOptions, PhysicsDebugRenderer};
#[allow(unused_imports)]
pub use terrain_renderer::{TerrainRenderer, TerrainVertex};
pub use widget::ViewportWidget;

