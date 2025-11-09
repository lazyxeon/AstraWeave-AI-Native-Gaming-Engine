# Advanced GOAP Roadmap

## Vision
- Deliver a learning, risk-aware planner that seamlessly replaces the current rule-based orchestrator while unlocking richer tactical and narrative behaviour across AstraWeave.
- Empower designers to author complex goals, priorities, and constraints without touching engine code.
- Establish a feedback loop where in-game execution data continuously improves planning quality across play sessions.

## Current Snapshot (Baseline Q4 2025)
- Rule-based `RuleOrchestrator` in `astraweave-ai` generates short deterministic plans.
- Existing runtime schema (`WorldSnapshot`, `PlanIntent`, `ActionStep`) is battle-tested but lacks GOAP-specific abstractions.
- Claude Sonnet 4.5 prototype (`advanced_goap.rs`, `goap_example.rs`) demonstrates:
  - Dynamic cost/risk scoring with action history.
  - Multi-goal prioritisation and deadlines.
  - Hierarchical goal placeholders (unused) and extended state types.
- Gaps to address before integration:
  - Non-deterministic state hashing via `format!("{:?}", state)`.
  - Exact-match `WorldState::satisfies` (no ranges or partial observability support).
  - `Goal::sub_goals` unused; no HTN/plan refinement yet.
  - Adapter layer between engine snapshot schema and GOAP world model unimplemented.

## Guiding Principles
- **No Regressions**: Maintain current orchestrator as fallback until GOAP exceeds baseline quality metrics.
- **Determinism First**: Ensure planner results and telemetry are reproducible across platforms/builds.
- **Incremental Rollout**: Introduce the planner behind feature flags with gradual entity coverage.
- **Telemetry-Driven**: Instrument decisions and outcomes; let data guide tuning.
- **Designer Friendly**: Expose goal/action authoring through data, not code whenever possible.

## Core Workstreams
- **Planner Core Hardening**: Deterministic hashing, heuristic validation, hierarchical planning support.
- **State & Action Modeling**: Map `WorldSnapshot` into GOAP `WorldState`; author domain actions that emit `ActionStep` payloads.
- **Learning & Persistence**: Wire `ActionHistory` into save/load systems; tune success probability models.
- **Tooling & UX**: Create debugging dashboards, visual plan inspectors, JSON/ron authoring schemas.
- **Rollout & Ops**: Feature flags, shadow mode comparisons, telemetry pipelines, balancing workflows.

## Phase Breakdown & Milestones

### Phase 0 – Discovery & Alignment (Week 0-1)
- Audit existing AI touchpoints (`astraweave-ai`, `astraweave-core`, director tooling) for planner dependencies.
- Define acceptance metrics (plan quality, runtime latency, failure rate).
- Schedule design reviews with gameplay, director, and tooling stakeholders.
- Deliverable: Roadmap sign-off, success-metric dashboard stub.

### Phase 1 – Technical Foundation (Weeks 1-3)
- Port Claude GOAP modules into `astraweave-ai::advanced_goap` under `planner_advanced` feature flag.
- Implement deterministic state hashing (`WorldStateKey`) and custom `Eq/Hash` derivations.
- Extend `StateValue` comparisons to support numeric ranges and optional tolerance predicates.
- Formalise heuristic unit tests (admissibility, consistency) with golden fixtures.
- Deliverable: `cargo test -p astraweave-ai` suite covering planner invariants.

### Phase 2 – Engine Integration (Weeks 3-6)
- Build adapter translating `WorldSnapshot` → GOAP `WorldState` (actions for movement, cover, revive, abilities).
- Implement action library with execution hooks that emit `ActionStep` sequences.
- Add plan-to-intent translator and reconciliation logic (handling partial execution, failure recovery).
- Introduce runtime shadow mode, logging GOAP vs rule-based plans plus deltas.
- Deliverable: Shadow-mode demo CLI (`cargo run -p hello_companion --features planner_advanced`) with side-by-side comparison output.

### Phase 3 – Learning & Persistence (Weeks 6-8)
- Wire `ActionHistory` into `astraweave-memory` save slots; define retention policy per entity type.
- Calibrate cost/risk weightings; expose tuning parameters via TOML config.
- Implement optional Bayesian or EWMA smoothing for success probabilities.
- Deliverable: Telemetry schema documenting recorded metrics, persisted history across reloads.

### Phase 4 – Hierarchical & Multi-Goal Expansion (Weeks 8-11)
- Enable `Goal::sub_goals` resolution (HTN-style) with recursive planning and plan stitching.
- Integrate director-level goals (pacing, encounter design) via multi-goal scheduling.
- Provide authoring templates for hierarchical goal configs (RON/TOML examples).
- Deliverable: Demo showing entity reacting to concurrent goals (protect player + secure objective).

