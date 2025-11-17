// Quest system for Veilweaver gameplay progression
// Supports multiple objective types (kill, repair, fetch, explore) with progress tracking

use glam::Vec3;
use std::collections::HashMap;

/// Quest state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestState {
    /// Quest exists but not yet activated
    Inactive,
    /// Quest is currently active and tracking progress
    Active,
    /// Quest objectives completed successfully
    Completed,
    /// Quest failed (optional failure conditions)
    Failed,
}

/// Objective types for quest progression
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveType {
    /// Kill N enemies of specific type
    Kill {
        target_type: String,
        required: usize,
        current: usize,
    },
    /// Repair N anchors to stability threshold
    Repair {
        required: usize,
        current: usize,
        min_stability: f32,
    },
    /// Collect N items and deliver to location
    Fetch {
        item_name: String,
        required: usize,
        current: usize,
        delivery_location: Vec3,
    },
    /// Reach specific location or discover area
    Explore {
        location_name: String,
        target_position: Vec3,
        radius: f32,
        discovered: bool,
    },
    /// Escort NPC to destination (from quest_types.rs)
    Escort {
        npc: crate::quest_types::EscortNPC,
    },
    /// Defend objective from waves (from quest_types.rs)
    Defend {
        objective: crate::quest_types::DefendObjective,
        required_waves: usize,
    },
    /// Time trial objective (from quest_types.rs)
    TimeTrial {
        objective: crate::quest_types::TimeTrialObjective,
    },
    /// Boss fight objective (from quest_types.rs)
    Boss {
        objective: crate::quest_types::BossObjective,
    },
    /// Collect scattered items (from quest_types.rs)
    Collect {
        objective: crate::quest_types::CollectObjective,
    },
}

impl ObjectiveType {
    /// Check if objective is complete
    pub fn is_complete(&self) -> bool {
        match self {
            ObjectiveType::Kill { required, current, .. } => current >= required,
            ObjectiveType::Repair { required, current, .. } => current >= required,
            ObjectiveType::Fetch { required, current, .. } => current >= required,
            ObjectiveType::Explore { discovered, .. } => *discovered,
            ObjectiveType::Escort { npc } => npc.reached_destination && npc.health > 0.0,
            ObjectiveType::Defend { objective, required_waves } => {
                objective.waves_survived >= *required_waves && objective.current_health > 0.0
            }
            ObjectiveType::TimeTrial { objective } => !objective.is_expired(),
            ObjectiveType::Boss { objective } => objective.is_defeated(),
            ObjectiveType::Collect { objective } => objective.is_complete(),
        }
    }

