pub mod camera;
pub mod clustered;
pub mod depth;
pub mod environment;
pub mod post; // compile-only WGSL placeholders & tests
pub mod primitives;
pub mod renderer;
pub mod terrain;
pub mod texture;
pub mod types; // clustered-lighting WGSL placeholders & tests

pub use camera::{Camera, CameraController};
pub use environment::{
    SkyConfig, SkyRenderer, TimeOfDay, WeatherParticles, WeatherSystem, WeatherType,
};
pub use renderer::Renderer;
pub use terrain::{TerrainMesh, TerrainRenderer, TerrainVertex, VegetationRenderInstance};
pub use texture::Texture;
pub use types::{Instance, Material, SkinnedVertex};

pub mod effects; // NEW
pub mod overlay; // NEW (for cutscene fades/letterbox later)

pub use effects::{WeatherFx, WeatherKind};
pub use overlay::{OverlayFx, OverlayParams};
