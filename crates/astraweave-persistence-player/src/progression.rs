//! Progression tracking (unlocks, achievements, stats)

use crate::PlayerProfile;

impl PlayerProfile {
    /// Unlock an ability
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use astraweave_persistence_player::PlayerProfile;
    /// let mut profile = PlayerProfile::default();
    /// profile.unlock_ability("Dash");
    /// profile.unlock_ability("Double Jump");
    /// ```
    pub fn unlock_ability(&mut self, ability: &str) {
        if !self.unlocks.abilities.contains(&ability.to_string()) {
            self.unlocks.abilities.push(ability.to_string());
            println!("✨ Ability Unlocked: {}", ability);
        }
    }

    /// Unlock an item
    pub fn unlock_item(&mut self, item: &str) {
        if !self.unlocks.items.contains(&item.to_string()) {
            self.unlocks.items.push(item.to_string());
            println!("✨ Item Unlocked: {}", item);
        }
    }

    /// Unlock a level
    pub fn unlock_level(&mut self, level: &str) {
        if !self.unlocks.levels.contains(&level.to_string()) {
            self.unlocks.levels.push(level.to_string());
            println!("✨ Level Unlocked: {}", level);
        }
    }

    /// Grant achievement
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use astraweave_persistence_player::PlayerProfile;
    /// let mut profile = PlayerProfile::default();
    /// profile.grant_achievement("First Blood");
    /// profile.grant_achievement("Level 10 Reached");
    /// ```
    pub fn grant_achievement(&mut self, achievement: &str) {
        if !self.stats.achievements.contains(&achievement.to_string()) {
            self.stats.achievements.push(achievement.to_string());
            println!("🏆 Achievement Unlocked: {}", achievement);
        }
    }

    /// Record an enemy kill
    pub fn record_kill(&mut self) {
        self.stats.enemies_defeated += 1;
    }

    /// Record a player death
    pub fn record_death(&mut self) {
        self.stats.deaths += 1;
    }

    /// Add playtime (in seconds)
    pub fn add_playtime(&mut self, seconds: u64) {
        self.stats.playtime_seconds += seconds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlock_ability() {
        let mut profile = PlayerProfile::default();

        profile.unlock_ability("Dash");
        assert_eq!(profile.unlocks.abilities.len(), 1);
        assert!(profile.unlocks.abilities.contains(&"Dash".to_string()));

        // Duplicate unlock should not add twice
        profile.unlock_ability("Dash");
        assert_eq!(profile.unlocks.abilities.len(), 1);
    }

    #[test]
    fn test_grant_achievement() {
        let mut profile = PlayerProfile::default();

        profile.grant_achievement("First Blood");
        assert_eq!(profile.stats.achievements.len(), 1);

        // Duplicate achievement should not add twice
        profile.grant_achievement("First Blood");
        assert_eq!(profile.stats.achievements.len(), 1);
    }

    #[test]
    fn test_stats_tracking() {
        let mut profile = PlayerProfile::default();

        profile.record_kill();
        profile.record_kill();
        profile.record_death();
        profile.add_playtime(3600); // 1 hour

        assert_eq!(profile.stats.enemies_defeated, 2);
        assert_eq!(profile.stats.deaths, 1);
        assert_eq!(profile.stats.playtime_seconds, 3600);
    }

    #[test]
    fn test_unlock_item() {
        let mut profile = PlayerProfile::default();

        profile.unlock_item("Sword");
        assert_eq!(profile.unlocks.items.len(), 1);
        assert!(profile.unlocks.items.contains(&"Sword".to_string()));

        // Duplicate should not add twice
        profile.unlock_item("Sword");
        assert_eq!(profile.unlocks.items.len(), 1);

        // Different item should add
        profile.unlock_item("Shield");
        assert_eq!(profile.unlocks.items.len(), 2);
    }

    #[test]
    fn test_unlock_level() {
        let mut profile = PlayerProfile::default();

        profile.unlock_level("level_01");
        assert_eq!(profile.unlocks.levels.len(), 1);
        assert!(profile.unlocks.levels.contains(&"level_01".to_string()));

        // Duplicate should not add twice
        profile.unlock_level("level_01");
        assert_eq!(profile.unlocks.levels.len(), 1);

        // Different level should add
        profile.unlock_level("level_02");
        assert_eq!(profile.unlocks.levels.len(), 2);
    }

    #[test]
    fn test_multiple_achievements() {
        let mut profile = PlayerProfile::default();

        profile.grant_achievement("First Blood");
        profile.grant_achievement("Level 10");
        profile.grant_achievement("Speed Runner");

        assert_eq!(profile.stats.achievements.len(), 3);
    }

    #[test]
    fn test_combined_progression() {
        let mut profile = PlayerProfile::default();

        // Unlock some content
        profile.unlock_ability("Dash");
        profile.unlock_ability("Double Jump");
        profile.unlock_item("Sword");
        profile.unlock_level("level_01");

        // Track some stats
        for _ in 0..10 {
            profile.record_kill();
        }
        profile.record_death();
        profile.add_playtime(7200); // 2 hours

        // Grant achievement
        profile.grant_achievement("First Blood");

        assert_eq!(profile.unlocks.abilities.len(), 2);
        assert_eq!(profile.unlocks.items.len(), 1);
        assert_eq!(profile.unlocks.levels.len(), 1);
        assert_eq!(profile.stats.enemies_defeated, 10);
        assert_eq!(profile.stats.deaths, 1);
        assert_eq!(profile.stats.playtime_seconds, 7200);
        assert_eq!(profile.stats.achievements.len(), 1);
    }
}
