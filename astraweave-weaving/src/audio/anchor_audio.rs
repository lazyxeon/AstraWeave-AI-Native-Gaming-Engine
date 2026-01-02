// Anchor Audio System - State-based audio playback for anchors
//
// This system manages audio playback for anchors based on their VFX state,
// including ambient hum, repair sounds, and Echo pickup chimes.

use glam::Vec3;
use std::collections::HashMap;

/// Audio state for a single anchor
#[derive(Debug, Clone)]
pub struct AnchorAudioState {
    /// Anchor ID
    pub anchor_id: usize,
    /// Current VFX state (0=Perfect, 1=Stable, 2=Unstable, 3=Critical, 4=Broken)
    pub vfx_state: u8,
    /// Previous VFX state (for transition detection)
    pub previous_vfx_state: u8,
    /// Anchor world position (for 3D audio)
    pub position: Vec3,
    /// Is anchor currently being repaired? (triggers repair sound)
    pub is_repairing: bool,
    /// Time since repair started (for 5s sound duration)
    pub repair_time: f32,
    /// Current hum volume (0.0-1.0, fades during transitions)
    pub hum_volume: f32,
    /// Target hum volume (based on VFX state)
    pub target_hum_volume: f32,
    /// Hum audio source ID (for crossfading, None = not playing)
    pub hum_source_id: Option<usize>,
}

impl AnchorAudioState {
    /// Create new audio state for anchor
    pub fn new(anchor_id: usize, vfx_state: u8, position: Vec3) -> Self {
        let target_volume = Self::volume_for_state(vfx_state);
        Self {
            anchor_id,
            vfx_state,
            previous_vfx_state: vfx_state,
            position,
            is_repairing: false,
            repair_time: 0.0,
            hum_volume: target_volume,
            target_hum_volume: target_volume,
            hum_source_id: None,
        }
    }

    /// Get hum volume for VFX state (0.0-1.0)
    pub fn volume_for_state(vfx_state: u8) -> f32 {
        match vfx_state {
            0 => 0.0, // Perfect - silent (pristine reality)
            1 => 0.2, // Stable - subtle hum
            2 => 0.5, // Unstable - moderate hum (warning)
            3 => 0.8, // Critical - loud static (danger)
            4 => 0.0, // Broken - silent (dead)
            _ => 0.0, // Unknown state
        }
    }

