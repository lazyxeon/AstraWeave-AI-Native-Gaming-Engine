//! Player Water Mechanics
//!
//! Implements swimming, diving, oxygen management, and water-related
//! player interactions (inspired by Enshrouded's water mechanics).

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Movement mode in water
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WaterMovementMode {
    /// Not in water, normal movement
    #[default]
    Dry,
    /// Shallow water (ankle to knee), slightly reduced speed
    Wading,
    /// Waist-deep water, significantly reduced speed
    WaistDeep,
    /// Surface swimming
    Swimming,
    /// Underwater diving
    Diving,
}

impl WaterMovementMode {
    /// Get movement speed multiplier for this mode
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            WaterMovementMode::Dry => 1.0,
            WaterMovementMode::Wading => 0.85,
            WaterMovementMode::WaistDeep => 0.6,
            WaterMovementMode::Swimming => 0.7,
            WaterMovementMode::Diving => 0.5,
        }
    }

    /// Get stamina drain multiplier
    pub fn stamina_drain_multiplier(&self) -> f32 {
        match self {
            WaterMovementMode::Dry => 1.0,
            WaterMovementMode::Wading => 1.1,
            WaterMovementMode::WaistDeep => 1.3,
            WaterMovementMode::Swimming => 1.5,
            WaterMovementMode::Diving => 2.0,
        }
    }

    /// Whether player can jump in this mode
    pub fn can_jump(&self) -> bool {
        matches!(
            self,
            WaterMovementMode::Dry | WaterMovementMode::Wading | WaterMovementMode::WaistDeep
        )
    }

    /// Whether player consumes oxygen in this mode
    pub fn consumes_oxygen(&self) -> bool {
        matches!(self, WaterMovementMode::Diving)
    }
}

/// Wet status debuff levels (matching Enshrouded)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WetStatus {
    /// Player is completely dry
    #[default]
    Dry,
    /// Slightly wet (touched water briefly)
    Damp,
    /// Wet (-50% stamina regeneration)
    Wet,
    /// Soaking wet (-50% stamina regen, reduced max stamina)
    Soaking,
}

impl WetStatus {
    /// Get stamina regeneration multiplier
    pub fn stamina_regen_multiplier(&self) -> f32 {
        match self {
            WetStatus::Dry => 1.0,
            WetStatus::Damp => 0.9,
            WetStatus::Wet => 0.5,
            WetStatus::Soaking => 0.5,
        }
    }

    /// Get stamina max multiplier
    pub fn stamina_max_multiplier(&self) -> f32 {
        match self {
            WetStatus::Dry => 1.0,
            WetStatus::Damp => 1.0,
            WetStatus::Wet => 1.0,
            WetStatus::Soaking => 0.8, // -20% max stamina when soaking
        }
    }

    /// Time to dry from this status (seconds)
    pub fn dry_time(&self) -> f32 {
        match self {
            WetStatus::Dry => 0.0,
            WetStatus::Damp => 10.0,
            WetStatus::Wet => 30.0,
            WetStatus::Soaking => 60.0,
        }
    }
}

/// Configuration for water player mechanics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaterPlayerConfig {
    /// Maximum oxygen capacity (seconds)
    pub max_oxygen: f32,
    /// Oxygen consumption rate while diving (per second)
    pub oxygen_drain_rate: f32,
    /// Oxygen recovery rate while not diving (per second)
    pub oxygen_recovery_rate: f32,
    /// Time until drowning damage starts after oxygen depleted
    pub drowning_grace_period: f32,
    /// Drowning damage per second
    pub drowning_damage_rate: f32,
    /// Player height for submersion calculation
    pub player_height: f32,
    /// Water level thresholds for movement modes
    pub wading_threshold: f32,
    pub waist_deep_threshold: f32,
    pub swimming_threshold: f32,
    pub diving_threshold: f32,
    /// Time to become fully soaked
    pub soak_time: f32,
    /// Wet Dog skill level (reduces wet debuff)
    pub wet_resistance_level: u8,
}

