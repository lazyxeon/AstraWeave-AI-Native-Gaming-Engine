# AstraWeave Debug Toolkit Demo

This example demonstrates how to integrate the AstraWeave debug toolkit with a game application.

## Features

- Integration of the debug HUD with a 3D rendering pipeline
- Real-time performance monitoring
- Event logging
- Script hot-reload demonstration
- Chrome tracing integration

## Running the Demo

```bash
cargo run -p debug_toolkit_demo
```

## Key Components

### Debug HUD Integration

The demo shows how to integrate the debug HUD with your game's rendering pipeline:

```rust
// Initialize the HUD
let mut hud = PerfHud::new();

// Update the HUD each frame
hud.frame();
hud.systems_snapshot = system_timers;
hud.entity_count = entity_count;

// Render the HUD using egui
egui::Window::new("Debug HUD").show(&egui_ctx, |ui| {
    hud.ui(ui);
});
```

### Event Logging

The demo logs various events during gameplay:

```rust
// Log important events
hud.log_event("system", "Application started");
hud.log_event("world", "World initialized with 3 entities");
hud.log_event("ai", "Companion evaluating plan options");
```

### Hot-Reload

The demo sets up watchers for script changes and reload signals:

```rust
// Watch for script changes
let _script_watcher = watch_scripts(content_dir.join("encounters"), || {
    println!("Script changed, reloading...");
    // In a real app, you would reload your scripts here
});

// Watch for reload signals
let _reload_watcher = watch_reload_signal(content_dir, || {
    println!("Reload signal detected, reloading level...");
    // In a real app, you would reload your level here
});
```

### Chrome Tracing

The demo initializes Chrome tracing for performance analysis:

```rust
// Initialize Chrome tracing
let _trace_guard = ChromeTraceGuard::init("astraweave_demo_trace.json");
```

## Integration with AstraWeave

This demo shows how to integrate the debug toolkit with:

- AstraWeave's core world model
- The rendering system
- The camera controller
- The egui overlay system

It provides a template for adding comprehensive debugging and profiling to any AstraWeave-based game.