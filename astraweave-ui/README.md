# astraweave-ui

In-game UI framework for AstraWeave, built on **egui**.

## Overview

Complete UI stack: menus (main, pause, settings), HUD (health bars, damage numbers, minimap, quest tracker, dialogue, tooltips), accessibility (colorblind modes), and gamepad support.

## Key Types

| Type | Description |
|------|-------------|
| `HudManager` | Central HUD state and rendering |
| `MenuManager` | Menu navigation state machine |
| `draw_ui()` | Main rendering entry point |
| `AccessibilitySettings` | Colorblind and contrast options |
| `GamepadManager` | Controller input handling |

## Modules

- **`hud`** — Health bars (smooth transitions), damage numbers (arc motion, combos), quest tracker, minimap, dialogue, tooltips
- **`menu` / `menus`** — Main menu, pause menu, settings panels
- **`panels`** — `draw_ui()` root rendering function
- **`persistence`** — TOML settings save/load
- **`accessibility`** — Colorblind modes (protanopia, deuteranopia, tritanopia)
- **`gamepad`** — Controller support via gilrs

## License

MIT