    /// Get audio file path for VFX state
    pub fn audio_file_for_state(vfx_state: u8) -> &'static str {
        match vfx_state {
            0 => "", // Perfect - no sound
            1 => "assets/audio/anchor/anchor_hum_stable.ogg",
            2 => "assets/audio/anchor/anchor_hum_unstable.ogg",
            3 => "assets/audio/anchor/anchor_hum_critical.ogg",
            4 => "", // Broken - no sound
            _ => "", // Unknown state
        }
    }

    /// Check if VFX state has transitioned
    pub fn has_transitioned(&self) -> bool {
        self.vfx_state != self.previous_vfx_state
    }

    /// Get crossfade duration for state transition (seconds)
    pub fn crossfade_duration(&self) -> f32 {
        if self.vfx_state == 0 || self.previous_vfx_state == 0 {
            1.0 // 1s fade for Perfect (silent) transitions
        } else if self.vfx_state == 4 || self.previous_vfx_state == 4 {
            2.0 // 2s fade for Broken transitions
        } else {
            0.5 // 0.5s crossfade for other transitions
        }
    }

    /// Update audio state (handle transitions, repair sounds, volume fading)
    pub fn update(&mut self, delta_time: f32) -> Vec<AudioCommand> {
        let mut commands = Vec::new();

        // Check for VFX state transition
        if self.has_transitioned() {
            // Stop old hum (if playing)
            if let Some(source_id) = self.hum_source_id {
                commands.push(AudioCommand::StopSound {
                    source_id,
                    fade_duration: self.crossfade_duration(),
                });
                self.hum_source_id = None;
            }

            // Update target volume
            self.target_hum_volume = Self::volume_for_state(self.vfx_state);

            // Start new hum (if not silent)
            if self.target_hum_volume > 0.0 {
                let file_path = Self::audio_file_for_state(self.vfx_state);
                if !file_path.is_empty() {
                    commands.push(AudioCommand::PlaySound {
                        file_path: file_path.to_string(),
                        position: self.position,
                        volume: 0.0, // Start at 0, fade in
                        looping: true,
                        fade_in: self.crossfade_duration(),
                    });
                    // Source ID will be assigned by audio system
                }
            }

            // Mark transition as processed
            self.previous_vfx_state = self.vfx_state;
        }

        // Fade hum volume toward target
        if (self.hum_volume - self.target_hum_volume).abs() > 0.01 {
            let fade_speed = 1.0 / self.crossfade_duration(); // Reach target in crossfade duration
            let delta =
                (self.target_hum_volume - self.hum_volume).signum() * fade_speed * delta_time;
            self.hum_volume = (self.hum_volume + delta).clamp(0.0, 1.0);

            // Update hum volume if playing
            if let Some(source_id) = self.hum_source_id {
                commands.push(AudioCommand::SetVolume {
                    source_id,
                    volume: self.hum_volume,
                });
            }
        }

        // Handle repair sound
        if self.is_repairing {
            self.repair_time += delta_time;

            // Play repair sound at start (5s duration)
            if self.repair_time <= delta_time {
                commands.push(AudioCommand::PlaySound {
                    file_path: "assets/audio/anchor/anchor_repair.ogg".to_string(),
                    position: self.position,
                    volume: 0.7, // 70% volume (overlays hum)
                    looping: false,
                    fade_in: 0.0, // Immediate start
                });
            }

            // Reset repair state after 5s
            if self.repair_time >= 5.0 {
                self.is_repairing = false;
                self.repair_time = 0.0;
            }
        }

        commands
    }
}

// ============================================================================
// Audio Commands (for audio system integration)
// ============================================================================

/// Audio command sent from anchor audio system to audio engine
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// Play sound at position (3D spatial audio)
    PlaySound {
        file_path: String,
        position: Vec3,
        volume: f32,
        looping: bool,
        fade_in: f32, // Fade in duration (seconds)
    },
    /// Stop sound with fade out
    StopSound {
        source_id: usize,
        fade_duration: f32,
    },
    /// Set volume for playing sound
    SetVolume { source_id: usize, volume: f32 },
    /// Set position for playing sound (for moving anchors, if needed)
    SetPosition { source_id: usize, position: Vec3 },
}

// ============================================================================
// Anchor Audio System Manager
// ============================================================================

/// Manages audio state for all anchors in scene
pub struct AnchorAudioSystem {
    /// Audio states for all anchors
    states: HashMap<usize, AnchorAudioState>,
}

impl AnchorAudioSystem {
    /// Create new audio system
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Add audio state for anchor
    pub fn add_anchor(&mut self, anchor_id: usize, vfx_state: u8, position: Vec3) {
        let state = AnchorAudioState::new(anchor_id, vfx_state, position);
        self.states.insert(anchor_id, state);
    }

    /// Remove audio state for anchor (stops all sounds)
    pub fn remove_anchor(&mut self, anchor_id: usize) -> Vec<AudioCommand> {
        if let Some(state) = self.states.remove(&anchor_id) {
            let mut commands = Vec::new();
            if let Some(source_id) = state.hum_source_id {
                commands.push(AudioCommand::StopSound {
                    source_id,
                    fade_duration: 1.0, // 1s fade out
                });
            }
            return commands;
        }
        Vec::new()
    }

    /// Update anchor state (VFX state, position, repair status)
    pub fn update_anchor(
        &mut self,
        anchor_id: usize,
        vfx_state: u8,
        position: Vec3,
        is_repairing: bool,
    ) {
        if let Some(state) = self.states.get_mut(&anchor_id) {
            state.vfx_state = vfx_state;
            state.position = position;
            state.is_repairing = is_repairing;
        }
    }

