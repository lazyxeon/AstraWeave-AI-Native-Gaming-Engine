# AstraWeave LLM Integration - Visual Roadmap

## 🎯 Strategic Vision

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CURRENT STATE (Phase 7 Complete)                  │
│                                                                       │
│  ✅ Production LLM Orchestrator (streaming, fallbacks, warmup)      │
│  ✅ Multiple Backends (Mock, Ollama, LocalHTTP)                     │
│  ✅ Tool Sandbox Security                                            │
│  ✅ Deterministic ECS Integration                                    │
│                                                                       │
│  ❌ NO Long-Term Memory (embeddings, RAG, vector search)            │
│  ❌ NO Context Management (conversation history)                    │
│  ❌ NO Personalization (persona-specific prompts)                   │
│  ❌ Limited Integration (only orchestrator uses LLM)                │
└─────────────────────────────────────────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────────┐
│                    TARGET STATE (16 Weeks)                           │
│                                                                       │
│  🎯 Persistent Companions That Learn (embeddings + RAG)             │
│  🎯 Adaptive Boss Systems (LLM-driven tactic evolution)             │
│  🎯 Dynamic Content Generation (quests, dialogue, behavior)          │
│  🎯 Production-Grade Performance (<100ms p95 latency)                │
│  🎯 Comprehensive Observability (tracing, metrics, dashboards)       │
│                                                                       │
│  ✅ 9/9 Crates Integrated                                            │
│  ✅ World's First AI-Native Game Engine                              │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 📅 16-Week Timeline Overview

```
┌────────────┬────────────┬────────────┬────────────┐
│  PHASE 1   │  PHASE 2   │  PHASE 3   │  PHASE 4   │
│ Foundation │ Core Feat. │  Advanced  │ Production │
│  (Weeks    │  (Weeks    │  (Weeks    │  (Weeks    │
│   1-4)     │   5-10)    │  11-14)    │  15-16)    │
└────────────┴────────────┴────────────┴────────────┘
     ↓             ↓            ↓            ↓
┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐
│Embeddings│  │   RAG   │  │   NPC   │  │ Observ- │
│ Context  │  │ Persona │  │Behavior │  │  ability│
│ Prompts  │  │Dialogue │  │ Optim.  │  │  A/B    │
│          │  │Director │  │         │  │Hardening│
│          │  │ Quests  │  │         │  │         │
└─────────┘  └─────────┘  └─────────┘  └─────────┘
```

---

## 🏗️ Architecture Layers

```
┌──────────────────────────────────────────────────────────────────┐
│                         GAME LAYER                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │ Persona  │ │ Dialogue │ │ Director │ │  Quests  │           │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘           │
│       │            │            │            │                  │
│  ┌────┴─────┐ ┌────┴─────┐ ┌────┴─────┐ ┌────┴─────┐           │
│  │   NPC    │ │ Behavior │ │  Memory  │ │Persistence│           │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘           │
└───────┼────────────┼────────────┼────────────┼─────────────────┘
        │            │            │            │
┌───────┼────────────┼────────────┼────────────┼─────────────────┐
│       │       INTELLIGENCE LAYER             │                  │
│       ↓                                      ↓                  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐           │
│  │ Context │  │ Prompts │  │   RAG   │  │Observ-  │           │
│  │ Manager │  │Templates│  │Pipeline │  │ability  │           │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘           │
└───────┼────────────┼────────────┼────────────┼─────────────────┘
        │            │            │            │
┌───────┼────────────┼────────────┼────────────┼─────────────────┐
│       │       FOUNDATION LAYER               │                  │
│       ↓                                      ↓                  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐           │
│  │Embeddings│  │LLM Client│ │Orchestr-│  │  Tool   │           │
│  │  Store   │  │(Existing)│ │  ator   │  │ Sandbox │           │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘           │
└──────────────────────────────────────────────────────────────────┘
```

---

## 🔄 Data Flow: Perception → Memory → Reasoning → Action

