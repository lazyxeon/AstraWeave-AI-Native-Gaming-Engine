//! Anchor Component - Core loom node system for Veilweaver
//!
//! Anchors are stabilized points in the game world that players can interact with
//! to repair reality distortions, unlock abilities, and deploy tactical structures.
//!
//! # Stability States
//! - **Perfect** (1.0): Pristine loom node, bright blue glow, 440 Hz hum
//! - **Stable** (0.7-0.99): Normal operation, dim blue glow, flickering hum
//! - **Unstable** (0.4-0.69): Reality warping, yellow glow, distorted hum
//! - **Critical** (0.1-0.39): Imminent failure, red glow, harsh static
//! - **Broken** (0.0): Inoperable, no glow, silence
//!
//! # Decay Mechanics
//! - Passive decay: -0.01 stability per 60 seconds (-1%/min)
//! - Combat stress: -0.05 stability per nearby enemy kill (-5%/kill)
//! - Repair bonus: +0.3 stability per repair (+30%)
//!
//! # Example
//! ```rust
//! use astraweave_weaving::anchor::{Anchor, AnchorVfxState, AbilityType};
//!
//! // Create a perfect anchor (Z0 tutorial)
//! let anchor = Anchor::new(1.0, 5, None);
//! assert_eq!(anchor.vfx_state(), AnchorVfxState::Perfect);
//!
//! // Create a decaying anchor with ability unlock (Z2 vista)
//! let vista_anchor = Anchor::new(0.7, 2, Some(AbilityType::EchoDash));
//! assert_eq!(vista_anchor.vfx_state(), AnchorVfxState::Stable);
//!
//! // Create a broken anchor for tactical use (Z1 combat)
//! let combat_anchor = Anchor::new(0.0, 1, None);
//! assert_eq!(combat_anchor.vfx_state(), AnchorVfxState::Broken);
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Anchor component - represents a loom node in the game world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anchor {
    /// Current stability (0.0 = broken, 1.0 = perfect)
    stability: f32,
    
    /// Decay rate per second (default: -0.01/60 = -0.000166)
    decay_rate: f32,
    
    /// Repair cost in Echoes
    repair_cost: u32,
    
    /// Current VFX state (computed from stability)
    #[serde(skip)]
    vfx_state: AnchorVfxState,
    
    /// Ability unlocked when repaired (if any)
    unlocks_ability: Option<AbilityType>,
    
    /// Proximity radius for interaction (default: 3m)
    proximity_radius: f32,
    
    /// Whether anchor has been repaired at least once
    repaired: bool,
    
    /// Time since last repair (for animation timing)
    time_since_repair: f32,
}

impl Anchor {
    /// Default decay rate: -0.01 stability per 60 seconds
    pub const DEFAULT_DECAY_RATE: f32 = -0.01 / 60.0;
    
    /// Combat stress decay: -0.05 stability per kill
    pub const COMBAT_STRESS_DECAY: f32 = -0.05;
    
    /// Repair bonus: +0.3 stability per repair
    pub const REPAIR_BONUS: f32 = 0.3;
    
    /// Default proximity radius: 3 meters
    pub const DEFAULT_PROXIMITY: f32 = 3.0;
    
    /// Repair animation duration: 5 seconds
    pub const REPAIR_ANIMATION_DURATION: f32 = 5.0;
    
    /// Create a new anchor with specified stability and repair cost
    ///
    /// # Arguments
    /// * `stability` - Initial stability (0.0-1.0)
    /// * `repair_cost` - Cost in Echoes to repair
    /// * `unlocks_ability` - Ability granted when repaired (optional)
    ///
    /// # Example
    /// ```rust
    /// use astraweave_weaving::anchor::{Anchor, AbilityType};
    ///
    /// // Z0 tutorial anchor (perfect, expensive)
    /// let z0_anchor = Anchor::new(1.0, 5, None);
    ///
    /// // Z2 vista anchor (decaying, unlocks Echo Dash)
    /// let z2_anchor = Anchor::new(0.7, 2, Some(AbilityType::EchoDash));
    ///
    /// // Z1 combat anchor (broken, cheap)
    /// let z1_anchor = Anchor::new(0.0, 1, None);
    /// ```
    pub fn new(stability: f32, repair_cost: u32, unlocks_ability: Option<AbilityType>) -> Self {
        let stability = stability.clamp(0.0, 1.0);
        let vfx_state = AnchorVfxState::from_stability(stability);
        
        Self {
            stability,
            decay_rate: Self::DEFAULT_DECAY_RATE,
            repair_cost,
            vfx_state,
            unlocks_ability,
            proximity_radius: Self::DEFAULT_PROXIMITY,
            repaired: false,
            time_since_repair: 0.0,
        }
    }
    
