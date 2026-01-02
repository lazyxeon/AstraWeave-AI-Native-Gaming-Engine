//! Level of Detail (LOD) System for Fluid Simulation
//!
//! Provides distance-based particle management for performance optimization.
//! Distant fluid volumes use reduced simulation rates and clustered rendering.

/// Configuration for fluid LOD management
#[derive(Clone, Debug)]
pub struct FluidLodConfig {
    /// Distance thresholds for LOD levels (in world units)
    pub lod_distances: [f32; 4],
    /// Simulation rate multipliers for each LOD level (1.0 = full rate)
    pub sim_rate_multipliers: [f32; 4],
    /// Whether to enable particle clustering for distant LODs
    pub enable_clustering: bool,
    /// Minimum particles per cluster at highest LOD
    pub cluster_min_particles: u32,
}

impl Default for FluidLodConfig {
    fn default() -> Self {
        Self {
            lod_distances: [20.0, 50.0, 100.0, 200.0],
            sim_rate_multipliers: [1.0, 0.5, 0.25, 0.1],
            enable_clustering: true,
            cluster_min_particles: 8,
        }
    }
}

/// LOD level for a fluid volume
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LodLevel {
    /// Full detail - all particles simulated every frame
    Full = 0,
    /// High detail - simulated every other frame
    High = 1,
    /// Medium detail - simulated every 4th frame
    Medium = 2,
    /// Low detail - simulated every 10th frame
    Low = 3,
    /// Culled - not simulated or rendered
    Culled = 4,
}

/// Manages LOD for a fluid system based on camera position
pub struct FluidLodManager {
    config: FluidLodConfig,
    current_lod: LodLevel,
    frame_accumulator: u32,
}

impl FluidLodManager {
    pub fn new(config: FluidLodConfig) -> Self {
        Self {
            config,
            current_lod: LodLevel::Full,
            frame_accumulator: 0,
        }
    }

    /// Update LOD based on distance from camera to fluid AABB center
    pub fn update(&mut self, camera_pos: [f32; 3], fluid_center: [f32; 3]) -> bool {
        let dx = camera_pos[0] - fluid_center[0];
        let dy = camera_pos[1] - fluid_center[1];
        let dz = camera_pos[2] - fluid_center[2];
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        self.current_lod = if distance < self.config.lod_distances[0] {
            LodLevel::Full
        } else if distance < self.config.lod_distances[1] {
            LodLevel::High
        } else if distance < self.config.lod_distances[2] {
            LodLevel::Medium
        } else if distance < self.config.lod_distances[3] {
            LodLevel::Low
        } else {
            LodLevel::Culled
        };

        self.frame_accumulator += 1;
        self.should_simulate_this_frame()
    }

    /// Check if fluid should be simulated this frame based on LOD
    pub fn should_simulate_this_frame(&self) -> bool {
        match self.current_lod {
            LodLevel::Full => true,
            LodLevel::High => self.frame_accumulator % 2 == 0,
            LodLevel::Medium => self.frame_accumulator % 4 == 0,
            LodLevel::Low => self.frame_accumulator % 10 == 0,
            LodLevel::Culled => false,
        }
    }

    /// Get current LOD level
    pub fn current_lod(&self) -> LodLevel {
        self.current_lod
    }

    /// Get simulation rate multiplier for current LOD
    pub fn sim_rate_multiplier(&self) -> f32 {
        match self.current_lod {
            LodLevel::Full => self.config.sim_rate_multipliers[0],
            LodLevel::High => self.config.sim_rate_multipliers[1],
            LodLevel::Medium => self.config.sim_rate_multipliers[2],
            LodLevel::Low => self.config.sim_rate_multipliers[3],
            LodLevel::Culled => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_distance_thresholds() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);

        // Camera at origin, fluid at origin -> Full LOD
        manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Full);

        // Camera far from fluid -> Low LOD
        manager.update([0.0, 0.0, 0.0], [150.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Low);

        // Camera very far from fluid -> Culled
        manager.update([0.0, 0.0, 0.0], [300.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Culled);
    }
}
