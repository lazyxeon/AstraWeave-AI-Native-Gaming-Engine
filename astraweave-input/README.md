# astraweave-input

Input binding and action mapping system for AstraWeave.

## Overview

Unified handling for keyboard, mouse, and gamepad input with serializable bindings, action definitions, and configuration persistence.

## Modules

| Module | Description |
|--------|-------------|
| `bindings` | Key/mouse/gamepad bindings (`Binding`, `BindingSet`) |
| `actions` | Game action definitions and enums |
| `manager` | Runtime input state polling and action queries |
| `save` | Input configuration persistence |

## Dependencies

- **winit** — Window event handling (keyboard/mouse)
- **gilrs** — Gamepad/controller input

## License

MIT
