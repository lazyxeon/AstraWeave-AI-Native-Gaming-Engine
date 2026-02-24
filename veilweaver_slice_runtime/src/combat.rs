//! Combat encounter system — manages Z2 combat gauntlet and future encounters.
//!
//! Models headless combat encounters as state machines: idle → active → cleared.
//! Each encounter has a wave of enemies, tracks kills, and emits events when
//! waves are cleared or the encounter finishes.
//!
//! Pure data model — headless-safe, no rendering or physics code.

use serde::{Deserialize, Serialize};
use tracing::info;

// ── Combat events ──────────────────────────────────────────────────────────

/// Events emitted by the combat system.
#[derive(Debug, Clone, PartialEq)]
pub enum CombatEvent {
    /// A new encounter has started.
    EncounterStarted {
        encounter_id: String,
        enemy_count: u32,
    },
    /// An enemy has been defeated in the current encounter.
    EnemyDefeated {
        encounter_id: String,
        remaining: u32,
    },
    /// A combat wave has been cleared.
    WaveCleared {
        encounter_id: String,
        wave: u32,
        next_wave: bool,
    },
    /// The entire encounter has been cleared.
    EncounterCleared { encounter_id: String },
    /// The player took damage during combat.
    PlayerDamaged { amount: f32, source: String },
    /// A combo was successfully landed.
    ComboLanded { combo_hits: u32 },
}

impl std::fmt::Display for CombatEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncounterStarted {
                encounter_id,
                enemy_count,
            } => {
                write!(
                    f,
                    "Encounter '{encounter_id}' started ({enemy_count} enemies)"
                )
            }
            Self::EnemyDefeated {
                encounter_id,
                remaining,
            } => {
                write!(
                    f,
                    "Enemy defeated in '{encounter_id}' ({remaining} remaining)"
                )
            }
            Self::WaveCleared {
                encounter_id,
                wave,
                next_wave,
            } => {
                write!(
                    f,
                    "Wave {wave} cleared in '{encounter_id}' (next: {next_wave})"
                )
            }
            Self::EncounterCleared { encounter_id } => {
                write!(f, "Encounter '{encounter_id}' cleared")
            }
            Self::PlayerDamaged { amount, source } => {
                write!(f, "Player damaged {amount:.1} by {source}")
            }
            Self::ComboLanded { combo_hits } => {
                write!(f, "Combo landed ({combo_hits} hits)")
            }
        }
    }
}

// ── Enemy template ─────────────────────────────────────────────────────────

/// A lightweight headless enemy for tracking combat state.
#[derive(Debug, Clone, PartialEq)]
pub struct CombatEnemy {
    pub id: u32,
    pub name: String,
    pub hp: f32,
    pub max_hp: f32,
    pub damage: f32,
    pub alive: bool,
}

impl CombatEnemy {
    /// Creates a new enemy.
    #[must_use]
    pub fn new(id: u32, name: impl Into<String>, hp: f32, damage: f32) -> Self {
        Self {
            id,
            name: name.into(),
            hp,
            max_hp: hp,
            damage,
            alive: true,
        }
    }

    /// Applies damage to this enemy. Returns `true` if the enemy died.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        if !self.alive || !amount.is_finite() || amount <= 0.0 {
            return false;
        }
        self.hp = (self.hp - amount).max(0.0);
        if self.hp <= 0.0 {
            self.alive = false;
            return true;
        }
        false
    }
}

// ── Combat wave ────────────────────────────────────────────────────────────

/// A wave of enemies in an encounter.
#[derive(Debug, Clone)]
pub struct CombatWave {
    pub enemies: Vec<CombatEnemy>,
}

impl CombatWave {
    /// Creates a wave from enemy templates.
    #[must_use]
    pub fn new(enemies: Vec<CombatEnemy>) -> Self {
        Self { enemies }
    }

    /// Returns the number of living enemies.
    #[must_use]
    pub fn alive_count(&self) -> u32 {
        self.enemies.iter().filter(|e| e.alive).count() as u32
    }

    /// Returns `true` if all enemies are dead.
    #[must_use]
    pub fn is_cleared(&self) -> bool {
        self.enemies.iter().all(|e| !e.alive)
    }
}

// ── Encounter state ────────────────────────────────────────────────────────

/// Lifecycle of a combat encounter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncounterPhase {
    /// Encounter not yet triggered.
    Idle,
    /// Combat in progress.
    Active,
    /// All waves cleared.
    Cleared,
}

