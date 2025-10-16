use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use astraweave_ecs::{App, Plugin};

pub mod llm_telemetry;
pub use llm_telemetry::*;

/// Configuration for observability stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub tracing_level: String,
    pub metrics_enabled: bool,
    pub crash_reporting_enabled: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            tracing_level: "INFO".to_string(),
            metrics_enabled: true,
            crash_reporting_enabled: true,
        }
    }
}

/// Resource for observability state
pub struct ObservabilityState {
    pub config: ObservabilityConfig,
}

impl ObservabilityState {
    pub fn new(config: ObservabilityConfig) -> Self {
        Self { config }
    }
}

/// Plugin for observability integration
pub struct ObservabilityPlugin {
    config: ObservabilityConfig,
}

impl ObservabilityPlugin {
    pub fn new(config: ObservabilityConfig) -> Self {
        Self { config }
    }
}

impl Default for ObservabilityPlugin {
    fn default() -> Self {
        Self::new(ObservabilityConfig::default())
    }
}

impl Plugin for ObservabilityPlugin {
    fn build(&self, app: &mut App) {
        // Initialize tracing
        init_tracing(&self.config).expect("Failed to initialize tracing");

        // Initialize metrics if enabled
        if self.config.metrics_enabled {
            init_metrics(&self.config).expect("Failed to initialize metrics");
        }

        // Initialize crash reporting if enabled
        if self.config.crash_reporting_enabled {
            init_crash_reporting();
        }

        // Add observability state as resource
        app.world
            .insert_resource(ObservabilityState::new(self.config.clone()));

        // Add observability systems
        app.add_system("presentation", observability_system);
    }
}

/// Initialize tracing with JSON output and filtering
fn init_tracing(config: &ObservabilityConfig) -> Result<()> {
    let level = match config.tracing_level.as_str() {
        "TRACE" => Level::TRACE,
        "DEBUG" => Level::DEBUG,
        "INFO" => Level::INFO,
        "WARN" => Level::WARN,
        "ERROR" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env().add_directive(level.into()))
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true),
        );

    subscriber.init();
    info!("Tracing initialized with level: {}", config.tracing_level);
    Ok(())
}

/// Initialize metrics with simple recorder
fn init_metrics(_config: &ObservabilityConfig) -> Result<()> {
    // For now, use a simple recorder that just logs metrics
    // In production, this could be extended to export to various backends
    info!("Metrics initialized with simple recorder");
    Ok(())
}

/// Initialize basic crash reporting (logs panics)
fn init_crash_reporting() {
    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::capture();
        error!("Panic occurred: {}\nBacktrace:\n{}", panic_info, backtrace);

        // In a real implementation, this would send to a crash reporting service
        // like Sentry, but for now we just log it
    }));

    info!("Crash reporting initialized");
}

/// System that collects observability metrics
fn observability_system(world: &mut astraweave_ecs::World) {
    if let Some(state) = world.get_resource::<ObservabilityState>() {
        if state.config.metrics_enabled {
            // For now, just log metrics instead of using metrics crate
            info!("Tick recorded");
        }
    }
}

/// Convenience function to initialize full observability stack
pub fn init_observability(config: ObservabilityConfig) -> Result<()> {
    init_tracing(&config)?;
    if config.metrics_enabled {
        init_metrics(&config)?;
    }
    if config.crash_reporting_enabled {
        init_crash_reporting();
    }
    Ok(())
}

/// Macros for common observability patterns
#[macro_export]
macro_rules! trace_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::TRACE, $name)
    };
}

#[macro_export]
macro_rules! debug_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::DEBUG, $name)
    };
}

#[macro_export]
macro_rules! info_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::INFO, $name)
    };
}

#[macro_export]
macro_rules! warn_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::WARN, $name)
    };
}

#[macro_export]
macro_rules! error_span {
    ($name:expr) => {
        tracing::span!(tracing::Level::ERROR, $name)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_ecs::App;

    #[test]
    fn test_observability_config_default() {
        let config = ObservabilityConfig::default();
        assert_eq!(config.tracing_level, "INFO");
        assert!(config.metrics_enabled);
        assert!(config.crash_reporting_enabled);
    }

    #[test]
    fn test_plugin_build() {
        let mut app = App::new();
        let plugin = ObservabilityPlugin::default();
        plugin.build(&mut app);

        // Check that the resource was inserted
        assert!(app.world.get_resource::<ObservabilityState>().is_some());
    }
}