impl Default for WaterPlayerConfig {
    fn default() -> Self {
        Self {
            max_oxygen: 30.0,           // 30 seconds of breath
            oxygen_drain_rate: 1.0,     // 1 oxygen per second
            oxygen_recovery_rate: 3.0,  // Recover 3x faster than drain
            drowning_grace_period: 3.0, // 3 seconds before damage
            drowning_damage_rate: 10.0, // 10 HP/sec drowning damage
            player_height: 1.8,         // Meters
            wading_threshold: 0.15,     // 15% submerged
            waist_deep_threshold: 0.4,  // 40% submerged
            swimming_threshold: 0.7,    // 70% submerged
            diving_threshold: 0.95,     // 95% submerged (head under)
            soak_time: 5.0,             // 5 seconds to become soaking
            wet_resistance_level: 0,    // No wet dog skill by default
        }
    }
}

/// Water interaction state for a player/entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaterPlayerState {
    /// Current submersion level (0.0 = dry, 1.0 = fully submerged)
    pub submersion: f32,
    /// Current movement mode
    pub mode: WaterMovementMode,
    /// Current oxygen remaining (seconds)
    pub oxygen: f32,
    /// Maximum oxygen (can be modified by equipment)
    pub max_oxygen: f32,
    /// Current wet status
    pub wet_status: WetStatus,
    /// Time spent in current wet status (for transitioning)
    pub wet_timer: f32,
    /// Time spent submerged (for soak calculation)
    pub submerge_time: f32,
    /// Drowning timer (counts up after oxygen depleted)
    pub drowning_timer: f32,
    /// Whether player is voluntarily diving (pressed dive key)
    pub is_diving: bool,
    /// Configuration
    config: WaterPlayerConfig,
}

impl Default for WaterPlayerState {
    fn default() -> Self {
        Self::new(WaterPlayerConfig::default())
    }
}

impl WaterPlayerState {
    /// Create a new water player state with configuration
    pub fn new(config: WaterPlayerConfig) -> Self {
        Self {
            submersion: 0.0,
            mode: WaterMovementMode::Dry,
            oxygen: config.max_oxygen,
            max_oxygen: config.max_oxygen,
            wet_status: WetStatus::Dry,
            wet_timer: 0.0,
            submerge_time: 0.0,
            drowning_timer: 0.0,
            is_diving: false,
            config,
        }
    }

    /// Update water state based on submersion level
    ///
    /// Returns drowning damage if applicable
    pub fn update(&mut self, submersion: f32, dt: f32) -> WaterUpdateResult {
        self.submersion = submersion.clamp(0.0, 1.0);

        // Determine movement mode based on submersion
        self.mode = if self.submersion < self.config.wading_threshold {
            WaterMovementMode::Dry
        } else if self.submersion < self.config.waist_deep_threshold {
            WaterMovementMode::Wading
        } else if self.submersion < self.config.swimming_threshold {
            WaterMovementMode::WaistDeep
        } else if self.submersion < self.config.diving_threshold && !self.is_diving {
            WaterMovementMode::Swimming
        } else {
            WaterMovementMode::Diving
        };

        // Update wet status
        self.update_wet_status(dt);

        // Update oxygen
        let drowning_damage = self.update_oxygen(dt);

        // Track submersion time
        if self.submersion > 0.0 {
            self.submerge_time += dt;
        } else {
            self.submerge_time = 0.0;
        }

        WaterUpdateResult {
            mode: self.mode,
            wet_status: self.wet_status,
            oxygen_percent: self.oxygen / self.max_oxygen,
            drowning_damage,
            speed_multiplier: self.get_speed_multiplier(),
            stamina_drain_multiplier: self.get_stamina_drain_multiplier(),
            stamina_regen_multiplier: self.get_stamina_regen_multiplier(),
        }
    }

    /// Update wet status based on submersion
    fn update_wet_status(&mut self, dt: f32) {
        if self.submersion > 0.0 {
            // Getting wetter
            self.wet_timer += dt;

            self.wet_status = if self.wet_timer < 1.0 {
                WetStatus::Damp
            } else if self.wet_timer < 3.0 {
                WetStatus::Wet
            } else if self.wet_timer >= self.config.soak_time {
                WetStatus::Soaking
            } else {
                WetStatus::Wet
            };
        } else if self.wet_status != WetStatus::Dry {
            // Drying off - timer counts down in real seconds
            // Wet Dog skill speeds up drying
            let skill_bonus = match self.config.wet_resistance_level {
                0 => 1.0,
                1 => 1.25, // 25% faster
                2 => 1.5,  // 50% faster
                _ => 2.0,  // 100% faster
            };

            self.wet_timer -= dt * skill_bonus;

            if self.wet_timer <= 0.0 {
                // Transition to previous wet status
                self.wet_status = match self.wet_status {
                    WetStatus::Soaking => WetStatus::Wet,
                    WetStatus::Wet => WetStatus::Damp,
                    WetStatus::Damp => WetStatus::Dry,
                    WetStatus::Dry => WetStatus::Dry,
                };

                if self.wet_status != WetStatus::Dry {
                    self.wet_timer = self.wet_status.dry_time();
                }
            }
        }
    }

