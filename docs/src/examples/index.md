# Working Examples

AstraWeave includes over 20 examples demonstrating different aspects of the engine. This page focuses on the **working examples** that you can build and run to learn the engine.

> **Note**: AstraWeave is under active development. Some examples have compilation issues due to API evolution. This page focuses on examples that are confirmed to work.

## Core AI Examples

These examples demonstrate the AI-native architecture:

### Hello Companion ✅
**Location**: `examples/hello_companion`  
**Status**: ✅ Working (expected panic)

The simplest example of AI perception, planning, and validation.

```bash
cargo run -p hello_companion --release
```

**What it demonstrates:**
- AI perception system capturing world state
- LLM-based planning generating intents
- Tool validation system (demonstrates failure case)
- Fixed-tick simulation loop

**Expected behavior**: Shows AI plan generation, then panics with "LosBlocked" error. This demonstrates that the AI cannot perform invalid actions.

[→ Detailed walkthrough](./hello-companion.md)

### Adaptive Boss ⚠️
**Location**: `examples/adaptive_boss`  
**Status**: ⚠️ Check compilation

Multi-phase boss that adapts its strategy based on player behavior.

```bash
cargo run -p adaptive_boss --release
```

**What it demonstrates:**
- Director system for complex AI behavior
- Phase-based AI state machines
- Dynamic strategy adaptation
- Boss pattern recognition

### Companion Profile ⚠️
**Location**: `examples/companion_profile`  
**Status**: ⚠️ Check compilation

Demonstrates persistent AI profiles that learn and adapt.

```bash
cargo run -p companion_profile --release
```

**What it demonstrates:**
- AI profile serialization/deserialization
- Learning from player interactions
- Personality trait adjustment
- Long-term memory systems

## Core Engine Examples

These examples showcase fundamental engine systems:

### Physics Demo 3D ⚠️
**Location**: `examples/physics_demo3d`  
**Status**: ⚠️ Check compilation

Demonstrates the Rapier3D physics integration.

```bash
cargo run -p physics_demo3d --release
```

**What it demonstrates:**
- 3D physics simulation
- Character controller integration
- Collision detection and response
- Physics-based AI movement

### Navmesh Demo ⚠️
**Location**: `examples/navmesh_demo`  
**Status**: ⚠️ Check compilation

Shows navigation mesh generation and pathfinding.

```bash
cargo run -p navmesh_demo --release
```

**What it demonstrates:**
- Automatic navmesh generation
- A* pathfinding
- Dynamic obstacle avoidance
- AI navigation coordination

### Audio Spatial Demo ⚠️
**Location**: `examples/audio_spatial_demo`  
**Status**: ⚠️ Check compilation

Spatial audio system with 3D positioning.

```bash
cargo run -p audio_spatial_demo --release
```

**What it demonstrates:**
- 3D positional audio
- Dynamic audio sources
- Environmental audio effects
- Audio-based AI perception

## Networking Examples

These examples show multiplayer and IPC capabilities:

### IPC Loopback ✅
**Location**: `examples/ipc_loopback`  
**Status**: ✅ Should work

Demonstrates inter-process communication for AI models.

```bash
cargo run -p ipc_loopback --release
```

**What it demonstrates:**
- Local/cloud AI model switching
- Process isolation for AI
- IPC message passing
- AI model hot-swapping

### Coop Server/Client ⚠️
**Location**: `examples/coop_server`, `examples/coop_client`  
**Status**: ⚠️ Check compilation

Basic multiplayer client-server architecture.

```bash
# Terminal 1
cargo run -p coop_server --release

# Terminal 2  
cargo run -p coop_client --release
```

**What it demonstrates:**
- Server-authoritative validation
- Intent-based networking
- AI agent synchronization
- Anti-cheat through determinism

## Tool and Planning Examples

These examples focus on AI planning and tool usage:

### LLM Tool Call ⚠️
**Location**: `examples/llm_toolcall`  
**Status**: ⚠️ Check compilation

Direct demonstration of LLM tool calling.

```bash
cargo run -p llm_toolcall --release
```

**What it demonstrates:**
- LLM integration
- Tool definition and usage
- Structured AI responses
- Planning validation

### Phase Director ⚠️
**Location**: `examples/phase_director`  
**Status**: ⚠️ Check compilation

Complex AI director managing multiple phases.

```bash
cargo run -p phase_director --release
```

**What it demonstrates:**
- Multi-phase AI behavior
- Director pattern implementation
- State machine management
- Complex AI coordination

## Development Examples

These examples help with engine development:

### Debug Overlay ❌
**Location**: `examples/debug_overlay`  
**Status**: ❌ Has compilation issues (egui API)

Debug UI overlay for development.

**Known issues**: egui API mismatches with current version.

### Persona Loader ⚠️
**Location**: `examples/persona_loader`  
**Status**: ⚠️ Check compilation

Loading and managing AI personas from files.

```bash
cargo run -p persona_loader --release
```

**What it demonstrates:**
- AI persona definition files
- Dynamic persona loading
- Personality trait configuration
- Behavioral parameter tuning

## Known Compilation Issues

Some examples have known issues due to API evolution:

### Graphics Examples ❌
- **visual_3d**: winit API mismatches
- **ui_controls_demo**: egui API compatibility issues
- **debug_overlay**: egui API changes

### Authoring Examples ❌
- **rhai_authoring**: Depends on broken astraweave-author crate
- Issues with rhai sync/send traits

### Complex Demos ❌
- **npc_town_demo**: Multiple API mismatches
- **weaving_playground**: Dependency issues
- **cutscene_render_demo**: Graphics API issues

## Testing Examples

To verify your installation is working:

### Minimal Test Sequence
```bash
# 1. Build core components
cargo build -p astraweave-core -p astraweave-ai -p hello_companion

# 2. Run the basic example
cargo run -p hello_companion --release

# 3. Run unit tests
cargo test -p astraweave-input
```

### Debugging Build Issues

If examples fail to compile:

1. **Check Rust version**: `rustc --version` should match `rust-toolchain.toml`
2. **Update dependencies**: `cargo update`
3. **Clean build**: `cargo clean && cargo build`
4. **Check system dependencies**: Ensure graphics and audio libraries are installed

### Reporting Issues

If you find compilation issues with examples marked as working:

1. Check your platform and Rust version
2. Ensure all system dependencies are installed
3. Try a clean build
4. Report the issue with full error output

## Building Your Own Examples

When creating new examples:

### Minimal Example Structure
```rust
// examples/my_example/src/main.rs
use astraweave_core::*;
use astraweave_ai::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the engine
    let mut world = World::new();
    
    // Add your systems
    world.add_system(my_custom_system);
    
    // Run the simulation
    world.run()?;
    
    Ok(())
}
```

### Cargo.toml Template
```toml
[package]
name = "my_example"
version.workspace = true
edition.workspace = true

[dependencies]
astraweave-core = { path = "../../astraweave-core" }
astraweave-ai = { path = "../../astraweave-ai" }
anyhow.workspace = true
```

## Next Steps

- **Start Simple**: Begin with [Hello Companion](./hello-companion.md)
- **Learn Architecture**: Read [AI-Native Design](../architecture/ai-native.md)
- **Build Something**: Follow [Building Your First Game](../game-dev/first-game.md)
- **Contribute**: Help fix broken examples in [Contributing Guide](../dev/contributing.md)

---

*The working examples are your best introduction to AstraWeave's capabilities. Start with hello_companion and work your way up to more complex scenarios.*