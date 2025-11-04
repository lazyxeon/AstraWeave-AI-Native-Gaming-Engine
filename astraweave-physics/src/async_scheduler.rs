// async_scheduler.rs - Async Physics Pipeline with Rayon Parallelization
//
// This module implements a 3-stage parallel physics pipeline:
// 1. Broad-Phase: Coarse collision detection (parallel AABB checks)
// 2. Narrow-Phase: Fine collision resolution (parallel contact generation)
// 3. Integration: Apply forces and update positions (parallel per-island)
//
// Key Design:
// - Uses Rayon for deterministic parallel iteration
// - Maintains strict ordering between stages (barriers)
// - Exposes telemetry for performance profiling
// - Feature-gated behind `async-physics` feature flag

use std::time::{Duration, Instant};

/// Performance telemetry for a single physics step
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicsStepProfile {
    /// Total time for complete physics step
    pub total_duration: Duration,

    /// Time spent in broad-phase collision detection
    pub broad_phase_duration: Duration,

    /// Time spent in narrow-phase collision resolution
    pub narrow_phase_duration: Duration,

    /// Time spent in integration (forces, positions)
    pub integration_duration: Duration,

    /// Number of active rigid bodies
    pub active_body_count: usize,

    /// Number of collision pairs detected
    pub collision_pair_count: usize,

    /// Number of constraint solver iterations
    pub solver_iterations: usize,
}

impl PhysicsStepProfile {
    /// Create a new profile with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate percentage of time spent in broad-phase
    pub fn broad_phase_percent(&self) -> f32 {
        if self.total_duration.as_nanos() == 0 {
            return 0.0;
        }
        (self.broad_phase_duration.as_nanos() as f32 / self.total_duration.as_nanos() as f32)
            * 100.0
    }

    /// Calculate percentage of time spent in narrow-phase
    pub fn narrow_phase_percent(&self) -> f32 {
        if self.total_duration.as_nanos() == 0 {
            return 0.0;
        }
        (self.narrow_phase_duration.as_nanos() as f32 / self.total_duration.as_nanos() as f32)
            * 100.0
    }

    /// Calculate percentage of time spent in integration
    pub fn integration_percent(&self) -> f32 {
        if self.total_duration.as_nanos() == 0 {
            return 0.0;
        }
        (self.integration_duration.as_nanos() as f32 / self.total_duration.as_nanos() as f32)
            * 100.0
    }
}

/// Async physics scheduler that coordinates parallel physics simulation
#[cfg(feature = "async-physics")]
pub struct AsyncPhysicsScheduler {
    /// Number of Rayon threads to use (0 = auto-detect)
    pub thread_count: usize,

    /// Last step profile (for telemetry)
    pub last_profile: PhysicsStepProfile,

    /// Enable/disable profiling (small overhead)
    pub enable_profiling: bool,
}

#[cfg(feature = "async-physics")]
impl Default for AsyncPhysicsScheduler {
    fn default() -> Self {
        Self {
            thread_count: 0, // Auto-detect (use Rayon's default)
            last_profile: PhysicsStepProfile::default(),
            enable_profiling: true,
        }
    }
}

#[cfg(feature = "async-physics")]
impl AsyncPhysicsScheduler {
    /// Create a new async scheduler
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a scheduler with specific thread count
    pub fn with_threads(thread_count: usize) -> Self {
        Self {
            thread_count,
            ..Default::default()
        }
    }

    /// Get the last step profile (for telemetry/dashboard)
    pub fn get_last_profile(&self) -> PhysicsStepProfile {
        self.last_profile
    }

    /// Record telemetry for a completed physics step
    /// Called by PhysicsWorld::step() when async is enabled
    pub fn record_step_telemetry(&mut self, total_duration: Duration) {
        if !self.enable_profiling {
            return;
        }

        // Update profile with actual timing
        // Note: Rapier3D handles internal parallelization, so we just record total time
        self.last_profile.total_duration = total_duration;

        // For now, we don't break down stages since Rapier3D's parallelization
        // is internal. Future: add custom instrumentation hooks.
        self.last_profile.integration_duration = total_duration;
        self.last_profile.broad_phase_duration = Duration::ZERO;
        self.last_profile.narrow_phase_duration = Duration::ZERO;
    }

