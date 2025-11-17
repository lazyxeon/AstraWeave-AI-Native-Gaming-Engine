//! Editor simulation runtime
//!
//! Provides deterministic play/pause/stop functionality with snapshot-based state management.

use anyhow::{Context, Result};
use astraweave_core::World;
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
}

impl Default for RuntimeStats {
    fn default() -> Self {
        Self {
            frame_time_ms: 0.0,
            entity_count: 0,
            tick_count: 0,
            fps: 0.0,
        }
    }
}

/// Editor simulation runtime
///
/// Manages the play/pause/stop lifecycle with deterministic snapshots.
pub struct EditorRuntime {
    /// Snapshot captured when entering play mode
    edit_snapshot: Option<SceneData>,
    
    /// Active simulation world (Some when playing/paused)
    sim_world: Option<World>,
    
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
            sim_world: None,
            tick_count: 0,
            state: RuntimeState::Editing,
            stats: RuntimeStats::default(),
            frame_times: Vec::with_capacity(60),
            last_frame_time: None,
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
        matches!(self.state, RuntimeState::Playing | RuntimeState::SteppingOneFrame)
    }

    /// Check if simulation is paused
    pub fn is_paused(&self) -> bool {
        self.state == RuntimeState::Paused
    }

    /// Get simulation world (if running)
    pub fn sim_world(&self) -> Option<&World> {
        self.sim_world.as_ref()
    }

    /// Get mutable simulation world (if running)
    pub fn sim_world_mut(&mut self) -> Option<&mut World> {
        self.sim_world.as_mut()
    }

    /// Capture current scene and enter play mode
    pub fn enter_play(&mut self, world: &World) -> Result<()> {
        if self.state != RuntimeState::Editing {
            return Ok(()); // Already playing
        }

        // Capture snapshot of edit state
        let snapshot = SceneData::from_world(world);
        
        // Clone world for simulation via snapshot
        let sim_world = snapshot.to_world();
        
        self.edit_snapshot = Some(snapshot);
        self.sim_world = Some(sim_world);
        self.tick_count = 0;
        self.state = RuntimeState::Playing;
        self.last_frame_time = Some(Instant::now());

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
        if self.sim_world.is_none() {
            return Ok(()); // Not in play mode
        }

        // Set state to stepping
        let was_paused = self.state == RuntimeState::Paused;
        self.state = RuntimeState::SteppingOneFrame;

        // Tick once at fixed 60Hz (16.67ms)
        self.tick(1.0 / 60.0)?;

        // Return to paused state
        if was_paused || self.state == RuntimeState::SteppingOneFrame {
            self.state = RuntimeState::Paused;
        }

        Ok(())
    }

    /// Advance simulation one frame
    pub fn tick(&mut self, _dt: f32) -> Result<()> {
        if !self.is_playing() {
            return Ok(());
        }

        let start = Instant::now();

        if let Some(world) = &mut self.sim_world {
            // Fixed 60Hz tick
            let fixed_dt = 1.0 / 60.0;
            
            // Update world time
            world.t += fixed_dt;
            
            self.tick_count += 1;
        }

        // Update frame timing
        let frame_time_ms = start.elapsed().as_secs_f32() * 1000.0;
        self.update_frame_time(frame_time_ms);

        // Update stats
        if let Some(world) = &self.sim_world {
            self.stats.entity_count = world.entities().len();
            self.stats.tick_count = self.tick_count;
        }

        // If stepping, we're done
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
        let restored_world = if let Some(snapshot) = &self.edit_snapshot {
            Some(snapshot.to_world())
        } else {
            None
        };

        // Reset runtime state
        self.sim_world = None;
        self.edit_snapshot = None;
        self.tick_count = 0;
        self.state = RuntimeState::Editing;
        self.stats = RuntimeStats::default();
        self.frame_times.clear();
        self.last_frame_time = None;

        Ok(restored_world)
    }

    /// Update frame time statistics
    fn update_frame_time(&mut self, frame_time_ms: f32) {
        self.frame_times.push(frame_time_ms);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        // Calculate average frame time and FPS
        if !self.frame_times.is_empty() {
            let avg_frame_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
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
            sim.spawn("runtime_entity", IVec2 { x: 20, y: 30 }, Team { id: 1 }, 50, 5);
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
}
