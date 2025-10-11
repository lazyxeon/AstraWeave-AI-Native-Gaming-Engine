# Phi-3 Demo - Interactive AI Showcase

**Live demonstration of AstraWeave's LLM integration with Microsoft Phi-3 Medium**

This example shows how Phi-3 generates tactical plans for different AI personas in real combat scenarios.

---

## 🚀 Quick Start

### Prerequisites

1. **Install Ollama**:
   ```bash
   # Windows
   winget install Ollama.Ollama
   
   # macOS
   brew install ollama
   
   # Linux
   curl https://ollama.ai/install.sh | sh
   ```

2. **Download Phi-3 Medium**:
   ```bash
   ollama pull phi3:medium
   ```
   
   This downloads ~7.9GB (takes 5-15 minutes).

3. **Start Ollama Server**:
   ```bash
   ollama serve
   ```
   
   Leave this running in a separate terminal.

### Run Demo

```bash
cd AstraWeave-AI-Native-Gaming-Engine

# Build and run (release mode recommended for speed)
cargo run -p phi3_demo --release
```

**Expected runtime**: 10-20 seconds (5 LLM queries × 2-4s each)

---

## 📋 What It Does

The demo creates a tactical combat scenario:
- **Player**: HP 75, crouched at (10, 10)
- **Companion** (you): Morale 80, ammo 18, at (12, 10)
- **Enemies**: 2 hostiles with cover at (25, 15) and (28, 12)
- **Points of Interest**: Ammo cache, health pack
- **Objective**: Eliminate all hostiles

Then queries Phi-3 with **5 different AI personas**:

### 1. TACTICAL AI (Aggressive)
- **Behavior**: Direct combat, move to cover, suppress enemies
- **Sample output**:
  ```json
  {
    "plan_id": "tactical-001",
    "reasoning": "Move to cover behind crate, suppress enemy with CoverFire",
    "steps": [
      {"act": "MoveTo", "x": 18, "y": 12},
      {"act": "CoverFire", "target_id": 99, "duration": 3.0}
    ]
  }
  ```

### 2. STEALTH AI (Cautious)
- **Behavior**: Silent movement, avoid detection, use distractions
- **Rules**: NEVER use CoverFire (alerts enemies)
- **Sample output**:
  ```json
  {
    "plan_id": "stealth-002",
    "reasoning": "Flank using obstacles, throw distraction, silent approach",
    "steps": [
      {"act": "MoveTo", "x": 18, "y": 14},
      {"act": "Throw", "item": "smoke", "x": 25, "y": 15},
      {"act": "MoveTo", "x": 30, "y": 20}
    ]
  }
  ```

### 3. SUPPORT AI (Team-focused)
- **Behavior**: Protect allies, prioritize revives, defensive positioning
- **Triggers**: Player HP < 50 → immediate support action
- **Sample output**:
  ```json
  {
    "plan_id": "support-003",
    "reasoning": "Player critical HP, provide smoke cover and fall back",
    "steps": [
      {"act": "Throw", "item": "smoke", "x": 12, "y": 10},
      {"act": "MoveTo", "x": 8, "y": 10}
    ]
  }
  ```

### 4. EXPLORATION AI (Curious)
- **Behavior**: Investigate POIs, avoid combat, reconnaissance
- **Sample output**:
  ```json
  {
    "plan_id": "explore-004",
    "reasoning": "Investigate ammo cache, avoid enemy LOS, map obstacles",
    "steps": [
      {"act": "MoveTo", "x": 15, "y": 8},
      {"act": "MoveTo", "x": 20, "y": 20}
    ]
  }
  ```

### 5. CUSTOM PROMPT (Builder API)
- **Demonstrates**: PromptBuilder with custom constraints
- **Constraints**:
  - Never cross open ground without smoke
  - Conserve ammo (prefer grenades)
  - Prioritize high-value targets in cover
- **Sample output**: Tactical plan respecting all constraints

---

## 🎨 Output Format

The demo uses **colored terminal output**:
- 🟢 **Green**: Success messages, completions
- 🟡 **Yellow**: Actions in progress
- 🔵 **Cyan**: Headers, titles
- ⚪ **White**: LLM responses (JSON)
- 🔴 **Red**: Errors (e.g., Ollama not running)

