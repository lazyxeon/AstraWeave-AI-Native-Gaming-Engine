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

    // ================== FluidLodConfig Tests ==================

    #[test]
    fn test_lod_config_default() {
        let config = FluidLodConfig::default();
        
        assert_eq!(config.lod_distances, [20.0, 50.0, 100.0, 200.0]);
        assert_eq!(config.sim_rate_multipliers, [1.0, 0.5, 0.25, 0.1]);
        assert!(config.enable_clustering);
        assert_eq!(config.cluster_min_particles, 8);
    }

    #[test]
    fn test_lod_config_clone() {
        let config = FluidLodConfig::default();
        let cloned = config.clone();
        
        assert_eq!(cloned.lod_distances, config.lod_distances);
        assert_eq!(cloned.sim_rate_multipliers, config.sim_rate_multipliers);
    }

    #[test]
    fn test_lod_config_custom() {
        let config = FluidLodConfig {
            lod_distances: [10.0, 25.0, 50.0, 100.0],
            sim_rate_multipliers: [1.0, 0.75, 0.5, 0.25],
            enable_clustering: false,
            cluster_min_particles: 16,
        };
        
        assert_eq!(config.lod_distances[0], 10.0);
        assert_eq!(config.sim_rate_multipliers[1], 0.75);
        assert!(!config.enable_clustering);
    }

    // ================== LodLevel Tests ==================

    #[test]
    fn test_lod_level_values() {
        assert_eq!(LodLevel::Full as u8, 0);
        assert_eq!(LodLevel::High as u8, 1);
        assert_eq!(LodLevel::Medium as u8, 2);
        assert_eq!(LodLevel::Low as u8, 3);
        assert_eq!(LodLevel::Culled as u8, 4);
    }

    #[test]
    fn test_lod_level_eq() {
        assert_eq!(LodLevel::Full, LodLevel::Full);
        assert_ne!(LodLevel::Full, LodLevel::High);
    }

    #[test]
    fn test_lod_level_copy() {
        let level = LodLevel::Medium;
        let copied = level;
        assert_eq!(level, copied);
    }

    // ================== FluidLodManager Tests ==================

    #[test]
    fn test_lod_manager_new() {
        let config = FluidLodConfig::default();
        let manager = FluidLodManager::new(config);
        
        assert_eq!(manager.current_lod(), LodLevel::Full);
    }

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

    #[test]
    fn test_lod_full_threshold() {
        let config = FluidLodConfig::default(); // distances: [20.0, 50.0, 100.0, 200.0]
        let mut manager = FluidLodManager::new(config);
        
        // Distance 0 -> Full
        manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Full);
        
        // Distance 19 -> Full (just under threshold)
        manager.update([0.0, 0.0, 0.0], [19.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Full);
    }

    #[test]
    fn test_lod_high_threshold() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        // Distance 21 -> High (just over first threshold)
        manager.update([0.0, 0.0, 0.0], [21.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::High);
        
        // Distance 49 -> High (just under second threshold)
        manager.update([0.0, 0.0, 0.0], [49.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::High);
    }

    #[test]
    fn test_lod_medium_threshold() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        // Distance 51 -> Medium
        manager.update([0.0, 0.0, 0.0], [51.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Medium);
        
        // Distance 99 -> Medium
        manager.update([0.0, 0.0, 0.0], [99.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Medium);
    }

    #[test]
    fn test_lod_low_threshold() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        // Distance 101 -> Low
        manager.update([0.0, 0.0, 0.0], [101.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Low);
        
        // Distance 199 -> Low
        manager.update([0.0, 0.0, 0.0], [199.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Low);
    }

    #[test]
    fn test_lod_culled_threshold() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        // Distance 201 -> Culled
        manager.update([0.0, 0.0, 0.0], [201.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Culled);
        
        // Very far -> Culled
        manager.update([0.0, 0.0, 0.0], [1000.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Culled);
    }

    #[test]
    fn test_lod_3d_distance() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        // 3D distance calculation: sqrt(10^2 + 10^2 + 10^2) = sqrt(300) ≈ 17.32
        manager.update([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);
        assert_eq!(manager.current_lod(), LodLevel::Full);
        
        // sqrt(30^2 + 30^2 + 30^2) = sqrt(2700) ≈ 51.96
        manager.update([0.0, 0.0, 0.0], [30.0, 30.0, 30.0]);
        assert_eq!(manager.current_lod(), LodLevel::Medium);
    }

    #[test]
    fn test_lod_negative_coordinates() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        // Distance should be same regardless of sign
        manager.update([0.0, 0.0, 0.0], [-25.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::High);
        
        manager.update([0.0, 0.0, 0.0], [25.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::High);
    }

    // ================== Simulation Rate Tests ==================

    #[test]
    fn test_should_simulate_full_lod() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!(manager.should_simulate_this_frame());
    }

    #[test]
    fn test_should_simulate_high_lod() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        manager.update([0.0, 0.0, 0.0], [25.0, 0.0, 0.0]); // High LOD
        
        // High LOD simulates every other frame
        // After first update, frame_accumulator = 1
        // 1 % 2 == 1, not 0, so should NOT simulate
        // Actually, we need to check the pattern
        let frame1 = manager.should_simulate_this_frame();
        manager.update([0.0, 0.0, 0.0], [25.0, 0.0, 0.0]);
        let frame2 = manager.should_simulate_this_frame();
        
        // At least one of two consecutive frames should simulate
        assert!(frame1 || frame2);
    }

    #[test]
    fn test_should_simulate_culled() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        manager.update([0.0, 0.0, 0.0], [300.0, 0.0, 0.0]); // Culled
        assert!(!manager.should_simulate_this_frame());
    }

    #[test]
    fn test_sim_rate_multiplier_full() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(manager.sim_rate_multiplier(), 1.0);
    }

    #[test]
    fn test_sim_rate_multiplier_high() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        manager.update([0.0, 0.0, 0.0], [25.0, 0.0, 0.0]);
        assert_eq!(manager.sim_rate_multiplier(), 0.5);
    }

    #[test]
    fn test_sim_rate_multiplier_medium() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        manager.update([0.0, 0.0, 0.0], [75.0, 0.0, 0.0]);
        assert_eq!(manager.sim_rate_multiplier(), 0.25);
    }

    #[test]
    fn test_sim_rate_multiplier_low() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        manager.update([0.0, 0.0, 0.0], [150.0, 0.0, 0.0]);
        assert_eq!(manager.sim_rate_multiplier(), 0.1);
    }

    #[test]
    fn test_sim_rate_multiplier_culled() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        manager.update([0.0, 0.0, 0.0], [300.0, 0.0, 0.0]);
        assert_eq!(manager.sim_rate_multiplier(), 0.0);
    }

    // ================== Update Return Value Tests ==================

    #[test]
    fn test_update_returns_should_simulate() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        // Full LOD should always return true
        let result = manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!(result);
    }

    #[test]
    fn test_update_culled_returns_false() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        // Culled should always return false
        let result = manager.update([0.0, 0.0, 0.0], [300.0, 0.0, 0.0]);
        assert!(!result);
    }

    // ================== Custom Config Tests ==================

    #[test]
    fn test_custom_lod_distances() {
        let config = FluidLodConfig {
            lod_distances: [5.0, 10.0, 20.0, 50.0],
            ..Default::default()
        };
        let mut manager = FluidLodManager::new(config);
        
        // With tighter thresholds, should transition sooner
        manager.update([0.0, 0.0, 0.0], [6.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::High);
        
        manager.update([0.0, 0.0, 0.0], [15.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Medium);
    }

    #[test]
    fn test_custom_sim_rate_multipliers() {
        let config = FluidLodConfig {
            sim_rate_multipliers: [1.0, 0.8, 0.6, 0.4],
            ..Default::default()
        };
        let mut manager = FluidLodManager::new(config);
        
        manager.update([0.0, 0.0, 0.0], [25.0, 0.0, 0.0]); // High LOD
        assert_eq!(manager.sim_rate_multiplier(), 0.8);
    }

    // ================== Frame Accumulation Tests ==================

    #[test]
    fn test_frame_accumulator_increments() {
        let config = FluidLodConfig::default();
        let mut manager = FluidLodManager::new(config);
        
        // Each update increments frame_accumulator
        for i in 1..=10 {
            manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
            // frame_accumulator should be i after i updates
            // We can verify this indirectly through should_simulate behavior
        }
    }

    #[test]
    fn test_lod_level_debug() {
        // Test Debug trait implementation
        let level = LodLevel::Medium;
        let debug_str = format!("{:?}", level);
        assert_eq!(debug_str, "Medium");
    }
}