    /// Update oxygen and return drowning damage
    fn update_oxygen(&mut self, dt: f32) -> f32 {
        if self.mode.consumes_oxygen() {
            // Consuming oxygen
            self.oxygen -= self.config.oxygen_drain_rate * dt;

            if self.oxygen <= 0.0 {
                self.oxygen = 0.0;
                self.drowning_timer += dt;

                // Apply drowning damage after grace period
                if self.drowning_timer > self.config.drowning_grace_period {
                    return self.config.drowning_damage_rate * dt;
                }
            }
        } else {
            // Recovering oxygen
            self.oxygen += self.config.oxygen_recovery_rate * dt;
            self.oxygen = self.oxygen.min(self.max_oxygen);
            self.drowning_timer = 0.0;
        }

        0.0
    }

    /// Start voluntary diving
    pub fn start_dive(&mut self) {
        self.is_diving = true;
    }

    /// Stop voluntary diving (surface)
    pub fn stop_dive(&mut self) {
        self.is_diving = false;
    }

    /// Toggle dive state
    pub fn toggle_dive(&mut self) {
        self.is_diving = !self.is_diving;
    }

    /// Get current oxygen as percentage (0.0-1.0)
    pub fn oxygen_percent(&self) -> f32 {
        self.oxygen / self.max_oxygen
    }

    /// Check if player is drowning (out of oxygen and past grace period)
    pub fn is_drowning(&self) -> bool {
        self.oxygen <= 0.0 && self.drowning_timer > self.config.drowning_grace_period
    }

    /// Check if player is at risk of drowning (low oxygen)
    pub fn is_low_oxygen(&self) -> bool {
        self.oxygen < self.max_oxygen * 0.25
    }

    /// Get movement speed multiplier
    pub fn get_speed_multiplier(&self) -> f32 {
        self.mode.speed_multiplier()
    }

    /// Get stamina drain multiplier
    pub fn get_stamina_drain_multiplier(&self) -> f32 {
        self.mode.stamina_drain_multiplier()
    }

    /// Get stamina regeneration multiplier (affected by wet status)
    pub fn get_stamina_regen_multiplier(&self) -> f32 {
        let wet_mult = self.wet_status.stamina_regen_multiplier();

        // Wet Dog skill reduces penalty
        let skill_reduction = match self.config.wet_resistance_level {
            0 => 1.0,
            1 => 0.75, // 25% reduction
            2 => 0.5,  // 50% reduction
            _ => 0.0,  // No penalty
        };

        1.0 - (1.0 - wet_mult) * skill_reduction
    }

    /// Get stamina max multiplier (affected by wet status)
    pub fn get_stamina_max_multiplier(&self) -> f32 {
        self.wet_status.stamina_max_multiplier()
    }

    /// Apply an oxygen boost (from breathing bubble, etc.)
    pub fn add_oxygen(&mut self, amount: f32) {
        self.oxygen = (self.oxygen + amount).min(self.max_oxygen);
    }

    /// Check if player can perform an action that requires breathing
    pub fn can_breathe(&self) -> bool {
        !self.mode.consumes_oxygen() || self.oxygen > 0.0
    }

    /// Set wet resistance skill level (Wet Dog / Wetter Dog)
    pub fn set_wet_resistance(&mut self, level: u8) {
        self.config.wet_resistance_level = level.min(3);
    }
}

/// Result of water state update
#[derive(Clone, Copy, Debug)]
pub struct WaterUpdateResult {
    /// Current movement mode
    pub mode: WaterMovementMode,
    /// Current wet status
    pub wet_status: WetStatus,
    /// Oxygen remaining (0.0-1.0)
    pub oxygen_percent: f32,
    /// Drowning damage this frame
    pub drowning_damage: f32,
    /// Movement speed multiplier to apply
    pub speed_multiplier: f32,
    /// Stamina drain multiplier to apply
    pub stamina_drain_multiplier: f32,
    /// Stamina regen multiplier to apply
    pub stamina_regen_multiplier: f32,
}

