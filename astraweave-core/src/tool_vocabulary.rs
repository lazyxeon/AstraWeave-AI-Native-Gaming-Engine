//! Tool Vocabulary - Complete metadata for all 37 action tools
//!
//! This module provides comprehensive metadata for LLM prompt engineering,
//! including descriptions, parameters, preconditions, and effects.

use serde::{Deserialize, Serialize};

/// Tool parameter definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub param_type: String, // "i32", "f32", "Entity", "IVec2", etc.
    pub required: bool,
    pub description: String,
}

/// Complete tool metadata for LLM prompting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub category: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub preconditions: Vec<String>,
    pub effects: Vec<String>,
    pub cooldown: Option<f32>,
    pub cost: Option<String>, // "ammo", "stamina", etc.
}

/// Get all tool metadata for LLM prompt engineering
pub fn get_all_tools() -> Vec<ToolMetadata> {
    vec![
        // ═══════════════════════════════════════
        // MOVEMENT (6 tools)
        // ═══════════════════════════════════════
        ToolMetadata {
            name: "move_to".into(),
            category: "Movement".into(),
            description: "Move to a specific position on the map".into(),
            parameters: vec![
                ToolParameter {
                    name: "x".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "Target X coordinate".into(),
                },
                ToolParameter {
                    name: "y".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "Target Y coordinate".into(),
                },
                ToolParameter {
                    name: "speed".into(),
                    param_type: "MovementSpeed".into(),
                    required: false,
                    description: "Movement speed (Walk, Run, Sprint)".into(),
                },
            ],
            preconditions: vec!["Path must be clear to target".into()],
            effects: vec!["Agent position changes to (x, y)".into()],
            cooldown: None,
            cost: Some("stamina (if sprinting)".into()),
        },
        ToolMetadata {
            name: "approach".into(),
            category: "Movement".into(),
            description: "Move toward target entity while maintaining specified distance".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to approach".into(),
                },
                ToolParameter {
                    name: "distance".into(),
                    param_type: "f32".into(),
                    required: true,
                    description: "Desired distance (e.g., 2 for melee, 15 for ranged)".into(),
                },
            ],
            preconditions: vec!["Target entity must exist".into()],
            effects: vec!["Agent moves closer to target".into()],
            cooldown: None,
            cost: None,
        },
        ToolMetadata {
            name: "retreat".into(),
            category: "Movement".into(),
            description: "Move away from target entity to safe distance".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to retreat from".into(),
                },
                ToolParameter {
                    name: "distance".into(),
                    param_type: "f32".into(),
                    required: true,
                    description: "Safe distance to reach".into(),
                },
            ],
            preconditions: vec!["Target entity must exist".into()],
            effects: vec!["Agent moves away from target".into()],
            cooldown: None,
            cost: None,
        },
        ToolMetadata {
            name: "take_cover".into(),
            category: "Movement".into(),
            description: "Take cover behind nearest obstacle or specified position".into(),
            parameters: vec![
                ToolParameter {
                    name: "position".into(),
                    param_type: "IVec2?".into(),
                    required: false,
                    description: "Optional: specific cover position".into(),
                },
            ],
            preconditions: vec!["Cover must be available".into()],
            effects: vec!["Agent moves to cover, gains defensive bonus".into()],
            cooldown: None,
            cost: None,
        },
        ToolMetadata {
            name: "strafe".into(),
            category: "Movement".into(),
            description: "Circle around target entity while maintaining line of sight".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to strafe around".into(),
                },
                ToolParameter {
                    name: "direction".into(),
                    param_type: "StrafeDirection".into(),
                    required: true,
                    description: "Direction to strafe (Left or Right)".into(),
                },
            ],
            preconditions: vec!["Target entity must exist".into()],
            effects: vec!["Agent circles target, harder to hit".into()],
            cooldown: None,
            cost: Some("stamina".into()),
        },
        ToolMetadata {
            name: "patrol".into(),
            category: "Movement".into(),
            description: "Patrol between multiple waypoints".into(),
            parameters: vec![
                ToolParameter {
                    name: "waypoints".into(),
                    param_type: "Vec<IVec2>".into(),
                    required: true,
                    description: "List of patrol waypoints".into(),
                },
            ],
            preconditions: vec!["At least 2 waypoints required".into()],
            effects: vec!["Agent moves between waypoints in sequence".into()],
            cooldown: None,
            cost: None,
        },
        
        // ═══════════════════════════════════════
        // OFFENSIVE (8 tools)
        // ═══════════════════════════════════════
        ToolMetadata {
            name: "attack".into(),
            category: "Offensive".into(),
            description: "Basic attack targeting entity (10 damage)".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to attack".into(),
                },
            ],
            preconditions: vec!["Target in range".into(), "Line of sight clear".into()],
            effects: vec!["Deals 10 damage to target".into()],
            cooldown: Some(1.0),
            cost: None,
        },
        ToolMetadata {
            name: "aimed_shot".into(),
            category: "Offensive".into(),
            description: "Aimed shot with higher accuracy and damage (15 damage)".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to target".into(),
                },
            ],
            preconditions: vec!["Target in range".into(), "Line of sight clear".into(), "Not moving".into()],
            effects: vec!["Deals 15 damage, higher accuracy".into()],
            cooldown: Some(2.0),
            cost: Some("ammo".into()),
        },
        ToolMetadata {
            name: "quick_attack".into(),
            category: "Offensive".into(),
            description: "Fast attack with lower damage (5 damage)".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to attack".into(),
                },
            ],
            preconditions: vec!["Target in melee range".into()],
            effects: vec!["Deals 5 damage, fast execution".into()],
            cooldown: Some(0.5),
            cost: None,
        },
        ToolMetadata {
            name: "heavy_attack".into(),
            category: "Offensive".into(),
            description: "Powerful attack with high damage (25 damage)".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to attack".into(),
                },
            ],
            preconditions: vec!["Target in melee range".into(), "Sufficient stamina".into()],
            effects: vec!["Deals 25 damage, slow execution".into()],
            cooldown: Some(3.0),
            cost: Some("stamina".into()),
        },
        ToolMetadata {
            name: "aoe_attack".into(),
            category: "Offensive".into(),
            description: "Area-of-effect attack damaging all entities in radius".into(),
            parameters: vec![
                ToolParameter {
                    name: "x".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "Center X coordinate".into(),
                },
                ToolParameter {
                    name: "y".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "Center Y coordinate".into(),
                },
                ToolParameter {
                    name: "radius".into(),
                    param_type: "f32".into(),
                    required: true,
                    description: "Effect radius".into(),
                },
            ],
            preconditions: vec!["Target area in range".into()],
            effects: vec!["Damages all entities in radius".into()],
            cooldown: Some(5.0),
            cost: Some("ability charge".into()),
        },
        ToolMetadata {
            name: "throw_explosive".into(),
            category: "Offensive".into(),
            description: "Throw grenade or explosive to target location".into(),
            parameters: vec![
                ToolParameter {
                    name: "x".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "Target X coordinate".into(),
                },
                ToolParameter {
                    name: "y".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "Target Y coordinate".into(),
                },
            ],
            preconditions: vec!["Grenade in inventory".into(), "Line of sight clear".into()],
            effects: vec!["Area damage at target location".into()],
            cooldown: Some(8.0),
            cost: Some("grenade".into()),
        },
        ToolMetadata {
            name: "cover_fire".into(),
            category: "Offensive".into(),
            description: "Suppressive fire on target for duration".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to suppress".into(),
                },
                ToolParameter {
                    name: "duration".into(),
                    param_type: "f32".into(),
                    required: true,
                    description: "Duration in seconds".into(),
                },
            ],
            preconditions: vec!["Ammo available".into(), "Line of sight clear".into()],
            effects: vec!["Target suppressed, continuous damage".into()],
            cooldown: Some(4.0),
            cost: Some("ammo".into()),
        },
        ToolMetadata {
            name: "charge".into(),
            category: "Offensive".into(),
            description: "Rush at target and attack".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to charge".into(),
                },
            ],
            preconditions: vec!["Target visible".into(), "Path clear".into()],
            effects: vec!["Rapid movement to target, melee attack".into()],
            cooldown: Some(6.0),
            cost: Some("stamina".into()),
        },
        
        // ═══════════════════════════════════════
        // DEFENSIVE (6 tools)
        // ═══════════════════════════════════════
        ToolMetadata {
            name: "block".into(),
            category: "Defensive".into(),
            description: "Block incoming attack".into(),
            parameters: vec![],
            preconditions: vec!["Enemy attack incoming".into()],
            effects: vec!["Reduces damage by 50%".into()],
            cooldown: Some(1.0),
            cost: None,
        },
        ToolMetadata {
            name: "dodge".into(),
            category: "Defensive".into(),
            description: "Dodge incoming attack".into(),
            parameters: vec![
                ToolParameter {
                    name: "direction".into(),
                    param_type: "StrafeDirection?".into(),
                    required: false,
                    description: "Optional: dodge direction".into(),
                },
            ],
            preconditions: vec!["Enemy attack incoming".into(), "Sufficient stamina".into()],
            effects: vec!["Avoid attack completely".into()],
            cooldown: Some(2.0),
            cost: Some("stamina".into()),
        },
        ToolMetadata {
            name: "parry".into(),
            category: "Defensive".into(),
            description: "Parry incoming attack and counter".into(),
            parameters: vec![],
            preconditions: vec!["Enemy attack incoming".into(), "Melee weapon equipped".into()],
            effects: vec!["Blocks attack, stuns attacker".into()],
            cooldown: Some(3.0),
            cost: None,
        },
        ToolMetadata {
            name: "throw_smoke".into(),
            category: "Defensive".into(),
            description: "Throw smoke grenade to obscure area".into(),
            parameters: vec![
                ToolParameter {
                    name: "x".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "Target X coordinate".into(),
                },
                ToolParameter {
                    name: "y".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "Target Y coordinate".into(),
                },
            ],
            preconditions: vec!["Smoke grenade in inventory".into(), "Line of sight clear".into()],
            effects: vec!["Blocks line of sight in area".into()],
            cooldown: Some(8.0),
            cost: Some("smoke grenade".into()),
        },
        ToolMetadata {
            name: "heal".into(),
            category: "Defensive".into(),
            description: "Heal self or ally (restores 20 HP)".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity?".into(),
                    required: false,
                    description: "Optional: ally to heal (self if not specified)".into(),
                },
            ],
            preconditions: vec!["Medical kit available".into()],
            effects: vec!["Restores 20 HP to target".into()],
            cooldown: Some(10.0),
            cost: Some("medical kit".into()),
        },
        ToolMetadata {
            name: "use_defensive_ability".into(),
            category: "Defensive".into(),
            description: "Activate defensive ability (shield, armor buff, etc.)".into(),
            parameters: vec![
                ToolParameter {
                    name: "ability_name".into(),
                    param_type: "String".into(),
                    required: true,
                    description: "Name of defensive ability".into(),
                },
            ],
            preconditions: vec!["Ability available".into(), "Ability off cooldown".into()],
            effects: vec!["Activates defensive effect".into()],
            cooldown: Some(15.0),
            cost: Some("ability charge".into()),
        },
        
        // ═══════════════════════════════════════
        // EQUIPMENT (5 tools)
        // ═══════════════════════════════════════
        ToolMetadata {
            name: "equip_weapon".into(),
            category: "Equipment".into(),
            description: "Equip weapon from inventory".into(),
            parameters: vec![
                ToolParameter {
                    name: "weapon_name".into(),
                    param_type: "String".into(),
                    required: true,
                    description: "Name of weapon to equip".into(),
                },
            ],
            preconditions: vec!["Weapon in inventory".into()],
            effects: vec!["Weapon equipped and ready".into()],
            cooldown: Some(1.0),
            cost: None,
        },
        ToolMetadata {
            name: "switch_weapon".into(),
            category: "Equipment".into(),
            description: "Switch to weapon in different slot".into(),
            parameters: vec![
                ToolParameter {
                    name: "slot".into(),
                    param_type: "u32".into(),
                    required: true,
                    description: "Weapon slot number (0, 1, 2)".into(),
                },
            ],
            preconditions: vec!["Weapon in slot".into()],
            effects: vec!["Switches active weapon".into()],
            cooldown: Some(0.5),
            cost: None,
        },
        ToolMetadata {
            name: "reload".into(),
            category: "Equipment".into(),
            description: "Reload current weapon to full ammo".into(),
            parameters: vec![],
            preconditions: vec!["Reserve ammo available".into()],
            effects: vec!["Restores weapon ammo to max".into()],
            cooldown: Some(2.0),
            cost: Some("reserve ammo".into()),
        },
        ToolMetadata {
            name: "use_item".into(),
            category: "Equipment".into(),
            description: "Use consumable item from inventory".into(),
            parameters: vec![
                ToolParameter {
                    name: "item_name".into(),
                    param_type: "String".into(),
                    required: true,
                    description: "Name of item to use".into(),
                },
            ],
            preconditions: vec!["Item in inventory".into()],
            effects: vec!["Item effect applied".into()],
            cooldown: Some(1.0),
            cost: Some("item consumed".into()),
        },
        ToolMetadata {
            name: "drop_item".into(),
            category: "Equipment".into(),
            description: "Drop item from inventory".into(),
            parameters: vec![
                ToolParameter {
                    name: "item_name".into(),
                    param_type: "String".into(),
                    required: true,
                    description: "Name of item to drop".into(),
                },
            ],
            preconditions: vec!["Item in inventory".into()],
            effects: vec!["Item removed from inventory".into()],
            cooldown: None,
            cost: None,
        },
        
        // ═══════════════════════════════════════
        // TACTICAL (7 tools)
        // ═══════════════════════════════════════
        ToolMetadata {
            name: "call_reinforcements".into(),
            category: "Tactical".into(),
            description: "Request allied reinforcements".into(),
            parameters: vec![
                ToolParameter {
                    name: "count".into(),
                    param_type: "u32".into(),
                    required: true,
                    description: "Number of reinforcements to call".into(),
                },
            ],
            preconditions: vec!["Reinforcements available".into()],
            effects: vec!["Allies spawn at friendly location".into()],
            cooldown: Some(60.0),
            cost: Some("command points".into()),
        },
        ToolMetadata {
            name: "mark_target".into(),
            category: "Tactical".into(),
            description: "Mark target for allies to focus fire".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to mark".into(),
                },
            ],
            preconditions: vec!["Target visible".into()],
            effects: vec!["Allies prioritize marked target".into()],
            cooldown: Some(5.0),
            cost: None,
        },
        ToolMetadata {
            name: "request_cover".into(),
            category: "Tactical".into(),
            description: "Request covering fire from allies".into(),
            parameters: vec![
                ToolParameter {
                    name: "duration".into(),
                    param_type: "f32".into(),
                    required: true,
                    description: "Duration in seconds".into(),
                },
            ],
            preconditions: vec!["Allies available".into()],
            effects: vec!["Allies provide suppressing fire".into()],
            cooldown: Some(10.0),
            cost: None,
        },
        ToolMetadata {
            name: "coordinate_attack".into(),
            category: "Tactical".into(),
            description: "Coordinate simultaneous attack with allies".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to attack together".into(),
                },
            ],
            preconditions: vec!["Allies available".into(), "Target visible".into()],
            effects: vec!["Synchronized attack with bonus damage".into()],
            cooldown: Some(15.0),
            cost: None,
        },
        ToolMetadata {
            name: "set_ambush".into(),
            category: "Tactical".into(),
            description: "Set up ambush at position".into(),
            parameters: vec![
                ToolParameter {
                    name: "position".into(),
                    param_type: "IVec2".into(),
                    required: true,
                    description: "Ambush position".into(),
                },
            ],
            preconditions: vec!["Position has cover".into()],
            effects: vec!["Gains stealth and first strike bonus".into()],
            cooldown: Some(20.0),
            cost: None,
        },
        ToolMetadata {
            name: "distract".into(),
            category: "Tactical".into(),
            description: "Distract enemy to draw attention".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to distract".into(),
                },
            ],
            preconditions: vec!["Target visible".into()],
            effects: vec!["Target focuses on agent, allies gain advantage".into()],
            cooldown: Some(8.0),
            cost: None,
        },
        ToolMetadata {
            name: "regroup".into(),
            category: "Tactical".into(),
            description: "Rally allies to position".into(),
            parameters: vec![
                ToolParameter {
                    name: "rally_point".into(),
                    param_type: "IVec2".into(),
                    required: true,
                    description: "Rally point coordinates".into(),
                },
            ],
            preconditions: vec!["Allies available".into()],
            effects: vec!["Allies move to rally point".into()],
            cooldown: Some(10.0),
            cost: None,
        },
        
        // ═══════════════════════════════════════
        // UTILITY (5 tools)
        // ═══════════════════════════════════════
        ToolMetadata {
            name: "scan".into(),
            category: "Utility".into(),
            description: "Scan area for threats and items".into(),
            parameters: vec![
                ToolParameter {
                    name: "radius".into(),
                    param_type: "f32".into(),
                    required: true,
                    description: "Scan radius".into(),
                },
            ],
            preconditions: vec![],
            effects: vec!["Reveals enemies and items in radius".into()],
            cooldown: Some(5.0),
            cost: None,
        },
        ToolMetadata {
            name: "wait".into(),
            category: "Utility".into(),
            description: "Wait for specified duration".into(),
            parameters: vec![
                ToolParameter {
                    name: "duration".into(),
                    param_type: "f32".into(),
                    required: true,
                    description: "Duration in seconds".into(),
                },
            ],
            preconditions: vec![],
            effects: vec!["Agent waits, no action taken".into()],
            cooldown: None,
            cost: None,
        },
        ToolMetadata {
            name: "interact".into(),
            category: "Utility".into(),
            description: "Interact with object or NPC".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Object or NPC to interact with".into(),
                },
            ],
            preconditions: vec!["Target in range".into(), "Target is interactable".into()],
            effects: vec!["Triggers interaction (open door, talk, etc.)".into()],
            cooldown: Some(1.0),
            cost: None,
        },
        ToolMetadata {
            name: "use_ability".into(),
            category: "Utility".into(),
            description: "Use special ability".into(),
            parameters: vec![
                ToolParameter {
                    name: "ability_name".into(),
                    param_type: "String".into(),
                    required: true,
                    description: "Name of ability to use".into(),
                },
            ],
            preconditions: vec!["Ability available".into(), "Ability off cooldown".into()],
            effects: vec!["Ability effect applied".into()],
            cooldown: Some(10.0),
            cost: Some("ability charge".into()),
        },
        ToolMetadata {
            name: "taunt".into(),
            category: "Utility".into(),
            description: "Taunt enemy to provoke attack".into(),
            parameters: vec![
                ToolParameter {
                    name: "target_id".into(),
                    param_type: "Entity".into(),
                    required: true,
                    description: "Entity to taunt".into(),
                },
            ],
            preconditions: vec!["Target visible".into()],
            effects: vec!["Target prioritizes agent for attack".into()],
            cooldown: Some(6.0),
            cost: None,
        },
    ]
}