    /// Update all audio states (returns audio commands to execute)
    pub fn update(&mut self, delta_time: f32) -> Vec<AudioCommand> {
        let mut all_commands = Vec::new();
        for state in self.states.values_mut() {
            let commands = state.update(delta_time);
            all_commands.extend(commands);
        }
        all_commands
    }

    /// Register audio source ID for anchor's hum (called by audio engine after PlaySound)
    pub fn register_hum_source(&mut self, anchor_id: usize, source_id: usize) {
        if let Some(state) = self.states.get_mut(&anchor_id) {
            state.hum_source_id = Some(source_id);
        }
    }

    /// Get audio state for anchor (for debugging/inspection)
    pub fn get_state(&self, anchor_id: usize) -> Option<&AnchorAudioState> {
        self.states.get(&anchor_id)
    }

    /// Get total number of managed anchors
    pub fn anchor_count(&self) -> usize {
        self.states.len()
    }

    /// Clear all audio states (stops all sounds)
    pub fn clear_all(&mut self) -> Vec<AudioCommand> {
        let mut commands = Vec::new();
        for state in self.states.values() {
            if let Some(source_id) = state.hum_source_id {
                commands.push(AudioCommand::StopSound {
                    source_id,
                    fade_duration: 0.5, // 0.5s fade out
                });
            }
        }
        self.states.clear();
        commands
    }
}

impl Default for AnchorAudioSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Echo Pickup Audio Helper
// ============================================================================

