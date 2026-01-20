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

#[cfg(test)]
mod tests {
    use super::*;

    // ================== FluidSnapshot Tests ==================

    #[test]
    fn test_fluid_snapshot_version_constant() {
        assert_eq!(FluidSnapshot::VERSION, 1);
    }

    #[test]
    fn test_fluid_snapshot_with_capacity() {
        let snapshot = FluidSnapshot::with_capacity(1000);
        
        assert_eq!(snapshot.version, FluidSnapshot::VERSION);
        assert_eq!(snapshot.positions.capacity(), 1000);
        assert_eq!(snapshot.velocities.capacity(), 1000);
        assert_eq!(snapshot.colors.capacity(), 1000);
        assert!(snapshot.positions.is_empty());
        assert!(snapshot.velocities.is_empty());
        assert!(snapshot.colors.is_empty());
        assert_eq!(snapshot.frame_index, 0);
        assert_eq!(snapshot.active_count, 0);
    }

    #[test]
    fn test_fluid_snapshot_with_zero_capacity() {
        let snapshot = FluidSnapshot::with_capacity(0);
        
        assert_eq!(snapshot.version, FluidSnapshot::VERSION);
        assert!(snapshot.positions.is_empty());
        assert!(snapshot.velocities.is_empty());
        assert!(snapshot.colors.is_empty());
    }

    #[test]
    fn test_fluid_snapshot_with_large_capacity() {
        let snapshot = FluidSnapshot::with_capacity(100_000);
        
        assert_eq!(snapshot.positions.capacity(), 100_000);
    }

    #[test]
    fn test_fluid_snapshot_to_bytes_empty() {
        let snapshot = FluidSnapshot::with_capacity(0);
        
        let bytes = snapshot.to_bytes();
        assert!(bytes.is_ok());
        assert!(!bytes.unwrap().is_empty());
    }

