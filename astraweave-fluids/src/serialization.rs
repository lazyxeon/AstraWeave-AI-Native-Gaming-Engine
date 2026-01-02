//! Fluid State Serialization
//!
//! Provides save/load functionality for fluid simulation snapshots.

use serde::{Deserialize, Serialize};

/// A serializable snapshot of the fluid simulation state.
#[derive(Clone, Serialize, Deserialize)]
pub struct FluidSnapshot {
    /// Version for forward compatibility
    pub version: u32,
    /// Particle positions (xyz + w=1.0)
    pub positions: Vec<[f32; 4]>,
    /// Particle velocities (xyz + w=0.0)
    pub velocities: Vec<[f32; 4]>,
    /// Particle colors (RGBA)
    pub colors: Vec<[f32; 4]>,
    /// Simulation parameters
    pub params: SnapshotParams,
    /// Frame index at time of snapshot
    pub frame_index: usize,
    /// Active particle count
    pub active_count: u32,
}

/// Serializable simulation parameters
#[derive(Clone, Serialize, Deserialize)]
pub struct SnapshotParams {
    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub viscosity: f32,
    pub surface_tension: f32,
    pub gravity: f32,
    pub iterations: u32,
    pub cell_size: f32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub grid_depth: u32,
}

impl FluidSnapshot {
    /// Current snapshot format version
    pub const VERSION: u32 = 1;

    /// Create an empty snapshot with the given capacity
    pub fn with_capacity(particle_count: usize) -> Self {
        Self {
            version: Self::VERSION,
            positions: Vec::with_capacity(particle_count),
            velocities: Vec::with_capacity(particle_count),
            colors: Vec::with_capacity(particle_count),
            params: SnapshotParams::default(),
            frame_index: 0,
            active_count: 0,
        }
    }

    /// Serialize to bytes using bincode
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

impl Default for SnapshotParams {
    fn default() -> Self {
        Self {
            smoothing_radius: 1.0,
            target_density: 12.0,
            pressure_multiplier: 300.0,
            viscosity: 10.0,
            surface_tension: 0.02,
            gravity: -9.8,
            iterations: 4,
            cell_size: 1.2,
            grid_width: 128,
            grid_height: 128,
            grid_depth: 128,
        }
    }
}
