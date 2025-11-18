//! Combat system for Veilweaver.
//!
//! Handles damage calculation, death detection, and combat events for both player
//! and enemies. Provides a centralized system for all combat interactions.

use glam::Vec3;
use std::collections::VecDeque;

use crate::Enemy;

/// Combat system managing player and enemy health, attacks, and combat events.
#[derive(Debug)]
pub struct CombatSystem {
    /// Player current health (0-100 HP)
    pub player_health: f32,
    /// Player maximum health (100 HP)
    pub player_max_health: f32,
    /// Player base attack damage (HP)
    pub player_attack_damage: f32,
    /// Echo Dash ability damage (HP, AoE)
    pub echo_dash_damage: f32,
    /// Echo Dash AoE radius (units)
    pub echo_dash_radius: f32,
    /// Combat events queue
    events: VecDeque<CombatEvent>,
}

/// Combat event types.
#[derive(Debug, Clone, PartialEq)]
pub enum CombatEvent {
    /// Player took damage
    PlayerDamaged { amount: f32, source_position: Vec3 },
    /// Enemy took damage
    EnemyDamaged {
        enemy_id: usize,
        amount: f32,
        was_critical: bool,
    },
    /// Enemy was killed
    EnemyKilled {
        enemy_id: usize,
        position: Vec3,
        killer: Killer,
    },
    /// Player was killed
    PlayerKilled { killer_position: Vec3 },
}

/// Who killed an enemy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Killer {
    /// Player basic attack
    Player,
    /// Echo Dash ability
    EchoDash,
    /// Environment (e.g., falling off map)
    Environment,
}

impl CombatSystem {
    /// Creates a new combat system with default values.
    ///
    /// # Example
    /// ```
    /// use astraweave_weaving::combat::CombatSystem;
    ///
    /// let combat = CombatSystem::new();
    /// assert_eq!(combat.player_health(), 100.0);
    /// ```
    pub fn new() -> Self {
        Self {
            player_health: 100.0,
            player_max_health: 100.0,
            player_attack_damage: 20.0,
            echo_dash_damage: 50.0,
            echo_dash_radius: 3.0,
            events: VecDeque::new(),
        }
    }

    /// Creates a combat system with custom stats.
    pub fn with_stats(
        player_max_health: f32,
        player_attack_damage: f32,
        echo_dash_damage: f32,
        echo_dash_radius: f32,
    ) -> Self {
        Self {
            player_health: player_max_health,
            player_max_health,
            player_attack_damage,
            echo_dash_damage,
            echo_dash_radius,
            events: VecDeque::new(),
        }
    }

    /// Returns current player health.
    pub fn player_health(&self) -> f32 {
        self.player_health
    }

    /// Returns player health as percentage (0.0 to 1.0).
    pub fn player_health_percentage(&self) -> f32 {
        self.player_health / self.player_max_health
    }

    /// Checks if player is alive.
    pub fn is_player_alive(&self) -> bool {
        self.player_health > 0.0
    }

    /// Player performs basic attack on enemy.
    ///
    /// # Arguments
    /// * `enemy_id` - ID of target enemy
    /// * `enemy` - Mutable reference to enemy
    /// * `enemy_position` - Position of enemy
    ///
    /// # Returns
    /// `Some(CombatEvent)` if enemy was damaged/killed, `None` otherwise
    ///
    /// # Example
    /// ```
    /// use astraweave_weaving::combat::CombatSystem;
    /// use astraweave_weaving::enemy::Enemy;
    /// use glam::Vec3;
    ///
    /// let mut combat = CombatSystem::new();
    /// let mut enemy = Enemy::new(Vec3::ZERO, 5.0);
    ///
    /// let event = combat.player_attack(0, &mut enemy, Vec3::ZERO);
    /// assert!(event.is_some());
    /// assert_eq!(enemy.health, 80.0); // 100 - 20 damage
    /// ```
    pub fn player_attack(
        &mut self,
        enemy_id: usize,
        enemy: &mut Enemy,
        enemy_position: Vec3,
    ) -> Option<CombatEvent> {
        let damage = self.player_attack_damage;
        let was_critical = false; // TODO: Critical hit system in future

        let was_killed = enemy.take_damage(damage);

        if was_killed {
            let event = CombatEvent::EnemyKilled {
                enemy_id,
                position: enemy_position,
                killer: Killer::Player,
            };
            self.events.push_back(event.clone());
            Some(event)
        } else {
            let event = CombatEvent::EnemyDamaged {
                enemy_id,
                amount: damage,
                was_critical,
            };
            self.events.push_back(event.clone());
            Some(event)
        }
    }

