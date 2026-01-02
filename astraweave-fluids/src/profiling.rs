//! Fluid System Profiling
//!
//! Provides timing statistics for fluid simulation passes.

use std::collections::HashMap;

/// Timing statistics for fluid simulation profiling
#[derive(Clone, Debug, Default)]
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
}