    /// Get current stability (0.0-1.0)
    pub fn stability(&self) -> f32 {
        self.stability
    }
    
    /// Get stability as percentage (0-100)
    pub fn stability_percent(&self) -> u32 {
        (self.stability * 100.0).round() as u32
    }
    
    /// Get current VFX state
    pub fn vfx_state(&self) -> AnchorVfxState {
        self.vfx_state
    }
    
    /// Get repair cost in Echoes
    pub fn repair_cost(&self) -> u32 {
        self.repair_cost
    }
    
    /// Get ability unlocked when repaired (if any)
    pub fn unlocks_ability(&self) -> Option<AbilityType> {
        self.unlocks_ability
    }
    
    /// Get proximity radius for interaction
    pub fn proximity_radius(&self) -> f32 {
        self.proximity_radius
    }
    
    /// Check if anchor has been repaired at least once
    pub fn is_repaired(&self) -> bool {
        self.repaired
    }
    
    /// Check if anchor is currently repairing (animation in progress)
    pub fn is_repairing(&self) -> bool {
        self.repaired && self.time_since_repair < Self::REPAIR_ANIMATION_DURATION
    }
    
    /// Apply passive decay (called per frame)
    ///
    /// # Arguments
    /// * `delta_time` - Time since last frame in seconds
    pub fn apply_decay(&mut self, delta_time: f32) {
        if self.stability > 0.0 {
            self.stability = (self.stability + self.decay_rate * delta_time).max(0.0);
            self.update_vfx_state();
        }
    }
    
    /// Apply combat stress decay (called when nearby enemy killed)
    pub fn apply_combat_stress(&mut self) {
        if self.stability > 0.0 {
            self.stability = (self.stability + Self::COMBAT_STRESS_DECAY).max(0.0);
            self.update_vfx_state();
        }
    }
    
    /// Manually adjust stability (for testing/integration)
    ///
    /// # Arguments
    /// * `delta` - Amount to adjust (positive = heal, negative = damage)
    ///
    /// # Examples
    /// ```
    /// let mut anchor = Anchor::new(0.5, 50, None);
    /// anchor.adjust_stability(-0.2); // Damage: 0.5 → 0.3
    /// anchor.adjust_stability(0.1);  // Heal: 0.3 → 0.4
    /// ```
    #[allow(dead_code)] // Used in integration tests
    pub(crate) fn adjust_stability(&mut self, delta: f32) {
        self.stability = (self.stability + delta).clamp(0.0, 1.0);
        self.update_vfx_state();
    }
    
    /// Repair anchor (restore stability, mark as repaired)
    ///
    /// Returns `true` if repair was successful, `false` if already at max stability
    pub fn repair(&mut self) -> bool {
        if self.stability >= 1.0 {
            return false;
        }
        
        self.stability = (self.stability + Self::REPAIR_BONUS).min(1.0);
        self.repaired = true;
        self.time_since_repair = 0.0;
        self.update_vfx_state();
        
        true
    }
    
    /// Update time since repair (for animation timing)
    pub fn update_repair_timer(&mut self, delta_time: f32) {
        if self.is_repairing() {
            self.time_since_repair += delta_time;
        }
    }
    
    /// Get repair animation progress (0.0-1.0)
    pub fn repair_animation_progress(&self) -> f32 {
        if !self.is_repairing() {
            return 1.0;
        }
        (self.time_since_repair / Self::REPAIR_ANIMATION_DURATION).min(1.0)
    }
    
    /// Update VFX state based on current stability
    fn update_vfx_state(&mut self) {
        self.vfx_state = AnchorVfxState::from_stability(self.stability);
    }
    
