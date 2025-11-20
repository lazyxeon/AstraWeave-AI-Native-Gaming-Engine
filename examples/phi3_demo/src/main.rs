//! Phi-3 Demo - Interactive AI Planning Showcase
//!
//! This example demonstrates AstraWeave's LLM integration with Phi-3 Medium.
//! It shows real-time tactical AI decision making using the Ollama backend.
//!
//! Prerequisites:
//! 1. Install Ollama: https://ollama.ai/download
//! 2. Run: `ollama pull phi3:medium`
//! 3. Run: `ollama serve` (in another terminal)
//!
//! Usage:
//! ```bash
//! cargo run -p phi3_demo --release
//! ```

use astraweave_core::{CompanionState, EnemyState, IVec2, PlayerState, Poi, WorldSnapshot};
use astraweave_llm::phi3_ollama::Phi3Ollama;
use astraweave_llm::prompts::{quick, PromptBuilder};
use astraweave_llm::LlmClient;
use colored::*;
use std::collections::BTreeMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "=== AstraWeave Phi-3 Demo ===".bright_cyan().bold());
    println!();

    // Step 1: Health Check
    println!("{}", "🔍 Checking Phi-3 setup...".yellow());

    // Use fast variant for low-latency demo (phi3:game - optimized mini for 6GB VRAM)
    let client = Phi3Ollama::fast();

    println!(
        "{}",
        "⚡ Using phi3:game model (3.8B params, optimized for 6GB VRAM)".bright_yellow()
    );
    println!("{}", "   Expected latency: 0.5-2s per request".dimmed());

    let health = client.health_check().await?;

    if !health.is_ready() {
        eprintln!("{}", "❌ Setup incomplete!".red().bold());
        eprintln!("{}", health.error_message().unwrap().red());
        std::process::exit(1);
    }

    println!("{}", "✅ Ollama server: Running".green());
    println!("{}", "✅ Model phi3:medium: Available".green());
    println!(
        "📦 Ollama version: {}",
        health.ollama_version.bright_white()
    );
    println!();

    // Step 2: Create Tactical Scenario
    println!("{}", "🎮 Creating tactical scenario...".yellow());
    let scenario = create_combat_scenario();
    print_scenario(&scenario);
    println!();

    // Step 3: Run All AI Roles
    run_tactical_demo(&client, &scenario).await?;
    run_stealth_demo(&client, &scenario).await?;
    run_support_demo(&client, &scenario).await?;
    run_exploration_demo(&client, &scenario).await?;

    // Step 4: Custom Prompt
    run_custom_prompt_demo(&client, &scenario).await?;

    println!();
    println!("{}", "🎉 Demo complete!".bright_green().bold());
    println!(
        "{}",
        "See docs/PHI3_SETUP.md for integration guides.".bright_white()
    );

    Ok(())
}

fn create_combat_scenario() -> WorldSnapshot {
    WorldSnapshot {
        t: 45.0,
        player: PlayerState {
            hp: 75,
            pos: IVec2 { x: 10, y: 10 },
            stance: "crouch".to_string(),
            orders: vec!["hold position".to_string()],
        },
        me: CompanionState {
            ammo: 18,
            cooldowns: BTreeMap::from([("grenade".to_string(), 0.0)]),
            morale: 80.0,
            pos: IVec2 { x: 12, y: 10 },
        },
        enemies: vec![
            EnemyState {
                id: 99, // Entity is just u32
                pos: IVec2 { x: 25, y: 15 },
                hp: 100,
                cover: "crate".to_string(),
                last_seen: 2.0,
            },
            EnemyState {
                id: 100,
                pos: IVec2 { x: 28, y: 12 },
                hp: 80,
                cover: "wall".to_string(),
                last_seen: 5.0,
            },
        ],
        pois: vec![
            Poi {
                k: "ammo_cache".to_string(),
                pos: IVec2 { x: 15, y: 8 },
            },
            Poi {
                k: "health_pack".to_string(),
                pos: IVec2 { x: 20, y: 20 },
            },
        ],
        obstacles: vec![
            IVec2 { x: 18, y: 12 },
            IVec2 { x: 18, y: 13 },
            IVec2 { x: 18, y: 14 },
        ],
        objective: Some("Eliminate all hostiles".to_string()),
    }
}

fn print_scenario(scenario: &WorldSnapshot) {
    println!("  ⏱️  Time: {:.1}s", scenario.t);
    println!(
        "  👤 Player: pos({}, {}) | HP: {} | Stance: {}",
        scenario.player.pos.x, scenario.player.pos.y, scenario.player.hp, scenario.player.stance
    );
    println!(
        "  🤖 Companion: pos({}, {}) | Morale: {:.0} | Ammo: {}",
        scenario.me.pos.x, scenario.me.pos.y, scenario.me.morale, scenario.me.ammo
    );
    println!("  ☠️  Enemies: {}", scenario.enemies.len());
    for e in &scenario.enemies {
        println!(
            "     - Enemy {}: pos({}, {}) | HP: {} | Cover: {} | Last seen: {:.1}s ago",
            e.id, e.pos.x, e.pos.y, e.hp, e.cover, e.last_seen
        );
    }
    println!("  📍 Points of Interest: {}", scenario.pois.len());
    for poi in &scenario.pois {
        println!("     - {}: pos({}, {})", poi.k, poi.pos.x, poi.pos.y);
    }
    println!("  🎯 Objective: {}", scenario.objective.as_ref().unwrap());
}

