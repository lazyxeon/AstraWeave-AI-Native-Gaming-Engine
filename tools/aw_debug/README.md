# AstraWeave Debug & Profiling Toolkit

A comprehensive toolkit for debugging, profiling, and monitoring the AstraWeave engine.

## Features

- Performance HUD with FPS counter and frame time graph
- System timing visualization
- Entity count tracking
- Event logging system
- Chrome tracing integration
- Script hot-reload support

## Components

### PerfHud

A performance heads-up display that shows:

- Current FPS
- Frame time graph
- System timings (physics, AI, rendering, etc.)
- Entity count
- Event log

```rust
// Example usage
let mut hud = PerfHud::new();
hud.entity_count = world.entity_count();
hud.systems_snapshot = profiler.last_timings();

// In your render loop
hud.frame(); // Update frame timing
hud.ui(&mut egui_ui); // Draw the HUD
```

### EventLog

A thread-safe event logging system:

```rust
// Log events from anywhere in your code
hud.log_event("ai", "Path recalculated");
hud.log_event("physics", "Collision detected");
hud.log_event("script", "Rhai script executed");
```

### ChromeTraceGuard

Integration with Chrome's tracing system for detailed performance analysis:

```rust
// Initialize at application start
let _trace = ChromeTraceGuard::init("trace.json");

// Later, open chrome://tracing in Chrome and load the trace.json file
```

### Script Watchers

Utilities for hot-reloading scripts and content:

```rust
// Watch a directory for script changes
let _watcher = watch_scripts(std::path::PathBuf::from("content/encounters"), || {
    // Reload scripts when changes are detected
    director.reload_budgets();
});

// Watch for the reload.signal file
let _reload_watcher = watch_reload_signal(content_dir, || {
    // Reload level when signal is detected
    game.reload_current_level();
});
```

## Integration

The debug toolkit is designed to integrate with the AstraWeave engine's existing egui overlay system. See the `examples/debug_toolkit_demo` for a complete integration example.

## Chrome Tracing

To use Chrome tracing:

1. Run your application with `ChromeTraceGuard` initialized
2. Open Chrome and navigate to `chrome://tracing`
3. Load the generated trace file
4. Analyze performance bottlenecks and system behavior

This provides microsecond-level timing information for all traced functions and systems.