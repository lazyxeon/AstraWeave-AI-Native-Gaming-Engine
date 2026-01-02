//! Editor simulation runtime
//!
//! Provides deterministic play/pause/stop functionality with snapshot-based state management.

use anyhow::Result;
use astraweave_core::ecs_adapter::build_app;
use astraweave_core::World;
use astraweave_ecs::App as SimulationApp;
use astraweave_profiling::{frame_mark, plot, span};
use std::time::Instant;

use crate::scene_serialization::SceneData;

/// Runtime execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// Not running (edit mode)
    Editing,
    /// Running continuously
    Playing,
    /// Paused at current tick
    Paused,
    /// Advance one tick then pause
    SteppingOneFrame,
}

/// Runtime performance statistics
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    /// Frame time in milliseconds
    pub frame_time_ms: f32,
    /// Entities in simulation
    pub entity_count: usize,
    /// Current tick number
    pub tick_count: u64,
    /// Frames per second
    pub fps: f32,
    /// Fixed 60 Hz steps executed during the last tick (for diagnostics)
    pub fixed_steps_last_tick: u32,
}

impl Default for RuntimeStats {
    fn default() -> Self {
        Self {
            frame_time_ms: 0.0,
            entity_count: 0,
            tick_count: 0,
            fps: 0.0,
            fixed_steps_last_tick: 0,
        }
    }
}

/// Editor simulation runtime
///
/// Manages the play/pause/stop lifecycle with deterministic snapshots.
pub struct EditorRuntime {
    /// Snapshot captured when entering play mode
    edit_snapshot: Option<SceneData>,

    /// Active simulation app (drives ECS/physics/audio once play mode starts)
    sim_app: Option<SimulationApp>,

    /// Current tick number (deterministic frame counter)
    tick_count: u64,

    /// Runtime state
    state: RuntimeState,

    /// Performance statistics
    stats: RuntimeStats,

    /// Frame time tracking (last 60 frames)
    frame_times: Vec<f32>,

    /// Last frame timestamp
    last_frame_time: Option<Instant>,

    /// Fixed 60 Hz timestep (seconds) used by the runtime
    fixed_dt: f32,

    /// Accumulated time waiting to be consumed by fixed steps
    time_accumulator: f32,
}

impl Default for EditorRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorRuntime {
    /// Create new runtime in editing state
    pub fn new() -> Self {
        Self {
            edit_snapshot: None,
            sim_app: None,
            tick_count: 0,
            state: RuntimeState::Editing,
            stats: RuntimeStats::default(),
            frame_times: Vec::with_capacity(60),
            last_frame_time: None,
            fixed_dt: 1.0 / 60.0,
            time_accumulator: 0.0,
        }
    }

    /// Get current runtime state
    pub fn state(&self) -> RuntimeState {
        self.state
    }

    /// Get current statistics
    pub fn stats(&self) -> &RuntimeStats {
        &self.stats
    }

    /// Check if simulation is running
    pub fn is_playing(&self) -> bool {
        matches!(
            self.state,
            RuntimeState::Playing | RuntimeState::SteppingOneFrame
        )
    }

    /// Check if simulation is paused
    pub fn is_paused(&self) -> bool {
        self.state == RuntimeState::Paused
    }

    /// Get simulation world (if running)
    pub fn sim_world(&self) -> Option<&World> {
        self.sim_app
            .as_ref()
            .and_then(|app| app.world.get_resource::<World>())
    }

    /// Get mutable simulation world (if running)
    pub fn sim_world_mut(&mut self) -> Option<&mut World> {
        self.sim_app
            .as_mut()
            .and_then(|app| app.world.get_resource_mut::<World>())
    }

    /// Capture current scene and enter play mode
    pub fn enter_play(&mut self, world: &World) -> Result<()> {
        if self.state != RuntimeState::Editing {
            return Ok(()); // Already playing
        }

        // Capture snapshot of edit state
        let snapshot = SceneData::from_world(world);

        // Clone world for simulation via snapshot and boot the ECS app
        let sim_world = snapshot.to_world();
        let sim_app = build_app(sim_world, self.fixed_dt);

        self.edit_snapshot = Some(snapshot);
        self.sim_app = Some(sim_app);
        self.tick_count = 0;
        self.state = RuntimeState::Playing;
        self.last_frame_time = Some(Instant::now());
        self.time_accumulator = 0.0;
        self.stats = RuntimeStats::default();
        self.frame_times.clear();

        if let Some(world) = self.sim_world() {
            self.stats.entity_count = world.entities().len();
        }

        Ok(())
    }

    /// Pause execution, preserving current state
    pub fn pause(&mut self) {
        if self.state == RuntimeState::Playing {
            self.state = RuntimeState::Paused;
        }
    }

    /// Resume from paused state
    pub fn resume(&mut self) {
        if self.state == RuntimeState::Paused {
            self.state = RuntimeState::Playing;
            self.last_frame_time = Some(Instant::now());
        }
    }

