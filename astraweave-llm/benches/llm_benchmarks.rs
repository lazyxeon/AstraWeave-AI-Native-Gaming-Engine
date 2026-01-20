// LLM Latency Benchmarks - Measure p50/p95/p99 percentiles
// Target: p99 < 100ms for cache hits, p99 < 500ms for LLM calls

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::time::Duration;

// Mock LLM client for benchmarking
struct MockLlmClient {
    latency_ms: u64,
}

impl MockLlmClient {
    fn new(latency_ms: u64) -> Self {
        Self { latency_ms }
    }

    async fn complete(&self, _prompt: &str) -> Result<String, String> {
        tokio::time::sleep(Duration::from_millis(self.latency_ms)).await;
        Ok("mock response".to_string())
    }
}

// Benchmark: Cache hit latency (should be < 1ms)
fn bench_cache_hit_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("cache_hit_latency", |b| {
        b.to_async(&rt).iter(|| async {
            // Simulate cache lookup (atomic read + hash lookup)
            let start = std::time::Instant::now();

            // Mock cache hit: hash lookup + string clone
            let _cached = black_box("cached response".to_string());

            start.elapsed()
        });
    });
}

// Benchmark: Cache miss + LLM call latency
fn bench_cache_miss_latency(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_miss_latency");

    for latency_ms in [10, 50, 100, 200].iter() {
        let client = MockLlmClient::new(*latency_ms);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}ms_llm", latency_ms)),
            latency_ms,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let start = std::time::Instant::now();

                    // Cache miss: LLM call + parse + store
                    let _response = client.complete(black_box("test prompt")).await.unwrap();

                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Prompt normalization overhead
fn bench_prompt_normalization(c: &mut Criterion) {
    let prompts = vec![
        "Move to (5,3). Attack enemy. t=1.234",
        "Enemy at (10,20). Defend position. [TIMESTAMP: 2025-10-14T12:34:56Z]",
        "Navigate  to   waypoint   A.   Use   stealth.   t=5.678",
    ];

    c.bench_function("prompt_normalization", |b| {
        b.iter(|| {
            for prompt in &prompts {
                // Simulate normalization: regex replace + whitespace collapse
                let normalized = black_box(prompt)
                    .replace("t=1.234", "t=X.XXX")
                    .replace("t=5.678", "t=X.XXX")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                black_box(normalized);
            }
        });
    });
}

// Benchmark: Telemetry overhead (atomic operations)
fn bench_telemetry_overhead(c: &mut Criterion) {
    use std::sync::atomic::{AtomicU64, Ordering};

    let counter = AtomicU64::new(0);

    c.bench_function("telemetry_record_request", |b| {
        b.iter(|| {
            // Simulate telemetry recording (3 atomic increments)
            counter.fetch_add(1, Ordering::Relaxed); // requests_total
            counter.fetch_add(1, Ordering::Relaxed); // requests_success
            counter.fetch_add(100, Ordering::Relaxed); // latency_ms
        });
    });
}

// Benchmark: Retry backoff calculation
fn bench_retry_backoff_calculation(c: &mut Criterion) {
    c.bench_function("retry_backoff_calculation", |b| {
        b.iter(|| {
            let initial_ms = 50u64;
            let multiplier = 2.0f64;
            let max_ms = 500u64;

            for attempt in 0i32..5 {
                let backoff =
                    (initial_ms as f64 * multiplier.powi(attempt)).min(max_ms as f64) as u64;

                // Add jitter (±25%)
                let jitter_range = (backoff as f64 * 0.25) as u64;
                let jitter = black_box(42u64 % (jitter_range * 2));
                let final_backoff = backoff.saturating_add(jitter);

                black_box(final_backoff);
            }
        });
    });
}

// Benchmark: Circuit breaker state check overhead
fn bench_circuit_breaker_check(c: &mut Criterion) {
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Copy)]
    #[allow(dead_code)]
    enum CircuitState {
        Closed,
        Open,
        HalfOpen,
    }

    let state = Arc::new(Mutex::new(CircuitState::Closed));

    c.bench_function("circuit_breaker_state_check", |b| {
        b.iter(|| {
            // Simulate circuit breaker check (mutex lock + state read)
            let s = state.lock().unwrap();
            let can_proceed = matches!(*s, CircuitState::Closed);
            black_box(can_proceed);
        });
    });
}

// Benchmark: End-to-end plan generation (with all overhead)
fn bench_end_to_end_plan_generation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("end_to_end_plan_generation");
    group.sample_size(50); // Fewer samples for longer operations

    for scenario in ["cache_hit", "cache_miss_fast", "cache_miss_slow"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario),
            scenario,
            |b, scenario| {
                b.to_async(&rt).iter(|| async {
                    let start = std::time::Instant::now();

                    // Simulate full pipeline
                    match *scenario {
                        "cache_hit" => {
                            // Cache hit: 1ms
                            tokio::time::sleep(Duration::from_micros(100)).await;
                        }
                        "cache_miss_fast" => {
                            // Cache miss + fast LLM: 50ms
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        }
                        "cache_miss_slow" => {
                            // Cache miss + slow LLM: 200ms
                            tokio::time::sleep(Duration::from_millis(200)).await;
                        }
                        _ => unreachable!(),
                    }

                    // Add overhead: telemetry (1µs) + validation (10µs)
                    tokio::time::sleep(Duration::from_micros(11)).await;

                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_hit_latency,
    bench_cache_miss_latency,
    bench_prompt_normalization,
    bench_telemetry_overhead,
    bench_retry_backoff_calculation,
    bench_circuit_breaker_check,
    bench_end_to_end_plan_generation,
);

criterion_main!(benches);
