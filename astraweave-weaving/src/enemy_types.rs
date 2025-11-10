// Advanced enemy types for Veilweaver combat variety
// Extends basic Enemy with: Riftstalker (fast flanker), Sentinel (tanky AOE), VoidBoss (multi-phase boss)

use glam::Vec3;

/// Enemy archetype determines behavior and stats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyArchetype {
    /// Basic enemy (existing Enemy type)
    Standard,
    /// Fast flanker - low HP, high speed, flanking behavior
    Riftstalker,
    /// Tanky defender - high HP, AOE attack, slow
    Sentinel,
    /// Boss enemy - multi-phase, special attacks
    VoidBoss,
}

/// Riftstalker - Fast flanking enemy
/// - 60 HP (lower than standard 100)
/// - 20 damage (higher than standard 15)
/// - 5.0 move speed (higher than standard 3.0)
/// - Flanking behavior: Circles player, attacks from behind
#[derive(Debug, Clone, PartialEq)]
pub struct Riftstalker {
    pub position: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub move_speed: f32,
    pub attack_cooldown: f32,
    pub time_since_attack: f32,
    pub flanking_angle: f32, // Angle offset for circling player
    pub flanking_radius: f32, // Distance to maintain from player
}

impl Riftstalker {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            health: 60.0,
            max_health: 60.0,
            damage: 20.0,
            move_speed: 5.0,
            attack_cooldown: 1.5, // Fast attacks
            time_since_attack: 0.0,
            flanking_angle: 0.0,
            flanking_radius: 4.0,
        }
    }

    /// Update position to flank player
    pub fn update(&mut self, player_pos: Vec3, delta_time: f32) {
        self.time_since_attack += delta_time;

        // Calculate flanking position (circle around player)
        self.flanking_angle += delta_time * 2.0; // Rotate 2 radians per second
        let offset_x = self.flanking_radius * self.flanking_angle.cos();
        let offset_z = self.flanking_radius * self.flanking_angle.sin();
        let target_pos = player_pos + Vec3::new(offset_x, 0.0, offset_z);

        // Move toward flanking position
        let direction = (target_pos - self.position).normalize_or_zero();
        self.position += direction * self.move_speed * delta_time;
    }

    /// Apply damage
    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }

    /// Check if alive
    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Check if can attack
    pub fn can_attack(&self, player_pos: Vec3) -> bool {
        self.time_since_attack >= self.attack_cooldown 
            && self.position.distance(player_pos) <= 2.0 // Melee range
    }

    /// Execute attack
    pub fn attack(&mut self) -> f32 {
        self.time_since_attack = 0.0;
        self.damage
    }

    /// Get health percentage
    pub fn health_percentage(&self) -> f32 {
        self.health / self.max_health
    }

    /// Check if behind player (successful flank)
    pub fn is_flanking(&self, player_pos: Vec3, player_forward: Vec3) -> bool {
        let to_enemy = (self.position - player_pos).normalize_or_zero();
        let dot = player_forward.dot(to_enemy);
        dot < -0.5 // Behind player (>120 degree angle)
    }

    /// Get damage multiplier for flanking
    pub fn flank_multiplier(&self, player_pos: Vec3, player_forward: Vec3) -> f32 {
        if self.is_flanking(player_pos, player_forward) {
            1.5 // +50% damage from behind
        } else {
            1.0
        }
    }
}

/// Sentinel - Tanky AOE enemy
/// - 200 HP (much higher than standard 100)
/// - 25 damage AOE (affects all in radius)
/// - 1.5 move speed (slower than standard 3.0)
/// - AOE attack: Damages all entities within radius
#[derive(Debug, Clone, PartialEq)]
pub struct Sentinel {
    pub position: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub move_speed: f32,
    pub attack_cooldown: f32,
    pub time_since_attack: f32,
    pub aoe_radius: f32,
    pub armor: f32, // Damage reduction percentage
}

