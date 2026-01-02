/*!
# AstraWeave Profiling

Zero-cost profiling abstraction layer with Tracy integration.

This crate provides profiling macros that compile to no-ops when profiling is disabled,
ensuring zero runtime cost in production builds.

## Features

- **profiling**: Enable basic Tracy profiling
- **profiling-sampling**: Enable Tracy sampling mode (8KHz, lower overhead)
- **profiling-system**: Enable system tracing (GPU, memory, locks)
- **profiling-full**: All profiling features combined

## Usage

```rust,no_run
use astraweave_profiling::{span, frame_mark};

fn game_loop() {
    loop {
        span!("frame");

        {
            span!("update");
            update_game();
        }

        {
            span!("render");
            render_game();
        }

        frame_mark!();
    }
}

fn update_game() {
    span!("physics");
    // Physics simulation

    span!("AI");
    // AI updates
}

fn render_game() {
    span!("submit");
    // Render submission
}
```

## Tracy Integration

When compiled with `--features profiling`, this crate integrates with Tracy Profiler:
1. Download Tracy from https://github.com/wolfpld/tracy
2. Build with profiling: `cargo build --features profiling`
3. Run Tracy server
4. Run your application
5. Tracy will automatically connect and capture profiling data

## Zero-Cost Abstraction

When profiling features are disabled (default), all macros compile to empty code blocks,
resulting in **zero runtime overhead**.
*/

#![cfg_attr(not(feature = "profiling"), no_std)]

#[cfg(feature = "profiling")]
pub use tracy_client;

/// Profiling span macro - measures execution time of a code block
///
/// # Examples
///
/// ```rust,no_run
/// use astraweave_profiling::span;
///
/// fn expensive_operation() {
///     span!("expensive");
///     // Your code here
/// }
/// ```
#[macro_export]
macro_rules! span {
    ($name:expr) => {
        #[cfg(feature = "profiling")]
        let _tracy_span = $crate::tracy_client::span!($name);
    };
}

/// Frame mark macro - marks the end of a frame
///
/// Use this at the end of your main loop iteration to help Tracy
/// visualize frame boundaries and calculate FPS.
///
/// # Examples
///
/// ```rust,no_run
/// use astraweave_profiling::frame_mark;
///
/// loop {
///     // Game loop code
///     frame_mark!();
/// }
/// ```
#[macro_export]
macro_rules! frame_mark {
    () => {
        #[cfg(feature = "profiling")]
        $crate::tracy_client::Client::running()
            .expect("Tracy client should be running")
            .frame_mark();
    };
}

/// Plot value macro - plots a numeric value over time
///
/// # Examples
///
/// ```rust,no_run
/// use astraweave_profiling::plot;
///
/// let fps = 60.0;
/// plot!("FPS", fps);
/// ```
#[macro_export]
macro_rules! plot {
    ($name:expr, $value:expr) => {
        #[cfg(feature = "profiling")]
        $crate::tracy_client::Client::running()
            .expect("Tracy client should be running")
            .plot(
                $crate::tracy_client::PlotName::new_leak($name.to_string()),
                $value as f64,
            );
    };
}

/// Message macro - emits a text message to Tracy
///
/// # Examples
///
/// ```rust,no_run
/// use astraweave_profiling::message;
///
/// message!("Player spawned");
/// message!("Level loaded: {}", level_name);
/// ```
#[macro_export]
macro_rules! message {
    ($($arg:tt)*) => {
        #[cfg(feature = "profiling")]
        {
            let msg = format!($($arg)*);
            $crate::tracy_client::Client::running()
                .expect("Tracy client should be running")
                .message(&msg, 0);
        }
    };
}

/// Memory allocation tracking macro
///
/// # Examples
///
/// ```rust,no_run
/// use astraweave_profiling::alloc;
///
/// let data = vec![0u8; 1024];
/// alloc!(data.as_ptr(), data.len());
/// ```
#[macro_export]
macro_rules! alloc {
    ($ptr:expr, $size:expr) => {
        #[cfg(feature = "profiling")]
        $crate::tracy_client::Client::running()
            .expect("Tracy client should be running")
            .alloc($ptr as *const u8, $size);
    };
}