/// Water-related player skills
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WaterSkills {
    /// "Splash Dash" - Forward lunge while swimming
    pub splash_dash: bool,
    /// Splash dash cooldown (seconds)
    pub splash_dash_cooldown: f32,
    /// Current splash dash timer
    pub splash_dash_timer: f32,
    /// "Wet Dog" skill level (0=none, 1=25%, 2=50%, 3=75% reduced penalty)
    pub wet_resistance_level: u8,
    /// "Deep Diver" - Increased oxygen capacity
    pub deep_diver_level: u8,
    /// "Swift Swimmer" - Increased swim speed
    pub swift_swimmer_level: u8,
}

impl WaterSkills {
    /// Check if splash dash is available
    pub fn can_splash_dash(&self) -> bool {
        self.splash_dash && self.splash_dash_timer <= 0.0
    }

    /// Use splash dash
    pub fn use_splash_dash(&mut self) {
        if self.can_splash_dash() {
            self.splash_dash_timer = self.splash_dash_cooldown;
        }
    }

    /// Update skill cooldowns
    pub fn update(&mut self, dt: f32) {
        if self.splash_dash_timer > 0.0 {
            self.splash_dash_timer -= dt;
        }
    }

    /// Get oxygen capacity bonus
    pub fn oxygen_bonus(&self) -> f32 {
        match self.deep_diver_level {
            0 => 1.0,
            1 => 1.25,
            2 => 1.5,
            _ => 2.0,
        }
    }

    /// Get swim speed bonus
    pub fn swim_speed_bonus(&self) -> f32 {
        match self.swift_swimmer_level {
            0 => 1.0,
            1 => 1.15,
            2 => 1.3,
            _ => 1.5,
        }
    }
}

/// Helper for computing movement in water
#[derive(Clone, Debug)]
pub struct WaterMovementHelper {
    /// Buoyancy force applied when underwater
    pub buoyancy_force: f32,
    /// Drag coefficient in water
    pub water_drag: f32,
    /// Swim force multiplier
    pub swim_force: f32,
}

impl Default for WaterMovementHelper {
    fn default() -> Self {
        Self {
            buoyancy_force: 15.0, // Upward force when submerged
            water_drag: 3.0,     // Slows movement
            swim_force: 8.0,     // Force from swimming input
        }
    }
}

impl WaterMovementHelper {
    /// Calculate water forces for a submerged entity
    pub fn calculate_water_forces(
        &self,
        velocity: Vec3,
        submersion: f32,
        swim_input: Vec3,
        mode: WaterMovementMode,
    ) -> WaterForces {
        let mut forces = WaterForces::default();

        // Buoyancy (only when submerged)
        if submersion > 0.0 {
            forces.buoyancy = Vec3::new(0.0, self.buoyancy_force * submersion, 0.0);
        }

        // Drag (opposes velocity)
        if velocity.length_squared() > 0.001 {
            let drag_force = velocity * velocity.length() * self.water_drag * submersion;
            forces.drag = -drag_force;
        }

        // Swim force from input
        if matches!(mode, WaterMovementMode::Swimming | WaterMovementMode::Diving) {
            forces.swim = swim_input * self.swim_force;
        }

        forces
    }
}

/// Forces acting on an entity in water
#[derive(Clone, Copy, Debug, Default)]
pub struct WaterForces {
    /// Upward buoyancy force
    pub buoyancy: Vec3,
    /// Drag force (opposes motion)
    pub drag: Vec3,
    /// Swim force from player input
    pub swim: Vec3,
}