    /// Get progress as percentage (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        match self {
            ObjectiveType::Kill { required, current, .. } => {
                (*current as f32 / *required as f32).min(1.0)
            }
            ObjectiveType::Repair { required, current, .. } => {
                (*current as f32 / *required as f32).min(1.0)
            }
            ObjectiveType::Fetch { required, current, .. } => {
                (*current as f32 / *required as f32).min(1.0)
            }
            ObjectiveType::Explore { discovered, .. } => {
                if *discovered { 1.0 } else { 0.0 }
            }
            ObjectiveType::Escort { npc } => {
                if npc.reached_destination { 1.0 } else { 0.5 }
            }
            ObjectiveType::Defend { objective, required_waves } => {
                (objective.waves_survived as f32 / *required_waves as f32).min(1.0)
            }
            ObjectiveType::TimeTrial { objective } => {
                1.0 - (objective.elapsed_seconds / objective.time_limit_seconds).min(1.0)
            }
            ObjectiveType::Boss { objective } => {
                1.0 - (objective.boss_health / 300.0).max(0.0)
            }
            ObjectiveType::Collect { objective } => {
                let collected = objective.items.iter().filter(|i| i.collected).count();
                collected as f32 / objective.items.len() as f32
            }
        }
    }

    /// Get description for UI display
    pub fn description(&self) -> String {
        match self {
            ObjectiveType::Kill { target_type, required, current } => {
                format!("Kill {}/{} {}", current, required, target_type)
            }
            ObjectiveType::Repair { required, current, min_stability } => {
                format!("Repair {}/{} anchors to {:.0}% stability", current, required, min_stability * 100.0)
            }
            ObjectiveType::Fetch { item_name, required, current, .. } => {
                format!("Collect {}/{} {}", current, required, item_name)
            }
            ObjectiveType::Explore { location_name, discovered, .. } => {
                format!("Explore {} {}", location_name, if *discovered { "✓" } else { "" })
            }
            ObjectiveType::Escort { npc } => {
                format!("Escort {} ({:.0}% health)", npc.name, npc.health_percentage() * 100.0)
            }
            ObjectiveType::Defend { objective, required_waves } => {
                format!("Defend: Wave {}/{} ({:.0} HP)", objective.waves_survived, required_waves, objective.current_health)
            }
            ObjectiveType::TimeTrial { objective } => {
                format!("Time Trial: {:.1}s remaining", objective.remaining_time())
            }
            ObjectiveType::Boss { objective } => {
                format!("Boss Fight: {:.0} HP ({:?})", objective.boss_health, objective.current_phase)
            }
            ObjectiveType::Collect { objective } => {
                let collected = objective.items.iter().filter(|i| i.collected).count();
                format!("Collect items: {}/{}", collected, objective.items.len())
            }
        }
    }
}

/// Quest reward types
#[derive(Debug, Clone, PartialEq)]
pub enum QuestReward {
    /// Echo currency reward
    EchoCurrency(i32),
    /// Unlock new ability
    AbilityUnlock(String),
    /// Increase max health/stamina
    StatBoost { stat: String, amount: f32 },
    /// Multiple rewards
    Multiple(Vec<QuestReward>),
}

/// Complete quest definition with objectives and rewards
#[derive(Debug, Clone, PartialEq)]
pub struct Quest {
    /// Unique quest identifier
    pub id: String,
    /// Display name
    pub title: String,
    /// Quest description for UI
    pub description: String,
    /// Current quest state
    pub state: QuestState,
    /// List of objectives (all must be completed)
    pub objectives: Vec<ObjectiveType>,
    /// Rewards granted on completion
    pub rewards: Vec<QuestReward>,
    /// Optional prerequisite quest IDs
    pub prerequisites: Vec<String>,
}