    /// Check if player is within proximity radius
    ///
    /// # Arguments
    /// * `player_pos` - Player position (x, y, z)
    /// * `anchor_pos` - Anchor position (x, y, z)
    pub fn is_in_proximity(
        &self,
        player_pos: (f32, f32, f32),
        anchor_pos: (f32, f32, f32),
    ) -> bool {
        let dx = player_pos.0 - anchor_pos.0;
        let dy = player_pos.1 - anchor_pos.1;
        let dz = player_pos.2 - anchor_pos.2;
        let distance_squared = dx * dx + dy * dy + dz * dz;
        distance_squared <= self.proximity_radius * self.proximity_radius
    }
}

/// VFX state computed from anchor stability
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorVfxState {
    /// Perfect (1.0): Bright blue glow, 440 Hz hum, no distortion
    Perfect,
    
    /// Stable (0.7-0.99): Dim blue glow, flickering hum, rare glitches
    Stable,
    
    /// Unstable (0.4-0.69): Yellow glow, distorted hum, frequent glitches
    Unstable,
    
    /// Critical (0.1-0.39): Red glow, harsh static, reality tears
    Critical,
    
    /// Broken (0.0): No glow, silence, inoperable
    Broken,
}

impl Default for AnchorVfxState {
    fn default() -> Self {
        Self::Broken
    }
}

impl AnchorVfxState {
    /// Compute VFX state from stability value
    pub fn from_stability(stability: f32) -> Self {
        if stability >= 1.0 {
            Self::Perfect
        } else if stability >= 0.7 {
            Self::Stable
        } else if stability >= 0.4 {
            Self::Unstable
        } else if stability >= 0.1 {
            Self::Critical
        } else {
            Self::Broken
        }
    }
    
    /// Get glow color (RGB, 0.0-1.0)
    pub fn glow_color(&self) -> (f32, f32, f32) {
        match self {
            Self::Perfect => (0.3, 0.7, 1.0),   // Bright blue
            Self::Stable => (0.2, 0.5, 0.8),    // Dim blue
            Self::Unstable => (0.9, 0.8, 0.2),  // Yellow
            Self::Critical => (1.0, 0.2, 0.2),  // Red
            Self::Broken => (0.0, 0.0, 0.0),    // No glow
        }
    }
    
    /// Get hum frequency (Hz)
    pub fn hum_frequency(&self) -> f32 {
        match self {
            Self::Perfect => 440.0,   // Pure A4
            Self::Stable => 430.0,    // Slightly flat
            Self::Unstable => 400.0,  // Distorted
            Self::Critical => 350.0,  // Harsh
            Self::Broken => 0.0,      // Silence
        }
    }
    
    /// Get particle emission rate (particles/second)
    pub fn particle_emission_rate(&self) -> f32 {
        match self {
            Self::Perfect => 0.0,     // No decay particles
            Self::Stable => 5.0,      // Few particles
            Self::Unstable => 20.0,   // Many particles
            Self::Critical => 50.0,   // Intense particles
            Self::Broken => 0.0,      // No particles (inoperable)
        }
    }
}

impl fmt::Display for AnchorVfxState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Perfect => write!(f, "Perfect"),
            Self::Stable => write!(f, "Stable"),
            Self::Unstable => write!(f, "Unstable"),
            Self::Critical => write!(f, "Critical"),
            Self::Broken => write!(f, "Broken"),
        }
    }
}

/// Ability types that can be unlocked by repairing anchors
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbilityType {
    /// Echo Dash: 5m teleport (1 Echo per use)
    /// Unlocked by Z2 vista_tutorial_anchor repair
    EchoDash,
    
    /// Barricade Deploy: 2m × 2m × 1m tactical cover
    /// Unlocked by Z1 cover anchor repair
    BarricadeDeploy,
    
    // Future abilities (Week 3+)
    // GravityShift,
    // TimeFlow,
    // PhaseShift,
}