    #[test]
    fn test_fluid_snapshot_roundtrip() {
        let mut snapshot = FluidSnapshot::with_capacity(3);
        snapshot.positions.push([1.0, 2.0, 3.0, 1.0]);
        snapshot.positions.push([4.0, 5.0, 6.0, 1.0]);
        snapshot.positions.push([7.0, 8.0, 9.0, 1.0]);
        snapshot.velocities.push([0.1, 0.2, 0.3, 0.0]);
        snapshot.velocities.push([0.4, 0.5, 0.6, 0.0]);
        snapshot.velocities.push([0.7, 0.8, 0.9, 0.0]);
        snapshot.colors.push([1.0, 0.0, 0.0, 1.0]);
        snapshot.colors.push([0.0, 1.0, 0.0, 1.0]);
        snapshot.colors.push([0.0, 0.0, 1.0, 1.0]);
        snapshot.frame_index = 42;
        snapshot.active_count = 3;
        
        let bytes = snapshot.to_bytes().expect("Serialization failed");
        let recovered = FluidSnapshot::from_bytes(&bytes).expect("Deserialization failed");
        
        assert_eq!(recovered.version, snapshot.version);
        assert_eq!(recovered.positions.len(), 3);
        assert_eq!(recovered.velocities.len(), 3);
        assert_eq!(recovered.colors.len(), 3);
        assert_eq!(recovered.frame_index, 42);
        assert_eq!(recovered.active_count, 3);
        
        // Verify positions
        assert_eq!(recovered.positions[0], [1.0, 2.0, 3.0, 1.0]);
        assert_eq!(recovered.positions[1], [4.0, 5.0, 6.0, 1.0]);
        assert_eq!(recovered.positions[2], [7.0, 8.0, 9.0, 1.0]);
        
        // Verify velocities
        assert_eq!(recovered.velocities[0], [0.1, 0.2, 0.3, 0.0]);
        
        // Verify colors
        assert_eq!(recovered.colors[0], [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_fluid_snapshot_from_bytes_invalid() {
        let invalid_bytes = vec![0, 1, 2, 3, 4];
        let result = FluidSnapshot::from_bytes(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_fluid_snapshot_from_bytes_empty() {
        let empty_bytes: Vec<u8> = vec![];
        let result = FluidSnapshot::from_bytes(&empty_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_fluid_snapshot_large_particle_count() {
        let mut snapshot = FluidSnapshot::with_capacity(1000);
        for i in 0..1000 {
            let f = i as f32;
            snapshot.positions.push([f, f + 1.0, f + 2.0, 1.0]);
            snapshot.velocities.push([0.0, 0.0, 0.0, 0.0]);
            snapshot.colors.push([1.0, 1.0, 1.0, 1.0]);
        }
        snapshot.active_count = 1000;
        snapshot.frame_index = 100;
        
        let bytes = snapshot.to_bytes().expect("Serialization failed");
        let recovered = FluidSnapshot::from_bytes(&bytes).expect("Deserialization failed");
        
        assert_eq!(recovered.positions.len(), 1000);
        assert_eq!(recovered.active_count, 1000);
        assert_eq!(recovered.frame_index, 100);
    }

    #[test]
    fn test_fluid_snapshot_params_preserved() {
        let mut snapshot = FluidSnapshot::with_capacity(0);
        snapshot.params.smoothing_radius = 2.5;
        snapshot.params.target_density = 25.0;
        snapshot.params.viscosity = 100.0;
        snapshot.params.gravity = -15.0;
        
        let bytes = snapshot.to_bytes().expect("Serialization failed");
        let recovered = FluidSnapshot::from_bytes(&bytes).expect("Deserialization failed");
        
        assert_eq!(recovered.params.smoothing_radius, 2.5);
        assert_eq!(recovered.params.target_density, 25.0);
        assert_eq!(recovered.params.viscosity, 100.0);
        assert_eq!(recovered.params.gravity, -15.0);
    }

    #[test]
    fn test_fluid_snapshot_clone() {
        let mut snapshot = FluidSnapshot::with_capacity(2);
        snapshot.positions.push([1.0, 2.0, 3.0, 1.0]);
        snapshot.frame_index = 50;
        
        let cloned = snapshot.clone();
        
        assert_eq!(cloned.positions.len(), 1);
        assert_eq!(cloned.positions[0], [1.0, 2.0, 3.0, 1.0]);
        assert_eq!(cloned.frame_index, 50);
    }

    // ================== SnapshotParams Tests ==================

    #[test]
    fn test_snapshot_params_default() {
        let params = SnapshotParams::default();
        
        assert_eq!(params.smoothing_radius, 1.0);
        assert_eq!(params.target_density, 12.0);
        assert_eq!(params.pressure_multiplier, 300.0);
        assert_eq!(params.viscosity, 10.0);
        assert_eq!(params.surface_tension, 0.02);
        assert_eq!(params.gravity, -9.8);
        assert_eq!(params.iterations, 4);
        assert_eq!(params.cell_size, 1.2);
        assert_eq!(params.grid_width, 128);
        assert_eq!(params.grid_height, 128);
        assert_eq!(params.grid_depth, 128);
    }

    #[test]
    fn test_snapshot_params_clone() {
        let params = SnapshotParams {
            viscosity: 50.0,
            ..Default::default()
        };
        
        let cloned = params.clone();
        
        assert_eq!(cloned.viscosity, 50.0);
    }

    #[test]
    fn test_snapshot_params_roundtrip() {
        let params = SnapshotParams {
            smoothing_radius: 5.5,
            target_density: 100.0,
            pressure_multiplier: 500.0,
            viscosity: 25.0,
            surface_tension: 0.1,
            gravity: -20.0,
            iterations: 8,
            cell_size: 2.0,
            grid_width: 256,
            grid_height: 256,
            grid_depth: 256,
        };
        
        let bytes = bincode::serialize(&params).expect("Serialization failed");
        let recovered: SnapshotParams = bincode::deserialize(&bytes).expect("Deserialization failed");
        
        assert_eq!(recovered.smoothing_radius, 5.5);
        assert_eq!(recovered.target_density, 100.0);
        assert_eq!(recovered.pressure_multiplier, 500.0);
        assert_eq!(recovered.viscosity, 25.0);
        assert_eq!(recovered.surface_tension, 0.1);
        assert_eq!(recovered.gravity, -20.0);
        assert_eq!(recovered.iterations, 8);
        assert_eq!(recovered.cell_size, 2.0);
        assert_eq!(recovered.grid_width, 256);
        assert_eq!(recovered.grid_height, 256);
        assert_eq!(recovered.grid_depth, 256);
    }

    // ================== Edge Cases ==================

    #[test]
    fn test_fluid_snapshot_extreme_values() {
        let mut snapshot = FluidSnapshot::with_capacity(1);
        snapshot.positions.push([f32::MAX, f32::MIN, f32::INFINITY, 1.0]);
        snapshot.velocities.push([0.0, 0.0, 0.0, 0.0]);
        snapshot.colors.push([0.0, 0.0, 0.0, 0.0]);
        
        let bytes = snapshot.to_bytes().expect("Serialization failed");
        let recovered = FluidSnapshot::from_bytes(&bytes).expect("Deserialization failed");
        
        assert_eq!(recovered.positions[0][0], f32::MAX);
        assert_eq!(recovered.positions[0][1], f32::MIN);
        assert!(recovered.positions[0][2].is_infinite());
    }

    #[test]
    fn test_fluid_snapshot_nan_values() {
        let mut snapshot = FluidSnapshot::with_capacity(1);
        snapshot.positions.push([f32::NAN, 0.0, 0.0, 1.0]);
        snapshot.velocities.push([0.0, 0.0, 0.0, 0.0]);
        snapshot.colors.push([0.0, 0.0, 0.0, 0.0]);
        
        let bytes = snapshot.to_bytes().expect("Serialization failed");
        let recovered = FluidSnapshot::from_bytes(&bytes).expect("Deserialization failed");
        
        assert!(recovered.positions[0][0].is_nan());
    }

    #[test]
    fn test_snapshot_params_zero_values() {
        let params = SnapshotParams {
            smoothing_radius: 0.0,
            gravity: 0.0,
            iterations: 0,
            ..Default::default()
        };
        
        let bytes = bincode::serialize(&params).expect("Serialization failed");
        let recovered: SnapshotParams = bincode::deserialize(&bytes).expect("Deserialization failed");
        
        assert_eq!(recovered.smoothing_radius, 0.0);
        assert_eq!(recovered.gravity, 0.0);
        assert_eq!(recovered.iterations, 0);
    }

    #[test]
    fn test_fluid_snapshot_version_mismatch_handling() {
        // Create a snapshot and serialize
        let snapshot = FluidSnapshot::with_capacity(0);
        let mut bytes = snapshot.to_bytes().expect("Serialization failed");
        
        // Tamper with version (first few bytes)
        bytes[0] = 255;
        
        // Deserialization should still work (version mismatch is not enforced in from_bytes)
        let recovered = FluidSnapshot::from_bytes(&bytes);
        if let Ok(snap) = recovered {
            // Version was modified
            assert_ne!(snap.version, FluidSnapshot::VERSION);
        }
    }

    #[test]
    fn test_multiple_serialization_roundtrips() {
        let mut snapshot = FluidSnapshot::with_capacity(5);
        for i in 0..5 {
            snapshot.positions.push([i as f32, 0.0, 0.0, 1.0]);
            snapshot.velocities.push([0.0, 0.0, 0.0, 0.0]);
            snapshot.colors.push([1.0, 1.0, 1.0, 1.0]);
        }
        snapshot.active_count = 5;
        
        // Serialize and deserialize multiple times
        let mut current = snapshot.clone();
        for _ in 0..10 {
            let bytes = current.to_bytes().expect("Serialization failed");
            current = FluidSnapshot::from_bytes(&bytes).expect("Deserialization failed");
        }
        
        // Verify data integrity after multiple roundtrips
        assert_eq!(current.positions.len(), 5);
        assert_eq!(current.active_count, 5);
        assert_eq!(current.positions[0][0], 0.0);
        assert_eq!(current.positions[4][0], 4.0);
    }
}