impl Quest {
    /// Create new quest in Inactive state
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            state: QuestState::Inactive,
            objectives: Vec::new(),
            rewards: Vec::new(),
            prerequisites: Vec::new(),
        }
    }

    /// Add objective to quest (builder pattern)
    pub fn with_objective(mut self, objective: ObjectiveType) -> Self {
        self.objectives.push(objective);
        self
    }

    /// Add reward to quest (builder pattern)
    pub fn with_reward(mut self, reward: QuestReward) -> Self {
        self.rewards.push(reward);
        self
    }

    /// Add prerequisite quest ID (builder pattern)
    pub fn with_prerequisite(mut self, quest_id: impl Into<String>) -> Self {
        self.prerequisites.push(quest_id.into());
        self
    }

    /// Activate quest (transition Inactive → Active)
    pub fn activate(&mut self) -> bool {
        if self.state == QuestState::Inactive {
            self.state = QuestState::Active;
            true
        } else {
            false
        }
    }

    /// Check if all objectives are complete
    pub fn check_completion(&mut self) -> bool {
        if self.state != QuestState::Active {
            return false;
        }

        let all_complete = self.objectives.iter().all(|obj| obj.is_complete());
        if all_complete {
            self.state = QuestState::Completed;
            true
        } else {
            false
        }
    }

    /// Mark quest as failed
    pub fn fail(&mut self) -> bool {
        if self.state == QuestState::Active {
            self.state = QuestState::Failed;
            true
        } else {
            false
        }
    }

    /// Get overall progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.objectives.is_empty() {
            return 0.0;
        }
        let total: f32 = self.objectives.iter().map(|obj| obj.progress()).sum();
        total / self.objectives.len() as f32
    }

    /// Update kill objective progress
    pub fn update_kill_progress(&mut self, target_type: &str, amount: usize) -> bool {
        let mut updated = false;
        for objective in &mut self.objectives {
            if let ObjectiveType::Kill { target_type: obj_type, current, .. } = objective {
                if obj_type == target_type {
                    *current += amount;
                    updated = true;
                }
            }
        }
        updated
    }

    /// Update repair objective progress
    pub fn update_repair_progress(&mut self, anchor_stability: f32) -> bool {
        let mut updated = false;
        for objective in &mut self.objectives {
            if let ObjectiveType::Repair { current, min_stability, .. } = objective {
                if anchor_stability >= *min_stability {
                    *current += 1;
                    updated = true;
                }
            }
        }
        updated
    }

    /// Update fetch objective progress
    pub fn update_fetch_progress(&mut self, item_name: &str, amount: usize) -> bool {
        let mut updated = false;
        for objective in &mut self.objectives {
            if let ObjectiveType::Fetch { item_name: obj_item, current, .. } = objective {
                if obj_item == item_name {
                    *current += amount;
                    updated = true;
                }
            }
        }
        updated
    }

    /// Update explore objective progress
    pub fn update_explore_progress(&mut self, player_pos: Vec3) -> bool {
        let mut updated = false;
        for objective in &mut self.objectives {
            if let ObjectiveType::Explore { target_position, radius, discovered, .. } = objective {
                if !*discovered && player_pos.distance(*target_position) <= *radius {
                    *discovered = true;
                    updated = true;
                }
            }
        }
        updated
    }
}

/// Quest manager for tracking multiple quests and progression
#[derive(Debug, Clone)]
pub struct QuestManager {
    /// All quests indexed by ID
    quests: HashMap<String, Quest>,
    /// Active quest ID (only one quest active at a time for now)
    active_quest_id: Option<String>,
    /// Completed quest IDs
    completed_quests: Vec<String>,
}

impl QuestManager {
    /// Create new quest manager
    pub fn new() -> Self {
        Self {
            quests: HashMap::new(),
            active_quest_id: None,
            completed_quests: Vec::new(),
        }
    }

    /// Register quest (must be done before activation)
    pub fn register_quest(&mut self, quest: Quest) {
        self.quests.insert(quest.id.clone(), quest);
    }

    /// Activate quest by ID (checks prerequisites)
    pub fn activate_quest(&mut self, quest_id: &str) -> Result<(), String> {
        // Check if quest exists
        let quest = self.quests.get(quest_id)
            .ok_or_else(|| format!("Quest {} not found", quest_id))?;

        // Check prerequisites
        for prereq_id in &quest.prerequisites {
            if !self.completed_quests.contains(prereq_id) {
                return Err(format!("Prerequisite quest {} not completed", prereq_id));
            }
        }

        // Check if already active or completed
        if quest.state != QuestState::Inactive {
            return Err(format!("Quest {} is already {:?}", quest_id, quest.state));
        }

        // Check if another quest is active
        if self.active_quest_id.is_some() {
            return Err("Another quest is already active".to_string());
        }

        // Activate quest
        let quest = self.quests.get_mut(quest_id).unwrap();
        quest.activate();
        self.active_quest_id = Some(quest_id.to_string());

        Ok(())
    }

    /// Get active quest (immutable)
    pub fn active_quest(&self) -> Option<&Quest> {
        self.active_quest_id.as_ref().and_then(|id| self.quests.get(id))
    }

    /// Get active quest (mutable)
    pub fn active_quest_mut(&mut self) -> Option<&mut Quest> {
        self.active_quest_id.as_ref().and_then(|id| self.quests.get_mut(id))
    }

