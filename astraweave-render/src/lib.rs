pub mod camera;
pub mod depth;
pub mod primitives;
pub mod renderer;
pub mod types;

pub use camera::{Camera, CameraController};
pub use renderer::{Renderer, Instance, Material};

pub mod effects;        // NEW
pub mod overlay;        // NEW (for cutscene fades/letterbox later)

pub use effects::{WeatherFx, WeatherKind};
pub use overlay::{OverlayFx, OverlayParams};
