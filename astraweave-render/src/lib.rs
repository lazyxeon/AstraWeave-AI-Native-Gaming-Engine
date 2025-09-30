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
pub mod ibl; // image-based lighting manager
pub mod mesh; // cpu mesh structures + utils
pub mod mesh_registry; // gpu upload & caching
pub mod material; // shared authored materials API + GPU arrays
pub mod material_loader; // internal builder helpers
#[cfg(any(feature = "gltf-assets", feature = "assets"))]
pub mod mesh_gltf; // glTF loader
#[cfg(any(feature = "obj-assets", feature = "assets"))]
pub mod mesh_obj; // OBJ fallback loader

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
pub use ibl::{IblManager, IblQuality, IblResources, SkyMode};
pub use mesh::{CpuMesh, MeshVertex, MeshVertexLayout};
pub use mesh_registry::{MeshHandle, MeshKey, MeshRegistry};
pub use material::{MaterialManager, MaterialGpuArrays, MaterialLoadStats, MaterialLayerDesc, MaterialPackDesc, ArrayLayout};