### Phase 5 – Tooling & Designer Enablement (Weeks 11-14)
- Build visualization tooling: plan tree explorer, action success heatmaps, risk timeline.
- Add editor validators for goal/action definitions (lint checks, schema validation).
- Document designer workflows (tutorials, troubleshooting guide, examples).
- Deliverable: `docs/advanced_goap_designer_guide.md` and interactive inspector prototype.

### Phase 6 – Rollout & Optimization (Weeks 14-18)
- Gradually activate GOAP per entity archetype, starting with non-critical companions.
- Monitor telemetry dashboards; iterate on cost/risk tuning and heuristic parameters.
- Address performance hotspots (memory churn, priority queue behavior, cache locality).
- Deliverable: Release note declaring GOAP default for first archetype, rollback plan validated.

### Phase 7 – Post-Launch Evolution (Ongoing)
- Integrate director feedback loops (global pacing goals adjusting companion plans).
- Explore ML-enhanced heuristics, opponent modeling, Monte Carlo sampling as optional modules.
- Maintain backlog of designer requests and difficulty tuning adjustments.

## Success Metrics
- **Plan Quality**: ≥20% reduction in plan execution failures vs baseline across test scenarios.
- **Reaction Time**: Planning latency ≤4 ms (P95) under typical combat snapshots.
- **Learning Impact**: ≥10% improvement in success probability for repeated encounter types over 30 minutes of play.
- **Designer Adoption**: ≥80% of authored scenarios use GOAP goals within first month post-launch.
- **Stability**: Zero new crashes attributable to planner in staging over two consecutive weeks.

## Key Deliverables by Workstream
- **Planner Core**: Deterministic state hashing, hierarchical planner, risk-weight config surface.
- **Action Library**: Validated action catalog with cost/risk annotations and execution hooks.
- **Persistence**: Action history serialization/deserialization, telemetry retention.
- **Tooling**: Visualization dashboards, schema validators, logging macros.
- **Documentation**: Integration guide for engineers, designer authoring handbook, troubleshooting FAQ.

## Risks & Mitigations
| Risk | Impact | Mitigation |
| --- | --- | --- |
| Non-deterministic state hashing | Planner loops, inconsistent results | Implement canonical key ordering and state hashing in Phase 1 |
| Performance regressions under many goals | Frame spikes | Add configurable iteration budget, incremental replanning, profiling harness |
| Designer complexity | Slow adoption | Provide templates, UX tooling, training sessions |
| History persistence bugs | Player progression issues | Add checksum validation, staging soak tests, opt-out toggle |
| Stochastic actions diverging from estimates | Plan instability | Instrument execution outcomes, adjust probability models, enable plan repair |

## Dependencies & Coordination
- **Engine Schema**: Any changes to `WorldSnapshot`/`ActionStep` require synchronised updates with GOAP adapter.
- **Persistence Layer**: Collaboration with `astraweave-memory` maintainers for save format changes.
- **Director System**: Work with `astraweave-director` to align goal priority semantics.
- **Tooling Team**: UI support for plan visualization within developer tools.

## Implementation Checklist
- [ ] Feature flag scaffolding (`planner_advanced`).
- [ ] GOAP core ported and namespaced under `astraweave_ai::advanced`.
- [ ] Deterministic `WorldState` hashing and equality implemented.
- [ ] Extended `StateValue` comparisons (numeric ranges, tolerance, optional states).
- [ ] Goal hierarchy execution path added.
- [ ] Adapter: snapshot → GOAP state; GOAP plan → `PlanIntent`.
- [ ] Action catalog implemented (movement, cover, revive, abilities, utility actions).
- [ ] Plan shadow-mode logging with diff visualisation.
- [ ] Telemetry + persistence integration.
- [ ] Designer tooling docs and samples.
- [ ] Rollout plan reviewed with gameplay leadership.

## Timeline Overview (High-Level)
- **Month 1**: Foundations, deterministic planner, adapter prototype.
- **Month 2**: Engine integration, learning/persistence, shadow mode trial.
- **Month 3**: Hierarchical goals, tooling, staged rollout to selected entities.
- **Beyond**: Continuous improvement, advanced heuristics, director integration.

## Open Questions
- Which entities should participate in first-wave rollout (companions, director agents, NPC factions)?
- What telemetry backend do we standardise on for plan analytics (existing tracing vs new pipeline)?
- Do we require plan repair or live replanning support prior to full rollout?
- How granular should designer-facing goal definitions be (single file per scenario vs shared library)?

## Next Actions (as of Roadmap Creation)
1. Confirm roadmap assumptions with gameplay, director, and tooling leads.
2. Prioritise unit test coverage for planner invariants before large-scale integration.
3. Assemble cross-functional task force (AI, tools, gameplay) for Phase 1 kick-off.
4. Establish telemetry dashboard scaffold for plan quality metrics.
