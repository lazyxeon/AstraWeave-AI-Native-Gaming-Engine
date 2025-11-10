// Advanced quest types for Veilweaver gameplay variety
// Extends basic quest system with: Escort, Defend, TimeTrial, Boss, Collect

use glam::Vec3;

/// NPC data for escort quests
#[derive(Debug, Clone, PartialEq)]
pub struct EscortNPC {
    pub name: String,
    pub health: f32,
    pub max_health: f32,
    pub position: Vec3,
    pub destination: Vec3,
    pub move_speed: f32,
    pub reached_destination: bool,
}

impl EscortNPC {
    pub fn new(name: &str, start: Vec3, destination: Vec3, health: f32) -> Self {
        Self {
            name: name.to_string(),
            health,
            max_health: health,
            position: start,
            destination,
            move_speed: 2.0, // 2 units per second
            reached_destination: false,
        }
    }

    /// Update NPC movement toward destination
    pub fn update(&mut self, delta_time: f32) {
        if self.reached_destination {
            return;
        }

        let direction = (self.destination - self.position).normalize_or_zero();
        let move_distance = self.move_speed * delta_time;
        self.position += direction * move_distance;

        // Check if reached destination (within 1.0 unit threshold)
        if self.position.distance(self.destination) <= 1.0 {
            self.reached_destination = true;
        }
    }

    /// Apply damage to NPC
    pub fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }

    /// Check if NPC is alive
    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    /// Get health percentage (for reward scaling)
    pub fn health_percentage(&self) -> f32 {
        self.health / self.max_health
    }
}

/// Defend quest data (survive waves at location)
#[derive(Debug, Clone, PartialEq)]
pub struct DefendObjective {
    pub location_name: String,
    pub protect_position: Vec3,
    pub protect_radius: f32,
    pub target_health: f32,
    pub current_health: f32,
    pub duration_seconds: f32,
    pub elapsed_seconds: f32,
    pub waves_survived: usize,
    pub total_waves: usize,
}

impl DefendObjective {
    pub fn new(
        location_name: &str,
        position: Vec3,
        radius: f32,
        health: f32,
        duration: f32,
        total_waves: usize,
    ) -> Self {
        Self {
            location_name: location_name.to_string(),
            protect_position: position,
            protect_radius: radius,
            target_health: health,
            current_health: health,
            duration_seconds: duration,
            elapsed_seconds: 0.0,
            waves_survived: 0,
            total_waves,
        }
    }

    /// Update timer
    pub fn update(&mut self, delta_time: f32) {
        self.elapsed_seconds += delta_time;
    }

    /// Apply damage to protected target
    pub fn take_damage(&mut self, amount: f32) {
        self.current_health = (self.current_health - amount).max(0.0);
    }

    /// Mark wave as survived
    pub fn complete_wave(&mut self) {
        self.waves_survived += 1;
    }

    /// Check if objective is complete (survived all waves and time)
    pub fn is_complete(&self) -> bool {
        self.waves_survived >= self.total_waves && self.elapsed_seconds >= self.duration_seconds
    }

    /// Check if objective failed (target destroyed)
    pub fn is_failed(&self) -> bool {
        self.current_health <= 0.0
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        let wave_progress = self.waves_survived as f32 / self.total_waves as f32;
        let time_progress = (self.elapsed_seconds / self.duration_seconds).min(1.0);
        (wave_progress + time_progress) / 2.0
    }
}

/// Time trial quest wrapper (adds time limit to existing objectives)
#[derive(Debug, Clone, PartialEq)]
pub struct TimeTrialObjective {
    pub time_limit_seconds: f32,
    pub elapsed_seconds: f32,
    pub bonus_time_threshold: f32, // Complete before this for bonus reward
}

impl TimeTrialObjective {
    pub fn new(time_limit: f32, bonus_threshold: f32) -> Self {
        Self {
            time_limit_seconds: time_limit,
            elapsed_seconds: 0.0,
            bonus_time_threshold: bonus_threshold,
        }
    }

    /// Update timer
    pub fn update(&mut self, delta_time: f32) {
        self.elapsed_seconds += delta_time;
    }

    /// Check if time expired
    pub fn is_expired(&self) -> bool {
        self.elapsed_seconds >= self.time_limit_seconds
    }

