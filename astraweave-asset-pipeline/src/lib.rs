//! Asset Pipeline - Texture compression and mesh optimization for AstraWeave
//!
//! This crate provides production-ready asset processing:
//! - **Texture Compression**: BC7 (desktop), ASTC (mobile)
//! - **Mesh Optimization**: Vertex cache, overdraw reduction
//! - **Validation**: Quality checks, size verification
//!
//! ## Features
//! - `bc7`: BC7 texture compression (default, desktop)
//! - `astc`: ASTC texture compression (mobile)
//!
//! ## Example
//! ```no_run
//! use astraweave_asset_pipeline::texture::compress_bc7;
//! use image::RgbaImage;
//!
//! # fn example() -> anyhow::Result<()> {
//! let rgba_image = image::open("texture.png")?.to_rgba8();
//! let compressed = compress_bc7(&rgba_image)?;
//!
//! println!("Compressed from {} to {} bytes ({:.1}% reduction)",
//!     rgba_image.len(),
//!     compressed.len(),
//!     100.0 * (1.0 - compressed.len() as f32 / rgba_image.len() as f32)
//! );
//! # Ok(())
//! # }
//! ```

pub mod mesh;
pub mod texture;
pub mod validator;

pub use mesh::{optimize_mesh, MeshOptimizationStats};
pub use texture::{compress_bc7, CompressionStats};
pub use validator::{AssetValidator, ValidationReport};

/// Re-export meshopt for direct access
pub use meshopt;
