// AstraWeave Render Bevy - Phase 1: Foundation
// Extracted from Bevy Engine v0.14.0 (MIT OR Apache-2.0)
// Adapted for AstraWeave ECS integration

//! Bevy-based renderer for AstraWeave
//!
//! This crate provides a production-ready PBR renderer by leveraging
//! Bevy's battle-tested rendering pipeline while maintaining compatibility
//! with AstraWeave's custom ECS.
//!
//! ## Architecture
//!
//! ```text
//! AstraWeave ECS → RenderAdapter → Bevy Render Pipeline → GPU
//!                      ↓
//!              Component Extraction
//!              (Transform, Mesh, Material, Light)
//! ```
//!
//! ## Features
//!
//! - **CSM Shadows**: 4-cascade directional light shadows (proven quality)
//! - **PBR Materials**: Albedo, normal, metallic-roughness workflow
//! - **Lighting**: Directional, point, spot lights with shadows
//! - **IBL**: Image-based lighting (diffuse + specular)
//! - **Post-FX**: Bloom, tonemapping (ACES, Reinhard, AgX)
//!
//! ## Phase 1 (Days 1-5)
//!
//! Foundation rendering with professional quality:
//! - Day 1: Extract Bevy PBR core ✅
//! - Day 2: Build ECS adapter
//! - Day 3: CSM + materials integration
//! - Day 4: Lighting + post-processing
//! - Day 5: Validation + documentation

#![warn(missing_docs)]

pub mod adapter;
pub mod render;

// Re-exports for convenience
pub use adapter::{
    DirectionalLight, ExtractionStats, PointLight, RenderAdapter, RenderExtractError,
    RenderMaterial, RenderMesh, RenderTransform, SpotLight,
};
pub use render::{
    shadow::{
        CascadeShadowConfig, ShadowCascade, ShadowRenderer, CASCADE_COUNT, CASCADE_RESOLUTION,
    },
    BevyRenderer, RenderConfig, Tonemapping,
};

/// Bevy renderer version
pub const VERSION: &str = "0.1.0";

/// Bevy source version this was extracted from
pub const BEVY_VERSION: &str = "0.14.0";

/// Phase 1 completion status
pub const PHASE_1_STATUS: &str =
    "Day 4: Shadow Demo Working (CSM Cascades Validated, Window Rendering)";