    /// Check if completed within bonus time
    pub fn is_bonus_time(&self) -> bool {
        self.elapsed_seconds <= self.bonus_time_threshold
    }

    /// Get remaining time
    pub fn remaining_time(&self) -> f32 {
        (self.time_limit_seconds - self.elapsed_seconds).max(0.0)
    }

    /// Get progress (1.0 at start, 0.0 at expiration)
    pub fn progress(&self) -> f32 {
        1.0 - (self.elapsed_seconds / self.time_limit_seconds).min(1.0)
    }
}

/// Boss fight phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossPhase {
    Phase1, // 100% - 66% health
    Phase2, // 66% - 33% health
    Phase3, // 33% - 0% health (enraged)
}

/// Boss fight objective
#[derive(Debug, Clone, PartialEq)]
pub struct BossObjective {
    pub boss_name: String,
    pub boss_health: f32,
    pub boss_max_health: f32,
    pub current_phase: BossPhase,
    pub boss_position: Vec3,
    pub arena_center: Vec3,
    pub arena_radius: f32,
    pub special_attack_cooldown: f32,
    pub time_since_special: f32,
}

impl BossObjective {
    pub fn new(name: &str, health: f32, position: Vec3, arena_center: Vec3, arena_radius: f32) -> Self {
        Self {
            boss_name: name.to_string(),
            boss_health: health,
            boss_max_health: health,
            current_phase: BossPhase::Phase1,
            boss_position: position,
            arena_center,
            arena_radius,
            special_attack_cooldown: 10.0, // Special attack every 10 seconds
            time_since_special: 0.0,
        }
    }

    /// Update boss state
    pub fn update(&mut self, delta_time: f32) {
        self.time_since_special += delta_time;
        
        // Update phase based on health
        let health_percentage = self.boss_health / self.boss_max_health;
        self.current_phase = if health_percentage > 0.66 {
            BossPhase::Phase1
        } else if health_percentage > 0.33 {
            BossPhase::Phase2
        } else {
            BossPhase::Phase3
        };
    }

    /// Apply damage to boss
    pub fn take_damage(&mut self, amount: f32) {
        self.boss_health = (self.boss_health - amount).max(0.0);
    }

    /// Check if boss defeated
    pub fn is_defeated(&self) -> bool {
        self.boss_health <= 0.0
    }

    /// Check if special attack ready
    pub fn can_use_special(&self) -> bool {
        self.time_since_special >= self.special_attack_cooldown
    }

    /// Trigger special attack (reset cooldown)
    pub fn use_special(&mut self) {
        self.time_since_special = 0.0;
    }

    /// Get boss health percentage
    pub fn health_percentage(&self) -> f32 {
        self.boss_health / self.boss_max_health
    }

    /// Get phase-specific attack multiplier
    pub fn attack_multiplier(&self) -> f32 {
        match self.current_phase {
            BossPhase::Phase1 => 1.0,
            BossPhase::Phase2 => 1.5,
            BossPhase::Phase3 => 2.0, // Enraged!
        }
    }
}

/// Collectible item data
#[derive(Debug, Clone, PartialEq)]
pub struct CollectItem {
    pub item_name: String,
    pub position: Vec3,
    pub collected: bool,
}

impl CollectItem {
    pub fn new(item_name: &str, position: Vec3) -> Self {
        Self {
            item_name: item_name.to_string(),
            position,
            collected: false,
        }
    }

    /// Check if player can collect (within radius)
    pub fn can_collect(&self, player_pos: Vec3, radius: f32) -> bool {
        !self.collected && self.position.distance(player_pos) <= radius
    }

    /// Mark as collected
    pub fn collect(&mut self) {
        self.collected = true;
    }
}

/// Collection quest objective
#[derive(Debug, Clone, PartialEq)]
pub struct CollectObjective {
    pub items: Vec<CollectItem>,
    pub collection_radius: f32,
    pub required_count: usize,
}

impl CollectObjective {
    pub fn new(item_name: &str, positions: Vec<Vec3>, radius: f32) -> Self {
        let items = positions.iter().map(|&pos| CollectItem::new(item_name, pos)).collect();
        let required_count = positions.len();
        Self {
            items,
            collection_radius: radius,
            required_count,
        }
    }

