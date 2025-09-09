use astraweave_core::{CompanionState, Constraints, EnemyState, IVec2, PlayerState, ToolSpec, ToolRegistry, WorldSnapshot};
use astraweave_llm::{plan_from_llm, MockLlm, parse_llm_plan};
use serde_json;

/// Integration test for end-to-end LLM workflow
#[tokio::test]
async fn test_llm_integration_workflow() {
    let world_snapshot = create_complex_scenario();
    let tool_registry = create_comprehensive_registry();
    
    // Test that MockLlm produces valid output
    let client = MockLlm;
    let plan = plan_from_llm(&client, &world_snapshot, &tool_registry).await
        .expect("MockLlm should produce valid plan");
    
    // Verify plan structure
    assert!(!plan.plan_id.is_empty());
    assert!(!plan.steps.is_empty());
    
    // Verify all steps are valid according to registry
    for step in &plan.steps {
        match step {
            astraweave_core::ActionStep::MoveTo { .. } => {
                assert!(tool_registry.tools.iter().any(|t| t.name == "move_to"));
            }
            astraweave_core::ActionStep::Throw { .. } => {
                assert!(tool_registry.tools.iter().any(|t| t.name == "throw"));
            }
            astraweave_core::ActionStep::CoverFire { .. } => {
                assert!(tool_registry.tools.iter().any(|t| t.name == "cover_fire"));
            }
            astraweave_core::ActionStep::Revive { .. } => {
                assert!(tool_registry.tools.iter().any(|t| t.name == "revive"));
            }
        }
    }
    
    // Test that the plan can be serialized back to JSON
    let json_output = serde_json::to_string(&plan)
        .expect("Plan should be serializable");
    
    // Test that serialized plan can be parsed back
    let reparsed_plan = parse_llm_plan(&json_output, &tool_registry)
        .expect("Serialized plan should be parsable");
    
    assert_eq!(plan.plan_id, reparsed_plan.plan_id);
    assert_eq!(plan.steps.len(), reparsed_plan.steps.len());
}

/// Test that the prompt generation includes all necessary context
#[test]
fn test_prompt_generation_comprehensive() {
    let world_snapshot = create_complex_scenario();
    let tool_registry = create_comprehensive_registry();
    
    let prompt = astraweave_llm::build_prompt(&world_snapshot, &tool_registry);
    
    // Verify prompt contains world state
    assert!(prompt.contains("85")); // Player HP
    assert!(prompt.contains("25")); // Companion ammo
    assert!(prompt.contains("enemies")); // Enemy references - should be plural
    assert!(prompt.contains("extract")); // Objective
    
    // Verify prompt contains tool specifications
    assert!(prompt.contains("move_to"));
    assert!(prompt.contains("throw"));
    assert!(prompt.contains("cover_fire"));
    assert!(prompt.contains("revive"));
    
    // Verify prompt contains constraints info
    assert!(prompt.contains("engine will validate"));
    assert!(prompt.contains("cooldown"));
    assert!(prompt.contains("LOS"));
    
    // Verify JSON schema is present
    assert!(prompt.contains("JSON"));
    assert!(prompt.contains("plan_id"));
    assert!(prompt.contains("steps"));
}

