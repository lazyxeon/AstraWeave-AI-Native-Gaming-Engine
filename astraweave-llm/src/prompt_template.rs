//! Prompt Template System - Phase 7 Enhanced Prompting
//!
//! This module provides robust prompt engineering with:
//! - JSON schema enforcement for all 37 tools
//! - Few-shot learning examples
//! - Tool vocabulary descriptions
//! - Hallucination prevention strategies

use astraweave_core::{ToolRegistry, WorldSnapshot, get_tools_by_category};

/// Prompt configuration options
#[derive(Clone, Debug)]
pub struct PromptConfig {
    /// Include few-shot examples
    pub include_examples: bool,
    /// Include detailed tool descriptions
    pub include_tool_descriptions: bool,
    /// Include JSON schema
    pub include_schema: bool,
    /// Maximum number of examples to include
    pub max_examples: usize,
    /// Emphasize JSON-only output
    pub strict_json_only: bool,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            include_examples: true,
            include_tool_descriptions: true,
            include_schema: true,
            max_examples: 5,
            strict_json_only: true,
        }
    }
}

/// Build enhanced prompt with Phase 7 improvements
pub fn build_enhanced_prompt(
    snap: &WorldSnapshot,
    reg: &ToolRegistry,
    config: &PromptConfig,
) -> String {
    let mut parts = Vec::new();

    // System message - role definition
    parts.push(build_system_message());

    // Tool vocabulary with descriptions
    if config.include_tool_descriptions {
        parts.push(build_tool_vocabulary());
    } else {
        parts.push(build_tool_list(reg));
    }

    // JSON schema
    if config.include_schema {
        parts.push(build_json_schema());
    }

    // Few-shot examples
    if config.include_examples {
        parts.push(build_few_shot_examples(config.max_examples));
    }

    // World snapshot
    parts.push(build_snapshot_section(snap));

    // Output instructions
    parts.push(build_output_instructions(config.strict_json_only));

    parts.join("\n\n")
}

/// System message defining the AI's role
fn build_system_message() -> String {
    r#"You are a tactical AI companion in a combat scenario. Your role is to:
1. Analyze the current world state
2. Generate a valid action plan using ONLY the allowed tools
3. Return STRICTLY VALID JSON matching the schema below

CRITICAL RULES:
- Use ONLY tools from the allowed list
- Do NOT invent new tools or parameters
- All tool names and parameters must EXACTLY match the schema
- Return ONLY JSON - no commentary, explanations, or markdown
- If uncertain, use simpler actions (MoveTo, Wait, Scan)"#.to_string()
}

/// Build comprehensive tool vocabulary with descriptions
fn build_tool_vocabulary() -> String {
    let mut output = String::from("═══════════════════════════════════════\nAVAILABLE TOOLS (37 total)\n═══════════════════════════════════════\n\n");

    let categories = vec!["Movement", "Offensive", "Defensive", "Equipment", "Tactical", "Utility"];
    
    for category in categories {
        output.push_str(&format!("## {} Tools:\n\n", category));
        
        let tools = get_tools_by_category(category);
        for tool in tools {
            output.push_str(&format!("• **{}**\n", tool.name));
            output.push_str(&format!("  Description: {}\n", tool.description));
            
            if !tool.parameters.is_empty() {
                output.push_str("  Parameters:\n");
                for param in &tool.parameters {
                    let req = if param.required { "REQUIRED" } else { "optional" };
                    output.push_str(&format!("    - {}: {} ({})\n", param.name, param.param_type, req));
                }
            }
            
            if let Some(cooldown) = tool.cooldown {
                output.push_str(&format!("  Cooldown: {:.1}s\n", cooldown));
            }
            
            output.push('\n');
        }
    }

    output
}

/// Build simple tool list (legacy format)
fn build_tool_list(reg: &ToolRegistry) -> String {
    let tool_list = reg
        .tools
        .iter()
        .map(|t| format!(" - {} {:?}", t.name, t.args))
        .collect::<Vec<_>>()
        .join("\n");
    
    format!("Allowed tools:\n{}", tool_list)
}