/// Memory deallocation tracking macro
///
/// # Examples
///
/// ```rust,no_run
/// use astraweave_profiling::free;
///
/// let data = vec![0u8; 1024];
/// let ptr = data.as_ptr();
/// drop(data);
/// free!(ptr);
/// ```
#[macro_export]
macro_rules! free {
    ($ptr:expr) => {
        #[cfg(feature = "profiling")]
        $crate::tracy_client::Client::running()
            .expect("Tracy client should be running")
            .free($ptr as *const u8);
    };
}

/// Colored span macro - creates a span with a custom color
///
/// Note: For simplicity, this maps to a regular span in Tracy 0.17.
/// Color support requires more complex macro implementation.
///
/// # Examples
///
/// ```rust,no_run
/// use astraweave_profiling::span_color;
///
/// fn critical_section() {
///     span_color!("critical", 0xFF0000); // Red  
///     // Your code
/// }
/// ```
#[macro_export]
macro_rules! span_color {
    ($name:expr, $_color:expr) => {
        #[cfg(feature = "profiling")]
        let _tracy_span = $crate::tracy_client::span!($name);
    };
}

/// Function-level profiling attribute macro
///
/// This is a compile-time wrapper that instruments entire functions.
/// Note: Requires proc-macro support (future enhancement).
///
/// # Examples
///
/// ```rust,ignore
/// #[profile]
/// fn my_function() {
///     // Automatically profiled
/// }
/// ```
// Note: This is a placeholder for future proc-macro implementation
// For now, use span!() at function entry
/// Profiling context for RAII-style span management
///
/// Note: Due to Tracy 0.17 API constraints, this uses marker-based approach.
/// The actual profiling is done via the span! macro in calling code.
#[cfg(feature = "profiling")]
pub struct ProfileSpan {
    _marker: std::marker::PhantomData<()>,
}

#[cfg(feature = "profiling")]
impl ProfileSpan {
    /// Create a new profiling span
    ///
    /// Note: Due to Tracy API constraints, callers should use `span!()` macro instead.
    /// This is provided for API compatibility but does not create a Tracy span.
    pub fn new(_name: &'static str) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a new profiling span with custom color
    ///
    /// Note: Color support requires different Tracy API. Use `span!()` macro instead.
    pub fn new_colored(_name: &'static str, _color: u32) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(not(feature = "profiling"))]
pub struct ProfileSpan;

#[cfg(not(feature = "profiling"))]
impl ProfileSpan {
    pub fn new(_name: &'static str) -> Self {
        Self
    }

    pub fn new_colored(_name: &'static str, _color: u32) -> Self {
        Self
    }
}

/// Profiling utilities
pub struct Profiler;

impl Profiler {
    /// Check if Tracy profiling is enabled at runtime
    pub fn is_enabled() -> bool {
        #[cfg(feature = "profiling")]
        {
            tracy_client::Client::is_running()
        }
        #[cfg(not(feature = "profiling"))]
        {
            false
        }
    }

    /// Get Tracy client version (if available)
    pub fn version() -> Option<&'static str> {
        #[cfg(feature = "profiling")]
        {
            Some(env!("CARGO_PKG_VERSION"))
        }
        #[cfg(not(feature = "profiling"))]
        {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_status() {
        // Profiling should be disabled in tests by default
        #[cfg(not(feature = "profiling"))]
        assert!(!Profiler::is_enabled());

        // Version should return None when profiling is disabled
        #[cfg(not(feature = "profiling"))]
        assert!(Profiler::version().is_none());
    }

    #[test]
    fn test_span_compiles() {
        // These should compile to no-ops without profiling feature
        span!("test");

        {
            span!("nested");
            // Nested span
        }
    }

    #[test]
    fn test_frame_mark_compiles() {
        // Should compile to no-op
        frame_mark!();
    }

    #[test]
    fn test_plot_compiles() {
        // Should compile to no-op
        plot!("test_metric", 42.0);
    }

    #[test]
    fn test_message_compiles() {
        // Should compile to no-op
        message!("Test message");
        message!("Test with arg: {}", 123);
    }

    #[test]
    fn test_profile_span_raii() {
        {
            let _span = ProfileSpan::new("test");
            // Span active here
        }
        // Span dropped here
    }

    #[test]
    fn test_colored_span() {
        {
            let _span = ProfileSpan::new_colored("critical", 0xFF0000);
            // Red span active
        }
    }
}
