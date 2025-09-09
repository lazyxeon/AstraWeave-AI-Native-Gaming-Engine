# AstraWeave Level & Encounter Editor

A non-programmer friendly GUI tool for creating and editing levels, encounters, and boss fights in AstraWeave.

## Features

- Visual editing of game levels with biomes, obstacles, and NPCs
- Fate thread system for creating trigger-based events
- Boss encounter configuration with Rhai scripting
- Hot-reload support for instant feedback

## Usage

```bash
cargo run -p aw_editor
```

## File Format

The editor saves levels in a human-readable TOML format:

```toml
title = "Forest Breach"
biome = "temperate_forest"
seed = 123456

[sky]
time_of_day = "dawn"
weather = "fog_light"

# Biome paints
[[biome_paints]]
kind = "grass_dense"
area = { cx = 0, cz = 0, radius = 64 }

# Obstacles
[[obstacles]]
id = "rock_big_01"
pos = [12.0, 0.0, -8.0]
yaw = 1.57
tags = ["cover", "climbable"]

# NPCs
[[npcs]]
archetype = "wolf_pack"
count = 3
spawn = { pos = [-15.0, 0.0, 12.0], radius = 3.0 }
behavior = "patrol"

# Fate threads (triggers → director ops)
[[fate_threads]]
name = "opening_ambush"
triggers = [{ kind = "enter_area", center = [0.0, 0.0, 0.0], radius = 6.0 }]
ops = [
  { op = "Fortify", area = { cx = 8, cz = -6, r = 6 } },
  { op = "SpawnWave", archetype = "wolf_pack", count = 2, scatter = 2.5 }
]

# Boss encounter
[boss]
director_budget_script = "content/encounters/forest_breach.budget.rhai"
phase_script = "content/encounters/forest_breach.phases.rhai"
```

## Integration

The editor creates a `reload.signal` file when saving, which the engine can watch to automatically reload content.

Rhai scripts for boss encounters are stored in the `content/encounters/` directory.