    /// Check active quest completion
    pub fn check_active_quest(&mut self) -> Option<Vec<QuestReward>> {
        let quest_id = self.active_quest_id.as_ref()?.clone();
        let quest = self.quests.get_mut(&quest_id)?;

        if quest.check_completion() {
            let rewards = quest.rewards.clone();
            self.completed_quests.push(quest_id.clone());
            self.active_quest_id = None;
            Some(rewards)
        } else {
            None
        }
    }

    /// Update kill progress for active quest
    pub fn update_kill(&mut self, target_type: &str, amount: usize) -> bool {
        if let Some(quest) = self.active_quest_mut() {
            quest.update_kill_progress(target_type, amount)
        } else {
            false
        }
    }

    /// Update repair progress for active quest
    pub fn update_repair(&mut self, anchor_stability: f32) -> bool {
        if let Some(quest) = self.active_quest_mut() {
            quest.update_repair_progress(anchor_stability)
        } else {
            false
        }
    }

    /// Update fetch progress for active quest
    pub fn update_fetch(&mut self, item_name: &str, amount: usize) -> bool {
        if let Some(quest) = self.active_quest_mut() {
            quest.update_fetch_progress(item_name, amount)
        } else {
            false
        }
    }

    /// Update explore progress for active quest
    pub fn update_explore(&mut self, player_pos: Vec3) -> bool {
        if let Some(quest) = self.active_quest_mut() {
            quest.update_explore_progress(player_pos)
        } else {
            false
        }
    }

    /// Get quest by ID
    pub fn quest(&self, quest_id: &str) -> Option<&Quest> {
        self.quests.get(quest_id)
    }

    /// Check if quest is completed
    pub fn is_completed(&self, quest_id: &str) -> bool {
        self.completed_quests.contains(&quest_id.to_string())
    }

    /// Get number of completed quests
    pub fn completed_count(&self) -> usize {
        self.completed_quests.len()
    }
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_creation() {
        let quest = Quest::new("quest1", "Test Quest", "A test quest");
        assert_eq!(quest.id, "quest1");
        assert_eq!(quest.title, "Test Quest");
        assert_eq!(quest.state, QuestState::Inactive);
        assert!(quest.objectives.is_empty());
        assert!(quest.rewards.is_empty());
    }

    #[test]
    fn test_quest_activation() {
        let mut quest = Quest::new("quest1", "Test", "Test");
        assert!(quest.activate());
        assert_eq!(quest.state, QuestState::Active);
        assert!(!quest.activate()); // Can't activate twice
    }

    #[test]
    fn test_kill_objective() {
        let mut obj = ObjectiveType::Kill {
            target_type: "enemy".to_string(),
            required: 10,
            current: 0,
        };

        assert!(!obj.is_complete());
        assert_eq!(obj.progress(), 0.0);

        if let ObjectiveType::Kill { current, .. } = &mut obj {
            *current = 5;
        }
        assert_eq!(obj.progress(), 0.5);

        if let ObjectiveType::Kill { current, .. } = &mut obj {
            *current = 10;
        }
        assert!(obj.is_complete());
        assert_eq!(obj.progress(), 1.0);
    }

    #[test]
    fn test_repair_objective() {
        let obj = ObjectiveType::Repair {
            required: 3,
            current: 2,
            min_stability: 0.8,
        };

        assert!(!obj.is_complete());
        assert!((obj.progress() - 0.666).abs() < 0.01);
        assert_eq!(obj.description(), "Repair 2/3 anchors to 80% stability");
    }

    #[test]
    fn test_fetch_objective() {
        let obj = ObjectiveType::Fetch {
            item_name: "echo_shard".to_string(),
            required: 5,
            current: 3,
            delivery_location: Vec3::new(0.0, 0.0, 0.0),
        };

        assert!(!obj.is_complete());
        assert_eq!(obj.progress(), 0.6);
        assert_eq!(obj.description(), "Collect 3/5 echo_shard");
    }