impl fmt::Display for AbilityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EchoDash => write!(f, "Echo Dash"),
            Self::BarricadeDeploy => write!(f, "Barricade Deploy"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_anchor_creation() {
        let anchor = Anchor::new(1.0, 5, None);
        assert_eq!(anchor.stability(), 1.0);
        assert_eq!(anchor.stability_percent(), 100);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Perfect);
        assert_eq!(anchor.repair_cost(), 5);
        assert_eq!(anchor.unlocks_ability(), None);
        assert!(!anchor.is_repaired());
    }
    
    #[test]
    fn test_anchor_with_ability() {
        let anchor = Anchor::new(0.7, 2, Some(AbilityType::EchoDash));
        assert_eq!(anchor.stability(), 0.7);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Stable);
        assert_eq!(anchor.unlocks_ability(), Some(AbilityType::EchoDash));
    }
    
    #[test]
    fn test_passive_decay() {
        let mut anchor = Anchor::new(1.0, 5, None);
        
        // Simulate 60 seconds of decay (should lose 0.01 stability)
        for _ in 0..60 {
            anchor.apply_decay(1.0);
        }
        
        assert!((anchor.stability() - 0.99).abs() < 0.001);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Stable);
    }
    
    #[test]
    fn test_combat_stress() {
        let mut anchor = Anchor::new(1.0, 5, None);
        
        // Simulate 5 nearby kills (should lose 0.25 stability)
        for _ in 0..5 {
            anchor.apply_combat_stress();
        }
        
        assert!((anchor.stability() - 0.75).abs() < 0.001);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Stable);
    }
    
    #[test]
    fn test_repair() {
        let mut anchor = Anchor::new(0.5, 2, None);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Unstable);
        
        let repaired = anchor.repair();
        assert!(repaired);
        assert!((anchor.stability() - 0.8).abs() < 0.001);
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Stable);
        assert!(anchor.is_repaired());
        assert!(anchor.is_repairing());
    }
    
    #[test]
    fn test_repair_animation() {
        let mut anchor = Anchor::new(0.5, 2, None);
        anchor.repair();
        
        assert_eq!(anchor.repair_animation_progress(), 0.0);
        
        anchor.update_repair_timer(2.5);
        assert!((anchor.repair_animation_progress() - 0.5).abs() < 0.001);
        
        anchor.update_repair_timer(2.5);
        assert_eq!(anchor.repair_animation_progress(), 1.0);
        assert!(!anchor.is_repairing());
    }
    
    #[test]
    fn test_vfx_state_transitions() {
        let mut anchor = Anchor::new(1.0, 5, None);
        
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Perfect);
        
        anchor.stability = 0.8;
        anchor.update_vfx_state();
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Stable);
        
        anchor.stability = 0.5;
        anchor.update_vfx_state();
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Unstable);
        
        anchor.stability = 0.2;
        anchor.update_vfx_state();
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Critical);
        
        anchor.stability = 0.0;
        anchor.update_vfx_state();
        assert_eq!(anchor.vfx_state(), AnchorVfxState::Broken);
    }
    
    #[test]
    fn test_proximity_detection() {
        let anchor = Anchor::new(1.0, 5, None);
        
        // Player at anchor position (0m distance)
        assert!(anchor.is_in_proximity((0.0, 0.0, 0.0), (0.0, 0.0, 0.0)));
        
        // Player 2m away (within 3m radius)
        assert!(anchor.is_in_proximity((2.0, 0.0, 0.0), (0.0, 0.0, 0.0)));
        
        // Player 3m away (exactly at radius)
        assert!(anchor.is_in_proximity((3.0, 0.0, 0.0), (0.0, 0.0, 0.0)));
        
        // Player 4m away (outside radius)
        assert!(!anchor.is_in_proximity((4.0, 0.0, 0.0), (0.0, 0.0, 0.0)));
    }
    
    #[test]
    fn test_vfx_state_properties() {
        assert_eq!(AnchorVfxState::Perfect.glow_color(), (0.3, 0.7, 1.0));
        assert_eq!(AnchorVfxState::Perfect.hum_frequency(), 440.0);
        assert_eq!(AnchorVfxState::Perfect.particle_emission_rate(), 0.0);
        
        assert_eq!(AnchorVfxState::Unstable.glow_color(), (0.9, 0.8, 0.2));
        assert_eq!(AnchorVfxState::Unstable.hum_frequency(), 400.0);
        assert_eq!(AnchorVfxState::Unstable.particle_emission_rate(), 20.0);
        
        assert_eq!(AnchorVfxState::Broken.glow_color(), (0.0, 0.0, 0.0));
        assert_eq!(AnchorVfxState::Broken.hum_frequency(), 0.0);
        assert_eq!(AnchorVfxState::Broken.particle_emission_rate(), 0.0);
    }
}