impl WaterForces {
    /// Get total force
    pub fn total(&self) -> Vec3 {
        self.buoyancy + self.drag + self.swim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_mode_from_submersion() {
        let mut state = WaterPlayerState::default();

        // Dry
        state.update(0.0, 0.1);
        assert_eq!(state.mode, WaterMovementMode::Dry);

        // Wading
        state.update(0.2, 0.1);
        assert_eq!(state.mode, WaterMovementMode::Wading);

        // Waist deep
        state.update(0.5, 0.1);
        assert_eq!(state.mode, WaterMovementMode::WaistDeep);

        // Swimming
        state.update(0.8, 0.1);
        assert_eq!(state.mode, WaterMovementMode::Swimming);

        // Diving
        state.update(0.99, 0.1);
        assert_eq!(state.mode, WaterMovementMode::Diving);
    }

    #[test]
    fn test_oxygen_consumption() {
        let mut state = WaterPlayerState::default();
        let initial_oxygen = state.oxygen;

        // Simulate diving for 5 seconds
        for _ in 0..50 {
            state.update(1.0, 0.1); // Fully submerged
        }

        assert!(state.oxygen < initial_oxygen);
        assert!(state.mode == WaterMovementMode::Diving);
    }

    #[test]
    fn test_oxygen_recovery() {
        let mut state = WaterPlayerState::default();
        state.oxygen = 10.0; // Low oxygen

        // Surface and recover
        for _ in 0..50 {
            state.update(0.0, 0.1); // Out of water
        }

        assert!(state.oxygen > 10.0);
    }

    #[test]
    fn test_drowning() {
        let mut state = WaterPlayerState::default();
        state.oxygen = 0.0; // No oxygen

        // Simulate drowning
        let mut total_damage = 0.0;
        for _ in 0..100 {
            let result = state.update(1.0, 0.1);
            total_damage += result.drowning_damage;
        }

        assert!(total_damage > 0.0);
        assert!(state.is_drowning());
    }

    #[test]
    fn test_wet_status() {
        let mut state = WaterPlayerState::default();
        assert_eq!(state.wet_status, WetStatus::Dry);

        // Get wet
        for _ in 0..50 {
            state.update(0.5, 0.1);
        }

        assert!(state.wet_status != WetStatus::Dry);
    }

    #[test]
    fn test_wet_drying() {
        let mut state = WaterPlayerState::default();

        // Get soaking wet
        state.wet_status = WetStatus::Soaking;
        state.wet_timer = 60.0;

        // Dry off (takes a while)
        for _ in 0..1000 {
            state.update(0.0, 0.1);
        }

        // Should eventually dry
        assert!(
            state.wet_status == WetStatus::Dry || state.wet_status == WetStatus::Damp,
            "Status: {:?}",
            state.wet_status
        );
    }

    #[test]
    fn test_voluntary_diving() {
        let mut state = WaterPlayerState::default();

        // At swimming depth
        state.update(0.8, 0.1);
        assert_eq!(state.mode, WaterMovementMode::Swimming);

        // Voluntarily dive
        state.start_dive();
        state.update(0.8, 0.1);
        assert_eq!(state.mode, WaterMovementMode::Diving);

        // Surface
        state.stop_dive();
        state.update(0.8, 0.1);
        assert_eq!(state.mode, WaterMovementMode::Swimming);
    }

    #[test]
    fn test_wet_resistance_skill() {
        let mut state = WaterPlayerState::default();
        state.set_wet_resistance(2);

        state.wet_status = WetStatus::Wet;

        // Should have reduced penalty
        let regen_mult = state.get_stamina_regen_multiplier();
        assert!(regen_mult > 0.5); // Better than base wet penalty
    }

    #[test]
    fn test_water_forces() {
        let helper = WaterMovementHelper::default();

        let forces = helper.calculate_water_forces(
            Vec3::new(0.0, -1.0, 0.0),
            0.5,
            Vec3::new(1.0, 0.0, 0.0),
            WaterMovementMode::Swimming,
        );

        // Should have upward buoyancy
        assert!(forces.buoyancy.y > 0.0);

        // Should have swim force
        assert!(forces.swim.x > 0.0);

        // Total should be computable
        let _ = forces.total();
    }

    #[test]
    fn test_water_skills() {
        let mut skills = WaterSkills {
            splash_dash: true,
            splash_dash_cooldown: 5.0,
            splash_dash_timer: 0.0,
            wet_resistance_level: 2,
            deep_diver_level: 1,
            swift_swimmer_level: 1,
        };

        assert!(skills.can_splash_dash());
        skills.use_splash_dash();
        assert!(!skills.can_splash_dash());

        // Update cooldown
        skills.update(6.0);
        assert!(skills.can_splash_dash());

        // Check bonuses
        assert!(skills.oxygen_bonus() > 1.0);
        assert!(skills.swim_speed_bonus() > 1.0);
    }
}