/// Get tool categories for organization
pub fn get_categories() -> Vec<String> {
    vec![
        "Movement".into(),
        "Offensive".into(),
        "Defensive".into(),
        "Equipment".into(),
        "Tactical".into(),
        "Utility".into(),
    ]
}

/// Get tools by category
pub fn get_tools_by_category(category: &str) -> Vec<ToolMetadata> {
    get_all_tools()
        .into_iter()
        .filter(|t| t.category == category)
        .collect()
}

/// Generate JSON schema for tool parameters (for LLM prompts)
pub fn generate_tool_schema(tool: &ToolMetadata) -> String {
    let mut schema = format!(r#"{{"act": "{}""#, tool.name);
    
    for param in &tool.parameters {
        if param.required {
            schema.push_str(&format!(r#", "{}": <{}>"#, param.name, param.param_type));
        } else {
            schema.push_str(&format!(r#", "{}": <{}>?"#, param.name, param.param_type));
        }
    }
    
    schema.push('}');
    schema
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_present() {
        let tools = get_all_tools();
        assert_eq!(tools.len(), 37, "Should have 37 tools");
    }

    #[test]
    fn test_categories() {
        let categories = get_categories();
        assert_eq!(categories.len(), 6, "Should have 6 categories");
        
        let tools = get_all_tools();
        for tool in &tools {
            assert!(categories.contains(&tool.category), 
                    "Tool {} has invalid category {}", tool.name, tool.category);
        }
    }

    #[test]
    fn test_tools_by_category() {
        let movement_tools = get_tools_by_category("Movement");
        assert_eq!(movement_tools.len(), 6, "Should have 6 movement tools");
        
        let offensive_tools = get_tools_by_category("Offensive");
        assert_eq!(offensive_tools.len(), 8, "Should have 8 offensive tools");
        
        let defensive_tools = get_tools_by_category("Defensive");
        assert_eq!(defensive_tools.len(), 6, "Should have 6 defensive tools");
        
        let equipment_tools = get_tools_by_category("Equipment");
        assert_eq!(equipment_tools.len(), 5, "Should have 5 equipment tools");
        
        let tactical_tools = get_tools_by_category("Tactical");
        assert_eq!(tactical_tools.len(), 7, "Should have 7 tactical tools");
        
        let utility_tools = get_tools_by_category("Utility");
        assert_eq!(utility_tools.len(), 5, "Should have 5 utility tools");
    }

    #[test]
    fn test_schema_generation() {
        let tool = ToolMetadata {
            name: "move_to".into(),
            category: "Movement".into(),
            description: "Test".into(),
            parameters: vec![
                ToolParameter {
                    name: "x".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "X coord".into(),
                },
                ToolParameter {
                    name: "y".into(),
                    param_type: "i32".into(),
                    required: true,
                    description: "Y coord".into(),
                },
            ],
            preconditions: vec![],
            effects: vec![],
            cooldown: None,
            cost: None,
        };
        
        let schema = generate_tool_schema(&tool);
        // Tool name is converted to snake_case
        assert!(schema.contains(r#""act": "move_to""#));
        assert!(schema.contains(r#""x": <i32>"#));
        assert!(schema.contains(r#""y": <i32>"#));
    }
}
