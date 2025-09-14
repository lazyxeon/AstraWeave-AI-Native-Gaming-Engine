pub mod camera;
pub mod depth;
pub mod primitives;
pub mod renderer;
pub mod types;
pub mod texture;
pub mod terrain;
pub mod environment;

pub use camera::{Camera, CameraController};
pub use renderer::Renderer;
pub use types::{Instance, Material};
pub use texture::Texture;
pub use terrain::{TerrainRenderer, TerrainMesh, TerrainVertex, VegetationRenderInstance};
pub use environment::{TimeOfDay, SkyRenderer, SkyConfig, WeatherSystem, WeatherType, WeatherParticles};

pub mod effects; // NEW
pub mod overlay; // NEW (for cutscene fades/letterbox later)

pub use effects::{WeatherFx, WeatherKind};
pub use overlay::{OverlayFx, OverlayParams};
