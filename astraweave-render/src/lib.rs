pub mod camera;
pub mod clustered;
pub mod clustered_forward; // Complete clustered forward rendering
pub mod depth;
pub mod environment;
pub mod gi; // Global Illumination (VXGI)
pub mod ibl; // image-based lighting manager
pub mod mesh; // cpu mesh structures + utils
pub mod mesh_registry;
pub mod post; // compile-only WGSL placeholders & tests
pub mod primitives;
pub mod renderer;
pub mod terrain;
pub mod texture;
pub mod types; // clustered-lighting WGSL placeholders & tests // gpu upload & caching
               // See MATERIALS.md for canonical materials arrays and WGSL bindings
pub mod animation;
pub mod culling; // GPU-driven frustum culling (Phase 2 Task 3)
pub mod culling_node; // Culling node for render graph
pub mod graph; // minimal render graph scaffolding (Phase 2)
pub mod graph_adapter; // runs a graph on Renderer frames
pub mod material; // shared authored materials API + GPU arrays
pub mod material_extended; // Phase PBR-E: Advanced materials (clearcoat, anisotropy, SSS, sheen, transmission)
pub mod terrain_material; // Phase PBR-F: Terrain layering with splat maps and triplanar projection
#[cfg(feature = "textures")]
pub mod material_loader; // internal builder helpers
#[cfg(any(feature = "gltf-assets", feature = "assets"))]
pub mod mesh_gltf; // glTF loader
#[cfg(any(feature = "obj-assets", feature = "assets"))]
pub mod mesh_obj;
pub mod residency; // asset streaming and residency management // OBJ fallback loader // Phase 2 Task 5: Skeletal animation with CPU/GPU skinning

#[cfg(feature = "skinning-gpu")]
pub mod skinning_gpu; // Phase 2 Task 5 Phase D: GPU skinning pipeline

// Nanite virtualized geometry system
#[cfg(feature = "nanite")]
pub mod nanite_gpu_culling;
#[cfg(feature = "nanite")]
pub mod nanite_render;
#[cfg(feature = "nanite")]
pub mod nanite_visibility; // NEW: GPU-driven culling and visibility

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

pub use culling::{
    batch_visible_instances, build_indirect_commands_cpu, cpu_frustum_cull, BatchId,
    CullingPipeline, CullingResources, DrawBatch, DrawIndirectCommand, FrustumPlanes, InstanceAABB,
};
pub use culling_node::CullingNode;
pub use effects::{WeatherFx, WeatherKind};
pub use ibl::{IblManager, IblQuality, IblResources, SkyMode};
pub use material::{
    ArrayLayout, MaterialGpu, MaterialGpuArrays, MaterialLayerDesc, MaterialLoadStats,
    MaterialManager, MaterialPackDesc,
};
pub use material_extended::{
    MaterialDefinitionExtended, MaterialGpuExtended, MATERIAL_FLAG_ANISOTROPY,
    MATERIAL_FLAG_CLEARCOAT, MATERIAL_FLAG_SHEEN, MATERIAL_FLAG_SUBSURFACE,
    MATERIAL_FLAG_TRANSMISSION,
};
pub use terrain_material::{
    TerrainLayerDesc, TerrainLayerGpu, TerrainMaterialDesc, TerrainMaterialGpu,
};
pub use mesh::{CpuMesh, MeshVertex, MeshVertexLayout};
pub use mesh_registry::{MeshHandle, MeshKey, MeshRegistry};
#[cfg(feature = "bloom")]
pub use post::{BloomConfig, BloomPipeline};
pub use residency::ResidencyManager;

// Phase 2 Task 5: Skeletal Animation exports
pub use animation::{
    compute_joint_matrices, skin_vertex_cpu, AnimationChannel, AnimationClip, AnimationState,
    ChannelData, Interpolation, Joint, JointMatrixGPU, JointPalette, Skeleton, Transform,
    MAX_JOINTS,
};

#[cfg(feature = "skinning-gpu")]
pub use skinning_gpu::{JointPaletteHandle, JointPaletteManager, SKINNING_GPU_SHADER};
