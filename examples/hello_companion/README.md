# hello_companion - AI Companion Demo

**Showcase of AstraWeave's hybrid AI architecture: Classical AI + LLM with graceful fallback**

## Overview

This example demonstrates:
- **Classical AI**: GOAP-based planning (always works, no dependencies)
- **LLM AI**: Phi-3 language model planning (optional, requires model download)
- **Hybrid Mode**: Try LLM first, fallback to classical on errors
- **Comparison Demo**: Side-by-side evaluation of both systems

## Quick Start

### Classical AI (Default)
```bash
# No dependencies, works immediately
cargo run -p hello_companion --release

# Expected output:
# 💡 LLM feature not enabled. Using classical AI.
# 🤖 Using Classical AI (RuleOrchestrator)...
#    Classical plan: 3 steps (0.023ms)
```

### LLM AI (Optional)
```bash
# Enable LLM feature (uses MockLlm for demo)
cargo run -p hello_companion --release --features llm

# Expected output:
# ✅ LLM feature enabled. Using hybrid AI (LLM + fallback).
# 🎯 Trying LLM AI with classical fallback...
#    LLM returned fallback plan: [reason]
```

### Comparison Demo
```bash
# Compare both systems side-by-side
cargo run -p hello_companion --release --features llm -- --demo-both

# Expected output:
# --- CLASSICAL AI ---
#    Classical plan: 3 steps (0.014ms)
#
# --- LLM AI (MockLlm) ---
#    LLM returned fallback plan: [reason]
#
# --- COMPARISON ---
# Classical steps: 3
# LLM steps:       0
```

## AI Modes

### Classical AI
- **Type**: Rule-based GOAP planner (`RuleOrchestrator`)
- **Latency**: ~0.02ms (sub-millisecond)
- **Reliability**: 100% (deterministic, no external dependencies)
- **Use Case**: Production gameplay (fast, reliable, always works)

### LLM AI
- **Type**: Phi-3 language model (via MockLlm demo or real Phi-3)
- **Latency**: 50-2000ms (model inference time)
- **Reliability**: 85-95% (depends on model, prompt, validation)
- **Fallback**: Automatic fallback to classical AI on errors
- **Use Case**: Complex reasoning, narrative generation, creative behaviors

### Hybrid Mode (Recommended)
- **Strategy**: Try LLM → Fallback to classical
- **Latency**: LLM latency on success, classical latency on fallback
- **Reliability**: 100% (classical AI guarantees valid plan)
- **Use Case**: Best of both worlds (LLM intelligence + classical reliability)

## Features

### Feature Flags
```toml
# Cargo.toml
[features]
default = []
llm = ["astraweave-llm", "astraweave-llm/llm_cache", "tokio"]
```

- **default**: Classical AI only (no dependencies, fast build)
- **llm**: Enables LLM integration (adds ~30s to first build)

### Command-Line Arguments
- `--demo-both`: Run side-by-side comparison of classical vs LLM

## Implementation Details

### Code Structure
```rust
// AI mode selection
enum AIMode {
    Classical,             // RuleOrchestrator (default)
    #[cfg(feature = "llm")]
    LLM,                  // MockLlm or Phi-3
    #[cfg(feature = "llm")]
    Hybrid,               // Try LLM, fallback to classical
}

// Plan generation
fn generate_plan(snap: &WorldSnapshot, mode: AIMode) -> Result<PlanIntent> {
    match mode {
        AIMode::Classical => generate_classical_plan(snap),
        AIMode::LLM => generate_llm_plan(snap).await,
        AIMode::Hybrid => /* try LLM, fallback to classical */
    }
}
```

### LLM Integration
```rust
#[cfg(feature = "llm")]
fn generate_llm_plan(snap: &WorldSnapshot) -> Result<PlanIntent> {
    use astraweave_llm::{plan_from_llm, MockLlm};
    
    let client = MockLlm;  // Or OllamaClient for real Phi-3
    let registry = create_tool_registry();
    
    // Call async LLM API
    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(async {
        plan_from_llm(&client, snap, &registry).await
    });
    
    match result {
        PlanSource::Llm(plan) => Ok(plan),      // LLM succeeded
        PlanSource::Fallback { plan, reason } => Ok(plan),  // LLM failed, heuristic plan
    }
}
```

### Tool Registry
The LLM has access to these tools:
- **MoveTo(x, y)**: Navigate to grid position
- **Throw(item, x, y)**: Throw smoke/grenade to position
- **CoverFire(target_id, duration)**: Suppress enemy with covering fire

## Performance

| Mode      | Latency   | Reliability | Use Case                |
|-----------|-----------|-------------|-------------------------|
| Classical | ~0.02ms   | 100%        | Production gameplay     |
| LLM       | 50-2000ms | 85-95%      | Complex reasoning       |
| Hybrid    | Variable  | 100%        | Best of both worlds     |

## Extending

### Add Real Phi-3 Model
```rust
// Replace MockLlm with OllamaClient
#[cfg(feature = "ollama")]
use astraweave_llm::OllamaClient;

let client = OllamaClient {
    url: "http://localhost:11434".to_string(),
    model: "phi3:mini".to_string(),
};
```

### Add Custom Tools
```rust
fn create_tool_registry() -> ToolRegistry {
    ToolRegistry {
        tools: vec![
            // Add your custom tool
            ToolSpec {
                name: "CustomAction".into(),
                args: {
                    let mut m = BTreeMap::new();
                    m.insert("param1".into(), "i32".into());
                    m
                },
            },
        ],
        // ...
    }
}
```

### Add Custom AI Mode
```rust
enum AIMode {
    Classical,
    LLM,
    Hybrid,
    Custom,  // Your custom AI
}

fn generate_plan(snap: &WorldSnapshot, mode: AIMode) -> Result<PlanIntent> {
    match mode {
        // ...
        AIMode::Custom => generate_custom_plan(snap),
    }
}
```

## Troubleshooting

### "LLM feature not enabled"
**Solution**: Add `--features llm` to cargo command
```bash
cargo run -p hello_companion --release --features llm
```

### "LLM returned fallback plan"
**Cause**: MockLlm intentionally triggers fallback for demo purposes

**Solution**: Use real Phi-3 model with OllamaClient:
1. Install Ollama: https://ollama.ai
2. Download Phi-3: `ollama pull phi3:mini`
3. Enable ollama feature: `cargo run -p hello_companion --features llm,ollama`

### Long first build with LLM
**Cause**: tokio + astraweave-llm dependencies (~30s)

**Solution**: Build without LLM for faster iteration:
```bash
cargo run -p hello_companion --release  # Classical only, fast build
```

## Related Examples

- **core_loop_goap_demo**: Pure GOAP planning (classical AI)
- **core_loop_bt_demo**: Behavior tree AI (classical AI)
- **unified_showcase**: Full game demo with AI companions

## See Also

- **COMPREHENSIVE_STRATEGIC_ANALYSIS.md**: AstraWeave LLM architecture
- **AI_NATIVE_VALIDATION_REPORT.md**: LLM performance benchmarks (81 tests)
- **astraweave-llm/README.md**: LLM crate documentation
- **astraweave-ai/README.md**: Classical AI documentation
