// Comprehensive action library mapping ActionStep variants to GOAP actions
// Phase 2: Engine Integration

use std::collections::BTreeMap;
use super::{Action, StateValue, WorldState, ActionHistory};

/// Movement action: Move toward a target position
pub struct MoveToAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl MoveToAction {
    pub fn new() -> Self {
        let preconditions = BTreeMap::new();
        
        let mut effects = BTreeMap::new();
        effects.insert("moved".to_string(), StateValue::Bool(true));
        effects.insert("position_changed".to_string(), StateValue::Bool(true));
        
        Self { preconditions, effects }
    }
}

impl Action for MoveToAction {
    fn name(&self) -> &str {
        "move_to"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        2.0
    }
}

/// Approach action: Move closer to enemy while maintaining distance
pub struct ApproachEnemyAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl ApproachEnemyAction {
    pub fn new() -> Self {
        let mut preconditions = BTreeMap::new();
        preconditions.insert("enemy_present".to_string(), StateValue::Bool(true));
        preconditions.insert("enemy_distance".to_string(), StateValue::IntRange(5, 999));
        
        let mut effects = BTreeMap::new();
        effects.insert("enemy_distance".to_string(), StateValue::IntRange(3, 8));
        effects.insert("in_range".to_string(), StateValue::Bool(true));
        
        Self { preconditions, effects }
    }
}

impl Action for ApproachEnemyAction {
    fn name(&self) -> &str {
        "approach_enemy"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        2.5
    }
    
    fn success_probability(&self, world: &WorldState, history: &ActionHistory) -> f32 {
        let base = history
            .get_action_stats(self.name())
            .map(|s| s.success_rate())
            .unwrap_or(0.85);
        
        // Higher risk when health is low
        if let Some(StateValue::Int(health)) = world.get("my_health") {
            if *health < 30 {
                return base * 0.7;
            }
        }
        base
    }
}

/// Attack action: Basic attack requiring ammo and close range
pub struct AttackAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl AttackAction {
    pub fn new() -> Self {
        let mut preconditions = BTreeMap::new();
        preconditions.insert("has_ammo".to_string(), StateValue::Bool(true));
        preconditions.insert("enemy_present".to_string(), StateValue::Bool(true));
        preconditions.insert("in_range".to_string(), StateValue::Bool(true));
        
        let mut effects = BTreeMap::new();
        effects.insert("enemy_damaged".to_string(), StateValue::Bool(true));
        effects.insert("ammo_spent".to_string(), StateValue::Bool(true));
        
        Self { preconditions, effects }
    }
}

impl Action for AttackAction {
    fn name(&self) -> &str {
        "attack"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        5.0
    }
    
    fn success_probability(&self, world: &WorldState, history: &ActionHistory) -> f32 {
        let base = history
            .get_action_stats(self.name())
            .map(|s| s.success_rate())
            .unwrap_or(0.75);
        
        // Success depends on health and ammo
        let mut modifier = 1.0;
        
        if let Some(StateValue::Int(health)) = world.get("my_health") {
            if *health < 40 {
                modifier *= 0.6; // Risky when low health
            }
        }
        
        if let Some(StateValue::Int(ammo)) = world.get("my_ammo") {
            if *ammo < 5 {
                modifier *= 0.8; // Less reliable when low ammo
            }
        }
        
        (base * modifier).clamp(0.2, 0.95)
    }
}

/// Cover Fire action: Sustained suppression fire
pub struct CoverFireAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl CoverFireAction {
    pub fn new() -> Self {
        let mut preconditions = BTreeMap::new();
        preconditions.insert("has_ammo".to_string(), StateValue::Bool(true));
        preconditions.insert("enemy_present".to_string(), StateValue::Bool(true));
        preconditions.insert("in_range".to_string(), StateValue::Bool(true));
        
        let mut effects = BTreeMap::new();
        effects.insert("enemy_suppressed".to_string(), StateValue::Bool(true));
        effects.insert("cover_provided".to_string(), StateValue::Bool(true));
        
        Self { preconditions, effects }
    }
}

impl Action for CoverFireAction {
    fn name(&self) -> &str {
        "cover_fire"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        6.0 // Higher cost due to sustained fire
    }
}

