// Telemetry module for LLM operations
// Thread-safe metrics collection with minimal overhead

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Global telemetry collector for LLM operations
pub struct LlmTelemetry {
    // Request metrics
    pub requests_total: AtomicU64,
    pub requests_success: AtomicU64,
    pub requests_error: AtomicU64,

    // Cache metrics (populated from cache stats)
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,

    // Retry/fallback metrics
    pub retries_attempted: AtomicU64,
    pub circuit_breaker_open: AtomicU64,
    pub fallbacks_triggered: AtomicU64,

    // Latency tracking (simplified - just total ms for averaging)
    pub latency_llm_call_ms: AtomicU64,
    pub latency_llm_call_count: AtomicU64,
    pub latency_plan_total_ms: AtomicU64,
    pub latency_plan_total_count: AtomicU64,
}

impl LlmTelemetry {
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_error: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            retries_attempted: AtomicU64::new(0),
            circuit_breaker_open: AtomicU64::new(0),
            fallbacks_triggered: AtomicU64::new(0),
            latency_llm_call_ms: AtomicU64::new(0),
            latency_llm_call_count: AtomicU64::new(0),
            latency_plan_total_ms: AtomicU64::new(0),
            latency_plan_total_count: AtomicU64::new(0),
        }
    }

    /// Record a request start
    pub fn record_request(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful request
    pub fn record_success(&self) {
        self.requests_success.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a failed request
    pub fn record_error(&self) {
        self.requests_error.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a retry attempt
    pub fn record_retry(&self) {
        self.retries_attempted.fetch_add(1, Ordering::Relaxed);
    }

    /// Record circuit breaker opening
    pub fn record_circuit_open(&self) {
        self.circuit_breaker_open.fetch_add(1, Ordering::Relaxed);
    }

    /// Record fallback activation
    pub fn record_fallback(&self) {
        self.fallbacks_triggered.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Record LLM call latency
    pub fn record_llm_latency(&self, duration: Duration) {
        self.latency_llm_call_ms
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        self.latency_llm_call_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record total plan generation latency
    pub fn record_plan_latency(&self, duration: Duration) {
        self.latency_plan_total_ms
            .fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        self.latency_plan_total_count
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> TelemetrySnapshot {
        let requests_total = self.requests_total.load(Ordering::Relaxed);
        let requests_success = self.requests_success.load(Ordering::Relaxed);
        let requests_error = self.requests_error.load(Ordering::Relaxed);

        let success_rate = if requests_total > 0 {
            (requests_success as f64 / requests_total as f64 * 100.0) as u32
        } else {
            0
        };

        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let cache_total = cache_hits + cache_misses;
        let cache_hit_rate = if cache_total > 0 {
            (cache_hits as f64 / cache_total as f64 * 100.0) as u32
        } else {
            0
        };

        let llm_call_count = self.latency_llm_call_count.load(Ordering::Relaxed);
        let avg_llm_latency_ms = if llm_call_count > 0 {
            self.latency_llm_call_ms.load(Ordering::Relaxed) / llm_call_count
        } else {
            0
        };

        let plan_count = self.latency_plan_total_count.load(Ordering::Relaxed);
        let avg_plan_latency_ms = if plan_count > 0 {
            self.latency_plan_total_ms.load(Ordering::Relaxed) / plan_count
        } else {
            0
        };

        TelemetrySnapshot {
            requests_total,
            requests_success,
            requests_error,
            success_rate,
            cache_hits,
            cache_misses,
            cache_hit_rate,
            retries_attempted: self.retries_attempted.load(Ordering::Relaxed),
            circuit_breaker_open: self.circuit_breaker_open.load(Ordering::Relaxed),
            fallbacks_triggered: self.fallbacks_triggered.load(Ordering::Relaxed),
            avg_llm_latency_ms,
            avg_plan_latency_ms,
        }
    }

    /// Reset all metrics (useful for testing)
    #[allow(dead_code)]
    pub fn reset(&self) {
        self.requests_total.store(0, Ordering::Relaxed);
        self.requests_success.store(0, Ordering::Relaxed);
        self.requests_error.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.retries_attempted.store(0, Ordering::Relaxed);
        self.circuit_breaker_open.store(0, Ordering::Relaxed);
        self.fallbacks_triggered.store(0, Ordering::Relaxed);
        self.latency_llm_call_ms.store(0, Ordering::Relaxed);
        self.latency_llm_call_count.store(0, Ordering::Relaxed);
        self.latency_plan_total_ms.store(0, Ordering::Relaxed);
        self.latency_plan_total_count.store(0, Ordering::Relaxed);
    }
}

impl Default for LlmTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of telemetry metrics at a point in time
#[derive(Debug, Clone, Copy)]
pub struct TelemetrySnapshot {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub success_rate: u32, // percentage 0-100
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: u32, // percentage 0-100
    pub retries_attempted: u64,
    pub circuit_breaker_open: u64,
    pub fallbacks_triggered: u64,
    pub avg_llm_latency_ms: u64,
    pub avg_plan_latency_ms: u64,
}

impl TelemetrySnapshot {
    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "LLM Telemetry:\n\
             Requests: {} total ({} success, {} error, {}% success rate)\n\
             Cache: {} hits, {} misses ({}% hit rate)\n\
             Retries: {}, Circuit Breaker Opens: {}, Fallbacks: {}\n\
             Latency: {}ms avg LLM call, {}ms avg plan total",
            self.requests_total,
            self.requests_success,
            self.requests_error,
            self.success_rate,
            self.cache_hits,
            self.cache_misses,
            self.cache_hit_rate,
            self.retries_attempted,
            self.circuit_breaker_open,
            self.fallbacks_triggered,
            self.avg_llm_latency_ms,
            self.avg_plan_latency_ms
        )
    }
}

/// Timer helper for measuring operation duration
pub struct OperationTimer {
    start: Instant,
}

impl OperationTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_creation() {
        let telemetry = LlmTelemetry::new();
        let snapshot = telemetry.snapshot();

        assert_eq!(snapshot.requests_total, 0);
        assert_eq!(snapshot.requests_success, 0);
        assert_eq!(snapshot.requests_error, 0);
        assert_eq!(snapshot.success_rate, 0);
    }

    #[test]
    fn test_record_requests() {
        let telemetry = LlmTelemetry::new();

        telemetry.record_request();
        telemetry.record_success();

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.requests_total, 1);
        assert_eq!(snapshot.requests_success, 1);
        assert_eq!(snapshot.success_rate, 100);
    }

    #[test]
    fn test_success_rate_calculation() {
        let telemetry = LlmTelemetry::new();

        // 3 requests: 2 success, 1 error
        telemetry.record_request();
        telemetry.record_success();

        telemetry.record_request();
        telemetry.record_success();

        telemetry.record_request();
        telemetry.record_error();

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.requests_total, 3);
        assert_eq!(snapshot.requests_success, 2);
        assert_eq!(snapshot.requests_error, 1);
        assert_eq!(snapshot.success_rate, 66); // 2/3 = 66%
    }

    #[test]
    fn test_cache_metrics() {
        let telemetry = LlmTelemetry::new();

        telemetry.record_cache_hit();
        telemetry.record_cache_hit();
        telemetry.record_cache_miss();

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.cache_hits, 2);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.cache_hit_rate, 66); // 2/3 = 66%
    }

    #[test]
    fn test_latency_tracking() {
        let telemetry = LlmTelemetry::new();

        telemetry.record_llm_latency(Duration::from_millis(100));
        telemetry.record_llm_latency(Duration::from_millis(200));

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.avg_llm_latency_ms, 150); // (100+200)/2
    }

    #[test]
    fn test_reset() {
        let telemetry = LlmTelemetry::new();

        telemetry.record_request();
        telemetry.record_success();
        telemetry.record_cache_hit();

        assert_eq!(telemetry.snapshot().requests_total, 1);

        telemetry.reset();

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.requests_total, 0);
        assert_eq!(snapshot.cache_hits, 0);
    }

    #[test]
    fn test_format_output() {
        let telemetry = LlmTelemetry::new();

        telemetry.record_request();
        telemetry.record_success();
        telemetry.record_cache_hit();

        let snapshot = telemetry.snapshot();
        let formatted = snapshot.format();

        assert!(formatted.contains("Requests: 1 total"));
        assert!(formatted.contains("100% success rate"));
        assert!(formatted.contains("Cache: 1 hits"));
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();

        assert!(elapsed.as_millis() >= 10);
    }
}