```
┌──────────────────────────────────────────────────────────────────┐
│  1. PERCEPTION                                                    │
│                                                                   │
│     ┌─────────────┐                                              │
│     │WorldSnapshot│ ← Current game state (existing)              │
│     └──────┬──────┘                                              │
│            │                                                      │
│            ├──→ RAG Retrieval (similar past experiences)         │
│            │                                                      │
│            └──→ Context Window (recent conversation)             │
└────────────────────────┬─────────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────────────┐
│  2. MEMORY CONSOLIDATION                                          │
│                                                                   │
│     New Experience → Embeddings → Vector Store → Prune Old       │
│                                                                   │
│     [Text] ──→ [384-dim vector] ──→ [HNSW Index] ──→ [Top-K]    │
└────────────────────────┬─────────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────────────┐
│  3. REASONING (LLM Inference)                                     │
│                                                                   │
│     Persona Prompt Template                                       │
│            +                                                      │
│     Context Injection (history + RAG)                            │
│            ↓                                                      │
│     ┌──────────────┐                                             │
│     │  LLM Client  │ ← Streaming inference (Ollama/Local)        │
│     └──────┬───────┘                                             │
│            │                                                      │
│            └──→ Parse JSON → Validate Schema                     │
└────────────────────────┬─────────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────────────┐
│  4. ACTION (Validated Execution)                                  │
│                                                                   │
│     Tool Sandbox Validation (existing)                            │
│            ↓                                                      │
│     Execute Actions (deterministic ECS)                           │
│            ↓                                                      │
│     Store Outcomes → Update Conversation History                 │
│            ↓                                                      │
│     Emit LLM Telemetry (latency, tokens, cost)                   │
└──────────────────────────────────────────────────────────────────┘
```

---

## 📦 New Crates Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  PHASE 1: Foundation (Weeks 1-4)                                 │
├─────────────────────────────────────────────────────────────────┤
│  📦 astraweave-embeddings                                        │
│     • EmbeddingClient trait (embed, embed_batch)                │
│     • MockEmbeddingClient (deterministic)                        │
│     • VectorStore (HNSW index, cosine similarity)               │
│     • Dependencies: rust-bert/ort, hnsw, nalgebra               │
│                                                                  │
│  📦 astraweave-context                                           │
│     • ConversationHistory (VecDeque<Message>)                   │
│     • Token counting (tiktoken)                                  │
│     • Sliding window pruning (FIFO)                             │
│     • Summarization strategy (LLM-based)                         │
│                                                                  │
│  📦 astraweave-prompts                                           │
│     • PromptTemplate (Handlebars/Tera)                          │
│     • Variable substitution                                      │
│     • Persona-specific generation                                │
│     • TOML-based prompt library                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  PHASE 2: Core Features (Weeks 5-10)                             │
├─────────────────────────────────────────────────────────────────┤
│  📦 astraweave-rag                                               │
│     • RagPipeline (retrieve, inject_context, consolidate)       │
│     • Memory consolidation (short-term → long-term)             │
│     • Forgetting curve (importance-based decay)                  │
│     • Optional: Cross-encoder reranking                          │
│                                                                  │
│  🔧 astraweave-persona [EXTENSIONS]                             │
│     • PersonaLlmConfig (temperature, top_p, factors)            │
│     • generate_prompt() (persona-specific)                       │
│     • evolve_from_outcome() (learning)                           │
│     • CPersonaLlm component (ECS integration)                    │
│                                                                  │
│  🔧 astraweave-dialogue [EXTENSIONS]                            │
│     • DialogueGenerator (LLM + RAG + context)                   │
│     • Multi-turn conversation                                    │
│     • Sentiment analysis                                         │
│     • CDialogueState component                                   │
│                                                                  │
│  🔧 astraweave-director [EXTENSIONS]                            │
│     • DirectorAI (adapt_tactics, adjust_difficulty)             │
│     • PlayerBehaviorModel                                        │
│     • CDirectorState component                                   │
│                                                                  │
│  🔧 astraweave-quests [EXTENSIONS]                              │
│     • QuestGenerator (generate_quest, branch_narrative)         │
│     • Quest validation                                           │
│     • CQuestGenerator component                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  PHASE 3: Advanced Features (Weeks 11-14)                        │
├─────────────────────────────────────────────────────────────────┤
│  🔧 astraweave-npc [EXTENSIONS]                                 │
│     • NpcAI (decide_action with LLM)                            │
│     • SocialGraph (relationships, affinity, trust)              │
│     • CNpcAI component                                           │
│                                                                  │
│  🔧 astraweave-behavior [EXTENSIONS]                            │
│     • BehaviorTreeGenerator (LLM-assisted)                      │
│     • Adaptive behavior (learning from outcomes)                 │
│     • Behavior library                                           │
│     • CBehaviorGenerator component                               │
│                                                                  │
│  ⚡ Optimization Features                                        │
│     • BatchInference (queue + batch processing)                  │
│     • PromptCache (LRU cache, 80%+ hit rate)                    │
│     • Token budget optimization                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  PHASE 4: Production (Weeks 15-16)                               │
├─────────────────────────────────────────────────────────────────┤
│  🔧 astraweave-observability [EXTENSIONS]                       │
│     • LlmTelemetry (traces, metrics, dashboards)                │
│     • Cost monitoring                                            │
│     • Alert thresholds                                           │
│     • CLlmTelemetry component                                    │
│                                                                  │
│  📊 A/B Testing Framework                                        │
│     • ABTestFramework (experiments, variants)                    │
│     • Statistical significance testing                           │
│     • TOML-based configuration                                   │
│                                                                  │
│  🛡️ Production Hardening                                        │
│     • RateLimiter (prevent API overload)                        │
│     • CircuitBreaker (failure isolation)                         │
│     • Backpressure handling                                      │
│     • Graceful degradation                                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🎯 Integration Points by Crate