impl Sentinel {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            health: 200.0,
            max_health: 200.0,
            damage: 25.0,
            move_speed: 1.5,
            attack_cooldown: 3.0, // Slow but powerful
            time_since_attack: 0.0,
            aoe_radius: 6.0,
            armor: 0.3, // 30% damage reduction
        }
    }

    /// Update position (move toward player slowly)
    pub fn update(&mut self, player_pos: Vec3, delta_time: f32) {
        self.time_since_attack += delta_time;

        let direction = (player_pos - self.position).normalize_or_zero();
        self.position += direction * self.move_speed * delta_time;
    }

    /// Apply damage (with armor reduction)
    pub fn take_damage(&mut self, amount: f32) {
        let reduced_damage = amount * (1.0 - self.armor);
        self.health = (self.health - reduced_damage).max(0.0);
    }

    /// Check if alive
    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Check if can attack (cooldown ready)
    pub fn can_attack(&self) -> bool {
        self.time_since_attack >= self.attack_cooldown
    }

    /// Execute AOE attack (returns damage and affected entities)
    pub fn attack_aoe(&mut self, entities: &[(Vec3, &str)]) -> Vec<(usize, f32)> {
        self.time_since_attack = 0.0;

        // Find all entities within AOE radius
        entities.iter()
            .enumerate()
            .filter(|(_, (pos, _))| self.position.distance(*pos) <= self.aoe_radius)
            .map(|(idx, _)| (idx, self.damage))
            .collect()
    }

    /// Get health percentage
    pub fn health_percentage(&self) -> f32 {
        self.health / self.max_health
    }

    /// Calculate effective health (accounting for armor)
    pub fn effective_health(&self) -> f32 {
        self.health / (1.0 - self.armor)
    }
}

/// VoidBoss phase states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoidBossPhase {
    Phase1, // 100% - 66% health (normal attacks)
    Phase2, // 66% - 33% health (summons adds)
    Phase3, // 33% - 0% health (enraged, AOE spam)
}

/// Special attack types for VoidBoss
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossSpecialAttack {
    VoidPulse,      // AOE damage around boss
    SummonAdds,     // Spawn 3 standard enemies
    TeleportStrike, // Teleport behind player and attack
}

/// VoidBoss - Multi-phase boss enemy
/// - 500 HP (5x standard)
/// - Phase-based attacks and behavior
/// - Special attacks unique to each phase
#[derive(Debug, Clone, PartialEq)]
pub struct VoidBoss {
    pub position: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub current_phase: VoidBossPhase,
    pub damage: f32,
    pub move_speed: f32,
    pub attack_cooldown: f32,
    pub time_since_attack: f32,
    pub special_cooldown: f32,
    pub time_since_special: f32,
    pub enrage_multiplier: f32,
}

