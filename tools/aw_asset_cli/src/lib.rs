//! AstraWeave Asset CLI library
//! 
//! This crate provides texture baking and asset processing functionality.
//! The types can be used by other crates to read texture metadata.

pub mod texture_baker;

// Re-export commonly used types
pub use texture_baker::{
    ColorSpace,
    CompressionFormat,
    NormalYConvention,
    TextureMetadata,
    BakeConfig,
};
