use astraweave_ai::{Orchestrator, RuleOrchestrator};
use astraweave_core::{
    build_snapshot, step, validate_and_execute, IVec2, PerceptionConfig, PlanIntent, SimConfig,
    Team, ValidateCfg, World, WorldSnapshot,
};

#[cfg(feature = "llm")]
use astraweave_core::ToolRegistry;

#[cfg(feature = "llm")]
use astraweave_llm::{plan_from_llm, MockLlm, PlanSource};

/// AI mode selection
#[derive(Debug, Clone, Copy)]
enum AIMode {
    Classical, // GOAP/RuleOrchestrator (default, always works)
    #[cfg(feature = "llm")]
    LLM, // Mock LLM (for demo purposes)
    #[cfg(feature = "llm")]
    Hybrid, // Try LLM, fallback to classical
}

fn main() -> anyhow::Result<()> {
    println!("=== AstraWeave AI Companion Demo ===\n");

    // Check if --demo-both flag is present
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--demo-both".to_string()) {
        #[cfg(feature = "llm")]
        return demo_both_ai_systems();

        #[cfg(not(feature = "llm"))]
        {
            println!("⚠️  --demo-both requires the 'llm' feature flag");
            println!("    Run: cargo run --release -p hello_companion --features llm -- --demo-both\n");
            return Ok(());
        }
    }

    // Select AI mode
    let ai_mode = select_ai_mode();
    println!("AI Mode: {:?}\n", ai_mode);

    // Build a tiny grid arena 20x10 with some obstacles
    let mut w = World::new();
    for x in 6..=6 {
        for y in 1..=8 {
            w.obstacles.insert((x, y));
        }
    } // a vertical wall

    // Spawn entities
    let player = w.spawn("Player", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let comp = w.spawn("Companion", IVec2 { x: 2, y: 3 }, Team { id: 1 }, 80, 30);
    let enemy = w.spawn("Rival", IVec2 { x: 12, y: 2 }, Team { id: 2 }, 60, 0);

    // Prime companion cooldowns
    if let Some(cd) = w.cooldowns_mut(comp) {
        cd.map.insert("throw:smoke".into(), 0.0);
    }

    let p_cfg = PerceptionConfig { los_max: 12 };
    let v_cfg = ValidateCfg {
        world_bounds: (0, 0, 19, 9),
    };
    let s_cfg = SimConfig { dt: 0.25 };

    // Build snapshot
    let enemies = vec![enemy];
    let snap = build_snapshot(&w, player, comp, &enemies, Some("extract".into()), &p_cfg);

    // Generate plan using selected AI mode
    let plan = generate_plan(&snap, ai_mode)?;

    let mut log = |line: String| {
        println!("{}", line);
    };

    println!("--- TICK 0, world time {:.2}", w.t);
    if let Err(e) = validate_and_execute(&mut w, comp, &plan, &v_cfg, &mut log) {
        println!("Plan validation/execution failed: {e}. Continuing without panic.");
    }

    // Progress a few seconds to simulate cooldowns & time
    for _ in 0..20 {
        step(&mut w, &s_cfg);
    }

    println!("--- Post-plan world state @ t={:.2}", w.t);
    println!(
        "Companion @ {:?}, Enemy @ {:?}, Enemy HP = {:?}",
        w.pos_of(comp)
            .expect("Companion entity should have Position component"),
        w.pos_of(enemy)
            .expect("Enemy entity should have Position component"),
        w.health(enemy)
            .expect("Enemy entity should have Health component")
            .hp
    );

    Ok(())
}

/// Select AI mode based on feature flags
fn select_ai_mode() -> AIMode {
    #[cfg(not(feature = "llm"))]
    {
        println!("💡 LLM feature not enabled. Using classical AI.");
        println!("   To enable: cargo run --release -p hello_companion --features llm\n");
        return AIMode::Classical;
    }

    #[cfg(feature = "llm")]
    {
        println!("✅ LLM feature enabled. Using hybrid AI (LLM + fallback).\n");
        AIMode::Hybrid
    }
}

/// Generate AI plan using selected mode
fn generate_plan(snap: &WorldSnapshot, mode: AIMode) -> anyhow::Result<PlanIntent> {
    match mode {
        AIMode::Classical => {
            println!("🤖 Using Classical AI (RuleOrchestrator)...");
            generate_classical_plan(snap)
        }

        #[cfg(feature = "llm")]
        AIMode::LLM => {
            println!("🧠 Using LLM AI...");
            generate_llm_plan(snap)
        }

        #[cfg(feature = "llm")]
        AIMode::Hybrid => {
            println!("🎯 Trying LLM AI with classical fallback...");
            match generate_llm_plan(snap) {
                Ok(plan) => {
                    println!("   ✅ LLM generated plan successfully");
                    Ok(plan)
                }
                Err(e) => {
                    println!("   ⚠️  LLM failed: {}. Falling back to classical AI...", e);
                    generate_classical_plan(snap)
                }
            }
        }
    }
}