    /// Player performs Echo Dash attack (AoE damage).
    ///
    /// # Arguments
    /// * `position` - Center position of Echo Dash
    /// * `enemies` - List of (enemy_id, enemy, enemy_position) tuples
    ///
    /// # Returns
    /// Vector of combat events (one per enemy hit)
    ///
    /// # Example
    /// ```
    /// use astraweave_weaving::combat::CombatSystem;
    /// use astraweave_weaving::enemy::Enemy;
    /// use glam::Vec3;
    ///
    /// let mut combat = CombatSystem::new();
    /// let mut enemy1 = Enemy::new(Vec3::new(1.0, 0.0, 0.0), 5.0);
    /// let mut enemy2 = Enemy::new(Vec3::new(5.0, 0.0, 0.0), 5.0); // Out of range
    ///
    /// let enemies = vec![
    ///     (0, &mut enemy1, Vec3::new(1.0, 0.0, 0.0)),
    ///     (1, &mut enemy2, Vec3::new(5.0, 0.0, 0.0)),
    /// ];
    ///
    /// let events = combat.echo_dash_attack(Vec3::ZERO, enemies);
    /// assert_eq!(events.len(), 1); // Only enemy1 hit
    /// assert_eq!(enemy1.health, 50.0); // 100 - 50 damage
    /// ```
    pub fn echo_dash_attack(
        &mut self,
        position: Vec3,
        enemies: Vec<(usize, &mut Enemy, Vec3)>,
    ) -> Vec<CombatEvent> {
        let mut events = Vec::new();

        for (enemy_id, enemy, enemy_position) in enemies {
            let distance = position.distance(enemy_position);

            if distance <= self.echo_dash_radius {
                let damage = self.echo_dash_damage;
                let was_critical = false;

                let was_killed = enemy.take_damage(damage);

                if was_killed {
                    let event = CombatEvent::EnemyKilled {
                        enemy_id,
                        position: enemy_position,
                        killer: Killer::EchoDash,
                    };
                    self.events.push_back(event.clone());
                    events.push(event);
                } else {
                    let event = CombatEvent::EnemyDamaged {
                        enemy_id,
                        amount: damage,
                        was_critical,
                    };
                    self.events.push_back(event.clone());
                    events.push(event);
                }
            }
        }

        events
    }

    /// Enemy attacks player.
    ///
    /// # Arguments
    /// * `enemy` - Mutable reference to attacking enemy
    /// * `enemy_position` - Position of attacking enemy
    ///
    /// # Returns
    /// `Some(CombatEvent)` if player was damaged/killed, `None` if enemy on cooldown
    ///
    /// # Example
    /// ```
    /// use astraweave_weaving::combat::CombatSystem;
    /// use astraweave_weaving::enemy::Enemy;
    /// use glam::Vec3;
    ///
    /// let mut combat = CombatSystem::new();
    /// let mut enemy = Enemy::new(Vec3::ZERO, 5.0);
    ///
    /// let event = combat.enemy_attack(&mut enemy, Vec3::new(1.0, 0.0, 0.0));
    /// assert!(event.is_some());
    /// assert_eq!(combat.player_health(), 90.0); // 100 - 10 damage
    /// ```
    pub fn enemy_attack(&mut self, enemy: &mut Enemy, enemy_position: Vec3) -> Option<CombatEvent> {
        if !enemy.can_attack() {
            return None;
        }

        let damage = enemy.attack();
        self.player_health = (self.player_health - damage).max(0.0);

        if self.player_health <= 0.0 {
            let event = CombatEvent::PlayerKilled {
                killer_position: enemy_position,
            };
            self.events.push_back(event.clone());
            Some(event)
        } else {
            let event = CombatEvent::PlayerDamaged {
                amount: damage,
                source_position: enemy_position,
            };
            self.events.push_back(event.clone());
            Some(event)
        }
    }

    /// Heals the player.
    ///
    /// # Example
    /// ```
    /// use astraweave_weaving::combat::CombatSystem;
    ///
    /// let mut combat = CombatSystem::new();
    /// combat.damage_player(50.0);
    /// combat.heal_player(30.0);
    /// assert_eq!(combat.player_health(), 80.0);
    /// ```
    pub fn heal_player(&mut self, amount: f32) {
        self.player_health = (self.player_health + amount).min(self.player_max_health);
    }

    /// Damages the player directly (e.g., environmental damage).
    ///
    /// # Returns
    /// `true` if player was killed, `false` otherwise
    pub fn damage_player(&mut self, amount: f32) -> bool {
        self.player_health = (self.player_health - amount).max(0.0);

        if self.player_health <= 0.0 {
            let event = CombatEvent::PlayerKilled {
                killer_position: Vec3::ZERO, // Environmental
            };
            self.events.push_back(event);
            true
        } else {
            false
        }
    }

