//! Advanced memory management system for LLM-powered game entities
//!
//! This crate provides sophisticated memory management including:
//! - Hierarchical memory structures (sensory, working, episodic, semantic)
//! - Intelligent forgetting curves and consolidation
//! - Context-aware memory retrieval
//! - Memory compression and summarization
//! - Cross-agent memory sharing

pub mod memory_types;
pub use memory_types::*;

pub mod memory_manager;
pub use memory_manager::*;

pub mod consolidation;
pub use consolidation::*;

pub mod forgetting;
pub use forgetting::*;

pub mod compression;
pub use compression::*;

pub mod retrieval;
pub use retrieval::*;

pub mod sharing;
pub use sharing::*;

#[cfg(feature = "bevy")]
pub mod components;
#[cfg(feature = "bevy")]
pub use components::*;