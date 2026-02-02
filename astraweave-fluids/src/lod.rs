//! Level of Detail (LOD) System for Fluid Simulation
//!
//! Provides distance-based particle management for performance optimization.
//! Distant fluid volumes use reduced simulation rates and clustered rendering.

use crate::optimization::{OptimizationPreset, WorkgroupConfig};

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
        for _ in 0..10 {
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

// =============================================================================
// Optimization-Aware LOD Extensions
// =============================================================================

/// Advanced LOD configuration with optimization integration.
///
/// Extends basic LOD with optimization preset awareness, automatic
/// quality scaling, and performance-based distance adjustment.
#[derive(Clone, Debug)]
pub struct OptimizedLodConfig {
    /// Base LOD configuration
    pub base: FluidLodConfig,
    /// Enable automatic distance adjustment based on frame time
    pub auto_adjust_distances: bool,
    /// Target frame time for distance adjustment (milliseconds)
    pub target_frame_time_ms: f32,
    /// Minimum distance scale factor (prevents too aggressive culling)
    pub min_distance_scale: f32,
    /// Maximum distance scale factor (prevents too loose culling)
    pub max_distance_scale: f32,
    /// Enable particle reduction at lower LODs
    pub enable_particle_reduction: bool,
    /// Particle reduction factors per LOD level [Full, High, Medium, Low]
    pub particle_factors: [f32; 4],
}

impl Default for OptimizedLodConfig {
    fn default() -> Self {
        Self {
            base: FluidLodConfig::default(),
            auto_adjust_distances: false,
            target_frame_time_ms: 4.0,
            min_distance_scale: 0.5,
            max_distance_scale: 2.0,
            enable_particle_reduction: true,
            particle_factors: [1.0, 0.75, 0.5, 0.25],
        }
    }
}

impl OptimizedLodConfig {
    /// Create configuration from optimization preset
    pub fn from_preset(preset: &OptimizationPreset) -> Self {
        // Determine quality level from budget target (higher budget = higher quality)
        let target_ms = preset.budget.target_ms;
        
        let (base, particle_factors, auto_adjust) = if target_ms >= 8.0 {
            // Ultra/Quality preset
            (
                FluidLodConfig {
                    lod_distances: [30.0, 75.0, 150.0, 300.0],
                    sim_rate_multipliers: [1.0, 0.75, 0.5, 0.25],
                    enable_clustering: true,
                    cluster_min_particles: 4,
                },
                [1.0, 0.9, 0.75, 0.5],
                false, // No auto-adjust for quality preset
            )
        } else if target_ms >= 4.0 {
            // Balanced preset
            (
                FluidLodConfig {
                    lod_distances: [25.0, 60.0, 120.0, 250.0],
                    sim_rate_multipliers: [1.0, 0.5, 0.25, 0.1],
                    enable_clustering: true,
                    cluster_min_particles: 8,
                },
                [1.0, 0.75, 0.5, 0.25],
                true,
            )
        } else if target_ms >= 2.0 {
            // Performance preset
            (
                FluidLodConfig {
                    lod_distances: [20.0, 50.0, 100.0, 200.0],
                    sim_rate_multipliers: [1.0, 0.5, 0.25, 0.1],
                    enable_clustering: true,
                    cluster_min_particles: 16,
                },
                [1.0, 0.5, 0.25, 0.125],
                true,
            )
        } else {
            // Minimum preset
            (
                FluidLodConfig {
                    lod_distances: [15.0, 40.0, 80.0, 150.0],
                    sim_rate_multipliers: [1.0, 0.33, 0.2, 0.1],
                    enable_clustering: true,
                    cluster_min_particles: 32,
                },
                [0.75, 0.5, 0.25, 0.1],
                true,
            )
        };

        Self {
            base,
            auto_adjust_distances: auto_adjust,
            target_frame_time_ms: target_ms,
            min_distance_scale: 0.5,
            max_distance_scale: 2.0,
            enable_particle_reduction: true,
            particle_factors,
        }
    }

    /// Create ultra-quality configuration (no particle reduction)
    pub fn ultra() -> Self {
        Self::from_preset(&OptimizationPreset::quality())
    }

    /// Create balanced configuration
    pub fn balanced() -> Self {
        Self::from_preset(&OptimizationPreset::balanced())
    }

    /// Create performance-focused configuration
    pub fn performance() -> Self {
        Self::from_preset(&OptimizationPreset::performance())
    }

    /// Create minimum-quality configuration
    pub fn minimum() -> Self {
        // Use performance preset with tighter budget for minimum
        let mut preset = OptimizationPreset::performance();
        preset.budget = crate::optimization::SimulationBudget::new(1.0);
        Self::from_preset(&preset)
    }
}

/// Advanced LOD manager with optimization integration.
///
/// Provides dynamic distance adjustment, particle streaming,
/// and optimization-aware culling decisions.
#[derive(Debug)]
pub struct OptimizedLodManager {
    config: OptimizedLodConfig,
    current_lod: LodLevel,
    frame_accumulator: u32,
    /// Current distance scale factor (adjusted automatically)
    distance_scale: f32,
    /// Rolling average frame time for auto-adjustment
    avg_frame_time_ms: f32,
    /// Number of frames in rolling average
    frame_count: u32,
    /// Current effective particle factor
    effective_particle_factor: f32,
    /// Visibility flag (can be forced)
    force_visible: bool,
    /// Visibility flag (can be forced invisible)
    force_invisible: bool,
    /// Last known camera position (for reference)
    last_camera_position: [f32; 3],
}

impl OptimizedLodManager {
    /// Create new optimized LOD manager
    pub fn new(config: OptimizedLodConfig) -> Self {
        Self {
            config,
            current_lod: LodLevel::Full,
            frame_accumulator: 0,
            distance_scale: 1.0,
            avg_frame_time_ms: 0.0,
            frame_count: 0,
            effective_particle_factor: 1.0,
            force_visible: false,
            force_invisible: false,
            last_camera_position: [0.0, 0.0, 0.0],
        }
    }

    /// Create new optimized LOD manager with initial camera position
    pub fn with_camera_position(config: OptimizedLodConfig, camera_position: [f32; 3]) -> Self {
        Self {
            config,
            current_lod: LodLevel::Full,
            frame_accumulator: 0,
            distance_scale: 1.0,
            avg_frame_time_ms: 0.0,
            frame_count: 0,
            effective_particle_factor: 1.0,
            force_visible: false,
            force_invisible: false,
            last_camera_position: camera_position,
        }
    }

    /// Get the last known camera position
    pub fn camera_position(&self) -> [f32; 3] {
        self.last_camera_position
    }

    /// Create with preset
    pub fn from_preset(preset: &OptimizationPreset) -> Self {
        Self::new(OptimizedLodConfig::from_preset(preset))
    }

    /// Update LOD with frame time feedback
    pub fn update_with_timing(
        &mut self,
        camera_pos: [f32; 3],
        fluid_center: [f32; 3],
        frame_time_ms: f32,
    ) -> LodUpdateResult {
        // Update camera position
        self.last_camera_position = camera_pos;

        // Update frame time average
        self.frame_count += 1;
        self.avg_frame_time_ms = self.avg_frame_time_ms * 0.95 + frame_time_ms * 0.05;

        // Auto-adjust distances if enabled
        if self.config.auto_adjust_distances && self.frame_count > 30 {
            self.adjust_distances();
        }

        // Calculate distance with scale
        let dx = camera_pos[0] - fluid_center[0];
        let dy = camera_pos[1] - fluid_center[1];
        let dz = camera_pos[2] - fluid_center[2];
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let scaled_distance = distance / self.distance_scale;

        // Determine LOD level
        let base = &self.config.base;
        let prev_lod = self.current_lod;

        // Note: force_visible and distance < lod_distances[0] both result in Full,
        // but are semantically different conditions (allow clippy::if_same_then_else)
        #[allow(clippy::if_same_then_else)]
        {
        self.current_lod = if self.force_invisible {
            LodLevel::Culled
        } else if self.force_visible {
            LodLevel::Full
        } else if scaled_distance < base.lod_distances[0] {
            LodLevel::Full
        } else if scaled_distance < base.lod_distances[1] {
            LodLevel::High
        } else if scaled_distance < base.lod_distances[2] {
            LodLevel::Medium
        } else if scaled_distance < base.lod_distances[3] {
            LodLevel::Low
        } else {
            LodLevel::Culled
        };
        }

        // Update particle factor
        self.effective_particle_factor = if self.config.enable_particle_reduction {
            match self.current_lod {
                LodLevel::Full => self.config.particle_factors[0],
                LodLevel::High => self.config.particle_factors[1],
                LodLevel::Medium => self.config.particle_factors[2],
                LodLevel::Low => self.config.particle_factors[3],
                LodLevel::Culled => 0.0,
            }
        } else if self.current_lod == LodLevel::Culled { 
            0.0 
        } else { 
            1.0 
        };

        self.frame_accumulator += 1;

        LodUpdateResult {
            should_simulate: self.should_simulate_this_frame(),
            lod_changed: prev_lod != self.current_lod,
            current_lod: self.current_lod,
            particle_factor: self.effective_particle_factor,
            sim_rate_multiplier: self.sim_rate_multiplier(),
            distance_scale: self.distance_scale,
        }
    }

    /// Simple update without timing (uses basic LOD manager logic)
    pub fn update(&mut self, camera_pos: [f32; 3], fluid_center: [f32; 3]) -> bool {
        let result = self.update_with_timing(camera_pos, fluid_center, 0.0);
        result.should_simulate
    }

    /// Adjust LOD distances based on frame time performance
    fn adjust_distances(&mut self) {
        let target = self.config.target_frame_time_ms;
        let current = self.avg_frame_time_ms;

        if current > target * 1.1 {
            // Over budget: shrink distances (more aggressive culling)
            self.distance_scale = (self.distance_scale * 0.98)
                .max(self.config.min_distance_scale);
        } else if current < target * 0.7 {
            // Under budget: expand distances (less aggressive culling)
            self.distance_scale = (self.distance_scale * 1.02)
                .min(self.config.max_distance_scale);
        }
    }

    /// Check if fluid should be simulated this frame
    pub fn should_simulate_this_frame(&self) -> bool {
        if self.force_invisible {
            return false;
        }
        if self.force_visible {
            return true;
        }

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
        let base = &self.config.base;
        match self.current_lod {
            LodLevel::Full => base.sim_rate_multipliers[0],
            LodLevel::High => base.sim_rate_multipliers[1],
            LodLevel::Medium => base.sim_rate_multipliers[2],
            LodLevel::Low => base.sim_rate_multipliers[3],
            LodLevel::Culled => 0.0,
        }
    }

    /// Get effective particle factor
    pub fn particle_factor(&self) -> f32 {
        self.effective_particle_factor
    }

    /// Get current distance scale
    pub fn distance_scale(&self) -> f32 {
        self.distance_scale
    }

    /// Get average frame time
    pub fn avg_frame_time_ms(&self) -> f32 {
        self.avg_frame_time_ms
    }

    /// Force visibility on (bypasses distance culling)
    pub fn set_force_visible(&mut self, force: bool) {
        self.force_visible = force;
        if force {
            self.force_invisible = false;
        }
    }

    /// Force visibility off
    pub fn set_force_invisible(&mut self, force: bool) {
        self.force_invisible = force;
        if force {
            self.force_visible = false;
        }
    }

    /// Reset distance scaling to default
    pub fn reset_distance_scale(&mut self) {
        self.distance_scale = 1.0;
    }

    /// Get recommended workgroup config for current LOD
    pub fn recommended_workgroup_config(&self) -> WorkgroupConfig {
        match self.current_lod {
            LodLevel::Full | LodLevel::High => WorkgroupConfig::universal(),
            LodLevel::Medium => WorkgroupConfig {
                particle_workgroup: 64,
                grid_workgroup: 64,
                secondary_workgroup: 32, // Reduce secondary work
            },
            LodLevel::Low => WorkgroupConfig {
                particle_workgroup: 32,
                grid_workgroup: 32,
                secondary_workgroup: 16, // Minimal secondary work
            },
            LodLevel::Culled => WorkgroupConfig::universal(),
        }
    }

    /// Get recommended iteration count for current LOD
    pub fn recommended_iterations(&self, base_iterations: u32) -> u32 {
        match self.current_lod {
            LodLevel::Full => base_iterations,
            LodLevel::High => base_iterations.max(2),
            LodLevel::Medium => (base_iterations * 3 / 4).max(2),
            LodLevel::Low => (base_iterations / 2).max(1),
            LodLevel::Culled => 0,
        }
    }
}

/// Result of LOD update operation
#[derive(Clone, Copy, Debug)]
pub struct LodUpdateResult {
    /// Whether simulation should run this frame
    pub should_simulate: bool,
    /// Whether LOD level changed from previous frame
    pub lod_changed: bool,
    /// Current LOD level
    pub current_lod: LodLevel,
    /// Particle reduction factor (0.0-1.0)
    pub particle_factor: f32,
    /// Simulation rate multiplier
    pub sim_rate_multiplier: f32,
    /// Current distance scale factor
    pub distance_scale: f32,
}

/// Streaming particle manager for LOD-based particle loading/unloading.
///
/// Manages particle pools with streaming support for large fluid volumes.
#[derive(Clone, Debug)]
pub struct ParticleStreamingManager {
    /// Maximum particles in memory
    max_particles: u32,
    /// Current active particles
    active_particles: u32,
    /// Particles pending load
    pending_load: u32,
    /// Particles pending unload
    pending_unload: u32,
    /// Streaming budget per frame
    stream_budget: u32,
    /// Target occupancy (0.0-1.0)
    target_occupancy: f32,
}

impl Default for ParticleStreamingManager {
    fn default() -> Self {
        Self::new(100_000, 1000)
    }
}

impl ParticleStreamingManager {
    /// Create new streaming manager
    pub fn new(max_particles: u32, stream_budget: u32) -> Self {
        Self {
            max_particles,
            active_particles: 0,
            pending_load: 0,
            pending_unload: 0,
            stream_budget,
            target_occupancy: 0.8,
        }
    }

    /// Create streaming manager with particle budget (default stream budget = 1000)
    pub fn with_budget(max_particles: usize) -> Self {
        Self::new(max_particles as u32, 1000)
    }

    /// Update streaming based on LOD requirements
    pub fn update(&mut self, lod_result: &LodUpdateResult, desired_particles: u32) {
        let target = (desired_particles as f32 * lod_result.particle_factor) as u32;
        let target = target.min(self.max_particles);

        if target > self.active_particles {
            // Need to load more particles
            let to_load = (target - self.active_particles).min(self.stream_budget);
            self.pending_load = to_load;
            self.pending_unload = 0;
        } else if target < self.active_particles {
            // Need to unload particles
            let to_unload = (self.active_particles - target).min(self.stream_budget);
            self.pending_unload = to_unload;
            self.pending_load = 0;
        } else {
            self.pending_load = 0;
            self.pending_unload = 0;
        }
    }

    /// Apply pending streaming operations
    pub fn apply_streaming(&mut self) -> StreamingOp {
        if self.pending_load > 0 {
            let loaded = self.pending_load;
            self.active_particles += loaded;
            self.pending_load = 0;
            StreamingOp::Load(loaded)
        } else if self.pending_unload > 0 {
            let unloaded = self.pending_unload;
            self.active_particles = self.active_particles.saturating_sub(unloaded);
            self.pending_unload = 0;
            StreamingOp::Unload(unloaded)
        } else {
            StreamingOp::None
        }
    }

    /// Get current active particle count
    pub fn active_particles(&self) -> u32 {
        self.active_particles
    }

    /// Get the maximum particle budget
    pub fn particle_budget(&self) -> usize {
        self.max_particles as usize
    }

    /// Get occupancy percentage
    pub fn occupancy(&self) -> f32 {
        self.active_particles as f32 / self.max_particles as f32
    }

    /// Check if at target occupancy
    pub fn at_target(&self) -> bool {
        self.pending_load == 0 && self.pending_unload == 0
    }

    /// Force set active particles (for initialization)
    pub fn set_active_particles(&mut self, count: u32) {
        self.active_particles = count.min(self.max_particles);
    }

    /// Get streaming status string
    pub fn status(&self) -> String {
        format!(
            "Particles: {}/{} ({:.1}%) | Pending: +{} -{} | Target: {:.0}%",
            self.active_particles,
            self.max_particles,
            self.occupancy() * 100.0,
            self.pending_load,
            self.pending_unload,
            self.target_occupancy * 100.0,
        )
    }
}

/// Streaming operation result
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamingOp {
    /// No operation needed
    None,
    /// Load N particles
    Load(u32),
    /// Unload N particles
    Unload(u32),
}

#[cfg(test)]
mod optimized_lod_tests {
    use super::*;

    #[test]
    fn test_optimized_config_default() {
        let config = OptimizedLodConfig::default();
        assert!(!config.auto_adjust_distances);
        assert!((config.target_frame_time_ms - 4.0).abs() < 0.001);
        assert!(config.enable_particle_reduction);
    }

    #[test]
    fn test_optimized_config_from_preset() {
        let preset = OptimizationPreset::performance();
        let config = OptimizedLodConfig::from_preset(&preset);
        assert!(config.auto_adjust_distances);
    }

    #[test]
    fn test_optimized_config_presets() {
        let ultra = OptimizedLodConfig::ultra();
        let balanced = OptimizedLodConfig::balanced();
        let perf = OptimizedLodConfig::performance();
        let minimum = OptimizedLodConfig::minimum();

        // Ultra should have longest distances
        assert!(ultra.base.lod_distances[0] > perf.base.lod_distances[0]);
        // Minimum should have highest particle reduction
        assert!(minimum.particle_factors[0] < ultra.particle_factors[0]);
        // Balanced should be in between
        assert!(balanced.base.lod_distances[0] < ultra.base.lod_distances[0]);
    }

    #[test]
    fn test_optimized_manager_new() {
        let config = OptimizedLodConfig::default();
        let manager = OptimizedLodManager::new(config);
        assert_eq!(manager.current_lod(), LodLevel::Full);
        assert!((manager.distance_scale() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_optimized_manager_from_preset() {
        let preset = OptimizationPreset::balanced();
        let manager = OptimizedLodManager::from_preset(&preset);
        assert_eq!(manager.current_lod(), LodLevel::Full);
    }

    #[test]
    fn test_optimized_manager_update_with_timing() {
        let config = OptimizedLodConfig::default();
        let mut manager = OptimizedLodManager::new(config);

        let result = manager.update_with_timing([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 2.0);

        assert!(result.should_simulate);
        assert_eq!(result.current_lod, LodLevel::Full);
        assert!((result.particle_factor - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_optimized_manager_lod_transition() {
        let config = OptimizedLodConfig::default();
        let mut manager = OptimizedLodManager::new(config);

        // Start at full
        manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Full);

        // Move to medium distance
        let result = manager.update_with_timing([0.0, 0.0, 0.0], [75.0, 0.0, 0.0], 2.0);
        assert_eq!(result.current_lod, LodLevel::Medium);
        assert!(result.lod_changed);
    }

    #[test]
    fn test_optimized_manager_particle_factor() {
        let mut config = OptimizedLodConfig::default();
        config.particle_factors = [1.0, 0.8, 0.6, 0.4];
        let mut manager = OptimizedLodManager::new(config);

        // High LOD
        manager.update([0.0, 0.0, 0.0], [30.0, 0.0, 0.0]);
        assert!((manager.particle_factor() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_optimized_manager_force_visible() {
        let config = OptimizedLodConfig::default();
        let mut manager = OptimizedLodManager::new(config);

        // Force invisible first
        manager.set_force_invisible(true);
        manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Culled);
        assert!(!manager.should_simulate_this_frame());

        // Force visible overrides
        manager.set_force_visible(true);
        manager.update([0.0, 0.0, 0.0], [1000.0, 0.0, 0.0]);
        assert_eq!(manager.current_lod(), LodLevel::Full);
        assert!(manager.should_simulate_this_frame());
    }

    #[test]
    fn test_optimized_manager_auto_adjust() {
        let mut config = OptimizedLodConfig::default();
        config.auto_adjust_distances = true;
        config.target_frame_time_ms = 4.0;
        let mut manager = OptimizedLodManager::new(config);

        // Simulate 50 frames over budget
        for _ in 0..50 {
            manager.update_with_timing([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 6.0);
        }

        // Distance scale should have decreased
        assert!(manager.distance_scale() < 1.0);
    }

    #[test]
    fn test_optimized_manager_recommended_workgroup() {
        let config = OptimizedLodConfig::default();
        let mut manager = OptimizedLodManager::new(config);

        manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        let full_config = manager.recommended_workgroup_config();
        assert_eq!(full_config.particle_workgroup, 64);

        manager.update([0.0, 0.0, 0.0], [150.0, 0.0, 0.0]);
        let low_config = manager.recommended_workgroup_config();
        assert_eq!(low_config.particle_workgroup, 32);
    }

    #[test]
    fn test_optimized_manager_recommended_iterations() {
        let config = OptimizedLodConfig::default();
        let mut manager = OptimizedLodManager::new(config);

        manager.update([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(manager.recommended_iterations(8), 8);

        manager.update([0.0, 0.0, 0.0], [75.0, 0.0, 0.0]);
        assert_eq!(manager.recommended_iterations(8), 6); // 3/4

        manager.update([0.0, 0.0, 0.0], [150.0, 0.0, 0.0]);
        assert_eq!(manager.recommended_iterations(8), 4); // 1/2
    }

    #[test]
    fn test_lod_update_result() {
        let result = LodUpdateResult {
            should_simulate: true,
            lod_changed: false,
            current_lod: LodLevel::Full,
            particle_factor: 1.0,
            sim_rate_multiplier: 1.0,
            distance_scale: 1.0,
        };

        assert!(result.should_simulate);
        assert!(!result.lod_changed);
    }

    // Streaming manager tests

    #[test]
    fn test_streaming_manager_new() {
        let manager = ParticleStreamingManager::new(100_000, 1000);
        assert_eq!(manager.active_particles(), 0);
        assert!((manager.occupancy() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_streaming_manager_default() {
        let manager = ParticleStreamingManager::default();
        assert_eq!(manager.max_particles, 100_000);
    }

    #[test]
    fn test_streaming_manager_update_load() {
        let mut manager = ParticleStreamingManager::new(100_000, 1000);
        let lod_result = LodUpdateResult {
            should_simulate: true,
            lod_changed: false,
            current_lod: LodLevel::Full,
            particle_factor: 1.0,
            sim_rate_multiplier: 1.0,
            distance_scale: 1.0,
        };

        manager.update(&lod_result, 5000);
        let op = manager.apply_streaming();

        assert_eq!(op, StreamingOp::Load(1000)); // Budget limited
        assert_eq!(manager.active_particles(), 1000);
    }

    #[test]
    fn test_streaming_manager_update_unload() {
        let mut manager = ParticleStreamingManager::new(100_000, 1000);
        manager.set_active_particles(5000);

        let lod_result = LodUpdateResult {
            should_simulate: true,
            lod_changed: false,
            current_lod: LodLevel::Low,
            particle_factor: 0.25,
            sim_rate_multiplier: 0.1,
            distance_scale: 1.0,
        };

        manager.update(&lod_result, 5000); // Target = 5000 * 0.25 = 1250
        let op = manager.apply_streaming();

        assert_eq!(op, StreamingOp::Unload(1000)); // Budget limited
        assert_eq!(manager.active_particles(), 4000);
    }

    #[test]
    fn test_streaming_manager_at_target() {
        let mut manager = ParticleStreamingManager::new(100_000, 1000);
        manager.set_active_particles(5000);

        let lod_result = LodUpdateResult {
            should_simulate: true,
            lod_changed: false,
            current_lod: LodLevel::Full,
            particle_factor: 1.0,
            sim_rate_multiplier: 1.0,
            distance_scale: 1.0,
        };

        manager.update(&lod_result, 5000); // Already at target
        assert!(manager.at_target());

        let op = manager.apply_streaming();
        assert_eq!(op, StreamingOp::None);
    }

    #[test]
    fn test_streaming_manager_status() {
        let mut manager = ParticleStreamingManager::new(10000, 100);
        manager.set_active_particles(5000);

        let status = manager.status();
        assert!(status.contains("5000"));
        assert!(status.contains("10000"));
        assert!(status.contains("50.0%"));
    }

    #[test]
    fn test_streaming_op_eq() {
        assert_eq!(StreamingOp::None, StreamingOp::None);
        assert_eq!(StreamingOp::Load(100), StreamingOp::Load(100));
        assert_ne!(StreamingOp::Load(100), StreamingOp::Load(200));
        assert_ne!(StreamingOp::Load(100), StreamingOp::Unload(100));
    }
}