    /// Polls combat events (drains event queue).
    ///
    /// # Example
    /// ```
    /// use astraweave_weaving::combat::CombatSystem;
    /// use astraweave_weaving::enemy::Enemy;
    /// use glam::Vec3;
    ///
    /// let mut combat = CombatSystem::new();
    /// let mut enemy = Enemy::new(Vec3::ZERO, 5.0);
    ///
    /// combat.player_attack(0, &mut enemy, Vec3::ZERO);
    /// let events = combat.poll_events();
    /// assert_eq!(events.len(), 1);
    ///
    /// // Events are consumed
    /// let events = combat.poll_events();
    /// assert_eq!(events.len(), 0);
    /// ```
    pub fn poll_events(&mut self) -> Vec<CombatEvent> {
        self.events.drain(..).collect()
    }

    /// Peeks at combat events without consuming them.
    pub fn peek_events(&self) -> Vec<CombatEvent> {
        self.events.iter().cloned().collect()
    }

    /// Clears all combat events.
    pub fn clear_events(&mut self) {
        self.events.clear();
    }
}

impl Default for CombatSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_system_creation() {
        let combat = CombatSystem::new();
        assert_eq!(combat.player_health(), 100.0);
        assert_eq!(combat.player_max_health, 100.0);
        assert_eq!(combat.player_attack_damage, 20.0);
        assert_eq!(combat.echo_dash_damage, 50.0);
        assert_eq!(combat.echo_dash_radius, 3.0);
        assert!(combat.is_player_alive());
    }

    #[test]
    fn test_combat_system_with_stats() {
        let combat = CombatSystem::with_stats(150.0, 30.0, 75.0, 5.0);
        assert_eq!(combat.player_health(), 150.0);
        assert_eq!(combat.player_attack_damage, 30.0);
        assert_eq!(combat.echo_dash_damage, 75.0);
        assert_eq!(combat.echo_dash_radius, 5.0);
    }

    #[test]
    fn test_player_attack() {
        let mut combat = CombatSystem::new();
        let mut enemy = Enemy::new(Vec3::ZERO, 5.0);

        let event = combat.player_attack(0, &mut enemy, Vec3::ZERO);
        assert!(event.is_some());

        if let Some(CombatEvent::EnemyDamaged {
            enemy_id,
            amount,
            was_critical,
        }) = event
        {
            assert_eq!(enemy_id, 0);
            assert_eq!(amount, 20.0);
            assert!(!was_critical);
        } else {
            panic!("Expected EnemyDamaged event");
        }

        assert_eq!(enemy.health, 80.0);
    }

    #[test]
    fn test_player_attack_kill() {
        let mut combat = CombatSystem::new();
        let mut enemy = Enemy::new(Vec3::ZERO, 5.0);
        enemy.take_damage(95.0); // Leave 5 HP

        let event = combat.player_attack(0, &mut enemy, Vec3::new(5.0, 0.0, 3.0));
        assert!(event.is_some());

        if let Some(CombatEvent::EnemyKilled {
            enemy_id,
            position,
            killer,
        }) = event
        {
            assert_eq!(enemy_id, 0);
            assert_eq!(position, Vec3::new(5.0, 0.0, 3.0));
            assert_eq!(killer, Killer::Player);
        } else {
            panic!("Expected EnemyKilled event");
        }

        assert_eq!(enemy.health, 0.0);
        assert!(enemy.is_dead());
    }

    #[test]
    fn test_echo_dash_attack() {
        let mut combat = CombatSystem::new();
        let mut enemy1 = Enemy::new(Vec3::ZERO, 5.0);
        let mut enemy2 = Enemy::new(Vec3::ZERO, 5.0);
        let mut enemy3 = Enemy::new(Vec3::ZERO, 5.0);

        let enemies = vec![
            (0, &mut enemy1, Vec3::new(1.0, 0.0, 0.0)), // In range
            (1, &mut enemy2, Vec3::new(2.5, 0.0, 0.0)), // In range
            (2, &mut enemy3, Vec3::new(5.0, 0.0, 0.0)), // Out of range
        ];

        let events = combat.echo_dash_attack(Vec3::ZERO, enemies);
        assert_eq!(events.len(), 2); // Only 2 enemies hit

        assert_eq!(enemy1.health, 50.0); // 100 - 50
        assert_eq!(enemy2.health, 50.0); // 100 - 50
        assert_eq!(enemy3.health, 100.0); // Not hit
    }

    #[test]
    fn test_echo_dash_kill() {
        let mut combat = CombatSystem::new();
        let mut enemy = Enemy::new(Vec3::ZERO, 5.0);
        enemy.take_damage(60.0); // Leave 40 HP

        let enemies = vec![(0, &mut enemy, Vec3::new(1.0, 0.0, 0.0))];
        let events = combat.echo_dash_attack(Vec3::ZERO, enemies);

        assert_eq!(events.len(), 1);
        if let CombatEvent::EnemyKilled { killer, .. } = &events[0] {
            assert_eq!(*killer, Killer::EchoDash);
        } else {
            panic!("Expected EnemyKilled event");
        }

        assert!(enemy.is_dead());
    }

    #[test]
    fn test_enemy_attack() {
        let mut combat = CombatSystem::new();
        let mut enemy = Enemy::new(Vec3::ZERO, 5.0);

        let event = combat.enemy_attack(&mut enemy, Vec3::new(2.0, 0.0, 1.0));
        assert!(event.is_some());

        if let Some(CombatEvent::PlayerDamaged {
            amount,
            source_position,
        }) = event
        {
            assert_eq!(amount, 10.0);
            assert_eq!(source_position, Vec3::new(2.0, 0.0, 1.0));
        } else {
            panic!("Expected PlayerDamaged event");
        }

        assert_eq!(combat.player_health(), 90.0);
        assert!(combat.is_player_alive());
    }

    #[test]
    fn test_enemy_attack_cooldown() {
        let mut combat = CombatSystem::new();
        let mut enemy = Enemy::new(Vec3::ZERO, 5.0);

        // First attack succeeds
        let event1 = combat.enemy_attack(&mut enemy, Vec3::ZERO);
        assert!(event1.is_some());

        // Second attack fails (cooldown)
        let event2 = combat.enemy_attack(&mut enemy, Vec3::ZERO);
        assert!(event2.is_none());

        assert_eq!(combat.player_health(), 90.0); // Only 1 attack landed
    }

    #[test]
    fn test_enemy_attack_kill_player() {
        let mut combat = CombatSystem::new();
        combat.damage_player(95.0); // Leave 5 HP

        let mut enemy = Enemy::new(Vec3::ZERO, 5.0);
        let event = combat.enemy_attack(&mut enemy, Vec3::new(3.0, 0.0, 2.0));

        if let Some(CombatEvent::PlayerKilled { killer_position }) = event {
            assert_eq!(killer_position, Vec3::new(3.0, 0.0, 2.0));
        } else {
            panic!("Expected PlayerKilled event");
        }

        assert_eq!(combat.player_health(), 0.0);
        assert!(!combat.is_player_alive());
    }

    #[test]
    fn test_heal_player() {
        let mut combat = CombatSystem::new();
        combat.damage_player(60.0);
        assert_eq!(combat.player_health(), 40.0);

        combat.heal_player(30.0);
        assert_eq!(combat.player_health(), 70.0);

        // Can't overheal
        combat.heal_player(50.0);
        assert_eq!(combat.player_health(), 100.0);
    }

    #[test]
    fn test_damage_player() {
        let mut combat = CombatSystem::new();

        let killed = combat.damage_player(30.0);
        assert!(!killed);
        assert_eq!(combat.player_health(), 70.0);

        let killed = combat.damage_player(80.0);
        assert!(killed);
        assert_eq!(combat.player_health(), 0.0);
    }

    #[test]
    fn test_player_health_percentage() {
        let mut combat = CombatSystem::new();
        assert_eq!(combat.player_health_percentage(), 1.0);

        combat.damage_player(50.0);
        assert_eq!(combat.player_health_percentage(), 0.5);

        combat.damage_player(50.0);
        assert_eq!(combat.player_health_percentage(), 0.0);
    }

    #[test]
    fn test_event_polling() {
        let mut combat = CombatSystem::new();
        let mut enemy = Enemy::new(Vec3::ZERO, 5.0);

        // Generate events
        combat.player_attack(0, &mut enemy, Vec3::ZERO);
        combat.player_attack(0, &mut enemy, Vec3::ZERO);

        // Poll events
        let events = combat.poll_events();
        assert_eq!(events.len(), 2);

        // Events consumed
        let events = combat.poll_events();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_event_peeking() {
        let mut combat = CombatSystem::new();
        let mut enemy = Enemy::new(Vec3::ZERO, 5.0);

        combat.player_attack(0, &mut enemy, Vec3::ZERO);

        // Peek doesn't consume
        let events1 = combat.peek_events();
        assert_eq!(events1.len(), 1);

        let events2 = combat.peek_events();
        assert_eq!(events2.len(), 1);

        // Poll does consume
        let events3 = combat.poll_events();
        assert_eq!(events3.len(), 1);

        let events4 = combat.peek_events();
        assert_eq!(events4.len(), 0);
    }

    #[test]
    fn test_clear_events() {
        let mut combat = CombatSystem::new();
        let mut enemy = Enemy::new(Vec3::ZERO, 5.0);

        combat.player_attack(0, &mut enemy, Vec3::ZERO);
        assert_eq!(combat.peek_events().len(), 1);

        combat.clear_events();
        assert_eq!(combat.peek_events().len(), 0);
    }
}