impl VoidBoss {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            health: 500.0,
            max_health: 500.0,
            current_phase: VoidBossPhase::Phase1,
            damage: 40.0,
            move_speed: 2.5,
            attack_cooldown: 2.0,
            time_since_attack: 0.0,
            special_cooldown: 8.0, // Special every 8 seconds
            time_since_special: 0.0,
            enrage_multiplier: 1.0,
        }
    }

    /// Update boss state (phase transitions, cooldowns)
    pub fn update(&mut self, delta_time: f32) {
        self.time_since_attack += delta_time;
        self.time_since_special += delta_time;

        // Update phase based on health
        let health_pct = self.health_percentage();
        let new_phase = if health_pct > 0.66 {
            VoidBossPhase::Phase1
        } else if health_pct > 0.33 {
            VoidBossPhase::Phase2
        } else {
            VoidBossPhase::Phase3
        };

        // Phase transition effects
        if new_phase != self.current_phase {
            self.on_phase_transition(new_phase);
        }

        self.current_phase = new_phase;
    }

    /// Handle phase transition (heal, special cooldown reset, etc.)
    fn on_phase_transition(&mut self, new_phase: VoidBossPhase) {
        match new_phase {
            VoidBossPhase::Phase2 => {
                // Reset special cooldown for immediate summon
                self.time_since_special = self.special_cooldown;
            }
            VoidBossPhase::Phase3 => {
                // Enrage: +50% damage
                self.enrage_multiplier = 1.5;
                // Reset special cooldown for immediate AOE
                self.time_since_special = self.special_cooldown;
            }
            _ => {}
        }
    }

    /// Apply damage
    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }

    /// Check if alive
    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Check if defeated
    pub fn is_defeated(&self) -> bool {
        self.health <= 0.0
    }

    /// Check if can use normal attack
    pub fn can_attack(&self) -> bool {
        self.time_since_attack >= self.attack_cooldown
    }

    /// Execute normal attack
    pub fn attack(&mut self) -> f32 {
        self.time_since_attack = 0.0;
        self.damage * self.enrage_multiplier
    }

    /// Check if can use special attack
    pub fn can_use_special(&self) -> bool {
        self.time_since_special >= self.special_cooldown
    }

    /// Get special attack for current phase
    pub fn get_special_attack(&self) -> BossSpecialAttack {
        match self.current_phase {
            VoidBossPhase::Phase1 => BossSpecialAttack::VoidPulse,
            VoidBossPhase::Phase2 => BossSpecialAttack::SummonAdds,
            VoidBossPhase::Phase3 => BossSpecialAttack::TeleportStrike,
        }
    }

    /// Execute special attack
    pub fn use_special(&mut self) -> BossSpecialAttack {
        self.time_since_special = 0.0;
        self.get_special_attack()
    }

    /// Get health percentage
    pub fn health_percentage(&self) -> f32 {
        self.health / self.max_health
    }

    /// Move toward player (with phase-specific behavior)
    pub fn update_movement(&mut self, player_pos: Vec3, delta_time: f32) {
        let speed = match self.current_phase {
            VoidBossPhase::Phase1 => self.move_speed,
            VoidBossPhase::Phase2 => self.move_speed * 0.8, // Slower when summoning
            VoidBossPhase::Phase3 => self.move_speed * 1.3, // Faster when enraged
        };

        let direction = (player_pos - self.position).normalize_or_zero();
        self.position += direction * speed * delta_time;
    }

    /// Execute teleport strike (teleport behind player)
    pub fn teleport_behind(&mut self, player_pos: Vec3, player_forward: Vec3) {
        // Teleport 5 units behind player
        self.position = player_pos - player_forward * 5.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Riftstalker Tests ===

    #[test]
    fn test_riftstalker_creation() {
        let riftstalker = Riftstalker::new(Vec3::ZERO);
        
        assert_eq!(riftstalker.health, 60.0);
        assert_eq!(riftstalker.max_health, 60.0);
        assert_eq!(riftstalker.damage, 20.0);
        assert_eq!(riftstalker.move_speed, 5.0);
        assert_eq!(riftstalker.attack_cooldown, 1.5);
    }

    #[test]
    fn test_riftstalker_movement() {
        let mut riftstalker = Riftstalker::new(Vec3::ZERO);
        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        
        let initial_angle = riftstalker.flanking_angle;
        riftstalker.update(player_pos, 1.0);
        
        // Angle should increase (circling)
        assert!(riftstalker.flanking_angle > initial_angle);
        
        // Position should change (moving toward flanking position)
        assert_ne!(riftstalker.position, Vec3::ZERO);
    }

    #[test]
    fn test_riftstalker_attack() {
        let mut riftstalker = Riftstalker::new(Vec3::ZERO);
        let player_pos = Vec3::new(1.0, 0.0, 0.0);
        
        // Not ready to attack initially
        assert!(!riftstalker.can_attack(player_pos));
        
        // Manually advance cooldown (test cooldown logic without movement complications)
        riftstalker.time_since_attack = 1.5;
        assert!(riftstalker.can_attack(player_pos));
        
        // Attack
        let damage = riftstalker.attack();
        assert_eq!(damage, 20.0);
        assert!(!riftstalker.can_attack(player_pos)); // Cooldown reset
    }

    #[test]
    fn test_riftstalker_flanking() {
        let mut riftstalker = Riftstalker::new(Vec3::new(0.0, 0.0, -5.0)); // Behind player
        let player_pos = Vec3::ZERO;
        let player_forward = Vec3::new(0.0, 0.0, 1.0); // Facing +Z
        
        // Riftstalker is behind player
        assert!(riftstalker.is_flanking(player_pos, player_forward));
        assert_eq!(riftstalker.flank_multiplier(player_pos, player_forward), 1.5);
        
        // Move Riftstalker in front of player
        riftstalker.position = Vec3::new(0.0, 0.0, 5.0);
        assert!(!riftstalker.is_flanking(player_pos, player_forward));
        assert_eq!(riftstalker.flank_multiplier(player_pos, player_forward), 1.0);
    }

    #[test]
    fn test_riftstalker_damage() {
        let mut riftstalker = Riftstalker::new(Vec3::ZERO);
        
        assert!(riftstalker.is_alive());
        assert_eq!(riftstalker.health_percentage(), 1.0);
        
        riftstalker.take_damage(30.0);
        assert_eq!(riftstalker.health, 30.0);
        assert_eq!(riftstalker.health_percentage(), 0.5);
        
        riftstalker.take_damage(40.0); // Overkill
        assert_eq!(riftstalker.health, 0.0);
        assert!(!riftstalker.is_alive());
    }

    // === Sentinel Tests ===

    #[test]
    fn test_sentinel_creation() {
        let sentinel = Sentinel::new(Vec3::ZERO);
        
        assert_eq!(sentinel.health, 200.0);
        assert_eq!(sentinel.max_health, 200.0);
        assert_eq!(sentinel.damage, 25.0);
        assert_eq!(sentinel.move_speed, 1.5);
        assert_eq!(sentinel.armor, 0.3);
        assert_eq!(sentinel.aoe_radius, 6.0);
    }

    #[test]
    fn test_sentinel_armor() {
        let mut sentinel = Sentinel::new(Vec3::ZERO);
        
        // 100 damage - 30% armor = 70 effective damage
        sentinel.take_damage(100.0);
        assert_eq!(sentinel.health, 130.0);
        
        // Effective health accounting for armor
        let effective = sentinel.effective_health();
        assert!((effective - 185.7).abs() < 0.1); // 130 / 0.7 ≈ 185.7
    }

    #[test]
    fn test_sentinel_aoe_attack() {
        let mut sentinel = Sentinel::new(Vec3::ZERO);
        
        // Entities at various distances
        let entities = vec![
            (Vec3::new(3.0, 0.0, 0.0), "player"),    // Within 6.0 radius
            (Vec3::new(5.0, 0.0, 0.0), "anchor"),    // Within 6.0 radius
            (Vec3::new(10.0, 0.0, 0.0), "enemy"),    // Outside 6.0 radius
        ];
        
        // Wait for cooldown
        sentinel.update(Vec3::ZERO, 3.0);
        assert!(sentinel.can_attack());
        
        // Execute AOE attack
        let hits = sentinel.attack_aoe(&entities);
        
        // Should hit first 2 entities, not the third
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].0, 0); // First entity
        assert_eq!(hits[0].1, 25.0); // Damage
        assert_eq!(hits[1].0, 1); // Second entity
    }

    #[test]
    fn test_sentinel_movement() {
        let mut sentinel = Sentinel::new(Vec3::ZERO);
        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        
        sentinel.update(player_pos, 1.0);
        
        // Should move toward player at speed 1.5
        assert!(sentinel.position.x > 0.0);
        assert!(sentinel.position.x <= 1.5);
    }

    #[test]
    fn test_sentinel_health() {
        let mut sentinel = Sentinel::new(Vec3::ZERO);
        
        assert!(sentinel.is_alive());
        assert_eq!(sentinel.health_percentage(), 1.0);
        
        sentinel.take_damage(100.0); // 70 effective damage
        assert_eq!(sentinel.health, 130.0);
        assert_eq!(sentinel.health_percentage(), 0.65);
        
        sentinel.take_damage(300.0); // Overkill (210 effective damage)
        assert_eq!(sentinel.health, 0.0);
        assert!(!sentinel.is_alive());
    }

    // === VoidBoss Tests ===

    #[test]
    fn test_void_boss_creation() {
        let boss = VoidBoss::new(Vec3::ZERO);
        
        assert_eq!(boss.health, 500.0);
        assert_eq!(boss.max_health, 500.0);
        assert_eq!(boss.damage, 40.0);
        assert_eq!(boss.current_phase, VoidBossPhase::Phase1);
        assert_eq!(boss.enrage_multiplier, 1.0);
    }

    #[test]
    fn test_void_boss_phases() {
        let mut boss = VoidBoss::new(Vec3::ZERO);
        
        assert_eq!(boss.current_phase, VoidBossPhase::Phase1);
        
        // Phase 2 transition (66% health = 330 HP)
        boss.take_damage(170.0); // 500 - 170 = 330
        boss.update(0.0);
        assert_eq!(boss.current_phase, VoidBossPhase::Phase2);
        
        // Phase 3 transition (33% health = 165 HP)
        boss.take_damage(165.0); // 330 - 165 = 165
        boss.update(0.0);
        assert_eq!(boss.current_phase, VoidBossPhase::Phase3);
        assert_eq!(boss.enrage_multiplier, 1.5); // Enraged
    }

    #[test]
    fn test_void_boss_attacks() {
        let mut boss = VoidBoss::new(Vec3::ZERO);
        
        // Normal attack
        boss.update(2.0);
        assert!(boss.can_attack());
        let damage = boss.attack();
        assert_eq!(damage, 40.0);
        
        // Special attack
        boss.update(8.0);
        assert!(boss.can_use_special());
        let special = boss.use_special();
        assert_eq!(special, BossSpecialAttack::VoidPulse); // Phase 1 special
    }

    #[test]
    fn test_void_boss_phase_specials() {
        let mut boss = VoidBoss::new(Vec3::ZERO);
        
        // Phase 1: Void Pulse
        assert_eq!(boss.get_special_attack(), BossSpecialAttack::VoidPulse);
        
        // Phase 2: Summon Adds
        boss.take_damage(200.0); // Enter Phase 2
        boss.update(0.0);
        assert_eq!(boss.get_special_attack(), BossSpecialAttack::SummonAdds);
        
        // Phase 3: Teleport Strike
        boss.take_damage(200.0); // Enter Phase 3
        boss.update(0.0);
        assert_eq!(boss.get_special_attack(), BossSpecialAttack::TeleportStrike);
    }

    #[test]
    fn test_void_boss_enrage() {
        let mut boss = VoidBoss::new(Vec3::ZERO);
        
        // Normal damage
        boss.update(2.0);
        let damage = boss.attack();
        assert_eq!(damage, 40.0);
        
        // Enraged damage (Phase 3)
        boss.take_damage(350.0); // Enter Phase 3
        boss.update(0.0);
        boss.update(2.0);
        let damage_enraged = boss.attack();
        assert_eq!(damage_enraged, 60.0); // 40 * 1.5
    }

    #[test]
    fn test_void_boss_teleport() {
        let mut boss = VoidBoss::new(Vec3::ZERO);
        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        let player_forward = Vec3::new(1.0, 0.0, 0.0);
        
        boss.teleport_behind(player_pos, player_forward);
        
        // Boss should be 5 units behind player
        let expected_pos = Vec3::new(5.0, 0.0, 0.0);
        assert_eq!(boss.position, expected_pos);
    }

    #[test]
    fn test_void_boss_movement() {
        let mut boss = VoidBoss::new(Vec3::ZERO);
        let player_pos = Vec3::new(10.0, 0.0, 0.0);
        
        // Phase 1 movement (normal speed 2.5)
        boss.update_movement(player_pos, 1.0);
        assert!(boss.position.x > 0.0);
        let phase1_dist = boss.position.x;
        
        // Phase 2 movement (slower 2.0)
        boss.position = Vec3::ZERO;
        boss.take_damage(200.0);
        boss.update(0.0);
        boss.update_movement(player_pos, 1.0);
        assert!(boss.position.x < phase1_dist);
        
        // Phase 3 movement (faster 3.25)
        boss.position = Vec3::ZERO;
        boss.take_damage(200.0);
        boss.update(0.0);
        boss.update_movement(player_pos, 1.0);
        assert!(boss.position.x > phase1_dist);
    }

    #[test]
    fn test_void_boss_defeat() {
        let mut boss = VoidBoss::new(Vec3::ZERO);
        
        assert!(boss.is_alive());
        assert!(!boss.is_defeated());
        
        boss.take_damage(500.0);
        assert!(!boss.is_alive());
        assert!(boss.is_defeated());
    }
}