**Example**:
```
=== AstraWeave Phi-3 Demo ===

🔍 Checking Phi-3 setup...
✅ Ollama server: Running
✅ Model phi3:medium: Available
📦 Ollama version: 0.5.0

🎮 Creating tactical scenario...
  ⏱️  Time: 45.0s
  👤 Player: pos(10, 10) | HP: 75 | Stance: crouch
  ...

━━━ TACTICAL AI (Aggressive) ━━━
Optimized for combat effectiveness and direct engagement

🧠 Querying Phi-3...
✅ Response received (1.82s)

📋 [TACTICAL]
────────────────────────────────────────────────────────────
{
  "plan_id": "tactical-001",
  ...
}
────────────────────────────────────────────────────────────

🆔 Plan ID: "tactical-001"
💡 Reasoning: "Move to cover, suppress enemy"
⚡ Steps: 2 actions
```

---

## ⚙️ Configuration

Edit `main.rs` to customize:

### Change Temperature (Creativity)

```rust
let client = Phi3Ollama::localhost()
    .with_temperature(0.3);  // More deterministic (0.0-1.0)
```

### Change Model

```rust
let client = Phi3Ollama::new("http://localhost:11434", "phi3:mini");
```

### Adjust Max Tokens (Plan Length)

```rust
let client = Phi3Ollama::localhost()
    .with_max_tokens(256);  // Shorter plans (128-1024)
```

### Add Your Own Scenario

```rust
let custom_scenario = WorldSnapshot {
    t: 0.0,
    player: PlayerState { hp: 100, pos: IVec2 { x: 0, y: 0 }, ... },
    // ... your scenario details
};

let prompt = quick::tactical_prompt(&custom_scenario, "Your objective");
let response = client.complete(&prompt).await?;
```

---

## 🐛 Troubleshooting

### "Failed to connect to Ollama server"

**Solution**:
```bash
# Check Ollama is running
curl http://localhost:11434/api/tags

# If not, start it
ollama serve
```

### "Model phi3:medium not found"

**Solution**:
```bash
# Download the model
ollama pull phi3:medium

# Verify it's available
ollama list
```

### Slow responses (>5 seconds)

**Possible causes**:
1. Running on CPU (not GPU) → Update GPU drivers
2. Low VRAM → Use `phi3:mini` instead
3. Other GPU-heavy apps → Close them
4. High temperature setting → Lower to 0.5

### JSON parsing errors

**Solution**: Lower temperature for more deterministic JSON:
```rust
let client = Phi3Ollama::localhost().with_temperature(0.3);
```

---

## 📊 Performance Expectations

**Hardware recommendations**:

| GPU | Model | Tokens/sec | Latency | Notes |
|-----|-------|------------|---------|-------|
| RTX 3060 (12GB) | phi3:medium | 30-40 | 1-2s | ⭐ Recommended |
| RTX 4090 (24GB) | phi3:medium | 50-80 | 0.5-1s | Best experience |
| GTX 1660 (6GB) | phi3:mini | 40-60 | 0.5-1s | Budget option |
| CPU only (16GB RAM) | phi3:mini | 5-10 | 5-10s | Not recommended |

**This demo** makes 5 sequential LLM calls:
- **Best case** (RTX 4090): ~3-5 seconds total
- **Recommended** (RTX 3060): ~8-12 seconds total
- **Budget** (GTX 1660 + mini): ~3-5 seconds total
- **CPU only**: ~30-60 seconds total

---

## 🔗 Next Steps

1. **Read setup guide**: `docs/PHI3_SETUP.md`
2. **Explore prompts**: `crates/astraweave-llm/src/prompts.rs`
3. **Integration example**: `examples/hello_companion`
4. **Non-blocking AI**: See LlmScheduler in Action 17 docs

---

**Version**: 1.0.0  
**Part of**: Week 4 Action 17 - Phi-3 Integration  
**Tested with**: Ollama 0.5.0+, Phi-3 Medium Q4