impl std::fmt::Display for EncounterPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Active => write!(f, "Active"),
            Self::Cleared => write!(f, "Cleared"),
        }
    }
}

// ── Combat encounter ───────────────────────────────────────────────────────

/// A multi-wave combat encounter.
#[derive(Debug, Clone)]
pub struct CombatEncounter {
    pub encounter_id: String,
    pub waves: Vec<CombatWave>,
    pub current_wave: usize,
    pub phase: EncounterPhase,
    pub total_kills: u32,
    pending_events: Vec<CombatEvent>,
}

impl CombatEncounter {
    /// Creates a new idle encounter.
    #[must_use]
    pub fn new(id: impl Into<String>, waves: Vec<CombatWave>) -> Self {
        Self {
            encounter_id: id.into(),
            waves,
            current_wave: 0,
            phase: EncounterPhase::Idle,
            total_kills: 0,
            pending_events: Vec::new(),
        }
    }

    /// Creates the default Z2 Echo Grove skirmish encounter.
    ///
    /// Two waves: 3 Threadborne + 2 Threadborne Brutes.
    #[must_use]
    pub fn z2_skirmish() -> Self {
        let wave1 = CombatWave::new(vec![
            CombatEnemy::new(1, "Threadborne Scout", 40.0, 8.0),
            CombatEnemy::new(2, "Threadborne Scout", 40.0, 8.0),
            CombatEnemy::new(3, "Threadborne Stalker", 60.0, 12.0),
        ]);
        let wave2 = CombatWave::new(vec![
            CombatEnemy::new(4, "Threadborne Brute", 80.0, 15.0),
            CombatEnemy::new(5, "Threadborne Brute", 80.0, 15.0),
        ]);
        Self::new("z2_skirmish", vec![wave1, wave2])
    }

    /// Starts the encounter. Transitions Idle → Active.
    pub fn start(&mut self) {
        if self.phase != EncounterPhase::Idle {
            return;
        }
        self.phase = EncounterPhase::Active;
        self.current_wave = 0;
        let count = self.current_alive_count();
        info!(
            "Combat encounter '{}' started — wave 1/{}, {} enemies",
            self.encounter_id,
            self.waves.len(),
            count
        );
        self.pending_events.push(CombatEvent::EncounterStarted {
            encounter_id: self.encounter_id.clone(),
            enemy_count: count,
        });
    }

    /// Simulates dealing damage to a specific enemy.
    ///
    /// Returns `true` if the enemy was killed.
    pub fn damage_enemy(&mut self, enemy_id: u32, amount: f32) -> bool {
        if self.phase != EncounterPhase::Active {
            return false;
        }

        if let Some(wave) = self.waves.get_mut(self.current_wave) {
            if let Some(enemy) = wave
                .enemies
                .iter_mut()
                .find(|e| e.id == enemy_id && e.alive)
            {
                let killed = enemy.take_damage(amount);
                if killed {
                    self.total_kills += 1;
                    self.pending_events.push(CombatEvent::EnemyDefeated {
                        encounter_id: self.encounter_id.clone(),
                        remaining: wave.alive_count(),
                    });
                    self.check_wave_advancement();
                }
                return killed;
            }
        }
        false
    }

    /// Simulates killing all remaining enemies in the current wave.
    ///
    /// Useful for testing and fast-forwarding.
    pub fn clear_current_wave(&mut self) {
        if self.phase != EncounterPhase::Active {
            return;
        }
        if let Some(wave) = self.waves.get_mut(self.current_wave) {
            let alive_ids: Vec<u32> = wave
                .enemies
                .iter()
                .filter(|e| e.alive)
                .map(|e| e.id)
                .collect();
            for id in alive_ids {
                self.damage_enemy(id, 9999.0);
            }
        }
    }

