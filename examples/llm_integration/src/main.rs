use astraweave_core::*;
use astraweave_llm::{plan_from_llm, LocalHttpClient, MockLlm, OllamaClient};
use std::env;

/// Comprehensive LLM integration example demonstrating multiple client types
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("AstraWeave LLM Integration Example");
    println!("==================================");

    // Create a test scenario
    let world_snapshot = create_test_scenario();
    let tool_registry = create_tool_registry();

    println!("\nWorld Snapshot:");
    println!("{}", serde_json::to_string_pretty(&world_snapshot)?);

    println!("\nTool Registry:");
    println!("{}", serde_json::to_string_pretty(&tool_registry)?);

    // 1. Test MockLlm (always available)
    println!("\n1. Testing MockLlm Client");
    println!("--------------------------");
    test_mock_client(&world_snapshot, &tool_registry).await?;

    // 2. Test Ollama (if URL provided)
    if let Ok(ollama_url) = env::var("OLLAMA_URL") {
        println!("\n2. Testing Ollama Client");
        println!("-------------------------");
        test_ollama_client(&world_snapshot, &tool_registry, &ollama_url).await?;
    } else {
        println!("\n2. Ollama Client (Skipped - set OLLAMA_URL environment variable to test)");
    }

    // 3. Test LocalHttpClient (if URL provided)
    if let Ok(local_url) = env::var("LOCAL_LLM_URL") {
        println!("\n3. Testing Local HTTP Client");
        println!("-----------------------------");
        test_local_http_client(&world_snapshot, &tool_registry, &local_url).await?;
    } else {
        println!(
            "\n3. Local HTTP Client (Skipped - set LOCAL_LLM_URL environment variable to test)"
        );
    }

    println!("\nExample completed successfully!");
    println!("\nTo test with real LLM services:");
    println!("  OLLAMA_URL=http://localhost:11434 OLLAMA_MODEL=llama2 cargo run");
    println!("  LOCAL_LLM_URL=http://localhost:5000 LOCAL_LLM_MODEL=gpt-3.5-turbo cargo run");

    Ok(())
}

async fn test_mock_client(snap: &WorldSnapshot, reg: &ToolRegistry) -> anyhow::Result<()> {
    let client = MockLlm;
    match plan_from_llm(&client, snap, reg).await {
        Ok(plan) => {
            println!("✓ MockLlm generated plan:");
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Err(e) => {
            println!("✗ MockLlm failed: {}", e);
        }
    }
    Ok(())
}

async fn test_ollama_client(
    snap: &WorldSnapshot,
    reg: &ToolRegistry,
    url: &str,
) -> anyhow::Result<()> {
    let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama2".to_string());
    let client = OllamaClient {
        url: url.to_string(),
        model,
    };

    println!("Connecting to Ollama at: {}", url);
    match plan_from_llm(&client, snap, reg).await {
        Ok(plan) => {
            println!("✓ Ollama generated plan:");
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Err(e) => {
            println!("✗ Ollama failed: {}", e);
            println!("  Make sure Ollama is running and the model is available");
        }
    }
    Ok(())
}

async fn test_local_http_client(
    snap: &WorldSnapshot,
    reg: &ToolRegistry,
    url: &str,
) -> anyhow::Result<()> {
    let model = env::var("LOCAL_LLM_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
    let client = if let Ok(api_key) = env::var("LOCAL_LLM_API_KEY") {
        LocalHttpClient::with_api_key(url.to_string(), model, api_key)
    } else {
        LocalHttpClient::new(url.to_string(), model)
    };

    println!("Connecting to local LLM at: {}", url);
    match plan_from_llm(&client, snap, reg).await {
        Ok(plan) => {
            println!("✓ Local HTTP client generated plan:");
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        Err(e) => {
            println!("✗ Local HTTP client failed: {}", e);
            println!(
                "  Make sure your local LLM service is running and compatible with OpenAI API"
            );
        }
    }
    Ok(())
}

fn create_test_scenario() -> WorldSnapshot {
    WorldSnapshot {
        t: 1.0,
        player: PlayerState {
            hp: 85,
            pos: IVec2 { x: 2, y: 3 },
            stance: "crouch".into(),
            orders: vec![],
        },
        me: CompanionState {
            ammo: 25,
            cooldowns: Default::default(),
            morale: 0.8,
            pos: IVec2 { x: 4, y: 3 },
        },
        enemies: vec![
            EnemyState {
                id: 101,
                pos: IVec2 { x: 15, y: 5 },
                hp: 75,
                cover: "high".into(),
                last_seen: 0.5,
            },
            EnemyState {
                id: 102,
                pos: IVec2 { x: 12, y: 8 },
                hp: 40,
                cover: "none".into(),
                last_seen: 1.0,
            },
        ],
        pois: vec![
            Poi {
                k: "extract_point".into(),
                pos: IVec2 { x: 20, y: 10 },
            },
            Poi {
                k: "ammo_cache".into(),
                pos: IVec2 { x: 8, y: 6 },
            },
        ],
        objective: Some("Reach extraction point while providing cover".into()),
    }
}

fn create_tool_registry() -> ToolRegistry {
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
                args: [
                    ("item", "enum[smoke,grenade,flashbang]"),
                    ("x", "i32"),
                    ("y", "i32"),
                ]
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
