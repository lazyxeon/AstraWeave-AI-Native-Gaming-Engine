---
layout: default
title: UI Subsystem
---

# User Interface (astraweave-ui)

AstraWeave's UI system is built on **egui 0.32** with wgpu rendering integration, providing in-game menus, HUD elements, and accessibility features.

## Implemented Features

### Menus
- Main menu (New Game, Continue, Settings, Quit)
- Pause menu (Resume, Settings, Quit to Main Menu)
- Settings panels: Graphics, Audio, Controls
- TOML-based settings persistence (save/load)

### HUD Elements
- Health bars with smooth easing, flash, and glow effects
- Damage numbers with arc motion, combos, and screen shake
- Quest tracker with objectives, checkmarks, and banners
- Minimap with POI markers
- FPS counter and debug overlay (F3 toggle)

### Dialogue System
- Branching NPC dialogue with 4-node trees
- Player response choices
- Tooltip system with hover detection

### Accessibility
- Gamepad navigation support
- Keyboard-only operation
- Configurable key bindings (click-to-rebind)
- Mouse sensitivity adjustment

## Architecture

```
egui::Context
    │
    ├── MenuManager ─── MainMenu / PauseMenu / SettingsMenu
    │
    ├── HudManager ──── HealthBar / DamageNumbers / QuestTracker / Minimap
    │
    └── DialogueManager ── DialogueTree / TooltipSystem
```

## Test Coverage

| Category | Tests |
|----------|-------|
| Core HUD logic | 25 |
| State management | 20 |
| Edge cases | 9 |
| **Total** | **51+** |

## Animation System

Health bars, damage numbers, and notifications use a shared animation framework:

- **Easing functions**: Linear, EaseInOut, Bounce, Elastic
- **Spring physics**: Critically-damped spring for smooth transitions
- **Flash/glow effects**: Color overlay on damage/heal events
- **Arc motion**: Parabolic trajectories for floating damage numbers

[← Back to Home](index.html) · [Architecture](architecture.html) · [Input](setup.html)