/// Test error handling for various invalid scenarios
#[tokio::test]
async fn test_error_handling_scenarios() {
    let world_snapshot = create_complex_scenario();
    let tool_registry = create_comprehensive_registry();
    
    // Test with client that returns invalid JSON
    struct BadJsonClient;
    
    #[async_trait::async_trait]
    impl astraweave_llm::LlmClient for BadJsonClient {
        async fn complete(&self, _prompt: &str) -> anyhow::Result<String> {
            Ok("This is not JSON at all!".to_string())
        }
    }
    
    let bad_client = BadJsonClient;
    let result = plan_from_llm(&bad_client, &world_snapshot, &tool_registry).await;
    assert!(result.is_err());
    
    // Test with client that returns JSON with disallowed tools
    struct DisallowedToolClient;
    
    #[async_trait::async_trait]
    impl astraweave_llm::LlmClient for DisallowedToolClient {
        async fn complete(&self, _prompt: &str) -> anyhow::Result<String> {
            Ok(r#"{"plan_id": "bad", "steps": [{"act": "Hack", "target": "mainframe"}]}"#.to_string())
        }
    }
    
    let disallowed_client = DisallowedToolClient;
    let result = plan_from_llm(&disallowed_client, &world_snapshot, &tool_registry).await;
    assert!(result.is_err());
}

/// Test validation with different tool registry configurations
#[test]
fn test_tool_registry_validation() {
    // Test with minimal registry
    let minimal_registry = ToolRegistry {
        tools: vec![
            ToolSpec {
                name: "move_to".into(),
                args: [("x", "i32"), ("y", "i32")]
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            }
        ],
        constraints: Constraints {
            enforce_cooldowns: false,
            enforce_los: false,
            enforce_stamina: false,
        },
    };
    
    let valid_json = r#"{"plan_id": "minimal", "steps": [{"act": "MoveTo", "x": 5, "y": 5}]}"#;
    let result = parse_llm_plan(valid_json, &minimal_registry);
    assert!(result.is_ok());
    
    let invalid_json = r#"{"plan_id": "invalid", "steps": [{"act": "Throw", "item": "grenade", "x": 5, "y": 5}]}"#;
    let result = parse_llm_plan(invalid_json, &minimal_registry);
    assert!(result.is_err());
}

fn create_complex_scenario() -> WorldSnapshot {
    WorldSnapshot {
        t: 42.5,
        player: PlayerState {
            hp: 85,
            pos: IVec2 { x: 10, y: 15 },
            stance: "crouch".into(),
            orders: vec!["hold_position".into(), "watch_six".into()],
        },
        me: CompanionState {
            ammo: 25,
            cooldowns: Default::default(),
            morale: 0.8,
            pos: IVec2 { x: 12, y: 15 },
        },
        enemies: vec![
            EnemyState {
                id: 201,
                pos: IVec2 { x: 25, y: 20 },
                hp: 90,
                cover: "heavy".into(),
                last_seen: 2.0,
            },
            EnemyState {
                id: 202,
                pos: IVec2 { x: 18, y: 12 },
                hp: 45,
                cover: "light".into(),
                last_seen: 0.5,
            },
            EnemyState {
                id: 203,
                pos: IVec2 { x: 30, y: 25 },
                hp: 75,
                cover: "none".into(),
                last_seen: 5.0,
            },
        ],
        pois: vec![
            astraweave_core::Poi {
                k: "extract_zone".into(),
                pos: IVec2 { x: 50, y: 50 },
            },
            astraweave_core::Poi {
                k: "ammo_resupply".into(),
                pos: IVec2 { x: 15, y: 8 },
            },
            astraweave_core::Poi {
                k: "high_ground".into(),
                pos: IVec2 { x: 20, y: 25 },
            },
        ],
        objective: Some("Reach extraction zone while eliminating hostiles".into()),
    }
}

fn create_comprehensive_registry() -> ToolRegistry {
    ToolRegistry {
        tools: vec![
            ToolSpec {
                name: "move_to".into(),
                args: [("x", "i32"), ("y", "i32")]
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            },
            ToolSpec {
                name: "throw".into(),
                args: [("item", "enum[smoke,grenade,flashbang]"), ("x", "i32"), ("y", "i32")]
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            },
            ToolSpec {
                name: "cover_fire".into(),
                args: [("target_id", "u32"), ("duration", "f32")]
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            },
            ToolSpec {
                name: "revive".into(),
                args: [("ally_id", "u32")]
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect(),
            },
        ],
        constraints: Constraints {
            enforce_cooldowns: true,
            enforce_los: true,
            enforce_stamina: true,
        },
    }
}