/// Build comprehensive JSON schema with all 37 tools
fn build_json_schema() -> String {
    r#"═══════════════════════════════════════
JSON SCHEMA (STRICT)
═══════════════════════════════════════

Your response MUST match this exact structure:

{
  "plan_id": "string (unique identifier)",
  "steps": [
    // Choose ONE of the following per step:
    
    // MOVEMENT
    {"act": "MoveTo", "x": INT, "y": INT, "speed": "walk|run|sprint"?},
    {"act": "Approach", "target_id": INT, "distance": FLOAT},
    {"act": "Retreat", "target_id": INT, "distance": FLOAT},
    {"act": "TakeCover", "position": {"x": INT, "y": INT}?},
    {"act": "Strafe", "target_id": INT, "direction": "left|right"},
    {"act": "Patrol", "waypoints": [{"x": INT, "y": INT}, ...]},
    
    // OFFENSIVE
    {"act": "Attack", "target_id": INT},
    {"act": "AimedShot", "target_id": INT},
    {"act": "QuickAttack", "target_id": INT},
    {"act": "HeavyAttack", "target_id": INT},
    {"act": "AoEAttack", "x": INT, "y": INT, "radius": FLOAT},
    {"act": "ThrowExplosive", "x": INT, "y": INT},
    {"act": "CoverFire", "target_id": INT, "duration": FLOAT},
    {"act": "Charge", "target_id": INT},
    
    // DEFENSIVE
    {"act": "Block"},
    {"act": "Dodge", "direction": "left|right"?},
    {"act": "Parry"},
    {"act": "ThrowSmoke", "x": INT, "y": INT},
    {"act": "Heal", "target_id": INT?},
    {"act": "UseDefensiveAbility", "ability_name": "string"},
    
    // EQUIPMENT
    {"act": "EquipWeapon", "weapon_name": "string"},
    {"act": "SwitchWeapon", "slot": INT},
    {"act": "Reload"},
    {"act": "UseItem", "item_name": "string"},
    {"act": "DropItem", "item_name": "string"},
    
    // TACTICAL
    {"act": "CallReinforcements", "count": INT},
    {"act": "MarkTarget", "target_id": INT},
    {"act": "RequestCover", "duration": FLOAT},
    {"act": "CoordinateAttack", "target_id": INT},
    {"act": "SetAmbush", "position": {"x": INT, "y": INT}},
    {"act": "Distract", "target_id": INT},
    {"act": "Regroup", "rally_point": {"x": INT, "y": INT}},
    
    // UTILITY
    {"act": "Scan", "radius": FLOAT},
    {"act": "Wait", "duration": FLOAT},
    {"act": "Interact", "target_id": INT},
    {"act": "UseAbility", "ability_name": "string"},
    {"act": "Taunt", "target_id": INT}
  ]
}

CRITICAL:
- "act" field MUST be one of the exact tool names above
- All parameters must match the types shown (INT, FLOAT, string)
- Optional parameters are marked with "?"
- Do NOT add extra fields or invent new tool names"#.to_string()
}

/// Build few-shot learning examples
fn build_few_shot_examples(max_examples: usize) -> String {
    let examples = vec![
        FewShotExample {
            scenario: "Enemy spotted at close range, low health",
            world_state: r#"{"t": 10.5, "me": {"pos": {"x": 5, "y": 5}, "hp": 30, "ammo": 15}, "enemies": [{"id": 42, "pos": {"x": 8, "y": 5}, "hp": 80}]}"#,
            plan: r#"{"plan_id": "ex1", "steps": [{"act": "ThrowSmoke", "x": 7, "y": 5}, {"act": "Retreat", "target_id": 42, "distance": 20.0}, {"act": "Heal"}]}"#,
            explanation: "Smoke for concealment, retreat to safety, heal",
        },
        FewShotExample {
            scenario: "Multiple enemies, good position",
            world_state: r#"{"t": 5.0, "me": {"pos": {"x": 10, "y": 10}, "ammo": 30}, "enemies": [{"id": 1, "pos": {"x": 15, "y": 10}}, {"id": 2, "pos": {"x": 16, "y": 11}}]}"#,
            plan: r#"{"plan_id": "ex2", "steps": [{"act": "TakeCover"}, {"act": "MarkTarget", "target_id": 1}, {"act": "AimedShot", "target_id": 1}, {"act": "Attack", "target_id": 2}]}"#,
            explanation: "Take cover, mark priority target, eliminate threats",
        },
        FewShotExample {
            scenario: "Need to reach objective",
            world_state: r#"{"t": 2.0, "me": {"pos": {"x": 0, "y": 0}}, "pois": [{"k": "objective", "pos": {"x": 20, "y": 20}}], "enemies": []}"#,
            plan: r#"{"plan_id": "ex3", "steps": [{"act": "Scan", "radius": 15.0}, {"act": "MoveTo", "x": 20, "y": 20, "speed": "run"}]}"#,
            explanation: "Scan for threats, then move to objective quickly",
        },
        FewShotExample {
            scenario: "Surrounded, low ammo",
            world_state: r#"{"t": 15.0, "me": {"pos": {"x": 10, "y": 10}, "ammo": 3}, "enemies": [{"id": 5, "pos": {"x": 8, "y": 10}}, {"id": 6, "pos": {"x": 12, "y": 10}}, {"id": 7, "pos": {"x": 10, "y": 12}}]}"#,
            plan: r#"{"plan_id": "ex4", "steps": [{"act": "ThrowSmoke", "x": 10, "y": 10}, {"act": "ThrowExplosive", "x": 10, "y": 12}, {"act": "Retreat", "target_id": 5, "distance": 30.0}, {"act": "Reload"}]}"#,
            explanation: "Create chaos with smoke/explosive, escape, reload",
        },
        FewShotExample {
            scenario: "Tactical advantage, allies available",
            world_state: r#"{"t": 20.0, "me": {"pos": {"x": 5, "y": 5}}, "enemies": [{"id": 10, "pos": {"x": 15, "y": 5}, "hp": 50}], "allies": 2}"#,
            plan: r#"{"plan_id": "ex5", "steps": [{"act": "MarkTarget", "target_id": 10}, {"act": "CoordinateAttack", "target_id": 10}, {"act": "CoverFire", "target_id": 10, "duration": 3.0}]}"#,
            explanation: "Coordinate with allies for focused assault",
        },
    ];

    let mut output = String::from("═══════════════════════════════════════\nFEW-SHOT EXAMPLES\n═══════════════════════════════════════\n\n");
    
    for (i, example) in examples.iter().take(max_examples).enumerate() {
        output.push_str(&format!("Example {}:\n", i + 1));
        output.push_str(&format!("Scenario: {}\n", example.scenario));
        output.push_str(&format!("World State: {}\n", example.world_state));
        output.push_str(&format!("Plan: {}\n", example.plan));
        output.push_str(&format!("Reasoning: {}\n\n", example.explanation));
    }

    output
}

