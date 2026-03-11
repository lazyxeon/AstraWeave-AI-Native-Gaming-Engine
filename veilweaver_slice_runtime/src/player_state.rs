//! Player state model — tracks the player's position, zone, health, and
//! gameplay resources throughout the vertical slice walkthrough.
//!
//! Pure data model — headless-safe, no rendering code.

use astraweave_scene::world_partition::GridCoord;
use serde::{Deserialize, Serialize};

// ── Player state ───────────────────────────────────────────────────────────

/// Core player state tracked across the vertical slice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerState {
    /// Current zone coordinate.
    pub zone: Option<GridCoord>,
    /// Current zone name.
    pub zone_name: String,
    /// Player world position (x, y, z).
    pub position: [f32; 3],
    /// Current hit points.
    pub hp: f32,
    /// Maximum hit points.
    pub max_hp: f32,
    /// Thread stability (0.0–1.0). Core gameplay resource for fate-weaving.
    pub thread_stability: f32,
    /// Echo shard balance. Currency for weaving abilities.
    pub echoes: u32,
    /// Number of anchors the player has repaired.
    pub anchors_repaired: u32,
    /// Whether the tutorial is complete.
    pub tutorial_complete: bool,
    /// Whether the player is alive (hp > 0).
    pub alive: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerState {
    /// Creates a new player at Z0 with full health.
    #[must_use]
    pub fn new() -> Self {
        Self {
            zone: None,
            zone_name: String::new(),
            position: [0.0, 0.0, 0.0],
            hp: 100.0,
            max_hp: 100.0,
            thread_stability: 1.0,
            echoes: 0,
            anchors_repaired: 0,
            tutorial_complete: false,
            alive: true,
        }
    }

    /// Moves the player to a new zone.
    pub fn enter_zone(&mut self, zone_name: impl Into<String>, coord: GridCoord) {
        self.zone_name = zone_name.into();
        self.zone = Some(coord);
    }

    /// Sets the player's world position.
    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = pos;
    }

    /// Applies damage to the player. Returns `true` if the player died.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        if !self.alive || amount <= 0.0 || !amount.is_finite() {
            return false;
        }
        self.hp = (self.hp - amount).max(0.0);
        if self.hp <= 0.0 {
            self.alive = false;
            return true;
        }
        false
    }

    /// Heals the player. Clamped to max_hp.
    pub fn heal(&mut self, amount: f32) {
        if !self.alive || !amount.is_finite() || amount <= 0.0 {
            return;
        }
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Adjusts thread stability. Clamped to 0.0–1.0.
    pub fn adjust_stability(&mut self, delta: f32) {
        if !delta.is_finite() {
            return;
        }
        self.thread_stability = (self.thread_stability + delta).clamp(0.0, 1.0);
    }

    /// Collects echo shards.
    pub fn collect_echoes(&mut self, amount: u32) {
        self.echoes = self.echoes.saturating_add(amount);
    }

    /// Spends echoes. Returns `false` if insufficient balance.
    pub fn spend_echoes(&mut self, amount: u32) -> bool {
        if self.echoes >= amount {
            self.echoes -= amount;
            true
        } else {
            false
        }
    }

    /// Records an anchor repair.
    pub fn repair_anchor(&mut self) {
        self.anchors_repaired = self.anchors_repaired.saturating_add(1);
    }

    /// Marks the tutorial as complete.
    pub fn complete_tutorial(&mut self) {
        self.tutorial_complete = true;
    }

    /// Returns the HP fraction (0.0–1.0).
    #[must_use]
    pub fn hp_fraction(&self) -> f32 {
        if self.max_hp <= 0.0 {
            return 0.0;
        }
        (self.hp / self.max_hp).clamp(0.0, 1.0)
    }

    /// Returns `true` if the player is at full health.
    #[must_use]
    pub fn is_full_health(&self) -> bool {
        (self.hp - self.max_hp).abs() < f32::EPSILON
    }

    /// Respawn (reset hp, keep echoes/progress).
    pub fn respawn(&mut self) {
        self.hp = self.max_hp;
        self.alive = true;
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player_defaults() {
        let p = PlayerState::new();
        assert!(p.alive);
        assert_eq!(p.hp, 100.0);
        assert_eq!(p.echoes, 0);
        assert_eq!(p.thread_stability, 1.0);
        assert!(!p.tutorial_complete);
        assert!(p.zone.is_none());
    }

    #[test]
    fn zone_entry() {
        let mut p = PlayerState::new();
        p.enter_zone("Z0_loomspire_sanctum", GridCoord::new(100, 0, 0));
        assert_eq!(p.zone_name, "Z0_loomspire_sanctum");
        assert_eq!(p.zone, Some(GridCoord::new(100, 0, 0)));
    }

    #[test]
    fn damage_and_death() {
        let mut p = PlayerState::new();
        let died = p.take_damage(60.0);
        assert!(!died);
        assert_eq!(p.hp, 40.0);

        let died = p.take_damage(50.0);
        assert!(died);
        assert!(!p.alive);
        assert_eq!(p.hp, 0.0);
    }

    #[test]
    fn heal_capped_at_max() {
        let mut p = PlayerState::new();
        p.take_damage(30.0);
        p.heal(50.0);
        assert_eq!(p.hp, p.max_hp);
    }

    #[test]
    fn dead_player_cannot_take_damage() {
        let mut p = PlayerState::new();
        p.take_damage(200.0);
        assert!(!p.alive);
        let died = p.take_damage(50.0);
        assert!(!died); // Already dead
    }

    #[test]
    fn dead_player_cannot_heal() {
        let mut p = PlayerState::new();
        p.take_damage(200.0);
        p.heal(50.0);
        assert_eq!(p.hp, 0.0);
    }

    #[test]
    fn stability_clamped() {
        let mut p = PlayerState::new();
        p.adjust_stability(-0.5);
        assert_eq!(p.thread_stability, 0.5);
        p.adjust_stability(2.0);
        assert_eq!(p.thread_stability, 1.0);
        p.adjust_stability(-5.0);
        assert_eq!(p.thread_stability, 0.0);
    }

    #[test]
    fn nan_guards() {
        let mut p = PlayerState::new();
        p.take_damage(f32::NAN);
        assert_eq!(p.hp, 100.0);
        p.heal(f32::NAN);
        assert_eq!(p.hp, 100.0);
        p.adjust_stability(f32::NAN);
        assert_eq!(p.thread_stability, 1.0);
    }

    #[test]
    fn echo_collection_and_spending() {
        let mut p = PlayerState::new();
        p.collect_echoes(5);
        assert_eq!(p.echoes, 5);
        assert!(p.spend_echoes(3));
        assert_eq!(p.echoes, 2);
        assert!(!p.spend_echoes(10));
        assert_eq!(p.echoes, 2);
    }

    #[test]
    fn anchor_repair_and_tutorial() {
        let mut p = PlayerState::new();
        p.repair_anchor();
        p.repair_anchor();
        assert_eq!(p.anchors_repaired, 2);
        assert!(!p.tutorial_complete);
        p.complete_tutorial();
        assert!(p.tutorial_complete);
    }

    #[test]
    fn respawn_restores_hp() {
        let mut p = PlayerState::new();
        p.take_damage(200.0);
        assert!(!p.alive);
        p.respawn();
        assert!(p.alive);
        assert_eq!(p.hp, 100.0);
    }

    #[test]
    fn hp_fraction() {
        let mut p = PlayerState::new();
        assert_eq!(p.hp_fraction(), 1.0);
        p.take_damage(50.0);
        assert_eq!(p.hp_fraction(), 0.5);
    }
}
