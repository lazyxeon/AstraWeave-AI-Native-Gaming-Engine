# Frame Timing Budget and Tracing (Stub)

This module provides hooks for per-subsystem profiling and Chrome tracing integration.

## Usage

- Use the `tracing` crate for instrumenting code:
  ```rust
  use tracing::{info_span, instrument};
  
  #[instrument]
  pub fn tick_physics() {
      // ...
  }
  ```
- To export Chrome trace:
  - Add `tracing-chrome` as a dev-dependency.
  - In main():
    ```rust
    let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();
    ```
- Set frame budget constants (e.g., physics ≤ 1.5ms, AI ≤ 2ms, render ≤ 7ms).
- Fail CI if exceeded (see benches or add runtime asserts).

## TODO
- Integrate tracing into all major subsystems.
- Add CI check for frame budget regression.
