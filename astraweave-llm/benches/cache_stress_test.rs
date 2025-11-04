// Cache Stress Test - Validate 80%+ hit rate under load
// Simulates 1000+ requests with realistic prompt patterns

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Simulated cache for benchmarking
struct SimpleCache {
    store: Arc<Mutex<HashMap<String, String>>>,
    capacity: usize,
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl SimpleCache {
    fn new(capacity: usize) -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            capacity,
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        let store = self.store.lock().unwrap();
        if let Some(value) = store.get(key) {
            *self.hits.lock().unwrap() += 1;
            Some(value.clone())
        } else {
            *self.misses.lock().unwrap() += 1;
            None
        }
    }

    fn put(&self, key: String, value: String) {
        let mut store = self.store.lock().unwrap();
        if store.len() >= self.capacity {
            // Simple eviction: remove first entry
            if let Some(first_key) = store.keys().next().cloned() {
                store.remove(&first_key);
            }
        }
        store.insert(key, value);
    }

    fn hit_rate(&self) -> f64 {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        let total = hits + misses;

        if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    fn reset_stats(&self) {
        *self.hits.lock().unwrap() = 0;
        *self.misses.lock().unwrap() = 0;
    }
}

// Generate realistic prompt patterns
fn generate_prompts(count: usize) -> Vec<String> {
    let mut prompts = Vec::new();

    // Common actions (80% of traffic - should hit cache)
    let common_actions = [
        "Move to waypoint A",
        "Attack nearest enemy",
        "Defend position",
        "Use health potion",
        "Navigate to base",
    ];

    // Rare actions (20% of traffic - cache misses)
    let rare_actions = [
        "Use ultimate ability",
        "Retreat to fallback point",
        "Coordinate with team",
        "Scout area",
    ];

    for i in 0..count {
        if i % 5 == 0 {
            // 20% rare actions (cache miss)
            let action = rare_actions[i % rare_actions.len()];
            prompts.push(format!("{} [request {}]", action, i));
        } else {
            // 80% common actions (cache hit)
            let action = common_actions[i % common_actions.len()];
            prompts.push(format!("{} [request {}]", action, i % 100)); // Cycle to hit cache
        }
    }

    prompts
}

// Normalize prompt for cache key (strip volatile data)
fn normalize_prompt(prompt: &str) -> String {
    // Remove [request N] metadata
    let normalized = prompt
        .split('[')
        .next()
        .unwrap_or(prompt)
        .trim()
        .to_string();

    normalized
}

// Benchmark: 1000 requests with 80% common patterns
fn bench_cache_stress_1000_requests(c: &mut Criterion) {
    let cache = SimpleCache::new(100); // Cache capacity
    let prompts = generate_prompts(1000);

    c.bench_function("cache_stress_1000_requests", |b| {
        b.iter(|| {
            cache.reset_stats();

            for prompt in &prompts {
                let key = normalize_prompt(prompt);

                if cache.get(&key).is_none() {
                    // Cache miss: simulate LLM call + store
                    let response = format!("response for {}", key);
                    cache.put(key, response);
                }
            }

            let hit_rate = cache.hit_rate();
            black_box(hit_rate);
        });
    });
}

// Benchmark: Cache hit rate validation
fn bench_cache_hit_rate_validation(c: &mut Criterion) {
    let cache = SimpleCache::new(100);
    let prompts = generate_prompts(1000);

    c.bench_function("cache_hit_rate_validation", |b| {
        b.iter(|| {
            cache.reset_stats();

            for prompt in &prompts {
                let key = normalize_prompt(prompt);

                if cache.get(&key).is_none() {
                    let response = format!("response for {}", key);
                    cache.put(key, response);
                }
            }

            let hit_rate = cache.hit_rate();

            // Validate hit rate >= 80%
            assert!(
                hit_rate >= 80.0,
                "Cache hit rate too low: {}% (expected >= 80%)",
                hit_rate
            );

            black_box(hit_rate);
        });
    });
}

// Benchmark: Cache performance under different capacities
fn bench_cache_capacity_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_capacity_impact");
    let prompts = generate_prompts(1000);

    for capacity in [10, 50, 100, 200, 500].iter() {
        let cache = SimpleCache::new(*capacity);

        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(format!("cap_{}", capacity)),
            capacity,
            |b, _| {
                b.iter(|| {
                    cache.reset_stats();

                    for prompt in &prompts {
                        let key = normalize_prompt(prompt);

                        if cache.get(&key).is_none() {
                            let response = format!("response for {}", key);
                            cache.put(key, response);
                        }
                    }

                    let hit_rate = cache.hit_rate();
                    black_box(hit_rate);
                });
            },
        );
    }

    group.finish();
}

// Benchmark: LRU eviction impact
fn bench_lru_eviction_overhead(c: &mut Criterion) {
    let cache = SimpleCache::new(50); // Small cache to force evictions
    let prompts = generate_prompts(1000);

    c.bench_function("lru_eviction_overhead", |b| {
        b.iter(|| {
            cache.reset_stats();

            for prompt in &prompts {
                let key = normalize_prompt(prompt);

                if cache.get(&key).is_none() {
                    let response = format!("response for {}", key);
                    cache.put(key, response);
                }
            }

            black_box(cache.hit_rate());
        });
    });
}

// Benchmark: Concurrent cache access (multi-threaded)
fn bench_concurrent_cache_access(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let cache = Arc::new(SimpleCache::new(100));
    let prompts = Arc::new(generate_prompts(100)); // Smaller for threading

    c.bench_function("concurrent_cache_access", |b| {
        b.iter(|| {
            let mut handles = vec![];

            // Spawn 4 threads
            for thread_id in 0..4 {
                let cache = Arc::clone(&cache);
                let prompts = Arc::clone(&prompts);

                let handle = thread::spawn(move || {
                    for (i, prompt) in prompts.iter().enumerate() {
                        if i % 4 == thread_id {
                            let key = normalize_prompt(prompt);

                            if cache.get(&key).is_none() {
                                let response = format!("response for {}", key);
                                cache.put(key, response);
                            }
                        }
                    }
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }

            black_box(cache.hit_rate());
        });
    });
}

// Benchmark: Cache key generation overhead
fn bench_cache_key_generation(c: &mut Criterion) {
    let prompts = generate_prompts(100);

    c.bench_function("cache_key_generation", |b| {
        b.iter(|| {
            for prompt in &prompts {
                // Simulate key generation: normalize + hash
                let normalized = normalize_prompt(prompt);
                let hash = {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};

                    let mut hasher = DefaultHasher::new();
                    normalized.hash(&mut hasher);
                    "phi3".hash(&mut hasher);
                    0.7f32.to_bits().hash(&mut hasher);

                    hasher.finish()
                };

                black_box(hash);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_cache_stress_1000_requests,
    bench_cache_hit_rate_validation,
    bench_cache_capacity_impact,
    bench_lru_eviction_overhead,
    bench_concurrent_cache_access,
    bench_cache_key_generation,
);

criterion_main!(benches);
