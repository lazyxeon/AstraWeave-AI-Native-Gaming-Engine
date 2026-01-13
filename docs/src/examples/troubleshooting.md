# Troubleshooting Examples

This guide helps resolve common issues when running AstraWeave examples.

## Working Examples

The following examples are verified to build and run:

| Example | Command | Notes |
|---------|---------|-------|
| hello_companion | `cargo run -p hello_companion` | Expected panic (no AI model) |
| ipc_loopback | `cargo run -p ipc_loopback` | IPC communication test |
| astraweave-input tests | `cargo test -p astraweave-input` | Input system tests |

## Known Compilation Issues

Some examples have compilation issues due to API evolution:

| Example | Issue | Workaround |
|---------|-------|------------|
| debug_toolkit_demo | egui/winit version mismatches | Use cargo check instead |
| aw_editor | eframe Send/Sync trait issues | Work in progress |
| visual_3d | Clippy deny-level warnings | Use `--allow warnings` |
| navmesh_demo | approx_constant lint | Use `--allow warnings` |
| physics_demo3d | approx_constant lint | Use `--allow warnings` |

## Build Aliases

Use these workspace aliases for safer builds:

```bash
cargo build-core
cargo check-all
cargo clippy-all
```

## Common Errors

### "No suitable graphics adapter"

```
thread 'main' panicked at 'No suitable graphics adapter found'
```

**Solution**: Update graphics drivers or use software rendering:
```bash
export WGPU_BACKEND=gl
cargo run -p hello_companion
```

### "AI model not found"

```
Error: Could not load AI model
```

**Solution**: Examples use mock AI by default. This panic is expected when no LLM is configured.

### Vulkan Not Found (Linux)

```bash
sudo apt-get install mesa-vulkan-drivers vulkan-tools
```

### Missing Build Tools (Windows)

Install Visual Studio with C++ build tools.

## Getting Help

- [Full Troubleshooting Guide](../resources/troubleshooting.md)
- [FAQ](../resources/faq.md)
- [Community](../resources/community.md)