    /// Try to collect items near player
    pub fn try_collect(&mut self, player_pos: Vec3) -> usize {
        let mut collected_count = 0;
        for item in &mut self.items {
            if item.can_collect(player_pos, self.collection_radius) {
                item.collect();
                collected_count += 1;
            }
        }
        collected_count
    }

    /// Get count of collected items
    pub fn collected_count(&self) -> usize {
        self.items.iter().filter(|item| item.collected).count()
    }

    /// Check if all items collected
    pub fn is_complete(&self) -> bool {
        self.collected_count() >= self.required_count
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        self.collected_count() as f32 / self.required_count as f32
    }

    /// Get uncollected item positions (for UI hints)
    pub fn uncollected_positions(&self) -> Vec<Vec3> {
        self.items.iter()
            .filter(|item| !item.collected)
            .map(|item| item.position)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === EscortNPC Tests ===

    #[test]
    fn test_escort_npc_creation() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let dest = Vec3::new(10.0, 0.0, 0.0);
        let npc = EscortNPC::new("Villager", start, dest, 100.0);
        
        assert_eq!(npc.name, "Villager");
        assert_eq!(npc.health, 100.0);
        assert_eq!(npc.max_health, 100.0);
        assert_eq!(npc.position, start);
        assert_eq!(npc.destination, dest);
        assert!(!npc.reached_destination);
    }

    #[test]
    fn test_escort_npc_movement() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let dest = Vec3::new(10.0, 0.0, 0.0);
        let mut npc = EscortNPC::new("Villager", start, dest, 100.0);
        
        // Move for 1 second (should move 2 units at speed 2.0)
        npc.update(1.0);
        assert_eq!(npc.position.x, 2.0);
        assert!(!npc.reached_destination);
        
