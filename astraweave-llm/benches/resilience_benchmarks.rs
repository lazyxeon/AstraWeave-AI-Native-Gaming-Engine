// Resilience Benchmarks - Circuit Breaker + Retry Logic
// Validates circuit breaker state transitions and retry backoff patterns

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Mock circuit breaker for benchmarking
#[derive(Clone, Copy, PartialEq, Debug)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct MockCircuitBreaker {
    state: Arc<Mutex<CircuitBreakerState>>,
    failure_threshold: u32,
    recovery_timeout_ms: u64,
}

struct CircuitBreakerState {
    current_state: CircuitState,
    failure_count: u32,
    success_count: u32,
    opened_at: Option<Instant>,
}

impl MockCircuitBreaker {
    fn new(failure_threshold: u32, recovery_timeout_ms: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState {
                current_state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                opened_at: None,
            })),
            failure_threshold,
            recovery_timeout_ms,
        }
    }

    fn can_proceed(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        
        match state.current_state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(opened_at) = state.opened_at {
                    if opened_at.elapsed() >= Duration::from_millis(self.recovery_timeout_ms) {
                        state.current_state = CircuitState::HalfOpen;
                        state.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        
        match state.current_state {
            CircuitState::Closed => {
                state.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= 2 {
                    state.current_state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.opened_at = None;
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        state.failure_count += 1;
        
        match state.current_state {
            CircuitState::Closed => {
                if state.failure_count >= self.failure_threshold {
                    state.current_state = CircuitState::Open;
                    state.opened_at = Some(Instant::now());
                }
            }
            CircuitState::HalfOpen => {
                state.current_state = CircuitState::Open;
                state.opened_at = Some(Instant::now());
                state.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }

    fn get_state(&self) -> CircuitState {
        self.state.lock().unwrap().current_state
    }

    fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        state.current_state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.opened_at = None;
    }
}

// Benchmark: Circuit breaker state transition overhead
fn bench_circuit_breaker_state_check(c: &mut Criterion) {
    let breaker = MockCircuitBreaker::new(5, 1000);
    
    c.bench_function("circuit_breaker_state_check", |b| {
        b.iter(|| {
            let can_proceed = breaker.can_proceed();
            black_box(can_proceed);
        });
    });
}

// Benchmark: Circuit breaker opening (5 failures)
fn bench_circuit_breaker_opening(c: &mut Criterion) {
    let breaker = MockCircuitBreaker::new(5, 1000);
    
    c.bench_function("circuit_breaker_opening", |b| {
        b.iter(|| {
            breaker.reset();
            
            // Trigger 5 failures to open circuit
            for _ in 0..5 {
                breaker.record_failure();
            }
            
            assert_eq!(breaker.get_state(), CircuitState::Open);
            black_box(breaker.get_state());
        });
    });
}

// Benchmark: Circuit breaker recovery cycle
fn bench_circuit_breaker_recovery(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("circuit_breaker_recovery", |b| {
        b.to_async(&rt).iter(|| async {
            let breaker = MockCircuitBreaker::new(5, 10); // 10ms recovery
            
            // Open circuit
            for _ in 0..5 {
                breaker.record_failure();
            }
            assert_eq!(breaker.get_state(), CircuitState::Open);
            
            // Wait for recovery timeout
            tokio::time::sleep(Duration::from_millis(15)).await;
            
            // Should transition to half-open
            assert!(breaker.can_proceed());
            assert_eq!(breaker.get_state(), CircuitState::HalfOpen);
            
            // Record 2 successes to close
            breaker.record_success();
            breaker.record_success();
            
            assert_eq!(breaker.get_state(), CircuitState::Closed);
            black_box(breaker.get_state());
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
            
            let mut backoffs = Vec::new();
            
            for attempt in 0..5 {
                let base = (initial_ms as f64 * multiplier.powi(attempt as i32))
                    .min(max_ms as f64) as u64;
                
                // Add jitter (±25%)
                let jitter_range = (base as f64 * 0.25) as u64;
                let jitter = (attempt * 42) % (jitter_range * 2);
                let final_backoff = base.saturating_add(jitter);
                
                backoffs.push(final_backoff);
            }
            
            black_box(backoffs);
        });
    });
}

// Benchmark: Retry execution with backoff
fn bench_retry_execution(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("retry_execution_3_attempts", |b| {
        b.to_async(&rt).iter(|| async {
            let mut attempt = 0;
            let max_attempts = 3;
            
            loop {
                attempt += 1;
                
                // Simulate operation (fails first 2 times, succeeds on 3rd)
                let success = attempt >= 3;
                
                if success {
                    break;
                }
                
                if attempt >= max_attempts {
                    break;
                }
                
                // Calculate backoff
                let backoff_ms = 50u64 * 2u64.pow(attempt - 1);
                tokio::time::sleep(Duration::from_millis(backoff_ms.min(500))).await;
            }
            
            black_box(attempt);
        });
    });
}

// Benchmark: Chaos test - Circuit breaker under load
fn bench_circuit_breaker_chaos_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("circuit_breaker_chaos");
    group.sample_size(20); // Fewer samples for long test
    
    for failure_rate in [0.1, 0.3, 0.5, 0.7].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}%_failures", (failure_rate * 100.0) as u32)),
            failure_rate,
            |b, &failure_rate| {
                b.iter(|| {
                    let breaker = MockCircuitBreaker::new(5, 1000);
                    let mut requests = 0;
                    let mut rejected = 0;
                    
                    // Simulate 100 requests
                    for i in 0..100 {
                        if !breaker.can_proceed() {
                            rejected += 1;
                            continue;
                        }
                        
                        requests += 1;
                        
                        // Simulate failure based on failure_rate
                        let fails = (i as f64 % (1.0 / failure_rate)) < 1.0;
                        
                        if fails {
                            breaker.record_failure();
                        } else {
                            breaker.record_success();
                        }
                    }
                    
                    black_box((requests, rejected));
                });
            },
        );
    }
    
    group.finish();
}

// Benchmark: Retry + Circuit Breaker combined
fn bench_retry_with_circuit_breaker(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("retry_with_circuit_breaker", |b| {
        b.to_async(&rt).iter(|| async {
            let breaker = MockCircuitBreaker::new(3, 10);
            let max_retries = 3;
            
            let mut attempt = 0;
            let mut success = false;
            
            while attempt < max_retries {
                // Check circuit breaker
                if !breaker.can_proceed() {
                    // Circuit open, immediate fallback
                    break;
                }
                
                attempt += 1;
                
                // Simulate operation (50% failure rate)
                let fails = attempt % 2 == 0;
                
                if fails {
                    breaker.record_failure();
                    
                    // Retry with backoff
                    let backoff_ms = 10u64 * 2u64.pow(attempt - 1);
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                } else {
                    breaker.record_success();
                    success = true;
                    break;
                }
            }
            
            black_box(success);
        });
    });
}

// Benchmark: Circuit breaker per-model isolation
fn bench_circuit_breaker_per_model(c: &mut Criterion) {
    use std::collections::HashMap;
    
    c.bench_function("circuit_breaker_per_model_isolation", |b| {
        b.iter(|| {
            let mut breakers: HashMap<&str, MockCircuitBreaker> = HashMap::new();
            breakers.insert("phi3", MockCircuitBreaker::new(5, 1000));
            breakers.insert("gpt4", MockCircuitBreaker::new(5, 1000));
            breakers.insert("claude", MockCircuitBreaker::new(5, 1000));
            
            // Phi3 fails
            for _ in 0..5 {
                breakers.get("phi3").unwrap().record_failure();
            }
            
            // GPT-4 and Claude still work
            assert!(breakers.get("gpt4").unwrap().can_proceed());
            assert!(breakers.get("claude").unwrap().can_proceed());
            
            // Phi3 circuit is open
            assert!(!breakers.get("phi3").unwrap().can_proceed());
            
            black_box(&breakers);
        });
    });
}

criterion_group!(
    benches,
    bench_circuit_breaker_state_check,
    bench_circuit_breaker_opening,
    bench_circuit_breaker_recovery,
    bench_retry_backoff_calculation,
    bench_retry_execution,
    bench_circuit_breaker_chaos_test,
    bench_retry_with_circuit_breaker,
    bench_circuit_breaker_per_model,
);

criterion_main!(benches);