    /// Checks if the current wave is cleared and advances if necessary.
    fn check_wave_advancement(&mut self) {
        let wave_cleared = self
            .waves
            .get(self.current_wave)
            .map(|w| w.is_cleared())
            .unwrap_or(false);

        if !wave_cleared {
            return;
        }

        let has_next = self.current_wave + 1 < self.waves.len();
        self.pending_events.push(CombatEvent::WaveCleared {
            encounter_id: self.encounter_id.clone(),
            wave: self.current_wave as u32,
            next_wave: has_next,
        });

        if has_next {
            self.current_wave += 1;
            info!(
                "Combat encounter '{}': wave {}/{} — {} enemies",
                self.encounter_id,
                self.current_wave + 1,
                self.waves.len(),
                self.current_alive_count()
            );
        } else {
            self.phase = EncounterPhase::Cleared;
            info!(
                "Combat encounter '{}' CLEARED — {} total kills",
                self.encounter_id, self.total_kills
            );
            self.pending_events.push(CombatEvent::EncounterCleared {
                encounter_id: self.encounter_id.clone(),
            });
        }
    }

    /// Drains pending combat events.
    pub fn drain_events(&mut self) -> Vec<CombatEvent> {
        std::mem::take(&mut self.pending_events)
    }

    // ── Queries ────────────────────────────────────────────────────────

    /// Returns the current encounter phase.
    #[must_use]
    pub fn phase(&self) -> EncounterPhase {
        self.phase
    }

    /// Returns `true` if the encounter is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.phase == EncounterPhase::Active
    }

    /// Returns `true` if the encounter is fully cleared.
    #[must_use]
    pub fn is_cleared(&self) -> bool {
        self.phase == EncounterPhase::Cleared
    }

    /// Returns the number of living enemies in the current wave.
    #[must_use]
    pub fn current_alive_count(&self) -> u32 {
        self.waves
            .get(self.current_wave)
            .map(|w| w.alive_count())
            .unwrap_or(0)
    }

    /// Returns total enemies across all waves.
    #[must_use]
    pub fn total_enemy_count(&self) -> u32 {
        self.waves.iter().map(|w| w.enemies.len() as u32).sum()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn z2_skirmish_has_two_waves() {
        let enc = CombatEncounter::z2_skirmish();
        assert_eq!(enc.waves.len(), 2);
        assert_eq!(enc.total_enemy_count(), 5);
        assert_eq!(enc.phase(), EncounterPhase::Idle);
    }

    #[test]
    fn encounter_start_and_kill() {
        let mut enc = CombatEncounter::z2_skirmish();
        enc.start();
        assert!(enc.is_active());
        assert_eq!(enc.current_alive_count(), 3);

        let killed = enc.damage_enemy(1, 50.0);
        assert!(killed);
        assert_eq!(enc.current_alive_count(), 2);
        assert_eq!(enc.total_kills, 1);

        let events = enc.drain_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, CombatEvent::EncounterStarted { .. })));
        assert!(events
            .iter()
            .any(|e| matches!(e, CombatEvent::EnemyDefeated { .. })));
    }

    #[test]
    fn wave_advancement() {
        let mut enc = CombatEncounter::z2_skirmish();
        enc.start();
        enc.drain_events();

        // Clear wave 1.
        enc.clear_current_wave();
        assert_eq!(enc.current_wave, 1);
        assert!(enc.is_active());

        let events = enc.drain_events();
        assert!(events.iter().any(|e| matches!(
            e,
            CombatEvent::WaveCleared {
                next_wave: true,
                ..
            }
        )));
    }

    #[test]
    fn full_encounter_clear() {
        let mut enc = CombatEncounter::z2_skirmish();
        enc.start();
        enc.drain_events();

        enc.clear_current_wave(); // Wave 1
        enc.drain_events();
        enc.clear_current_wave(); // Wave 2
        assert!(enc.is_cleared());
        assert_eq!(enc.total_kills, 5);

        let events = enc.drain_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, CombatEvent::EncounterCleared { .. })));
    }

    #[test]
    fn idle_encounter_ignores_damage() {
        let mut enc = CombatEncounter::z2_skirmish();
        let killed = enc.damage_enemy(1, 9999.0);
        assert!(!killed);
    }

    #[test]
    fn enemy_nan_guard() {
        let mut enemy = CombatEnemy::new(1, "Test", 50.0, 10.0);
        let killed = enemy.take_damage(f32::NAN);
        assert!(!killed);
        assert_eq!(enemy.hp, 50.0);
    }

    #[test]
    fn cleared_encounter_ignores_damage() {
        let mut enc = CombatEncounter::z2_skirmish();
        enc.start();
        enc.clear_current_wave();
        enc.clear_current_wave();
        assert!(enc.is_cleared());
        let killed = enc.damage_enemy(99, 100.0);
        assert!(!killed);
    }
}