/// Create audio command for Echo pickup chime
pub fn echo_pickup_audio_command(position: Vec3) -> AudioCommand {
    AudioCommand::PlaySound {
        file_path: "assets/audio/anchor/echo_pickup.ogg".to_string(),
        position,
        volume: 0.5, // 50% volume (subtle chime)
        looping: false,
        fade_in: 0.0, // Immediate play
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_state_creation() {
        let state = AnchorAudioState::new(1, 1, Vec3::ZERO); // Stable

        assert_eq!(state.anchor_id, 1);
        assert_eq!(state.vfx_state, 1);
        assert_eq!(state.hum_volume, 0.2); // Stable = 20% volume
        assert_eq!(state.target_hum_volume, 0.2);
        assert!(!state.is_repairing);
    }

    #[test]
    fn test_volume_for_state() {
        assert_eq!(AnchorAudioState::volume_for_state(0), 0.0); // Perfect - silent
        assert_eq!(AnchorAudioState::volume_for_state(1), 0.2); // Stable - subtle
        assert_eq!(AnchorAudioState::volume_for_state(2), 0.5); // Unstable - moderate
        assert_eq!(AnchorAudioState::volume_for_state(3), 0.8); // Critical - loud
        assert_eq!(AnchorAudioState::volume_for_state(4), 0.0); // Broken - silent
    }

    #[test]
    fn test_audio_file_for_state() {
        assert_eq!(AnchorAudioState::audio_file_for_state(0), "");
        assert_eq!(
            AnchorAudioState::audio_file_for_state(1),
            "assets/audio/anchor/anchor_hum_stable.ogg"
        );
        assert_eq!(
            AnchorAudioState::audio_file_for_state(2),
            "assets/audio/anchor/anchor_hum_unstable.ogg"
        );
        assert_eq!(
            AnchorAudioState::audio_file_for_state(3),
            "assets/audio/anchor/anchor_hum_critical.ogg"
        );
        assert_eq!(AnchorAudioState::audio_file_for_state(4), "");
    }

    #[test]
    fn test_state_transition_detection() {
        let mut state = AnchorAudioState::new(1, 1, Vec3::ZERO);
        assert!(!state.has_transitioned());

        state.vfx_state = 2; // Stable → Unstable
        assert!(state.has_transitioned());
    }

    #[test]
    fn test_transition_generates_commands() {
        let mut state = AnchorAudioState::new(1, 1, Vec3::ZERO); // Stable
        state.hum_source_id = Some(100); // Pretend hum is playing

        state.vfx_state = 2; // Transition to Unstable

        let commands = state.update(0.016); // 1 frame @ 60 FPS

        // Should generate StopSound (old hum) and PlaySound (new hum)
        assert_eq!(commands.len(), 2);
        assert!(matches!(commands[0], AudioCommand::StopSound { .. }));
        assert!(matches!(commands[1], AudioCommand::PlaySound { .. }));
    }

    #[test]
    fn test_repair_sound_trigger() {
        let mut state = AnchorAudioState::new(1, 1, Vec3::ZERO);
        state.is_repairing = true;

        let commands = state.update(0.016); // First frame

        // Should play repair sound
        assert!(commands.iter().any(|c| matches!(
            c,
            AudioCommand::PlaySound { file_path, looping, .. } if file_path.contains("repair") && !looping
        )));
    }

    #[test]
    fn test_repair_sound_duration() {
        let mut state = AnchorAudioState::new(1, 1, Vec3::ZERO);
        state.is_repairing = true;

        // Update for 5.1s (repair duration + margin for float precision)
        for _ in 0..320 {
            state.update(0.016); // 60 FPS × 5.12s = 320 frames
        }

        // Repair should be finished
        assert!(!state.is_repairing);
        assert_eq!(state.repair_time, 0.0);
    }

    #[test]
    fn test_volume_fade() {
        let mut state = AnchorAudioState::new(1, 0, Vec3::ZERO); // Perfect (silent)
        state.vfx_state = 1; // Transition to Stable (20% volume)
        state.target_hum_volume = 0.2;

        // Update multiple frames to fade volume
        for _ in 0..60 {
            state.update(0.016); // 1 second @ 60 FPS
        }

        // Volume should be close to target (within fade tolerance)
        assert!((state.hum_volume - 0.2).abs() < 0.05);
    }

    #[test]
    fn test_audio_system_manager() {
        let mut system = AnchorAudioSystem::new();

        // Add 3 anchors
        system.add_anchor(1, 1, Vec3::ZERO);
        system.add_anchor(2, 2, Vec3::new(5.0, 0.0, 0.0));
        system.add_anchor(3, 3, Vec3::new(10.0, 0.0, 0.0));

        assert_eq!(system.anchor_count(), 3);

        // Update
        let commands = system.update(0.016);

        // Should generate some commands (hum playback, etc.)
        // Exact count depends on state transitions, so just verify it doesn't crash
        let _ = commands.len(); // May be 0 if no transitions
    }

    #[test]
    fn test_remove_anchor_stops_sound() {
        let mut system = AnchorAudioSystem::new();
        system.add_anchor(1, 1, Vec3::ZERO);

        // Register hum source
        system.register_hum_source(1, 100);

        // Remove anchor
        let commands = system.remove_anchor(1);

        // Should stop hum
        assert_eq!(commands.len(), 1);
        assert!(matches!(
            commands[0],
            AudioCommand::StopSound { source_id: 100, .. }
        ));
    }

    #[test]
    fn test_echo_pickup_audio() {
        let command = echo_pickup_audio_command(Vec3::new(1.0, 2.0, 3.0));

        match command {
            AudioCommand::PlaySound {
                file_path,
                position,
                volume,
                looping,
                ..
            } => {
                assert!(file_path.contains("echo_pickup"));
                assert_eq!(position, Vec3::new(1.0, 2.0, 3.0));
                assert_eq!(volume, 0.5);
                assert!(!looping);
            }
            _ => panic!("Expected PlaySound command"),
        }
    }

    #[test]
    fn test_clear_all_stops_sounds() {
        let mut system = AnchorAudioSystem::new();
        system.add_anchor(1, 1, Vec3::ZERO);
        system.add_anchor(2, 2, Vec3::new(5.0, 0.0, 0.0));

        // Register hum sources
        system.register_hum_source(1, 100);
        system.register_hum_source(2, 101);

        // Clear all
        let commands = system.clear_all();

        // Should stop both hums
        assert_eq!(commands.len(), 2);
        assert!(commands
            .iter()
            .all(|c| matches!(c, AudioCommand::StopSound { .. })));
    }
}