    /// Advance exactly one frame then pause
    pub fn step_frame(&mut self) -> Result<()> {
        if self.sim_app.is_none() {
            return Ok(()); // Not in play mode
        }

        // Set state to stepping
        let was_paused = self.state == RuntimeState::Paused;
        self.state = RuntimeState::SteppingOneFrame;
        self.time_accumulator = 0.0;
        let start = Instant::now();

        // Tick once at fixed 60Hz (16.67ms)
        self.run_fixed_steps(1)?;
        self.finish_tick(start, 1);

        // Return to paused state
        if was_paused || self.state == RuntimeState::SteppingOneFrame {
            self.state = RuntimeState::Paused;
        }

        Ok(())
    }

    /// Advance simulation using variable `dt`, internally respecting the fixed 60 Hz step.
    pub fn tick(&mut self, dt: f32) -> Result<()> {
        if !self.is_playing() {
            return Ok(());
        }

        span!("editor_runtime.tick");
        let start = Instant::now();

        let clamped_dt = if dt.is_finite() {
            dt.clamp(0.0, self.fixed_dt * 5.0)
        } else {
            self.fixed_dt
        };
        self.time_accumulator = (self.time_accumulator + clamped_dt).min(self.fixed_dt * 5.0);

        let mut steps_to_run = 0u32;
        while self.time_accumulator + f32::EPSILON >= self.fixed_dt {
            self.time_accumulator -= self.fixed_dt;
            steps_to_run += 1;
        }

        self.run_fixed_steps(steps_to_run)?;
        self.finish_tick(start, steps_to_run);

        if self.state == RuntimeState::SteppingOneFrame {
            self.state = RuntimeState::Paused;
        }

        Ok(())
    }

    /// Exit play mode and restore edit snapshot
    pub fn exit_play(&mut self) -> Result<Option<World>> {
        if self.state == RuntimeState::Editing {
            return Ok(None); // Already in edit mode
        }

        // Restore edit snapshot
        let restored_world = self.edit_snapshot.as_ref().map(|snapshot| snapshot.to_world());

        // Reset runtime state
        self.sim_app = None;
        self.edit_snapshot = None;
        self.tick_count = 0;
        self.state = RuntimeState::Editing;
        self.stats = RuntimeStats::default();
        self.frame_times.clear();
        self.last_frame_time = None;
        self.time_accumulator = 0.0;

        Ok(restored_world)
    }

    fn run_fixed_steps(&mut self, steps: u32) -> Result<()> {
        if steps == 0 {
            return Ok(());
        }

        if let Some(mut app) = self.sim_app.take() {
            for _ in 0..steps {
                span!("editor_runtime.fixed_step");
                app = app.run_fixed(1);
                self.tick_count += 1;
            }
            self.sim_app = Some(app);
        }

        Ok(())
    }

    fn finish_tick(&mut self, start: Instant, steps: u32) {
        self.stats.fixed_steps_last_tick = steps;
        let frame_time_ms = start.elapsed().as_secs_f32() * 1000.0;
        self.update_frame_time(frame_time_ms);

        if let Some(world) = self.sim_world() {
            self.stats.entity_count = world.entities().len();
        } else {
            self.stats.entity_count = 0;
        }
        self.stats.tick_count = self.tick_count;

        plot!("EditorRuntime::frame_ms", self.stats.frame_time_ms as f64);
        plot!("EditorRuntime::entities", self.stats.entity_count as f64);
        frame_mark!();
    }

