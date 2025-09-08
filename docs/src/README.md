# AstraWeave: AI-Native Game Engine Documentation

Welcome to the comprehensive documentation for **AstraWeave**, the first game engine built from the ground up for AI-native gameplay.

## What is AstraWeave?

AstraWeave is a deterministic, ECS-based game engine where **AI agents are first-class citizens**. Unlike traditional engines where AI is bolted on as an afterthought, AstraWeave implements the core AI loop (**Perception → Reasoning → Planning → Action**) directly into the simulation architecture.

## Key Features

🧠 **AI-Native Architecture** - Agents plan through sandboxed tools with full engine validation  
🎯 **Deterministic Simulation** - 60Hz fixed-tick simulation with authoritative validation  
🛡️ **Tool Sandbox Security** - AI can only act through validated verbs (no cheating)  
🤝 **Persistent Companions** - AI profiles that learn and adapt across sessions  
🎭 **Adaptive Boss Systems** - Directors that evolve tactics and reshape battlefields  
🌐 **Local-First AI** - 7B-12B quantized LLMs for low-latency decisions  

## Who is This For?

This documentation serves different types of users:

### 🎮 Game Developers
- Want to build games with intelligent AI companions
- Need dynamic bosses that adapt to player strategies
- Seek emergent gameplay from AI agent interactions
- **Start with:** [Quick Start Guide](./getting-started/quick-start.md)

### 🔧 Engine Contributors
- Want to contribute to the engine's core systems
- Interested in AI-native architecture design
- Need to understand the codebase structure
- **Start with:** [Contributing Guide](./dev/contributing.md)

### 🧠 AI Researchers
- Studying AI-native game architectures
- Interested in perception, planning, and validation systems
- Want to understand tool-based AI validation
- **Start with:** [AI-Native Design](./architecture/ai-native.md)

### 📚 Students & Learners
- Learning about game engine architecture
- Understanding ECS and deterministic simulation
- Exploring AI in games
- **Start with:** [Architecture Overview](./architecture/overview.md)

## Quick Navigation

- **New to AstraWeave?** → [Quick Start Guide](./getting-started/quick-start.md)
- **Want to build a game?** → [Building Your First Game](./game-dev/first-game.md)
- **Need API reference?** → [API Documentation](./api/index.md)
- **Looking for examples?** → [Working Examples](./examples/index.md)
- **Want to contribute?** → [Contributing Guide](./dev/contributing.md)
- **Having issues?** → [Troubleshooting](./resources/troubleshooting.md)

## Reference Implementation: Veilweaver

**Veilweaver: Threads of Eternity** serves as AstraWeave's reference implementation—a complete AI-native Action RPG that demonstrates the engine's capabilities. Learn more in the [Veilweaver section](./veilweaver/overview.md).

---

*Ready to build the future of AI-native gaming? Let's get started!*