```
┌──────────────────┬─────────────┬──────────┬──────────────────┐
│      Crate       │ Integration │ Priority │  Effort (Weeks)  │
├──────────────────┼─────────────┼──────────┼──────────────────┤
│ astraweave-      │ Prompt gen, │   HIGH   │        2         │
│ persona          │ personality │          │                  │
│                  │ evolution   │          │                  │
├──────────────────┼─────────────┼──────────┼──────────────────┤
│ astraweave-      │ Conversation│   HIGH   │        2         │
│ dialogue         │ generation, │          │                  │
│                  │ RAG         │          │                  │
├──────────────────┼─────────────┼──────────┼──────────────────┤
│ astraweave-      │ Boss AI,    │   HIGH   │        2         │
│ director         │ tactic      │          │                  │
│                  │ adaptation  │          │                  │
├──────────────────┼─────────────┼──────────┼──────────────────┤
│ astraweave-      │ Quest       │  MEDIUM  │        2         │
│ quests           │ generation, │          │                  │
│                  │ branching   │          │                  │
├──────────────────┼─────────────┼──────────┼──────────────────┤
│ astraweave-      │ Emergent    │  MEDIUM  │        2         │
│ npc              │ behavior,   │          │                  │
│                  │ social graph│          │                  │
├──────────────────┼─────────────┼──────────┼──────────────────┤
│ astraweave-      │ Behavior    │  MEDIUM  │        2         │
│ behavior         │ tree        │          │                  │
│                  │ synthesis   │          │                  │
├──────────────────┼─────────────┼──────────┼──────────────────┤
│ astraweave-      │ LLM tracing,│   HIGH   │        1         │
│ observability    │ metrics     │          │                  │
├──────────────────┼─────────────┼──────────┼──────────────────┤
│ astraweave-      │ Conversation│  MEDIUM  │        1         │
│ persistence-ecs  │ history     │          │                  │
│                  │ save/load   │          │                  │
├──────────────────┼─────────────┼──────────┼──────────────────┤
│ astraweave-      │ Long-term   │   HIGH   │        1         │
│ memory           │ memory      │          │                  │
│                  │ persistence │          │                  │
└──────────────────┴─────────────┴──────────┴──────────────────┘
                          Total Effort: 15 weeks
                    + 1 week buffer = 16 weeks
```

---