    #[test]
    fn test_explore_objective() {
        let mut obj = ObjectiveType::Explore {
            location_name: "Ancient Ruins".to_string(),
            target_position: Vec3::new(100.0, 0.0, 100.0),
            radius: 10.0,
            discovered: false,
        };

        assert!(!obj.is_complete());
        assert_eq!(obj.progress(), 0.0);

        if let ObjectiveType::Explore { discovered, .. } = &mut obj {
            *discovered = true;
        }
        assert!(obj.is_complete());
        assert_eq!(obj.progress(), 1.0);
    }

    #[test]
    fn test_quest_with_objectives() {
        let quest = Quest::new("quest1", "Test", "Test")
            .with_objective(ObjectiveType::Kill {
                target_type: "enemy".to_string(),
                required: 10,
                current: 0,
            })
            .with_objective(ObjectiveType::Repair {
                required: 3,
                current: 0,
                min_stability: 0.8,
            })
            .with_reward(QuestReward::EchoCurrency(100));

        assert_eq!(quest.objectives.len(), 2);
        assert_eq!(quest.rewards.len(), 1);
    }

    #[test]
    fn test_quest_completion() {
        let mut quest = Quest::new("quest1", "Test", "Test")
            .with_objective(ObjectiveType::Kill {
                target_type: "enemy".to_string(),
                required: 2,
                current: 0,
            });

        quest.activate();
        assert_eq!(quest.state, QuestState::Active);

        // Incomplete
        assert!(!quest.check_completion());
        assert_eq!(quest.state, QuestState::Active);

        // Complete objectives
        quest.update_kill_progress("enemy", 2);
        assert!(quest.check_completion());
        assert_eq!(quest.state, QuestState::Completed);
    }

    #[test]
    fn test_quest_progress() {
        let quest = Quest::new("quest1", "Test", "Test")
            .with_objective(ObjectiveType::Kill {
                target_type: "enemy".to_string(),
                required: 10,
                current: 5,
            })
            .with_objective(ObjectiveType::Repair {
                required: 4,
                current: 2,
                min_stability: 0.8,
            });

        let progress = quest.progress();
        assert!((progress - 0.5).abs() < 0.01); // (0.5 + 0.5) / 2 = 0.5
    }

    #[test]
    fn test_quest_manager_registration() {
        let mut manager = QuestManager::new();
        let quest = Quest::new("quest1", "Test", "Test");
        
        manager.register_quest(quest);
        assert!(manager.quest("quest1").is_some());
        assert!(manager.quest("quest2").is_none());
    }

    #[test]
    fn test_quest_manager_activation() {
        let mut manager = QuestManager::new();
        let quest = Quest::new("quest1", "Test", "Test");
        
        manager.register_quest(quest);
        assert!(manager.activate_quest("quest1").is_ok());
        assert!(manager.active_quest().is_some());
        assert_eq!(manager.active_quest().unwrap().state, QuestState::Active);
    }

    #[test]
    fn test_quest_manager_prerequisites() {
        let mut manager = QuestManager::new();
        
        let quest1 = Quest::new("quest1", "First", "First quest");
        let quest2 = Quest::new("quest2", "Second", "Second quest")
            .with_prerequisite("quest1");
        
        manager.register_quest(quest1);
        manager.register_quest(quest2);
        
        // Can't activate quest2 without completing quest1
        assert!(manager.activate_quest("quest2").is_err());
        
        // Complete quest1
        manager.activate_quest("quest1").unwrap();
        manager.active_quest_mut().unwrap().state = QuestState::Completed;
        manager.completed_quests.push("quest1".to_string());
        manager.active_quest_id = None;
        
        // Now can activate quest2
        assert!(manager.activate_quest("quest2").is_ok());
    }

    #[test]
    fn test_quest_manager_one_active_at_time() {
        let mut manager = QuestManager::new();
        
        manager.register_quest(Quest::new("quest1", "First", "First"));
        manager.register_quest(Quest::new("quest2", "Second", "Second"));
        
        assert!(manager.activate_quest("quest1").is_ok());
        assert!(manager.activate_quest("quest2").is_err()); // Already active
    }

