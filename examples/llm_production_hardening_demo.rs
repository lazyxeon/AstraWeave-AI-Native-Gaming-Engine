use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{info, warn, error};

use astraweave_llm::{
    production_hardening::{ProductionHardeningLayer, HardeningConfig, HardenedRequest},
    backpressure::Priority,
};

/// Comprehensive demo showcasing Phase 4 LLM Production Hardening features
/// This demo simulates real-world production scenarios with various failure modes
/// and demonstrates how the hardening layer provides reliability and observability.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for observability
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🚀 Starting LLM Production Hardening Demo");

    // Create production hardening configuration
    let config = create_production_config();

    // Initialize the production hardening layer
    let hardening_layer = Arc::new(ProductionHardeningLayer::new(config));

    // Start all background services
    hardening_layer.start().await?;
    info!("✅ Production hardening layer started");

    // Demo 1: Basic successful request processing
    info!("\n=== Demo 1: Successful Request Processing ===");
    demo_successful_requests(&hardening_layer).await?;

    // Demo 2: Rate limiting behavior
    info!("\n=== Demo 2: Rate Limiting ===");
    demo_rate_limiting(&hardening_layer).await?;

    // Demo 3: Circuit breaker behavior
    info!("\n=== Demo 3: Circuit Breaker ===");
    demo_circuit_breaker(&hardening_layer).await?;

    // Demo 4: Backpressure and queue management
    info!("\n=== Demo 4: Backpressure Management ===");
    demo_backpressure(&hardening_layer).await?;

    // Demo 5: System health monitoring
    info!("\n=== Demo 5: System Health Monitoring ===");
    demo_health_monitoring(&hardening_layer).await?;

    // Demo 6: Error scenarios and graceful degradation
    info!("\n=== Demo 6: Error Handling and Graceful Degradation ===");
    demo_error_handling(&hardening_layer).await?;

    // Demo 7: Production metrics and observability
    info!("\n=== Demo 7: Production Metrics ===");
    demo_production_metrics(&hardening_layer).await?;

    // Shutdown
    info!("\n🛑 Shutting down production hardening layer");
    hardening_layer.shutdown().await?;
    info!("✅ Shutdown complete");

    Ok(())
}

fn create_production_config() -> HardeningConfig {
    use astraweave_llm::{
        rate_limiter::RateLimiterConfig,
        circuit_breaker::CircuitBreakerConfig,
        backpressure::BackpressureConfig,
        ab_testing::ABTestConfig,
    };
    use astraweave_observability::llm_telemetry::TelemetryConfig;

    HardeningConfig {
        rate_limiter: RateLimiterConfig {
            default_rpm: 10, // Low for demo
            default_tpm: 1000,
            user_rpm: 5,
            global_rpm: 20,
            allow_burst: true,
            burst_multiplier: 1.5,
            window_duration: Duration::from_secs(10), // Short for demo
            adaptive_limiting: true,
        },
        circuit_breaker: CircuitBreakerConfig {
            failure_threshold: 3, // Low for demo
            failure_window: 10,   // Short for demo
            minimum_requests: 2,
            recovery_timeout: 5,  // Short for demo
            success_threshold: 2,
            enabled: true,
        },
        backpressure: BackpressureConfig {
            max_concurrent_requests: 5, // Low for demo
            max_queue_size: 10,
            request_timeout: Duration::from_secs(5),
            processing_interval: Duration::from_millis(100),
            adaptive_concurrency: true,
            target_latency_ms: 500,
            load_shedding_threshold: 0.8,
            enable_graceful_degradation: true,
        },
        ab_testing: ABTestConfig::default(),
        telemetry: TelemetryConfig {
            max_traces: 100,
            log_content: false, // Privacy
            enable_cost_tracking: true,
            enable_prometheus: false,
            enable_opentelemetry: false,
            alert_thresholds: Default::default(),
            sampling_rate: 1.0, // Sample all for demo
        },
        health_check: Default::default(),
        graceful_shutdown_timeout: Duration::from_secs(10),
    }
}

async fn demo_successful_requests(layer: &Arc<ProductionHardeningLayer>) -> Result<()> {
    info!("Processing successful requests...");

    for i in 0..3 {
        let request = create_test_request(
            format!("user_{}", i),
            "gpt-3.5-turbo".to_string(),
            format!("Hello, this is request {}", i),
            Priority::Normal,
        );

        let result = layer.process_request(request, || async {
            // Simulate successful LLM call
            sleep(Duration::from_millis(100)).await;
            Ok::<String, anyhow::Error>(format!("Response to request {}", i))
        }).await;

        match result {
            astraweave_llm::production_hardening::HardeningResult::Success(response) => {
                info!("✅ Request {} succeeded: {}", i, response);
            }
            other => {
                warn!("⚠️  Request {} had unexpected result: {:?}", i, other);
            }
        }

        sleep(Duration::from_millis(200)).await;
    }

    Ok(())
}