/// Reload action: Restore ammo
pub struct ReloadAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl ReloadAction {
    pub fn new() -> Self {
        let mut preconditions = BTreeMap::new();
        preconditions.insert("has_ammo".to_string(), StateValue::Bool(false));
        
        let mut effects = BTreeMap::new();
        effects.insert("has_ammo".to_string(), StateValue::Bool(true));
        effects.insert("my_ammo".to_string(), StateValue::Int(30));
        effects.insert("reloaded".to_string(), StateValue::Bool(true));
        
        Self { preconditions, effects }
    }
}

impl Action for ReloadAction {
    fn name(&self) -> &str {
        "reload"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        3.0
    }
    
    fn success_probability(&self, _world: &WorldState, _history: &ActionHistory) -> f32 {
        0.98 // Reload is very reliable
    }
}

/// Take Cover action: Move to defensive position
pub struct TakeCoverAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl TakeCoverAction {
    pub fn new() -> Self {
        let mut preconditions = BTreeMap::new();
        preconditions.insert("enemy_present".to_string(), StateValue::Bool(true));
        preconditions.insert("in_cover".to_string(), StateValue::Bool(false));
        
        let mut effects = BTreeMap::new();
        effects.insert("in_cover".to_string(), StateValue::Bool(true));
        effects.insert("protected".to_string(), StateValue::Bool(true));
        
        Self { preconditions, effects }
    }
}

impl Action for TakeCoverAction {
    fn name(&self) -> &str {
        "take_cover"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        2.0
    }
    
    fn state_cost_modifier(&self, world: &WorldState) -> f32 {
        // Taking cover becomes more urgent when health is low
        if let Some(StateValue::Int(health)) = world.get("my_health") {
            if *health < 30 {
                return 0.5; // Lower cost = higher priority
            } else if *health < 60 {
                return 0.8;
            }
        }
        1.0
    }
}

/// Heal action: Restore health using resources
pub struct HealAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl HealAction {
    pub fn new() -> Self {
        let mut preconditions = BTreeMap::new();
        preconditions.insert("has_medkit".to_string(), StateValue::Bool(true));
        preconditions.insert("my_health".to_string(), StateValue::IntRange(1, 80));
        
        let mut effects = BTreeMap::new();
        effects.insert("my_health".to_string(), StateValue::Int(100));
        effects.insert("healed".to_string(), StateValue::Bool(true));
        effects.insert("has_medkit".to_string(), StateValue::Bool(false));
        
        Self { preconditions, effects }
    }
}

impl Action for HealAction {
    fn name(&self) -> &str {
        "heal"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        4.0
    }
    
    fn state_cost_modifier(&self, world: &WorldState) -> f32 {
        // Healing becomes cheaper (higher priority) when critically wounded
        if let Some(StateValue::Int(health)) = world.get("my_health") {
            if *health < 20 {
                return 0.3; // Very urgent
            } else if *health < 50 {
                return 0.6;
            }
        }
        1.0
    }
}

/// Throw Smoke action: Deploy smoke for concealment
pub struct ThrowSmokeAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl ThrowSmokeAction {
    pub fn new() -> Self {
        let mut preconditions = BTreeMap::new();
        preconditions.insert("smoke_available".to_string(), StateValue::Bool(true));
        preconditions.insert("smoke_cooldown".to_string(), StateValue::FloatApprox(0.0, 0.1));
        
        let mut effects = BTreeMap::new();
        effects.insert("smoke_deployed".to_string(), StateValue::Bool(true));
        effects.insert("concealed".to_string(), StateValue::Bool(true));
        effects.insert("smoke_cooldown".to_string(), StateValue::Float(super::OrderedFloat(8.0)));
        
        Self { preconditions, effects }
    }
}

impl Action for ThrowSmokeAction {
    fn name(&self) -> &str {
        "throw_smoke"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        4.0
    }
}

/// Retreat action: Fall back from enemy
pub struct RetreatAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl RetreatAction {
    pub fn new() -> Self {
        let mut preconditions = BTreeMap::new();
        preconditions.insert("enemy_present".to_string(), StateValue::Bool(true));
        preconditions.insert("enemy_distance".to_string(), StateValue::IntRange(0, 8));
        
        let mut effects = BTreeMap::new();
        effects.insert("enemy_distance".to_string(), StateValue::IntRange(10, 20));
        effects.insert("safe_distance".to_string(), StateValue::Bool(true));
        
        Self { preconditions, effects }
    }
}

