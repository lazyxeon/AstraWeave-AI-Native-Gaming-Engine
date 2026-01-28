//! Fluid System Profiling
//!
//! Provides timing statistics for fluid simulation passes.

use std::collections::HashMap;

/// Timing statistics for fluid simulation profiling
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FluidTimingStats {
    /// Total time for the last simulation step (microseconds)
    pub total_step_us: u64,
    /// Time for SDF generation pass
    pub sdf_gen_us: u64,
    /// Time for predict pass
    pub predict_us: u64,
    /// Time for grid build pass
    pub grid_build_us: u64,
    /// Time for constraint solving (lambda + delta_pos)
    pub constraint_solve_us: u64,
    /// Time for integration pass
    pub integrate_us: u64,
    /// Time for secondary particle passes
    pub secondary_us: u64,
    /// Time for heat diffusion pass
    pub heat_diffuse_us: u64,
    /// Frame index when stats were collected
    pub frame: u64,
}

impl FluidTimingStats {
    /// Get total simulation time in milliseconds
    pub fn total_ms(&self) -> f32 {
        self.total_step_us as f32 / 1000.0
    }

    /// Get breakdown as percentages
    pub fn breakdown(&self) -> HashMap<&'static str, f32> {
        let total = self.total_step_us.max(1) as f32;
        [
            ("sdf_gen", self.sdf_gen_us as f32 / total * 100.0),
            ("predict", self.predict_us as f32 / total * 100.0),
            ("grid_build", self.grid_build_us as f32 / total * 100.0),
            (
                "constraint_solve",
                self.constraint_solve_us as f32 / total * 100.0,
            ),
            ("integrate", self.integrate_us as f32 / total * 100.0),
            ("secondary", self.secondary_us as f32 / total * 100.0),
            ("heat_diffuse", self.heat_diffuse_us as f32 / total * 100.0),
        ]
        .into_iter()
        .collect()
    }
}

/// Profiler for tracking fluid simulation performance
pub struct FluidProfiler {
    enabled: bool,
    stats: FluidTimingStats,
    frame_count: u64,
    accumulated_stats: FluidTimingStats,
}

impl Default for FluidProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl FluidProfiler {
    pub fn new() -> Self {
        Self {
            enabled: false,
            stats: FluidTimingStats::default(),
            frame_count: 0,
            accumulated_stats: FluidTimingStats::default(),
        }
    }

    /// Enable or disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the latest timing stats
    pub fn stats(&self) -> &FluidTimingStats {
        &self.stats
    }

    /// Get average stats over all frames
    pub fn average_stats(&self) -> FluidTimingStats {
        if self.frame_count == 0 {
            return FluidTimingStats::default();
        }
        let n = self.frame_count;
        FluidTimingStats {
            total_step_us: self.accumulated_stats.total_step_us / n,
            sdf_gen_us: self.accumulated_stats.sdf_gen_us / n,
            predict_us: self.accumulated_stats.predict_us / n,
            grid_build_us: self.accumulated_stats.grid_build_us / n,
            constraint_solve_us: self.accumulated_stats.constraint_solve_us / n,
            integrate_us: self.accumulated_stats.integrate_us / n,
            secondary_us: self.accumulated_stats.secondary_us / n,
            heat_diffuse_us: self.accumulated_stats.heat_diffuse_us / n,
            frame: self.frame_count,
        }
    }

    /// Record a frame's timing data (called by FluidSystem internally)
    pub fn record_frame(&mut self, stats: FluidTimingStats) {
        if !self.enabled {
            return;
        }
        self.frame_count += 1;
        self.accumulated_stats.total_step_us += stats.total_step_us;
        self.accumulated_stats.sdf_gen_us += stats.sdf_gen_us;
        self.accumulated_stats.predict_us += stats.predict_us;
        self.accumulated_stats.grid_build_us += stats.grid_build_us;
        self.accumulated_stats.constraint_solve_us += stats.constraint_solve_us;
        self.accumulated_stats.integrate_us += stats.integrate_us;
        self.accumulated_stats.secondary_us += stats.secondary_us;
        self.accumulated_stats.heat_diffuse_us += stats.heat_diffuse_us;
        self.stats = stats;
    }