/// Generate plan using classical AI (RuleOrchestrator)
fn generate_classical_plan(snap: &WorldSnapshot) -> anyhow::Result<PlanIntent> {
    use std::time::Instant;

    let start = Instant::now();
    let orch = RuleOrchestrator;
    let plan = orch.propose_plan(snap);
    let elapsed = start.elapsed();

    println!(
        "   Classical plan: {} steps ({:.3}ms)",
        plan.steps.len(),
        elapsed.as_secs_f64() * 1000.0
    );

    Ok(plan)
}

/// Generate plan using LLM (MockLlm for demo)
#[cfg(feature = "llm")]
fn generate_llm_plan(snap: &WorldSnapshot) -> anyhow::Result<PlanIntent> {
    use std::time::Instant;

    // Create tool registry (same as classical AI uses)
    let registry = create_tool_registry();

    // Use MockLlm client for demo (no actual Phi-3 model needed)
    let client = MockLlm;

    // Create async runtime for LLM call
    let rt = tokio::runtime::Runtime::new()?;

    let start = Instant::now();
    let result = rt.block_on(async { plan_from_llm(&client, snap, &registry).await });
    let elapsed = start.elapsed();

    match result {
        PlanSource::Llm(plan) => {
            println!(
                "   LLM plan: {} steps ({:.3}ms)",
                plan.steps.len(),
                elapsed.as_secs_f64() * 1000.0
            );

            // Show LLM reasoning
            println!("   LLM reasoning:");
            for (i, step) in plan.steps.iter().enumerate() {
                println!("      {}. {:?}", i + 1, step);
            }

            Ok(plan)
        }
        PlanSource::Fallback { plan, reason } => {
            println!("   LLM returned fallback plan: {}", reason);
            Ok(plan)
        }
    }
}

/// Create tool registry for LLM
#[cfg(feature = "llm")]
fn create_tool_registry() -> ToolRegistry {
    use astraweave_core::{Constraints, ToolSpec};
    use std::collections::BTreeMap;

    ToolRegistry {
        tools: vec![
            ToolSpec {
                name: "MoveTo".into(),
                args: {
                    let mut m = BTreeMap::new();
                    m.insert("x".into(), "i32".into());
                    m.insert("y".into(), "i32".into());
                    m
                },
            },
            ToolSpec {
                name: "Throw".into(),
                args: {
                    let mut m = BTreeMap::new();
                    m.insert("item".into(), "enum[smoke,grenade]".into());
                    m.insert("x".into(), "i32".into());
                    m.insert("y".into(), "i32".into());
                    m
                },
            },
            ToolSpec {
                name: "CoverFire".into(),
                args: {
                    let mut m = BTreeMap::new();
                    m.insert("target_id".into(), "u32".into());
                    m.insert("duration".into(), "f32".into());
                    m
                },
            },
        ],
        constraints: Constraints {
            enforce_cooldowns: true,
            enforce_los: true,
            enforce_stamina: true,
        },
    }
}

/// Compare both AI systems side-by-side
#[cfg(feature = "llm")]
fn demo_both_ai_systems() -> anyhow::Result<()> {
    println!("=== AstraWeave AI Comparison Demo ===\n");

    // Setup world
    let mut w = World::new();
    for x in 6..=6 {
        for y in 1..=8 {
            w.obstacles.insert((x, y));
        }
    }

    let player = w.spawn("Player", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
    let comp = w.spawn("Companion", IVec2 { x: 2, y: 3 }, Team { id: 1 }, 80, 30);
    let enemy = w.spawn("Rival", IVec2 { x: 12, y: 2 }, Team { id: 2 }, 60, 0);

    if let Some(cd) = w.cooldowns_mut(comp) {
        cd.map.insert("throw:smoke".into(), 0.0);
    }

    let p_cfg = PerceptionConfig { los_max: 12 };
    let enemies = vec![enemy];
    let snap = build_snapshot(&w, player, comp, &enemies, Some("extract".into()), &p_cfg);

    // Run classical AI
    println!("--- CLASSICAL AI ---");
    let classical_plan = generate_classical_plan(&snap)?;

    println!();

    // Run LLM AI
    println!("--- LLM AI (MockLlm) ---");
    let llm_plan = match generate_llm_plan(&snap) {
        Ok(plan) => plan,
        Err(e) => {
            println!("   LLM failed: {}", e);
            return Ok(());
        }
    };

    // Compare
    println!("\n--- COMPARISON ---");
    println!("Classical steps: {}", classical_plan.steps.len());
    println!("LLM steps:       {}", llm_plan.steps.len());

    if classical_plan.steps.len() == llm_plan.steps.len() {
        println!("✅ Both generated {} step plans", classical_plan.steps.len());
    }

    println!("\n💡 Note: Using MockLlm for demo. Enable real Phi-3 with --features phi3");

    Ok(())
}