impl Action for RetreatAction {
    fn name(&self) -> &str {
        "retreat"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        2.5
    }
    
    fn state_cost_modifier(&self, world: &WorldState) -> f32 {
        // Retreating becomes cheaper (more attractive) when health is low
        if let Some(StateValue::Int(health)) = world.get("my_health") {
            if *health < 25 {
                return 0.4; // Very attractive when critical
            } else if *health < 50 {
                return 0.7;
            }
        }
        1.0
    }
}

/// Revive action: Restore downed ally
pub struct ReviveAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl ReviveAction {
    pub fn new() -> Self {
        let mut preconditions = BTreeMap::new();
        preconditions.insert("ally_downed".to_string(), StateValue::Bool(true));
        preconditions.insert("near_ally".to_string(), StateValue::Bool(true));
        
        let mut effects = BTreeMap::new();
        effects.insert("ally_downed".to_string(), StateValue::Bool(false));
        effects.insert("ally_revived".to_string(), StateValue::Bool(true));
        
        Self { preconditions, effects }
    }
}

impl Action for ReviveAction {
    fn name(&self) -> &str {
        "revive"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        5.0
    }
}

/// Wait/Scan action: Observe before acting
pub struct ScanAction {
    preconditions: BTreeMap<String, StateValue>,
    effects: BTreeMap<String, StateValue>,
}

impl ScanAction {
    pub fn new() -> Self {
        let preconditions = BTreeMap::new();
        
        let mut effects = BTreeMap::new();
        effects.insert("scanned".to_string(), StateValue::Bool(true));
        effects.insert("aware".to_string(), StateValue::Bool(true));
        
        Self { preconditions, effects }
    }
}

impl Action for ScanAction {
    fn name(&self) -> &str {
        "scan"
    }
    
    fn preconditions(&self) -> &BTreeMap<String, StateValue> {
        &self.preconditions
    }
    
    fn effects(&self) -> &BTreeMap<String, StateValue> {
        &self.effects
    }
    
    fn base_cost(&self) -> f32 {
        1.0
    }
}

/// Register all actions into a GOAP planner
pub fn register_all_actions(planner: &mut crate::goap::AdvancedGOAP) {
    planner.add_action(Box::new(MoveToAction::new()));
    planner.add_action(Box::new(ApproachEnemyAction::new()));
    planner.add_action(Box::new(AttackAction::new()));
    planner.add_action(Box::new(CoverFireAction::new()));
    planner.add_action(Box::new(ReloadAction::new()));
    planner.add_action(Box::new(TakeCoverAction::new()));
    planner.add_action(Box::new(HealAction::new()));
    planner.add_action(Box::new(ThrowSmokeAction::new()));
    planner.add_action(Box::new(RetreatAction::new()));
    planner.add_action(Box::new(ReviveAction::new()));
    planner.add_action(Box::new(ScanAction::new()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_preconditions() {
        let action = AttackAction::new();
        assert_eq!(action.preconditions().len(), 3);
        assert!(action.preconditions().contains_key("has_ammo"));
        assert!(action.preconditions().contains_key("enemy_present"));
    }

    #[test]
    fn test_heal_state_cost_modifier() {
        let action = HealAction::new();
        
        let mut critical_health = WorldState::new();
        critical_health.set("my_health", StateValue::Int(15));
        assert!(action.state_cost_modifier(&critical_health) < 0.5);
        
        let mut full_health = WorldState::new();
        full_health.set("my_health", StateValue::Int(100));
        assert_eq!(action.state_cost_modifier(&full_health), 1.0);
    }

    #[test]
    fn test_retreat_becomes_attractive_when_wounded() {
        let action = RetreatAction::new();
        
        let mut critical = WorldState::new();
        critical.set("my_health", StateValue::Int(20));
        assert!(action.state_cost_modifier(&critical) < 0.5);
    }

    #[test]
    fn test_action_registration() {
        let mut planner = crate::goap::AdvancedGOAP::new();
        register_all_actions(&mut planner);
        
        assert_eq!(planner.action_count(), 11);
        let names = planner.action_names();
        assert!(names.contains(&"attack".to_string()));
        assert!(names.contains(&"heal".to_string()));
        assert!(names.contains(&"retreat".to_string()));
    }
}