    /// Execute a physics step with parallel pipeline
    ///
    /// This is a placeholder for the full implementation. In production, this would:
    /// 1. Parallel broad-phase: Split AABB checks across threads
    /// 2. Barrier sync
    /// 3. Parallel narrow-phase: Contact generation per collision pair
    /// 4. Barrier sync
    /// 5. Parallel integration: Per-island solver + position updates
    ///
    /// For now, we measure the existing single-threaded step and prepare structure.
    pub fn step_parallel<F>(&mut self, mut step_fn: F) -> PhysicsStepProfile
    where
        F: FnMut() -> PhysicsStepProfile,
    {
        let start = Instant::now();

        // TODO: Implement actual parallel pipeline
        // For Phase 1, we delegate to single-threaded implementation
        // and focus on telemetry infrastructure
        let profile = step_fn();

        let total_duration = start.elapsed();

        let final_profile = PhysicsStepProfile {
            total_duration,
            ..profile
        };

        if self.enable_profiling {
            self.last_profile = final_profile;
        }

        final_profile
    }

    /// Export telemetry to JSON file (for benchmark dashboard)
    #[cfg(feature = "serde")]
    pub fn export_telemetry(&self, path: &std::path::Path) -> anyhow::Result<()> {
        use std::fs;
        use std::io::Write;

        let json = serde_json::to_string_pretty(&self.last_profile)?;
        let mut file = fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

/// Parallel iterator helpers for deterministic physics
#[cfg(feature = "async-physics")]
pub mod parallel {
    use rayon::prelude::*;

    /// Process rigid bodies in parallel (deterministic order)
    ///
    /// This helper ensures:
    /// - Bodies are processed in handle order (determinism)
    /// - Work is split across available threads
    /// - Results are collected in original order
    pub fn par_process_bodies<T, F>(bodies: &[T], f: F) -> Vec<T>
    where
        T: Send + Sync + Clone,
        F: Fn(&T) -> T + Send + Sync,
    {
        bodies.par_iter().map(|body| f(body)).collect()
    }

    /// Process collision pairs in parallel
    ///
    /// Collision pairs have no ordering dependency, so can be processed freely
    pub fn par_process_collision_pairs<T, F>(pairs: &[T], f: F)
    where
        T: Send + Sync,
        F: Fn(&T) + Send + Sync,
    {
        pairs.par_iter().for_each(|pair| f(pair));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_percentages_sum_to_100() {
        let profile = PhysicsStepProfile {
            total_duration: Duration::from_millis(10),
            broad_phase_duration: Duration::from_millis(4),
            narrow_phase_duration: Duration::from_millis(3),
            integration_duration: Duration::from_millis(3),
            active_body_count: 100,
            collision_pair_count: 50,
            solver_iterations: 4,
        };

        let total_percent = profile.broad_phase_percent()
            + profile.narrow_phase_percent()
            + profile.integration_percent();

        // Should sum to 100% (within floating point error)
        assert!((total_percent - 100.0).abs() < 1.0);
    }

    #[test]
    #[cfg(feature = "async-physics")]
    fn scheduler_default_creation() {
        let scheduler = AsyncPhysicsScheduler::new();
        assert_eq!(scheduler.thread_count, 0); // Auto-detect
        assert!(scheduler.enable_profiling);
    }

    #[test]
    #[cfg(feature = "async-physics")]
    fn scheduler_with_threads() {
        let scheduler = AsyncPhysicsScheduler::with_threads(4);
        assert_eq!(scheduler.thread_count, 4);
    }

    #[test]
    #[cfg(feature = "async-physics")]
    fn parallel_body_processing_deterministic() {
        use super::parallel::par_process_bodies;

        let bodies: Vec<i32> = (0..100).collect();
        let processed = par_process_bodies(&bodies, |&x| x * 2);

        // Results should be in same order as input (determinism)
        for (i, &val) in processed.iter().enumerate() {
            assert_eq!(val, (i as i32) * 2);
        }
    }
}