    /// Reset all accumulated stats
    pub fn reset(&mut self) {
        self.frame_count = 0;
        self.accumulated_stats = FluidTimingStats::default();
        self.stats = FluidTimingStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ================== FluidTimingStats Tests ==================

    #[test]
    fn test_fluid_timing_stats_default() {
        let stats = FluidTimingStats::default();
        
        assert_eq!(stats.total_step_us, 0);
        assert_eq!(stats.sdf_gen_us, 0);
        assert_eq!(stats.predict_us, 0);
        assert_eq!(stats.grid_build_us, 0);
        assert_eq!(stats.constraint_solve_us, 0);
        assert_eq!(stats.integrate_us, 0);
        assert_eq!(stats.secondary_us, 0);
        assert_eq!(stats.heat_diffuse_us, 0);
        assert_eq!(stats.frame, 0);
    }

    #[test]
    fn test_fluid_timing_stats_clone() {
        let stats = FluidTimingStats {
            total_step_us: 1000,
            sdf_gen_us: 100,
            predict_us: 200,
            grid_build_us: 150,
            constraint_solve_us: 350,
            integrate_us: 100,
            secondary_us: 50,
            heat_diffuse_us: 50,
            frame: 42,
        };
        
        let cloned = stats.clone();
        
        assert_eq!(cloned.total_step_us, 1000);
        assert_eq!(cloned.frame, 42);
    }

    #[test]
    fn test_total_ms_conversion() {
        let stats = FluidTimingStats {
            total_step_us: 1000,
            ..Default::default()
        };
        
        assert_eq!(stats.total_ms(), 1.0);
    }

    #[test]
    fn test_total_ms_zero() {
        let stats = FluidTimingStats::default();
        assert_eq!(stats.total_ms(), 0.0);
    }

    #[test]
    fn test_total_ms_fractional() {
        let stats = FluidTimingStats {
            total_step_us: 500,
            ..Default::default()
        };
        
        assert_eq!(stats.total_ms(), 0.5);
    }

    #[test]
    fn test_total_ms_large_value() {
        let stats = FluidTimingStats {
            total_step_us: 16_667, // ~60fps in microseconds
            ..Default::default()
        };
        
        let ms = stats.total_ms();
        assert!((ms - 16.667).abs() < 0.001);
    }

    #[test]
    fn test_stats_breakdown() {
        let stats = FluidTimingStats {
            total_step_us: 1000,
            sdf_gen_us: 100,
            predict_us: 200,
            grid_build_us: 150,
            constraint_solve_us: 350,
            integrate_us: 100,
            secondary_us: 50,
            heat_diffuse_us: 50,
            frame: 1,
        };
        let breakdown = stats.breakdown();
        assert_eq!(breakdown["sdf_gen"], 10.0);
        assert_eq!(breakdown["constraint_solve"], 35.0);
    }

    #[test]
    fn test_breakdown_percentages_sum() {
        let stats = FluidTimingStats {
            total_step_us: 1000,
            sdf_gen_us: 100,
            predict_us: 200,
            grid_build_us: 150,
            constraint_solve_us: 350,
            integrate_us: 100,
            secondary_us: 50,
            heat_diffuse_us: 50,
            frame: 1,
        };
        let breakdown = stats.breakdown();
        
        let sum: f32 = breakdown.values().sum();
        assert_eq!(sum, 100.0);
    }

    #[test]
    fn test_breakdown_all_keys_present() {
        let stats = FluidTimingStats::default();
        let breakdown = stats.breakdown();
        
        assert!(breakdown.contains_key("sdf_gen"));
        assert!(breakdown.contains_key("predict"));
        assert!(breakdown.contains_key("grid_build"));
        assert!(breakdown.contains_key("constraint_solve"));
        assert!(breakdown.contains_key("integrate"));
        assert!(breakdown.contains_key("secondary"));
        assert!(breakdown.contains_key("heat_diffuse"));
    }

    #[test]
    fn test_breakdown_zero_total() {
        let stats = FluidTimingStats {
            total_step_us: 0,
            sdf_gen_us: 100,
            ..Default::default()
        };
        
        // With zero total, breakdown uses max(1) to avoid division by zero
        let breakdown = stats.breakdown();
        
        // 100 / 1 * 100 = 10000%
        assert_eq!(breakdown["sdf_gen"], 10000.0);
    }

    // ================== FluidProfiler Tests ==================

    #[test]
    fn test_fluid_profiler_new() {
        let profiler = FluidProfiler::new();
        
        assert!(!profiler.is_enabled());
        assert_eq!(profiler.stats().total_step_us, 0);
    }

    #[test]
    fn test_fluid_profiler_default() {
        let profiler = FluidProfiler::default();
        
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn test_profiler_enable_disable() {
        let mut profiler = FluidProfiler::new();
        
        assert!(!profiler.is_enabled());
        
        profiler.set_enabled(true);
        assert!(profiler.is_enabled());
        
        profiler.set_enabled(false);
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn test_profiler_stats_access() {
        let profiler = FluidProfiler::new();
        let stats = profiler.stats();
        
        assert_eq!(stats.total_step_us, 0);
    }

    #[test]
    fn test_profiler_record_frame_when_disabled() {
        let mut profiler = FluidProfiler::new();
        profiler.set_enabled(false);
        
        let stats = FluidTimingStats {
            total_step_us: 1000,
            ..Default::default()
        };
        
        profiler.record_frame(stats);
        
        // Stats should not be updated when disabled
        assert_eq!(profiler.stats().total_step_us, 0);
    }

    #[test]
    fn test_profiler_record_frame_when_enabled() {
        let mut profiler = FluidProfiler::new();
        profiler.set_enabled(true);
        
        let stats = FluidTimingStats {
            total_step_us: 1000,
            sdf_gen_us: 100,
            predict_us: 200,
            grid_build_us: 150,
            constraint_solve_us: 350,
            integrate_us: 100,
            secondary_us: 50,
            heat_diffuse_us: 50,
            frame: 1,
        };
        
        profiler.record_frame(stats);
        
        assert_eq!(profiler.stats().total_step_us, 1000);
        assert_eq!(profiler.stats().sdf_gen_us, 100);
    }

    #[test]
    fn test_profiler_average_stats_no_frames() {
        let profiler = FluidProfiler::new();
        let avg = profiler.average_stats();
        
        assert_eq!(avg.total_step_us, 0);
    }

    #[test]
    fn test_profiler_average_stats_single_frame() {
        let mut profiler = FluidProfiler::new();
        profiler.set_enabled(true);
        
        let stats = FluidTimingStats {
            total_step_us: 1000,
            sdf_gen_us: 100,
            ..Default::default()
        };
        
        profiler.record_frame(stats);
        let avg = profiler.average_stats();
        
        assert_eq!(avg.total_step_us, 1000);
        assert_eq!(avg.sdf_gen_us, 100);
        assert_eq!(avg.frame, 1);
    }

    #[test]
    fn test_profiler_average_stats_multiple_frames() {
        let mut profiler = FluidProfiler::new();
        profiler.set_enabled(true);
        
        // Frame 1: 1000us
        profiler.record_frame(FluidTimingStats {
            total_step_us: 1000,
            sdf_gen_us: 100,
            ..Default::default()
        });
        
        // Frame 2: 2000us
        profiler.record_frame(FluidTimingStats {
            total_step_us: 2000,
            sdf_gen_us: 200,
            ..Default::default()
        });
        
        // Frame 3: 3000us
        profiler.record_frame(FluidTimingStats {
            total_step_us: 3000,
            sdf_gen_us: 300,
            ..Default::default()
        });
        
        let avg = profiler.average_stats();
        
        // Average: (1000 + 2000 + 3000) / 3 = 2000
        assert_eq!(avg.total_step_us, 2000);
        assert_eq!(avg.sdf_gen_us, 200);
        assert_eq!(avg.frame, 3);
    }

    #[test]
    fn test_profiler_reset() {
        let mut profiler = FluidProfiler::new();
        profiler.set_enabled(true);
        
        profiler.record_frame(FluidTimingStats {
            total_step_us: 1000,
            ..Default::default()
        });
        
        assert_eq!(profiler.stats().total_step_us, 1000);
        
        profiler.reset();
        
        assert_eq!(profiler.stats().total_step_us, 0);
        assert_eq!(profiler.average_stats().total_step_us, 0);
    }

    #[test]
    fn test_profiler_reset_maintains_enabled_state() {
        let mut profiler = FluidProfiler::new();
        profiler.set_enabled(true);
        
        profiler.record_frame(FluidTimingStats {
            total_step_us: 1000,
            ..Default::default()
        });
        
        profiler.reset();
        
        // Enabled state should be preserved
        assert!(profiler.is_enabled());
    }

    #[test]
    fn test_profiler_accumulated_stats() {
        let mut profiler = FluidProfiler::new();
        profiler.set_enabled(true);
        
        for i in 0..100 {
            profiler.record_frame(FluidTimingStats {
                total_step_us: 1000,
                sdf_gen_us: i as u64 * 10,
                ..Default::default()
            });
        }
        
        let avg = profiler.average_stats();
        
        assert_eq!(avg.total_step_us, 1000);
        assert_eq!(avg.frame, 100);
        // Average of 0, 10, 20, ..., 990 = sum(0..100) * 10 / 100 = 4950 * 10 / 100 = 495
        assert_eq!(avg.sdf_gen_us, 495);
    }

    #[test]
    fn test_profiler_latest_stats_override() {
        let mut profiler = FluidProfiler::new();
        profiler.set_enabled(true);
        
        profiler.record_frame(FluidTimingStats {
            total_step_us: 1000,
            frame: 1,
            ..Default::default()
        });
        
        profiler.record_frame(FluidTimingStats {
            total_step_us: 2000,
            frame: 2,
            ..Default::default()
        });
        
        // Latest stats should be the most recent frame
        assert_eq!(profiler.stats().total_step_us, 2000);
        assert_eq!(profiler.stats().frame, 2);
    }

    // ================== Integration Tests ==================

    #[test]
    fn test_profiler_with_stats_workflow() {
        let mut profiler = FluidProfiler::new();
        profiler.set_enabled(true);
        
        // Simulate a fluid step
        let stats = FluidTimingStats {
            total_step_us: 16_667, // ~60 FPS
            sdf_gen_us: 2000,
            predict_us: 1500,
            grid_build_us: 3000,
            constraint_solve_us: 5000,
            integrate_us: 2000,
            secondary_us: 1667,
            heat_diffuse_us: 1500,
            frame: 1,
        };
        
        profiler.record_frame(stats);
        
        // Verify total time
        assert!((profiler.stats().total_ms() - 16.667).abs() < 0.001);
        
        // Verify breakdown
        let breakdown = profiler.stats().breakdown();
        assert!(breakdown["constraint_solve"] > 25.0); // Should be largest
    }
}
