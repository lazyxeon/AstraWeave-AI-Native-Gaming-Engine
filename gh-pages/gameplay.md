---
layout: default
title: Gameplay Subsystem
---

# Gameplay (astraweave-gameplay)

AstraWeave's gameplay crate provides combat, crafting, quests, item management, biome definitions, and the Veilweaver game mechanic.

## Key Systems

### Combat Physics

Raycast-based attack sweep with cone filtering:

```rust
use astraweave_gameplay::combat_physics::perform_attack_sweep;

let hits = perform_attack_sweep(
    &physics_world,
    attacker_id,
    &attacker_position,
    &target_list,
    attack_range,
    &mut stats_map,
    &mut parry_map,
    &mut iframe_map,
);
```

Supports:
- Cone-angle filtering for melee / ranged
- Parry detection with timing windows
- Invincibility frame tracking
- Damage calculation with armor/resistance

### Crafting System

Recipe-based crafting with material requirements, unlockable tiers, and failure chances.

### Quest System

- Quest definitions with objectives
- Progress tracking per-player
- Reward distribution on completion
- Integration with HUD quest tracker

### Items & Inventory

- Typed item database
- Stack management
- Equipment slots
- Consumable effects

### Biome Definitions

Material and terrain configuration per biome, integrated with the terrain and PCG systems.

### Veilweaver Mechanic

The fate-weaving system — AstraWeave's signature game mechanic:

- **94.26% test coverage** (64 tests)
- Thread manipulation affecting NPC destinies
- Deterministic outcome resolution
- Integration with AI perception (NPCs react to fate changes)

## Foundation Status

- Weaving system: **A+** grade (production-ready)
- Vertical slice: 6-8 weeks to 30-minute playable demo
- [Foundation Audit Report](../docs/current/VEILWEAVER_FOUNDATION_AUDIT_REPORT.md)

[← Back to Home](index.html) · [Architecture](architecture.html) · [Physics](physics.html)