struct FewShotExample {
    scenario: &'static str,
    world_state: &'static str,
    plan: &'static str,
    explanation: &'static str,
}

/// Build world snapshot section
fn build_snapshot_section(snap: &WorldSnapshot) -> String {
    let snap_json = serde_json::to_string_pretty(snap).unwrap_or_else(|_| "{}".to_string());
    
    format!(
        r#"═══════════════════════════════════════
CURRENT WORLD STATE
═══════════════════════════════════════

{}

Analyze this state and generate your tactical plan."#,
        snap_json
    )
}

/// Build output instructions
fn build_output_instructions(strict_json: bool) -> String {
    if strict_json {
        r#"═══════════════════════════════════════
OUTPUT INSTRUCTIONS
═══════════════════════════════════════

Return ONLY the JSON plan object. No explanations, no commentary, no markdown.

CORRECT: {"plan_id": "p1", "steps": [{"act": "MoveTo", "x": 10, "y": 5}]}

INCORRECT: 
```json
{"plan_id": "p1", "steps": [...]}
```

INCORRECT:
Here's my plan: {"plan_id": "p1", ...}

Generate your plan now:"#.to_string()
    } else {
        "Generate your tactical plan based on the world state above.".to_string()
    }
}

/// Build prompt with legacy compatibility (for gradual migration)
pub fn build_prompt_legacy(snap: &WorldSnapshot, reg: &ToolRegistry) -> String {
    build_enhanced_prompt(snap, reg, &PromptConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{default_tool_registry, WorldSnapshot};

    #[test]
    fn test_build_enhanced_prompt() {
        let snap = WorldSnapshot::default();
        let reg = default_tool_registry();
        let config = PromptConfig::default();
        
        let prompt = build_enhanced_prompt(&snap, &reg, &config);
        
        // Verify key sections are present
        assert!(prompt.contains("tactical AI companion"));
        assert!(prompt.contains("AVAILABLE TOOLS"));
        assert!(prompt.contains("JSON SCHEMA"));
        assert!(prompt.contains("FEW-SHOT EXAMPLES"));
        assert!(prompt.contains("CURRENT WORLD STATE"));
        assert!(prompt.contains("OUTPUT INSTRUCTIONS"));
    }

    #[test]
    fn test_config_options() {
        let snap = WorldSnapshot::default();
        let reg = default_tool_registry();
        
        // Minimal config
        let config = PromptConfig {
            include_examples: false,
            include_tool_descriptions: false,
            include_schema: false,
            max_examples: 0,
            strict_json_only: false,
        };
        
        let prompt = build_enhanced_prompt(&snap, &reg, &config);
        assert!(!prompt.contains("FEW-SHOT EXAMPLES"));
        assert!(!prompt.contains("JSON SCHEMA"));
    }

    #[test]
    fn test_few_shot_examples_count() {
        let snap = WorldSnapshot::default();
        let reg = default_tool_registry();
        
        // Request 3 examples
        let config = PromptConfig {
            max_examples: 3,
            ..Default::default()
        };
        
        let prompt = build_enhanced_prompt(&snap, &reg, &config);
        
        // Should have Example 1, 2, 3 but not 4, 5
        assert!(prompt.contains("Example 1:"));
        assert!(prompt.contains("Example 2:"));
        assert!(prompt.contains("Example 3:"));
        assert!(!prompt.contains("Example 4:"));
    }

    #[test]
    fn test_tool_vocabulary_includes_all_categories() {
        let vocab = build_tool_vocabulary();
        
        assert!(vocab.contains("Movement Tools"));
        assert!(vocab.contains("Offensive Tools"));
        assert!(vocab.contains("Defensive Tools"));
        assert!(vocab.contains("Equipment Tools"));
        assert!(vocab.contains("Tactical Tools"));
        assert!(vocab.contains("Utility Tools"));
    }

    #[test]
    fn test_json_schema_has_all_tools() {
        let schema = build_json_schema();
        
        // Spot check a few from each category
        assert!(schema.contains(r#""act": "MoveTo""#));
        assert!(schema.contains(r#""act": "Attack""#));
        assert!(schema.contains(r#""act": "ThrowSmoke""#));
        assert!(schema.contains(r#""act": "Reload""#));
        assert!(schema.contains(r#""act": "MarkTarget""#));
        assert!(schema.contains(r#""act": "Scan""#));
    }
}