## 🚀 Deployment Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│  WEEK 4: Phase 1 Deployment                                      │
├─────────────────────────────────────────────────────────────────┤
│  ✅ astraweave-embeddings v0.1.0                                │
│  ✅ astraweave-context v0.1.0                                   │
│  ✅ astraweave-prompts v0.1.0                                   │
│                                                                  │
│  Integration: Add to astraweave-llm                              │
│  CI: Unit tests, benchmarks                                      │
│  Docs: API docs, usage examples                                  │
└─────────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│  WEEK 10: Phase 2 Deployment                                     │
├─────────────────────────────────────────────────────────────────┤
│  ✅ astraweave-rag v0.1.0                                       │
│  ✅ astraweave-persona v0.2.0 (extensions)                      │
│  ✅ astraweave-dialogue v0.2.0 (extensions)                     │
│  ✅ astraweave-director v0.2.0 (extensions)                     │
│  ✅ astraweave-quests v0.2.0 (extensions)                       │
│                                                                  │
│  Integration: Feature flags (default off)                        │
│  Examples: hello_companion, unified_showcase updates            │
│  CI: Integration tests, performance tests                        │
└─────────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│  WEEK 14: Phase 3 Deployment                                     │
├─────────────────────────────────────────────────────────────────┤
│  ✅ astraweave-npc v0.2.0 (extensions)                          │
│  ✅ astraweave-behavior v0.2.0 (extensions)                     │
│  ✅ Optimization features (batching, caching)                   │
│                                                                  │
│  Integration: Feature flags (default on for production)          │
│  Example: NPC town with emergent behavior                        │
│  CI: Stress tests (100 NPCs)                                     │
└─────────────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────────────┐
│  WEEK 16: Phase 4 Deployment (PRODUCTION READY)                  │
├─────────────────────────────────────────────────────────────────┤
│  ✅ astraweave-observability v0.2.0 (extensions)                │
│  ✅ A/B testing framework                                       │
│  ✅ Production hardening (rate limiting, circuit breakers)      │
│                                                                  │
│  Integration: Production-ready (default enabled)                 │
│  Dashboard: aw_editor LLM metrics panel                          │
│  CI: Failure scenarios, load tests                              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📊 Success Metrics Dashboard

```
┌─────────────────────────────────────────────────────────────────┐
│  TECHNICAL METRICS                                               │
├─────────────────────────────────────────────────────────────────┤
│  ✅ 9/9 crates integrated                        [████████] 100% │
│  ✅ <100ms p95 latency (orchestrator)            [████████] PASS │
│  ✅ <500ms p95 latency (dialogue)                [████████] PASS │
│  ✅ 95%+ uptime with graceful degradation        [████████] PASS │
│  ✅ Zero crashes from LLM errors                 [████████] PASS │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  QUALITY METRICS                                                 │
├─────────────────────────────────────────────────────────────────┤
│  ✅ Companions remember 100+ interactions        [████████] PASS │
│  ✅ Bosses adapt in 5+ encounters                [████████] PASS │
│  ✅ Quests feel personalized (user testing)      [████████] PASS │
│  ✅ NPCs exhibit emergent behavior               [████████] PASS │
│  ✅ Dialogue maintains personality consistency   [████████] PASS │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  USER IMPACT                                                     │
├─────────────────────────────────────────────────────────────────┤
│  🎯 Companion engagement                         +50% (target)   │
│  🎯 Boss replayability                           +30% (target)   │
│  🎯 Quest completion rate                        +20% (target)   │
│  🎯 NPC interaction rate                         +40% (target)   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🎉 Completion Milestones

```
Week  4: 🏗️  Foundation Complete
         • Embeddings, Context, Prompts ready
         • 30% progress

Week 10: 🎮  Core Features Complete
         • Persona, Dialogue, Director, Quests integrated
         • 65% progress

Week 14: 🚀  Advanced Features Complete
         • NPCs, Behavior Trees, Optimization done
         • 90% progress

Week 16: ✅  PRODUCTION READY
         • Observability, A/B Testing, Hardening complete
         • 100% progress
         • World's First AI-Native Game Engine 🎉
```

---

## 📖 Documentation Links

- **[Master Plan](LLM_INTEGRATION_MASTER_PLAN.md)** - Full 16-week plan with technical details
- **[Quick Start](LLM_INTEGRATION_QUICK_START.md)** - Get started in <30 minutes
- **[This Roadmap](LLM_INTEGRATION_ROADMAP.md)** - Visual overview (current doc)

---

**Ready to transform AstraWeave into the world's first AI-native game engine?** 🚀

**Let's begin Phase 1!** 🎯