async fn run_tactical_demo(client: &Phi3Ollama, scenario: &WorldSnapshot) -> anyhow::Result<()> {
    println!(
        "{}",
        "━━━ TACTICAL AI (Aggressive) ━━━".bright_magenta().bold()
    );
    println!(
        "{}",
        "Optimized for combat effectiveness and direct engagement".dimmed()
    );
    println!();

    let prompt = quick::tactical_prompt(scenario, "Eliminate all hostiles");

    println!("{}", "🧠 Querying Phi-3...".yellow());
    let start = std::time::Instant::now();

    let response = client.complete(&prompt).await?;

    let duration = start.elapsed();
    println!(
        "{} ({:.2}s)",
        "✅ Response received".green(),
        duration.as_secs_f32()
    );
    println!();

    print_llm_response("TACTICAL", &response);
    Ok(())
}

async fn run_stealth_demo(client: &Phi3Ollama, scenario: &WorldSnapshot) -> anyhow::Result<()> {
    println!();
    println!(
        "{}",
        "━━━ STEALTH AI (Cautious) ━━━".bright_magenta().bold()
    );
    println!(
        "{}",
        "Optimized for silent infiltration and avoidance".dimmed()
    );
    println!();

    let prompt = quick::stealth_prompt(scenario, "pos(30, 20)");

    println!("{}", "🧠 Querying Phi-3...".yellow());
    let start = std::time::Instant::now();

    let response = client.complete(&prompt).await?;

    let duration = start.elapsed();
    println!(
        "{} ({:.2}s)",
        "✅ Response received".green(),
        duration.as_secs_f32()
    );
    println!();

    print_llm_response("STEALTH", &response);
    Ok(())
}

async fn run_support_demo(client: &Phi3Ollama, scenario: &WorldSnapshot) -> anyhow::Result<()> {
    println!();
    println!(
        "{}",
        "━━━ SUPPORT AI (Team-focused) ━━━".bright_magenta().bold()
    );
    println!(
        "{}",
        "Optimized for ally protection and defensive tactics".dimmed()
    );
    println!();

    // Modify scenario: player is wounded
    let mut support_scenario = scenario.clone();
    support_scenario.player.hp = 35; // Critical HP!

    let prompt = quick::support_prompt(&support_scenario, 0); // Entity 0 = player

    println!("{}", "🧠 Querying Phi-3...".yellow());
    let start = std::time::Instant::now();

    let response = client.complete(&prompt).await?;

    let duration = start.elapsed();
    println!(
        "{} ({:.2}s)",
        "✅ Response received".green(),
        duration.as_secs_f32()
    );
    println!();

    print_llm_response("SUPPORT", &response);
    Ok(())
}

async fn run_exploration_demo(client: &Phi3Ollama, scenario: &WorldSnapshot) -> anyhow::Result<()> {
    println!();
    println!(
        "{}",
        "━━━ EXPLORATION AI (Curious) ━━━".bright_magenta().bold()
    );
    println!(
        "{}",
        "Optimized for reconnaissance and POI investigation".dimmed()
    );
    println!();

    let prompt = quick::exploration_prompt(scenario);

    println!("{}", "🧠 Querying Phi-3...".yellow());
    let start = std::time::Instant::now();

    let response = client.complete(&prompt).await?;

    let duration = start.elapsed();
    println!(
        "{} ({:.2}s)",
        "✅ Response received".green(),
        duration.as_secs_f32()
    );
    println!();

    print_llm_response("EXPLORATION", &response);
    Ok(())
}

async fn run_custom_prompt_demo(
    client: &Phi3Ollama,
    scenario: &WorldSnapshot,
) -> anyhow::Result<()> {
    println!();
    println!(
        "{}",
        "━━━ CUSTOM PROMPT (Builder API) ━━━"
            .bright_magenta()
            .bold()
    );
    println!(
        "{}",
        "Demonstrating PromptBuilder with custom constraints".dimmed()
    );
    println!();

    let prompt = PromptBuilder::new()
        .system_role("tactical")
        .add_snapshot(scenario)
        .add_goal("Flank enemy position from the west")
        .add_constraint("Never cross open ground without smoke cover")
        .add_constraint("Conserve ammo - use grenades if available")
        .add_constraint("Prioritize high-value targets (enemies in cover)")
        .build();

    println!(
        "{}",
        "🧠 Querying Phi-3 with custom constraints...".yellow()
    );
    let start = std::time::Instant::now();

    let response = client.complete(&prompt).await?;

    let duration = start.elapsed();
    println!(
        "{} ({:.2}s)",
        "✅ Response received".green(),
        duration.as_secs_f32()
    );
    println!();

    print_llm_response("CUSTOM", &response);
    Ok(())
}

fn print_llm_response(role: &str, response: &str) {
    println!(
        "{} {}",
        "📋".bright_yellow(),
        format!("[{}]", role).bright_white().bold()
    );
    println!("{}", "─".repeat(60).dimmed());

    // Try to parse as JSON and pretty-print
    match serde_json::from_str::<serde_json::Value>(response) {
        Ok(json) => {
            let pretty = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", pretty.bright_white());

            // Highlight key fields
            if let Some(plan_id) = json.get("plan_id") {
                println!();
                println!("{} {}", "🆔 Plan ID:".bright_cyan(), plan_id);
            }
            if let Some(reasoning) = json.get("reasoning") {
                println!("{} {}", "💡 Reasoning:".bright_green(), reasoning);
            }
            if let Some(steps) = json.get("steps").and_then(|s| s.as_array()) {
                println!("{} {} actions", "⚡ Steps:".bright_yellow(), steps.len());
            }
        }
        Err(_) => {
            // Fallback: just print raw response
            println!("{}", response.bright_white());
        }
    }

    println!("{}", "─".repeat(60).dimmed());
}