    /// Update frame time statistics
    fn update_frame_time(&mut self, frame_time_ms: f32) {
        self.frame_times.push(frame_time_ms);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        // Calculate average frame time and FPS
        if !self.frame_times.is_empty() {
            let avg_frame_time: f32 =
                self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            self.stats.frame_time_ms = avg_frame_time;
            self.stats.fps = if avg_frame_time > 0.0 {
                1000.0 / avg_frame_time
            } else {
                0.0
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{IVec2, Team};

    #[test]
    fn runtime_starts_in_editing_state() {
        let runtime = EditorRuntime::new();
        assert_eq!(runtime.state(), RuntimeState::Editing);
        assert!(!runtime.is_playing());
        assert!(runtime.sim_world().is_none());
    }

    #[test]
    fn enter_play_captures_snapshot_and_creates_sim_world() {
        let mut runtime = EditorRuntime::new();
        let mut world = World::new();
        world.spawn("test", IVec2 { x: 5, y: 10 }, Team { id: 0 }, 100, 10);

        runtime.enter_play(&world).expect("enter play");

        assert_eq!(runtime.state(), RuntimeState::Playing);
        assert!(runtime.is_playing());
        assert!(runtime.sim_world().is_some());
        assert!(runtime.edit_snapshot.is_some());
        assert_eq!(runtime.tick_count, 0);
    }

    #[test]
    fn tick_advances_simulation_count() {
        let mut runtime = EditorRuntime::new();
        let world = World::new();
        runtime.enter_play(&world).expect("enter play");

        runtime.tick(1.0 / 60.0).expect("tick 1");
        assert_eq!(runtime.tick_count, 1);

        runtime.tick(1.0 / 60.0).expect("tick 2");
        assert_eq!(runtime.tick_count, 2);
    }

    #[test]
    fn pause_stops_ticking() {
        let mut runtime = EditorRuntime::new();
        let world = World::new();
        runtime.enter_play(&world).expect("enter play");

        runtime.tick(1.0 / 60.0).expect("tick 1");
        let tick_before_pause = runtime.tick_count;

        runtime.pause();
        assert_eq!(runtime.state(), RuntimeState::Paused);
        assert!(runtime.is_paused());

        // Ticking while paused does nothing
        runtime.tick(1.0 / 60.0).expect("tick while paused");
        assert_eq!(runtime.tick_count, tick_before_pause);
    }

    #[test]
    fn resume_continues_from_paused() {
        let mut runtime = EditorRuntime::new();
        let world = World::new();
        runtime.enter_play(&world).expect("enter play");
        runtime.tick(1.0 / 60.0).expect("tick 1");
        runtime.pause();

        runtime.resume();
        assert_eq!(runtime.state(), RuntimeState::Playing);
        assert!(runtime.is_playing());

        runtime.tick(1.0 / 60.0).expect("tick after resume");
        assert_eq!(runtime.tick_count, 2);
    }

    #[test]
    fn step_frame_advances_one_tick_then_pauses() {
        let mut runtime = EditorRuntime::new();
        let world = World::new();
        runtime.enter_play(&world).expect("enter play");
        runtime.pause();

        runtime.step_frame().expect("step 1");
        assert_eq!(runtime.tick_count, 1);
        assert_eq!(runtime.state(), RuntimeState::Paused);

        runtime.step_frame().expect("step 2");
        assert_eq!(runtime.tick_count, 2);
        assert_eq!(runtime.state(), RuntimeState::Paused);
    }

    #[test]
    fn exit_play_restores_edit_snapshot() {
        let mut runtime = EditorRuntime::new();
        let mut world = World::new();
        let entity = world.spawn("test", IVec2 { x: 5, y: 10 }, Team { id: 0 }, 100, 10);

        runtime.enter_play(&world).expect("enter play");

        // Simulate modifications during play
        if let Some(sim) = runtime.sim_world_mut() {
            sim.spawn(
                "runtime_entity",
                IVec2 { x: 20, y: 30 },
                Team { id: 1 },
                50,
                5,
            );
        }

        let restored = runtime.exit_play().expect("exit play");
        assert_eq!(runtime.state(), RuntimeState::Editing);
        assert!(runtime.sim_world().is_none());
        assert_eq!(runtime.tick_count, 0);

        // Verify restored world matches original (not runtime modifications)
        if let Some(restored_world) = restored {
            assert_eq!(restored_world.entities().len(), 1); // Only original entity
            assert!(restored_world.pose(entity).is_some());
        }
    }

    #[test]
    fn stats_track_entity_count_and_tick_count() {
        let mut runtime = EditorRuntime::new();
        let mut world = World::new();
        world.spawn("e1", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 10);
        world.spawn("e2", IVec2 { x: 1, y: 1 }, Team { id: 0 }, 100, 10);

        runtime.enter_play(&world).expect("enter play");
        runtime.tick(1.0 / 60.0).expect("tick");

        let stats = runtime.stats();
        assert_eq!(stats.tick_count, 1);
        assert_eq!(stats.entity_count, 2); // Should match spawned entities
    }

    #[test]
    fn tick_accumulates_until_full_step() {
        let mut runtime = EditorRuntime::new();
        let world = World::new();
        runtime.enter_play(&world).expect("enter play");

        let half_dt = (1.0 / 60.0) * 0.5;
        runtime.tick(half_dt).expect("fractional tick");
        assert_eq!(runtime.stats().tick_count, 0);
        assert_eq!(runtime.stats().fixed_steps_last_tick, 0);

        runtime.tick(half_dt).expect("second fractional tick");
        assert_eq!(runtime.stats().tick_count, 1);
        assert_eq!(runtime.stats().fixed_steps_last_tick, 1);
    }

    #[test]
    fn step_frame_executes_single_fixed_step() {
        let mut runtime = EditorRuntime::new();
        let world = World::new();
        runtime.enter_play(&world).expect("enter play");
        runtime.pause();

        runtime.step_frame().expect("step frame");

        assert_eq!(runtime.stats().tick_count, 1);
        assert_eq!(runtime.stats().fixed_steps_last_tick, 1);
        assert!(runtime.is_paused());
    }
}