        // Move for 4 more seconds (total 10 units)
        for _ in 0..4 {
            npc.update(1.0);
        }
        assert!(npc.reached_destination);
    }

    #[test]
    fn test_escort_npc_damage() {
        let npc_pos = Vec3::ZERO;
        let mut npc = EscortNPC::new("Villager", npc_pos, Vec3::X * 10.0, 100.0);
        
        assert!(npc.is_alive());
        assert_eq!(npc.health_percentage(), 1.0);
        
        npc.take_damage(30.0);
        assert_eq!(npc.health, 70.0);
        assert_eq!(npc.health_percentage(), 0.7);
        
        npc.take_damage(80.0); // Overkill
        assert_eq!(npc.health, 0.0);
        assert!(!npc.is_alive());
    }

    #[test]
    fn test_escort_npc_health_percentage() {
        let npc_pos = Vec3::ZERO;
        let mut npc = EscortNPC::new("Villager", npc_pos, Vec3::X * 10.0, 200.0);
        
        npc.take_damage(50.0);
        assert_eq!(npc.health_percentage(), 0.75);
        
        npc.take_damage(100.0);
        assert_eq!(npc.health_percentage(), 0.25);
    }

    // === DefendObjective Tests ===

    #[test]
    fn test_defend_creation() {
        let defend = DefendObjective::new("Anchor", Vec3::ZERO, 5.0, 100.0, 60.0, 3);
        
        assert_eq!(defend.location_name, "Anchor");
        assert_eq!(defend.protect_position, Vec3::ZERO);
        assert_eq!(defend.protect_radius, 5.0);
        assert_eq!(defend.current_health, 100.0);
        assert_eq!(defend.duration_seconds, 60.0);
        assert_eq!(defend.total_waves, 3);
        assert_eq!(defend.waves_survived, 0);
    }

    #[test]
    fn test_defend_timer() {
        let mut defend = DefendObjective::new("Anchor", Vec3::ZERO, 5.0, 100.0, 60.0, 3);
        
        defend.update(15.0);
        assert_eq!(defend.elapsed_seconds, 15.0);
        
        defend.update(45.0);
        assert_eq!(defend.elapsed_seconds, 60.0);
    }

    #[test]
    fn test_defend_waves() {
        let mut defend = DefendObjective::new("Anchor", Vec3::ZERO, 5.0, 100.0, 60.0, 3);
        
        assert!(!defend.is_complete());
        
        defend.complete_wave();
        assert_eq!(defend.waves_survived, 1);
        assert!(!defend.is_complete()); // Not enough waves
        
        defend.complete_wave();
        defend.complete_wave();
        assert_eq!(defend.waves_survived, 3);
        assert!(!defend.is_complete()); // Not enough time
        
        defend.update(60.0);
        assert!(defend.is_complete()); // Both waves and time
    }

    #[test]
    fn test_defend_failure() {
        let mut defend = DefendObjective::new("Anchor", Vec3::ZERO, 5.0, 100.0, 60.0, 3);
        
        assert!(!defend.is_failed());
        
        defend.take_damage(50.0);
        assert!(!defend.is_failed());
        
        defend.take_damage(60.0); // Destroy target
        assert!(defend.is_failed());
    }

    #[test]
    fn test_defend_progress() {
        let mut defend = DefendObjective::new("Anchor", Vec3::ZERO, 5.0, 100.0, 60.0, 3);
        
        // Initial progress 0%
        assert_eq!(defend.progress(), 0.0);
        
        // Complete 1 wave (33.3%) + 30s time (50%) = 41.65%
        defend.complete_wave();
        defend.update(30.0);
        let progress = defend.progress();
        assert!((progress - 0.4165).abs() < 0.01);
        
        // Complete all waves + full time = 100%
        defend.complete_wave();
        defend.complete_wave();
        defend.update(30.0);
        assert_eq!(defend.progress(), 1.0);
    }

    // === TimeTrialObjective Tests ===

    #[test]
    fn test_time_trial_creation() {
        let timer = TimeTrialObjective::new(120.0, 90.0);
        
        assert_eq!(timer.time_limit_seconds, 120.0);
        assert_eq!(timer.bonus_time_threshold, 90.0);
        assert_eq!(timer.elapsed_seconds, 0.0);
    }

    #[test]
    fn test_time_trial_expiration() {
        let mut timer = TimeTrialObjective::new(60.0, 45.0);
        
        assert!(!timer.is_expired());
        assert!(timer.is_bonus_time());
        
        timer.update(50.0);
        assert!(!timer.is_expired());
        assert!(!timer.is_bonus_time()); // Past bonus threshold
        
        timer.update(15.0); // Total 65s > 60s limit
        assert!(timer.is_expired());
    }

    #[test]
    fn test_time_trial_remaining_time() {
        let mut timer = TimeTrialObjective::new(100.0, 70.0);
        
        assert_eq!(timer.remaining_time(), 100.0);
        
        timer.update(30.0);
        assert_eq!(timer.remaining_time(), 70.0);
        
        timer.update(80.0); // Expired
        assert_eq!(timer.remaining_time(), 0.0);
    }

    #[test]
    fn test_time_trial_progress() {
        let mut timer = TimeTrialObjective::new(100.0, 70.0);
        
        assert_eq!(timer.progress(), 1.0); // Full time remaining
        
        timer.update(50.0);
        assert_eq!(timer.progress(), 0.5); // Half time remaining
        
        timer.update(50.0);
        assert_eq!(timer.progress(), 0.0); // No time remaining
    }

    // === BossObjective Tests ===

    #[test]
    fn test_boss_creation() {
        let boss = BossObjective::new("Void Lord", 500.0, Vec3::ZERO, Vec3::ZERO, 20.0);
        
        assert_eq!(boss.boss_name, "Void Lord");
        assert_eq!(boss.boss_health, 500.0);
        assert_eq!(boss.boss_max_health, 500.0);
        assert_eq!(boss.current_phase, BossPhase::Phase1);
    }

    #[test]
    fn test_boss_phases() {
        let mut boss = BossObjective::new("Void Lord", 300.0, Vec3::ZERO, Vec3::ZERO, 20.0);
        
        assert_eq!(boss.current_phase, BossPhase::Phase1);
        
        boss.take_damage(100.0); // 200/300 = 66.7%
        boss.update(0.0);
        assert_eq!(boss.current_phase, BossPhase::Phase1);
        
        boss.take_damage(50.0); // 150/300 = 50%
        boss.update(0.0);
        assert_eq!(boss.current_phase, BossPhase::Phase2);
        
        boss.take_damage(100.0); // 50/300 = 16.7%
        boss.update(0.0);
        assert_eq!(boss.current_phase, BossPhase::Phase3);
    }

    #[test]
    fn test_boss_defeat() {
        let mut boss = BossObjective::new("Void Lord", 100.0, Vec3::ZERO, Vec3::ZERO, 20.0);
        
        assert!(!boss.is_defeated());
        
        boss.take_damage(100.0);
        assert!(boss.is_defeated());
        assert_eq!(boss.boss_health, 0.0);
    }

    #[test]
    fn test_boss_special_attack() {
        let mut boss = BossObjective::new("Void Lord", 500.0, Vec3::ZERO, Vec3::ZERO, 20.0);
        
        assert!(!boss.can_use_special());
        
        boss.update(10.0);
        assert!(boss.can_use_special());
        
        boss.use_special();
        assert!(!boss.can_use_special()); // Cooldown reset
        
        boss.update(10.0);
        assert!(boss.can_use_special()); // Ready again
    }

    #[test]
    fn test_boss_attack_multiplier() {
        let mut boss = BossObjective::new("Void Lord", 300.0, Vec3::ZERO, Vec3::ZERO, 20.0);
        
        assert_eq!(boss.attack_multiplier(), 1.0); // Phase 1
        
        boss.take_damage(120.0); // Phase 2 (60%)
        boss.update(0.0);
        assert_eq!(boss.attack_multiplier(), 1.5);
        
        boss.take_damage(120.0); // Phase 3 (20%)
        boss.update(0.0);
        assert_eq!(boss.attack_multiplier(), 2.0);
    }

    // === CollectObjective Tests ===

    #[test]
    fn test_collect_creation() {
        let positions = vec![
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(-10.0, 0.0, 0.0),
        ];
        let collect = CollectObjective::new("Echo Shard", positions, 2.0);
        
        assert_eq!(collect.items.len(), 3);
        assert_eq!(collect.required_count, 3);
        assert_eq!(collect.collection_radius, 2.0);
        assert_eq!(collect.collected_count(), 0);
    }

    #[test]
    fn test_collect_items() {
        let positions = vec![
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(20.0, 0.0, 0.0),
        ];
        let mut collect = CollectObjective::new("Echo Shard", positions, 2.0);
        
        // Player at first item
        let count = collect.try_collect(Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(count, 1);
        assert_eq!(collect.collected_count(), 1);
        
        // Player between items (too far from second)
        let count = collect.try_collect(Vec3::new(15.0, 0.0, 0.0));
        assert_eq!(count, 0);
        
        // Player at second item
        let count = collect.try_collect(Vec3::new(20.0, 0.0, 0.0));
        assert_eq!(count, 1);
        assert_eq!(collect.collected_count(), 2);
        assert!(collect.is_complete());
    }

    #[test]
    fn test_collect_progress() {
        let positions = vec![Vec3::ZERO, Vec3::X * 10.0, Vec3::Z * 10.0];
        let mut collect = CollectObjective::new("Echo Shard", positions, 2.0);
        
        assert_eq!(collect.progress(), 0.0);
        
        collect.try_collect(Vec3::ZERO);
        assert!((collect.progress() - 0.333).abs() < 0.01);
        
        collect.try_collect(Vec3::X * 10.0);
        assert!((collect.progress() - 0.666).abs() < 0.01);
        
        collect.try_collect(Vec3::Z * 10.0);
        assert_eq!(collect.progress(), 1.0);
    }

    #[test]
    fn test_collect_uncollected_positions() {
        let positions = vec![Vec3::ZERO, Vec3::X * 10.0, Vec3::Z * 10.0];
        let mut collect = CollectObjective::new("Echo Shard", positions, 2.0);
        
        let uncollected = collect.uncollected_positions();
        assert_eq!(uncollected.len(), 3);
        
        collect.try_collect(Vec3::ZERO);
        let uncollected = collect.uncollected_positions();
        assert_eq!(uncollected.len(), 2);
        assert!(!uncollected.contains(&Vec3::ZERO));
    }

    #[test]
    fn test_collect_multiple_at_once() {
        // Items close together
        let positions = vec![
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(10.5, 0.0, 0.0),
            Vec3::new(11.0, 0.0, 0.0),
        ];
        let mut collect = CollectObjective::new("Echo Shard", positions, 2.0);
        
        // Player near all 3 items (radius 2.0 covers all)
        let count = collect.try_collect(Vec3::new(10.5, 0.0, 0.0));
        assert_eq!(count, 3);
        assert!(collect.is_complete());
    }
}