async fn demo_rate_limiting(layer: &Arc<ProductionHardeningLayer>) -> Result<()> {
    info!("Testing rate limiting behavior...");

    // Rapidly send requests to trigger rate limiting
    let mut handles = Vec::new();

    for i in 0..15 {
        let layer = Arc::clone(layer);
        let handle = tokio::spawn(async move {
            let request = create_test_request(
                "rate_limit_user".to_string(),
                "gpt-3.5-turbo".to_string(),
                format!("Rate limit test {}", i),
                Priority::Normal,
            );

            let result = layer.process_request(request, || async {
                sleep(Duration::from_millis(50)).await;
                Ok::<String, anyhow::Error>(format!("Rate limited response {}", i))
            }).await;

            (i, result)
        });
        handles.push(handle);
    }

    for handle in handles {
        let (i, result) = handle.await?;
        match result {
            astraweave_llm::production_hardening::HardeningResult::Success(response) => {
                info!("✅ Request {} passed rate limiting: {}", i, response);
            }
            astraweave_llm::production_hardening::HardeningResult::RateLimited { retry_after, reason } => {
                warn!("🚫 Request {} rate limited: {} (retry after {:?})", i, reason, retry_after);
            }
            other => {
                warn!("⚠️  Request {} had unexpected result: {:?}", i, other);
            }
        }
    }

    Ok(())
}