    #[test]
    fn test_quest_manager_update_kill() {
        let mut manager = QuestManager::new();
        
        let quest = Quest::new("quest1", "Kill Quest", "Kill enemies")
            .with_objective(ObjectiveType::Kill {
                target_type: "enemy".to_string(),
                required: 5,
                current: 0,
            });
        
        manager.register_quest(quest);
        manager.activate_quest("quest1").unwrap();
        
        assert!(manager.update_kill("enemy", 3));
        let active = manager.active_quest().unwrap();
        if let ObjectiveType::Kill { current, .. } = &active.objectives[0] {
            assert_eq!(*current, 3);
        }
    }

    #[test]
    fn test_quest_manager_update_repair() {
        let mut manager = QuestManager::new();
        
        let quest = Quest::new("quest1", "Repair Quest", "Repair anchors")
            .with_objective(ObjectiveType::Repair {
                required: 3,
                current: 0,
                min_stability: 0.8,
            });
        
        manager.register_quest(quest);
        manager.activate_quest("quest1").unwrap();
        
        assert!(manager.update_repair(0.85)); // Above threshold
        assert!(!manager.update_repair(0.7)); // Below threshold
        
        let active = manager.active_quest().unwrap();
        if let ObjectiveType::Repair { current, .. } = &active.objectives[0] {
            assert_eq!(*current, 1);
        }
    }

    #[test]
    fn test_quest_manager_completion() {
        let mut manager = QuestManager::new();
        
        let quest = Quest::new("quest1", "Simple Quest", "Complete this")
            .with_objective(ObjectiveType::Kill {
                target_type: "enemy".to_string(),
                required: 2,
                current: 0,
            })
            .with_reward(QuestReward::EchoCurrency(100));
        
        manager.register_quest(quest);
        manager.activate_quest("quest1").unwrap();
        
        // Incomplete
        assert!(manager.check_active_quest().is_none());
        
        // Complete objectives
        manager.update_kill("enemy", 2);
        let rewards = manager.check_active_quest();
        assert!(rewards.is_some());
        assert_eq!(rewards.unwrap().len(), 1);
        
        // Quest should be completed and no longer active
        assert!(manager.active_quest().is_none());
        assert!(manager.is_completed("quest1"));
        assert_eq!(manager.completed_count(), 1);
    }

    #[test]
    fn test_quest_manager_update_fetch() {
        let mut manager = QuestManager::new();
        
        let quest = Quest::new("quest1", "Fetch Quest", "Collect items")
            .with_objective(ObjectiveType::Fetch {
                item_name: "echo_shard".to_string(),
                required: 5,
                current: 0,
                delivery_location: Vec3::ZERO,
            });
        
        manager.register_quest(quest);
        manager.activate_quest("quest1").unwrap();
        
        assert!(manager.update_fetch("echo_shard", 3));
        assert!(!manager.update_fetch("wrong_item", 1));
        
        let active = manager.active_quest().unwrap();
        if let ObjectiveType::Fetch { current, .. } = &active.objectives[0] {
            assert_eq!(*current, 3);
        }
    }

    #[test]
    fn test_quest_manager_update_explore() {
        let mut manager = QuestManager::new();
        
        let quest = Quest::new("quest1", "Explore Quest", "Find the ruins")
            .with_objective(ObjectiveType::Explore {
                location_name: "Ruins".to_string(),
                target_position: Vec3::new(100.0, 0.0, 100.0),
                radius: 10.0,
                discovered: false,
            });
        
        manager.register_quest(quest);
        manager.activate_quest("quest1").unwrap();
        
        // Too far
        assert!(!manager.update_explore(Vec3::new(50.0, 0.0, 50.0)));
        
        // Within radius
        assert!(manager.update_explore(Vec3::new(105.0, 0.0, 105.0)));
        
        let active = manager.active_quest().unwrap();
        if let ObjectiveType::Explore { discovered, .. } = &active.objectives[0] {
            assert!(*discovered);
        }
    }
}