async fn demo_circuit_breaker(layer: &Arc<ProductionHardeningLayer>) -> Result<()> {
    info!("Testing circuit breaker behavior...");

    // Send failing requests to open the circuit breaker
    info!("Sending failing requests to trigger circuit breaker...");
    for i in 0..5 {
        let request = create_test_request(
            format!("circuit_user_{}", i),
            "failing-model".to_string(),
            format!("This will fail {}", i),
            Priority::Normal,
        );

        let result = layer.process_request(request, || async {
            // Simulate failing LLM call
            sleep(Duration::from_millis(100)).await;
            Err::<String, anyhow::Error>(anyhow::anyhow!("Simulated failure"))
        }).await;

        match result {
            astraweave_llm::production_hardening::HardeningResult::Error(e) => {
                warn!("❌ Expected failure {}: {}", i, e);
            }
            astraweave_llm::production_hardening::HardeningResult::CircuitOpen { model, retry_after } => {
                warn!("🔓 Circuit breaker opened for model {}: retry after {:?}", model, retry_after);
                break;
            }
            other => {
                warn!("⚠️  Unexpected result for failure {}: {:?}", i, other);
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    // Wait for recovery and test successful request
    info!("Waiting for circuit breaker recovery...");
    sleep(Duration::from_secs(6)).await; // Wait longer than recovery timeout

    let request = create_test_request(
        "recovery_user".to_string(),
        "failing-model".to_string(),
        "Recovery test".to_string(),
        Priority::Normal,
    );

    let result = layer.process_request(request, || async {
        // Simulate successful recovery
        sleep(Duration::from_millis(100)).await;
        Ok::<String, anyhow::Error>("Recovery successful".to_string())
    }).await;

    match result {
        astraweave_llm::production_hardening::HardeningResult::Success(response) => {
            info!("✅ Circuit breaker recovered: {}", response);
        }
        other => {
            warn!("⚠️  Recovery attempt failed: {:?}", other);
        }
    }

    Ok(())
}

async fn demo_backpressure(layer: &Arc<ProductionHardeningLayer>) -> Result<()> {
    info!("Testing backpressure management...");

    // Submit many requests simultaneously to trigger queuing
    let mut handles = Vec::new();

    for i in 0..12 {
        let layer = Arc::clone(layer);
        let priority = match i % 3 {
            0 => Priority::High,
            1 => Priority::Normal,
            _ => Priority::Low,
        };

        let handle = tokio::spawn(async move {
            let request = create_test_request(
                format!("backpressure_user_{}", i),
                "gpt-4".to_string(),
                format!("Backpressure test {}", i),
                priority,
            );

            let start = std::time::Instant::now();
            let result = layer.process_request(request, || async {
                // Simulate longer processing time
                sleep(Duration::from_millis(500)).await;
                Ok::<String, anyhow::Error>(format!("Backpressure response {}", i))
            }).await;

            let elapsed = start.elapsed();
            (i, priority, result, elapsed)
        });
        handles.push(handle);

        // Small delay to stagger requests
        sleep(Duration::from_millis(10)).await;
    }

    for handle in handles {
        let (i, priority, result, elapsed) = handle.await?;
        match result {
            astraweave_llm::production_hardening::HardeningResult::Success(response) => {
                info!("✅ Request {} ({:?}) completed in {:?}: {}", i, priority, elapsed, response);
            }
            astraweave_llm::production_hardening::HardeningResult::Queued { position, estimated_wait } => {
                info!("📋 Request {} queued at position {} (estimated wait: {:?})", i, position, estimated_wait);
            }
            astraweave_llm::production_hardening::HardeningResult::Rejected { reason, retry_after } => {
                warn!("🚫 Request {} rejected: {} (retry after: {:?})", i, reason, retry_after);
            }
            other => {
                warn!("⚠️  Request {} had unexpected result: {:?}", i, other);
            }
        }
    }

    Ok(())
}

async fn demo_health_monitoring(layer: &Arc<ProductionHardeningLayer>) -> Result<()> {
    info!("Demonstrating system health monitoring...");

    // Get initial system status
    let status = layer.get_system_status().await;
    info!("📊 System Health Status:");
    info!("  Overall: {:?}", status.health.overall_status);
    info!("  Components: {} healthy", status.health.components.len());
    info!("  Uptime: {} seconds", status.health.uptime_seconds);

    // Display rate limiter status
    info!("📈 Rate Limiter Status:");
    info!("  Global: {}/{}", status.rate_limiter.global_current, status.rate_limiter.global_max);
    for (model, model_status) in &status.rate_limiter.model_status {
        info!("  Model {}: {}/{} requests, {}/{} tokens",
              model, model_status.current_requests, model_status.max_requests,
              model_status.current_tokens, model_status.max_tokens);
        info!("    Success rate: {:.2}%, Adaptive multiplier: {:.2}",
              model_status.success_rate * 100.0, model_status.adaptive_multiplier);
    }

    // Display circuit breaker status
    info!("🔌 Circuit Breakers:");
    for breaker in &status.circuit_breakers {
        info!("  {}: {:?} ({} failures, {} requests)",
              breaker.model, breaker.state, breaker.failure_count, breaker.request_count);
    }

    // Display backpressure metrics
    info!("🚰 Backpressure Metrics:");
    info!("  Active requests: {}", status.backpressure.current_active_requests);
    info!("  Queue size: {}", status.backpressure.current_queue_size);
    info!("  Load factor: {:.2}", status.backpressure.load_factor);
    info!("  Avg processing time: {:.1}ms", status.backpressure.average_processing_time_ms);

    // Display telemetry metrics
    info!("📡 Telemetry Metrics:");
    info!("  Total requests: {}", status.telemetry.total_requests);
    info!("  Success rate: {:.2}%", (1.0 - status.telemetry.error_rate) * 100.0);
    info!("  Avg latency: {:.1}ms", status.telemetry.average_latency_ms);
    info!("  Total cost: ${:.4}", status.telemetry.total_cost_usd);

    Ok(())
}

async fn demo_error_handling(layer: &Arc<ProductionHardeningLayer>) -> Result<()> {
    info!("Testing error handling and graceful degradation...");

    // Test various error scenarios
    let error_scenarios = vec![
        ("timeout", Priority::Normal),
        ("invalid_model", Priority::Low),
        ("network_error", Priority::High),
        ("server_overload", Priority::Background),
    ];

    for (scenario, priority) in error_scenarios {
        info!("Testing scenario: {} with priority {:?}", scenario, priority);

        let request = create_test_request(
            format!("error_user_{}", scenario),
            format!("error-{}", scenario),
            format!("Error test: {}", scenario),
            priority,
        );

        let result = timeout(Duration::from_secs(3), layer.process_request(request, || async {
            match scenario {
                "timeout" => {
                    sleep(Duration::from_secs(10)).await; // Will timeout
                    Ok::<String, anyhow::Error>("Should not reach here".to_string())
                }
                "invalid_model" => {
                    Err(anyhow::anyhow!("Model not found: invalid-model-name"))
                }
                "network_error" => {
                    Err(anyhow::anyhow!("Network unreachable"))
                }
                "server_overload" => {
                    Err(anyhow::anyhow!("Server overloaded: 503 Service Unavailable"))
                }
                _ => Ok("Unexpected scenario".to_string()),
            }
        })).await;

        match result {
            Ok(hardening_result) => {
                match hardening_result {
                    astraweave_llm::production_hardening::HardeningResult::Error(e) => {
                        warn!("❌ Expected error for {}: {}", scenario, e);
                    }
                    astraweave_llm::production_hardening::HardeningResult::Rejected { reason, .. } => {
                        info!("🚫 Request rejected for {}: {}", scenario, reason);
                    }
                    other => {
                        warn!("⚠️  Unexpected result for {}: {:?}", scenario, other);
                    }
                }
            }
            Err(_) => {
                warn!("⏰ Timeout occurred for scenario: {}", scenario);
            }
        }

        sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

async fn demo_production_metrics(layer: &Arc<ProductionHardeningLayer>) -> Result<()> {
    info!("Generating final production metrics report...");

    let status = layer.get_system_status().await;

    // Generate comprehensive metrics report
    info!("\n📊 PRODUCTION METRICS REPORT");
    info!("=====================================");

    info!("\n🏥 SYSTEM HEALTH");
    info!("Overall Status: {:?}", status.health.overall_status);
    info!("Uptime: {} seconds", status.health.uptime_seconds);
    info!("Components Status:");
    for (component, health) in &status.health.components {
        info!("  {}: {:?} (failures: {})", component, health.status, health.consecutive_failures);
        if let Some(response_time) = health.response_time_ms {
            info!("    Response time: {}ms", response_time);
        }
        if let Some(error) = &health.last_error {
            info!("    Last error: {}", error);
        }
    }

    info!("\n🚦 RATE LIMITING");
    info!("Global Usage: {}/{} requests", status.rate_limiter.global_current, status.rate_limiter.global_max);
    info!("Model Performance:");
    for (model, metrics) in &status.rate_limiter.model_status {
        let success_rate = metrics.success_rate * 100.0;
        info!("  {}: {:.1}% success, {:.2}x multiplier", model, success_rate, metrics.adaptive_multiplier);
    }

    info!("\n🔌 CIRCUIT BREAKERS");
    let healthy_breakers = status.circuit_breakers.iter()
        .filter(|b| b.state == astraweave_llm::circuit_breaker::CircuitState::Closed)
        .count();
    info!("Healthy circuits: {}/{}", healthy_breakers, status.circuit_breakers.len());
    for breaker in &status.circuit_breakers {
        if breaker.state != astraweave_llm::circuit_breaker::CircuitState::Closed {
            info!("  {}: {:?} (failures: {})", breaker.model, breaker.state, breaker.failure_count);
        }
    }

    info!("\n🚰 BACKPRESSURE");
    info!("Active Requests: {}", status.backpressure.current_active_requests);
    info!("Queue Depth: {}", status.backpressure.current_queue_size);
    info!("Load Factor: {:.1}%", status.backpressure.load_factor * 100.0);
    info!("Processing Stats:");
    info!("  Avg Queue Time: {:.1}ms", status.backpressure.average_queue_time_ms);
    info!("  Avg Processing Time: {:.1}ms", status.backpressure.average_processing_time_ms);

    info!("\n📈 PERFORMANCE METRICS");
    info!("Total Requests: {}", status.telemetry.total_requests);
    info!("Success Rate: {:.2}%", (1.0 - status.telemetry.error_rate) * 100.0);
    info!("Average Latency: {:.1}ms", status.telemetry.average_latency_ms);
    info!("Throughput: {:.1} req/s", status.telemetry.requests_per_second);
    info!("Token Rate: {:.0} tokens/s", status.telemetry.tokens_per_second);

    info!("\n💰 COST TRACKING");
    info!("Total Cost: ${:.4}", status.telemetry.total_cost_usd);
    info!("Cost per Hour: ${:.2}", status.telemetry.cost_per_hour_usd);
    info!("Total Tokens: {}", status.telemetry.total_tokens);

    info!("\n📋 REQUEST BREAKDOWN");
    info!("Successful: {}", status.telemetry.successful_requests);
    info!("Failed: {}", status.telemetry.failed_requests);
    if status.backpressure.total_requests > 0 {
        info!("Queued: {} ({:.1}%)", status.backpressure.queued_requests,
              (status.backpressure.queued_requests as f32 / status.backpressure.total_requests as f32) * 100.0);
        info!("Rejected: {} ({:.1}%)", status.backpressure.rejected_requests,
              (status.backpressure.rejected_requests as f32 / status.backpressure.total_requests as f32) * 100.0);
    }

    info!("\n=====================================");
    info!("✅ Production metrics report complete");

    Ok(())
}

fn create_test_request(user_id: String, model: String, prompt: String, priority: Priority) -> HardenedRequest {
    let mut metadata = HashMap::new();
    metadata.insert("demo".to_string(), "true".to_string());
    metadata.insert("source".to_string(), "production_hardening_demo".to_string());

    HardenedRequest {
        user_id: Some(user_id),
        session_id: Some(uuid::Uuid::new_v4().to_string()),
        model,
        prompt,
        estimated_tokens: 50,
        priority,
        timeout: Some(Duration::from_secs(5)),
        metadata,
    }
}

