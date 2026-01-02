//! Adversarial Optimization Benchmarks
//!
//! Stress testing for batch inference, prompt caching, compression, load balancing, and token optimization.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-optimization API)
// ============================================================================

#[derive(Clone, Debug)]
struct InferenceRequest {
    id: u64,
    prompt: String,
    max_tokens: u32,
    temperature: f32,
    priority: Priority,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, Debug)]
struct BatchedRequest {
    requests: Vec<InferenceRequest>,
    combined_tokens: usize,
    max_batch_size: usize,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    key: String,
    tokens: Vec<u32>,
    embedding: Vec<f32>,
    hit_count: u64,
    last_access: u64,
}

#[derive(Clone, Debug)]
struct PromptCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
    current_size: usize,
}

impl PromptCache {
    fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            current_size: 0,
        }
    }
    
    fn get(&mut self, key: &str, timestamp: u64) -> Option<&CacheEntry> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.hit_count += 1;
            entry.last_access = timestamp;
            return Some(entry);
        }
        None
    }
    
    fn insert(&mut self, key: String, tokens: Vec<u32>, embedding: Vec<f32>, timestamp: u64) {
        let size = tokens.len() * 4 + embedding.len() * 4;
        
        // Evict if necessary
        while self.current_size + size > self.max_size && !self.entries.is_empty() {
            // Find LRU entry
            if let Some((lru_key, lru_size)) = self.entries
                .iter()
                .min_by_key(|(_, e)| e.last_access)
                .map(|(k, e)| (k.clone(), e.tokens.len() * 4 + e.embedding.len() * 4))
            {
                self.entries.remove(&lru_key);
                self.current_size -= lru_size;
            }
        }
        
        self.entries.insert(key.clone(), CacheEntry {
            key,
            tokens,
            embedding,
            hit_count: 0,
            last_access: timestamp,
        });
        self.current_size += size;
    }
}

#[derive(Clone, Debug)]
struct LoadBalancer {
    workers: Vec<WorkerState>,
    strategy: LoadBalanceStrategy,
}

#[derive(Clone, Debug)]
struct WorkerState {
    id: u32,
    current_load: f32,
    capacity: f32,
    queue_length: usize,
    latency_ms: f32,
}

#[derive(Clone, Copy, Debug)]
enum LoadBalanceStrategy {
    RoundRobin,
    LeastLoaded,
    WeightedRandom,
    LatencyAware,
}

impl LoadBalancer {
    fn select_worker(&self, _request_size: usize) -> Option<u32> {
        match self.strategy {
            LoadBalanceStrategy::RoundRobin => {
                self.workers.first().map(|w| w.id)
            }
            LoadBalanceStrategy::LeastLoaded => {
                self.workers
                    .iter()
                    .filter(|w| w.current_load < w.capacity)
                    .min_by(|a, b| a.current_load.partial_cmp(&b.current_load).unwrap())
                    .map(|w| w.id)
            }
            LoadBalanceStrategy::WeightedRandom => {
                // Simple weighted selection
                let total_capacity: f32 = self.workers.iter().map(|w| w.capacity - w.current_load).sum();
                if total_capacity <= 0.0 {
                    return None;
                }
                // Just return first available for benchmark
                self.workers.iter().find(|w| w.current_load < w.capacity).map(|w| w.id)
            }
            LoadBalanceStrategy::LatencyAware => {
                self.workers
                    .iter()
                    .filter(|w| w.current_load < w.capacity)
                    .min_by(|a, b| a.latency_ms.partial_cmp(&b.latency_ms).unwrap())
                    .map(|w| w.id)
            }
        }
    }
}

#[derive(Clone, Debug)]
struct TokenOptimizer {
    vocab_size: usize,
    merges: Vec<(String, String, String)>, // (a, b, merged)
}

impl TokenOptimizer {
    fn tokenize(&self, text: &str) -> Vec<u32> {
        // Simple character-level tokenization for benchmark
        text.chars()
            .map(|c| (c as u32) % self.vocab_size as u32)
            .collect()
    }
    
    fn optimize_tokens(&self, tokens: &[u32]) -> Vec<u32> {
        // Simulate token merging
        let mut result = tokens.to_vec();
        let merge_count = result.len() / 4;
        result.truncate(result.len() - merge_count);
        result
    }
}

// ============================================================================
// CATEGORY 1: BATCH INFERENCE
// ============================================================================

fn bench_batch_inference(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_adversarial/batch_inference");
    
    // Test 1: Request batching
    for batch_size in [16, 32, 64] {
        group.throughput(Throughput::Elements(batch_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("request_batching", batch_size),
            &batch_size,
            |bencher, &batch_size| {
                let requests: Vec<InferenceRequest> = (0..1000)
                    .map(|i| InferenceRequest {
                        id: i as u64,
                        prompt: format!("This is prompt number {} with some content", i),
                        max_tokens: 100 + (i % 100) as u32,
                        temperature: 0.7,
                        priority: match i % 4 {
                            0 => Priority::Low,
                            1 => Priority::Normal,
                            2 => Priority::High,
                            _ => Priority::Critical,
                        },
                    })
                    .collect();
                
                let max_tokens_per_batch = batch_size * 150;
                
                bencher.iter(|| {
                    let mut batches: Vec<BatchedRequest> = Vec::new();
                    let mut current_batch: Vec<InferenceRequest> = Vec::new();
                    let mut current_tokens = 0usize;
                    
                    for req in &requests {
                        let req_tokens = req.prompt.len() + req.max_tokens as usize;
                        
                        if current_batch.len() >= batch_size || current_tokens + req_tokens > max_tokens_per_batch {
                            if !current_batch.is_empty() {
                                batches.push(BatchedRequest {
                                    requests: std::mem::take(&mut current_batch),
                                    combined_tokens: current_tokens,
                                    max_batch_size: batch_size,
                                });
                                current_tokens = 0;
                            }
                        }
                        
                        current_batch.push(req.clone());
                        current_tokens += req_tokens;
                    }
                    
                    if !current_batch.is_empty() {
                        batches.push(BatchedRequest {
                            requests: current_batch,
                            combined_tokens: current_tokens,
                            max_batch_size: batch_size,
                        });
                    }
                    
                    std_black_box(batches.len())
                });
            },
        );
    }
    
    // Test 2: Priority sorting
    group.bench_function("priority_sorting_5000", |bencher| {
        let mut requests: Vec<InferenceRequest> = (0..5000)
            .map(|i| InferenceRequest {
                id: i as u64,
                prompt: format!("Prompt {}", i),
                max_tokens: 100,
                temperature: 0.7,
                priority: match i % 4 {
                    0 => Priority::Low,
                    1 => Priority::Normal,
                    2 => Priority::High,
                    _ => Priority::Critical,
                },
            })
            .collect();
        
        bencher.iter(|| {
            requests.sort_by(|a, b| b.priority.cmp(&a.priority));
            
            let critical_count = requests.iter().filter(|r| r.priority == Priority::Critical).count();
            std_black_box(critical_count)
        });
    });
    
    // Test 3: Batch padding calculation
    group.bench_function("batch_padding_1000", |bencher| {
        let batches: Vec<Vec<usize>> = (0..1000)
            .map(|i| {
                (0..32)
                    .map(|j| 50 + (i * j) % 200)
                    .collect()
            })
            .collect();
        
        bencher.iter(|| {
            let padding_stats: Vec<(usize, usize, f32)> = batches
                .iter()
                .map(|batch| {
                    let max_len = batch.iter().max().copied().unwrap_or(0);
                    let total_tokens: usize = batch.iter().sum();
                    let padded_tokens = batch.len() * max_len;
                    let efficiency = total_tokens as f32 / padded_tokens as f32;
                    (total_tokens, padded_tokens, efficiency)
                })
                .collect();
            
            let avg_efficiency: f32 = padding_stats.iter().map(|(_, _, e)| e).sum::<f32>()
                / padding_stats.len() as f32;
            
            std_black_box(avg_efficiency)
        });
    });
    
    // Test 4: Dynamic batch sizing
    group.bench_function("dynamic_batch_sizing_2000", |bencher| {
        let request_sizes: Vec<usize> = (0..2000)
            .map(|i| 20 + (i % 500))
            .collect();
        
        let memory_budget = 50000usize;
        
        bencher.iter(|| {
            let mut batches: Vec<(usize, usize)> = Vec::new(); // (count, total_size)
            let mut current_count = 0;
            let mut current_size = 0;
            
            for &size in &request_sizes {
                if current_size + size > memory_budget {
                    batches.push((current_count, current_size));
                    current_count = 0;
                    current_size = 0;
                }
                
                current_count += 1;
                current_size += size;
            }
            
            if current_count > 0 {
                batches.push((current_count, current_size));
            }
            
            std_black_box(batches.len())
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 2: PROMPT CACHING
// ============================================================================

fn bench_prompt_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_adversarial/prompt_caching");
    
    // Test 1: Cache lookup
    group.bench_function("cache_lookup_10000", |bencher| {
        let mut cache = PromptCache::new(10_000_000);
        
        // Pre-populate cache
        for i in 0..1000 {
            let key = format!("prompt_{}", i);
            let tokens: Vec<u32> = (0..100).map(|j| (i * 100 + j) as u32).collect();
            let embedding: Vec<f32> = (0..768).map(|j| (i + j) as f32 * 0.001).collect();
            cache.insert(key, tokens, embedding, i as u64);
        }
        
        let lookups: Vec<String> = (0..10000)
            .map(|i| format!("prompt_{}", i % 1000))
            .collect();
        
        bencher.iter(|| {
            let mut hits = 0;
            let mut timestamp = 10000u64;
            
            for key in &lookups {
                if cache.get(key, timestamp).is_some() {
                    hits += 1;
                }
                timestamp += 1;
            }
            
            std_black_box(hits)
        });
    });
    
    // Test 2: Cache insertion with eviction
    group.bench_function("cache_eviction_5000", |bencher| {
        bencher.iter(|| {
            let mut cache = PromptCache::new(100_000); // Small cache to trigger eviction
            
            for i in 0..5000 {
                let key = format!("prompt_{}", i);
                let tokens: Vec<u32> = (0..50).map(|j| (i * 50 + j) as u32).collect();
                let embedding: Vec<f32> = (0..384).map(|j| (i + j) as f32 * 0.001).collect();
                cache.insert(key, tokens, embedding, i as u64);
            }
            
            std_black_box(cache.entries.len())
        });
    });
    
    // Test 3: Prefix matching
    group.bench_function("prefix_matching_5000", |bencher| {
        let cached_prefixes: Vec<(String, Vec<u32>)> = (0..500)
            .map(|i| {
                let prefix = format!("You are a helpful assistant. Task {}: ", i);
                let tokens: Vec<u32> = (0..20).map(|j| (i * 20 + j) as u32).collect();
                (prefix, tokens)
            })
            .collect();
        
        let prompts: Vec<String> = (0..5000)
            .map(|i| {
                format!(
                    "You are a helpful assistant. Task {}: Please help me with {}",
                    i % 500,
                    i
                )
            })
            .collect();
        
        bencher.iter(|| {
            let matches: Vec<Option<usize>> = prompts
                .iter()
                .map(|prompt| {
                    cached_prefixes
                        .iter()
                        .enumerate()
                        .find(|(_, (prefix, _))| prompt.starts_with(prefix))
                        .map(|(idx, _)| idx)
                })
                .collect();
            
            let hit_count = matches.iter().filter(|m| m.is_some()).count();
            std_black_box(hit_count)
        });
    });
    
    // Test 4: Cache statistics
    group.bench_function("cache_statistics_1000", |bencher| {
        let mut cache = PromptCache::new(10_000_000);
        
        // Build cache with varying hit patterns
        for i in 0..1000 {
            let key = format!("prompt_{}", i);
            let tokens: Vec<u32> = (0..100).collect();
            let embedding: Vec<f32> = vec![0.0; 768];
            cache.insert(key, tokens, embedding, i as u64);
        }
        
        // Simulate hit patterns
        for i in 0..5000 {
            // Hot items get more hits
            let key = format!("prompt_{}", i % 100);
            cache.get(&key, 1000 + i as u64);
        }
        
        bencher.iter(|| {
            let stats: Vec<(u64, u64)> = cache.entries
                .values()
                .map(|e| (e.hit_count, e.last_access))
                .collect();
            
            let total_hits: u64 = stats.iter().map(|(h, _)| h).sum();
            let avg_hits = total_hits as f64 / stats.len() as f64;
            
            // Find hot entries (above average hits)
            let hot_count = stats.iter().filter(|(h, _)| *h > avg_hits as u64).count();
            
            std_black_box(hot_count)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 3: COMPRESSION
// ============================================================================

fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_adversarial/compression");
    
    // Test 1: Token compression
    group.bench_function("token_compression_10000", |bencher| {
        let token_sequences: Vec<Vec<u32>> = (0..10000)
            .map(|i| {
                (0..100)
                    .map(|j| ((i * 100 + j) % 50000) as u32)
                    .collect()
            })
            .collect();
        
        bencher.iter(|| {
            let compressed: Vec<Vec<u8>> = token_sequences
                .iter()
                .map(|tokens| {
                    // Simulate variable-length encoding
                    let mut bytes = Vec::with_capacity(tokens.len() * 2);
                    
                    for &token in tokens {
                        if token < 128 {
                            bytes.push(token as u8);
                        } else if token < 16384 {
                            bytes.push((token >> 7) as u8 | 0x80);
                            bytes.push((token & 0x7F) as u8);
                        } else {
                            bytes.push((token >> 14) as u8 | 0x80);
                            bytes.push(((token >> 7) & 0x7F) as u8 | 0x80);
                            bytes.push((token & 0x7F) as u8);
                        }
                    }
                    
                    bytes
                })
                .collect();
            
            let total_bytes: usize = compressed.iter().map(|c| c.len()).sum();
            let original_bytes = token_sequences.len() * 100 * 4;
            let ratio = total_bytes as f64 / original_bytes as f64;
            
            std_black_box(ratio)
        });
    });
    
    // Test 2: Embedding quantization
    group.bench_function("embedding_quantization_1000", |bencher| {
        let embeddings: Vec<Vec<f32>> = (0..1000)
            .map(|i| {
                (0..768)
                    .map(|j| ((i + j) as f32 * 0.001 - 0.5).sin())
                    .collect()
            })
            .collect();
        
        bencher.iter(|| {
            // Quantize to int8
            let quantized: Vec<Vec<i8>> = embeddings
                .iter()
                .map(|emb| {
                    let max_val = emb.iter().cloned().fold(0.0f32, f32::max).max(0.001);
                    let min_val = emb.iter().cloned().fold(0.0f32, f32::min);
                    let scale = (max_val - min_val) / 255.0;
                    
                    emb.iter()
                        .map(|&v| ((v - min_val) / scale - 128.0) as i8)
                        .collect()
                })
                .collect();
            
            std_black_box(quantized.len())
        });
    });
    
    // Test 3: KV cache compression
    group.bench_function("kv_cache_compression_500", |bencher| {
        // Simulate key-value cache entries
        let kv_cache: Vec<(Vec<f32>, Vec<f32>)> = (0..500)
            .map(|i| {
                let keys: Vec<f32> = (0..64).map(|j| (i + j) as f32 * 0.01).collect();
                let values: Vec<f32> = (0..64).map(|j| (i + j) as f32 * 0.02).collect();
                (keys, values)
            })
            .collect();
        
        bencher.iter(|| {
            // Group similar entries
            let mut groups: Vec<Vec<usize>> = Vec::new();
            let threshold = 0.1f32;
            
            for (i, (keys_i, _)) in kv_cache.iter().enumerate() {
                let mut found_group = false;
                
                for group in groups.iter_mut() {
                    if let Some(&first) = group.first() {
                        let (keys_first, _) = &kv_cache[first];
                        
                        // Simple cosine similarity
                        let dot: f32 = keys_i.iter().zip(keys_first).map(|(a, b)| a * b).sum();
                        let norm_i: f32 = keys_i.iter().map(|x| x * x).sum::<f32>().sqrt();
                        let norm_first: f32 = keys_first.iter().map(|x| x * x).sum::<f32>().sqrt();
                        
                        let similarity = dot / (norm_i * norm_first + 0.0001);
                        
                        if similarity > 1.0 - threshold {
                            group.push(i);
                            found_group = true;
                            break;
                        }
                    }
                }
                
                if !found_group {
                    groups.push(vec![i]);
                }
            }
            
            std_black_box(groups.len())
        });
    });
    
    // Test 4: Sparse attention compression
    group.bench_function("sparse_attention_1000", |bencher| {
        let attention_matrices: Vec<Vec<Vec<f32>>> = (0..100)
            .map(|i| {
                (0..32)
                    .map(|j| {
                        (0..32)
                            .map(|k| {
                                if (j as i32 - k as i32).abs() < 5 || (i + j + k) % 10 == 0 {
                                    ((i + j + k) as f32 * 0.1).exp() / 10.0
                                } else {
                                    0.0
                                }
                            })
                            .collect()
                    })
                    .collect()
            })
            .collect();
        
        bencher.iter(|| {
            // Convert to sparse format (COO)
            let sparse: Vec<Vec<(usize, usize, f32)>> = attention_matrices
                .iter()
                .map(|matrix| {
                    matrix
                        .iter()
                        .enumerate()
                        .flat_map(|(i, row)| {
                            row.iter()
                                .enumerate()
                                .filter(|(_, &v)| v.abs() > 0.001)
                                .map(move |(j, &v)| (i, j, v))
                        })
                        .collect()
                })
                .collect();
            
            let total_nonzero: usize = sparse.iter().map(|s| s.len()).sum();
            let total_elements = attention_matrices.len() * 32 * 32;
            let sparsity = 1.0 - (total_nonzero as f64 / total_elements as f64);
            
            std_black_box(sparsity)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 4: LOAD BALANCING
// ============================================================================

fn bench_load_balancing(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_adversarial/load_balancing");
    
    // Test 1: Worker selection strategies
    for strategy in [
        LoadBalanceStrategy::RoundRobin,
        LoadBalanceStrategy::LeastLoaded,
        LoadBalanceStrategy::WeightedRandom,
        LoadBalanceStrategy::LatencyAware,
    ] {
        group.bench_with_input(
            BenchmarkId::new("worker_selection", format!("{:?}", strategy)),
            &strategy,
            |bencher, &strategy| {
                let balancer = LoadBalancer {
                    workers: (0..8)
                        .map(|i| WorkerState {
                            id: i,
                            current_load: (i as f32 * 10.0) % 80.0,
                            capacity: 100.0,
                            queue_length: (i as usize * 5) % 50,
                            latency_ms: 10.0 + (i as f32 * 2.0),
                        })
                        .collect(),
                    strategy,
                };
                
                let request_sizes: Vec<usize> = (0..10000)
                    .map(|i| 100 + (i % 1000))
                    .collect();
                
                bencher.iter(|| {
                    let selections: Vec<Option<u32>> = request_sizes
                        .iter()
                        .map(|&size| balancer.select_worker(size))
                        .collect();
                    
                    let successful = selections.iter().filter(|s| s.is_some()).count();
                    std_black_box(successful)
                });
            },
        );
    }
    
    // Test 2: Load rebalancing
    group.bench_function("load_rebalancing_1000", |bencher| {
        let initial_loads: Vec<f32> = vec![90.0, 85.0, 10.0, 5.0, 50.0, 45.0, 95.0, 15.0];
        let target_load = 50.0f32;
        
        bencher.iter(|| {
            let mut loads = initial_loads.clone();
            let mut migrations = 0;
            
            // Iterative rebalancing
            for _ in 0..1000 {
                let max_idx = loads
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap();
                
                let min_idx = loads
                    .iter()
                    .enumerate()
                    .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap();
                
                if loads[max_idx] - loads[min_idx] < 10.0 {
                    break;
                }
                
                // Migrate 10% of load
                let transfer = (loads[max_idx] - target_load) * 0.1;
                loads[max_idx] -= transfer;
                loads[min_idx] += transfer;
                migrations += 1;
            }
            
            std_black_box(migrations)
        });
    });
    
    // Test 3: Health checking
    group.bench_function("health_checking_5000", |bencher| {
        let workers: Vec<(u32, f32, f32, bool)> = (0..100)
            .map(|i| {
                let latency = 10.0 + (i as f32 * 0.5);
                let error_rate = (i % 10) as f32 * 0.01;
                let is_healthy = latency < 50.0 && error_rate < 0.05;
                (i, latency, error_rate, is_healthy)
            })
            .collect();
        
        let checks: Vec<(u32, f32, f32)> = (0..5000)
            .map(|i| {
                let worker_id = (i % 100) as u32;
                let latency = workers[worker_id as usize].1 + (i % 10) as f32;
                let error_rate = workers[worker_id as usize].2 + (i % 5) as f32 * 0.001;
                (worker_id, latency, error_rate)
            })
            .collect();
        
        bencher.iter(|| {
            let mut health_status: HashMap<u32, (f32, f32, bool)> = HashMap::new();
            
            for (worker_id, latency, error_rate) in &checks {
                let entry = health_status.entry(*worker_id).or_insert((0.0, 0.0, true));
                
                // Exponential moving average
                entry.0 = entry.0 * 0.9 + latency * 0.1;
                entry.1 = entry.1 * 0.9 + error_rate * 0.1;
                entry.2 = entry.0 < 100.0 && entry.1 < 0.1;
            }
            
            let healthy_count = health_status.values().filter(|(_, _, h)| *h).count();
            std_black_box(healthy_count)
        });
    });
    
    // Test 4: Request routing
    group.bench_function("request_routing_10000", |bencher| {
        let workers: Vec<(u32, Vec<String>)> = (0..8)
            .map(|i| {
                let capabilities: Vec<String> = match i % 4 {
                    0 => vec!["text".to_string(), "code".to_string()],
                    1 => vec!["text".to_string(), "image".to_string()],
                    2 => vec!["code".to_string(), "math".to_string()],
                    _ => vec!["text".to_string()],
                };
                (i, capabilities)
            })
            .collect();
        
        let requests: Vec<String> = (0..10000)
            .map(|i| {
                ["text", "code", "image", "math"][i % 4].to_string()
            })
            .collect();
        
        bencher.iter(|| {
            let routes: Vec<Option<u32>> = requests
                .iter()
                .map(|req_type| {
                    workers
                        .iter()
                        .find(|(_, caps)| caps.contains(req_type))
                        .map(|(id, _)| *id)
                })
                .collect();
            
            let routed = routes.iter().filter(|r| r.is_some()).count();
            std_black_box(routed)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 5: TOKEN OPTIMIZATION
// ============================================================================

fn bench_token_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_adversarial/token_optimization");
    
    // Test 1: Tokenization
    group.bench_function("tokenization_5000", |bencher| {
        let optimizer = TokenOptimizer {
            vocab_size: 50000,
            merges: Vec::new(),
        };
        
        let texts: Vec<String> = (0..5000)
            .map(|i| {
                format!(
                    "This is sample text number {} for tokenization benchmarking purposes.",
                    i
                )
            })
            .collect();
        
        bencher.iter(|| {
            let tokenized: Vec<Vec<u32>> = texts
                .iter()
                .map(|t| optimizer.tokenize(t))
                .collect();
            
            let total_tokens: usize = tokenized.iter().map(|t| t.len()).sum();
            std_black_box(total_tokens)
        });
    });
    
    // Test 2: Token merging
    group.bench_function("token_merging_2000", |bencher| {
        let optimizer = TokenOptimizer {
            vocab_size: 50000,
            merges: Vec::new(),
        };
        
        let token_sequences: Vec<Vec<u32>> = (0..2000)
            .map(|i| {
                (0..200)
                    .map(|j| ((i * 200 + j) % 50000) as u32)
                    .collect()
            })
            .collect();
        
        bencher.iter(|| {
            let optimized: Vec<Vec<u32>> = token_sequences
                .iter()
                .map(|tokens| optimizer.optimize_tokens(tokens))
                .collect();
            
            let reduction: usize = token_sequences.iter().map(|t| t.len()).sum::<usize>()
                - optimized.iter().map(|t| t.len()).sum::<usize>();
            
            std_black_box(reduction)
        });
    });
    
    // Test 3: Context window packing
    group.bench_function("context_packing_1000", |bencher| {
        let documents: Vec<(usize, Vec<u32>)> = (0..1000)
            .map(|i| {
                let len = 50 + (i % 500);
                let tokens: Vec<u32> = (0..len).map(|j| (i * len + j) as u32).collect();
                (len, tokens)
            })
            .collect();
        
        let context_window = 4096usize;
        
        bencher.iter(|| {
            let mut packed_contexts: Vec<Vec<&Vec<u32>>> = Vec::new();
            let mut current_context: Vec<&Vec<u32>> = Vec::new();
            let mut current_len = 0usize;
            
            for (len, tokens) in &documents {
                if current_len + len > context_window {
                    packed_contexts.push(std::mem::take(&mut current_context));
                    current_len = 0;
                }
                
                current_context.push(tokens);
                current_len += len;
            }
            
            if !current_context.is_empty() {
                packed_contexts.push(current_context);
            }
            
            // Calculate packing efficiency
            let total_tokens: usize = documents.iter().map(|(l, _)| l).sum();
            let total_capacity = packed_contexts.len() * context_window;
            let efficiency = total_tokens as f64 / total_capacity as f64;
            
            std_black_box(efficiency)
        });
    });
    
    // Test 4: Token budget allocation
    group.bench_function("budget_allocation_500", |bencher| {
        let requests: Vec<(u64, usize, Priority)> = (0..500)
            .map(|i| {
                let id = i as u64;
                let requested_tokens = 100 + (i % 1000);
                let priority = match i % 4 {
                    0 => Priority::Low,
                    1 => Priority::Normal,
                    2 => Priority::High,
                    _ => Priority::Critical,
                };
                (id, requested_tokens, priority)
            })
            .collect();
        
        let total_budget = 100000usize;
        
        bencher.iter(|| {
            // Priority-weighted allocation
            let total_requested: usize = requests.iter().map(|(_, t, _)| t).sum();
            
            let allocations: Vec<(u64, usize)> = if total_requested <= total_budget {
                requests.iter().map(|(id, tokens, _)| (*id, *tokens)).collect()
            } else {
                let mut sorted = requests.clone();
                sorted.sort_by(|a, b| b.2.cmp(&a.2));
                
                let mut remaining = total_budget;
                let mut result = Vec::new();
                
                for (id, requested, _) in sorted {
                    let allocated = requested.min(remaining);
                    result.push((id, allocated));
                    remaining -= allocated;
                }
                
                result
            };
            
            let total_allocated: usize = allocations.iter().map(|(_, a)| a).sum();
            std_black_box(total_allocated)
        });
    });
    
    group.finish();
}

// ============================================================================
// CATEGORY 6: MODEL OPTIMIZATION
// ============================================================================

fn bench_model_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_adversarial/model_optimization");
    
    // Test 1: Weight pruning simulation
    group.bench_function("weight_pruning_10000", |bencher| {
        let weights: Vec<f32> = (0..10000)
            .map(|i| ((i as f32 * 0.001 - 5.0).sin() * 0.5))
            .collect();
        
        let pruning_threshold = 0.1f32;
        
        bencher.iter(|| {
            let pruned: Vec<f32> = weights
                .iter()
                .map(|&w| if w.abs() < pruning_threshold { 0.0 } else { w })
                .collect();
            
            let zero_count = pruned.iter().filter(|&&w| w == 0.0).count();
            let sparsity = zero_count as f64 / pruned.len() as f64;
            
            std_black_box(sparsity)
        });
    });
    
    // Test 2: Knowledge distillation data prep
    group.bench_function("distillation_prep_1000", |bencher| {
        // Teacher model outputs
        let teacher_outputs: Vec<Vec<f32>> = (0..1000)
            .map(|i| {
                (0..50000)
                    .map(|j| ((i + j) as f32 * 0.0001).exp())
                    .collect()
            })
            .collect();
        
        let temperature = 2.0f32;
        
        bencher.iter(|| {
            // Compute soft targets
            let soft_targets: Vec<Vec<f32>> = teacher_outputs
                .iter()
                .map(|logits| {
                    let scaled: Vec<f32> = logits.iter().map(|&l| l / temperature).collect();
                    let max_val = scaled.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                    let exp_sum: f32 = scaled.iter().map(|&s| (s - max_val).exp()).sum();
                    
                    scaled.iter().map(|&s| (s - max_val).exp() / exp_sum).collect()
                })
                .collect();
            
            std_black_box(soft_targets.len())
        });
    });
    
    // Test 3: Activation checkpointing simulation
    group.bench_function("activation_checkpointing_500", |bencher| {
        let layer_count = 24;
        let checkpoint_interval = 4;
        
        let activations: Vec<Vec<f32>> = (0..500)
            .map(|i| {
                (0..1024)
                    .map(|j| (i + j) as f32 * 0.001)
                    .collect()
            })
            .collect();
        
        bencher.iter(|| {
            let mut checkpointed: Vec<Vec<f32>> = Vec::new();
            let mut recompute_count = 0;
            
            for layer in 0..layer_count {
                if layer % checkpoint_interval == 0 {
                    // Save checkpoint
                    checkpointed.push(activations[layer % activations.len()].clone());
                } else {
                    // Would need recomputation
                    recompute_count += 1;
                }
            }
            
            let memory_saved = (layer_count - checkpointed.len()) as f64 / layer_count as f64;
            std_black_box((memory_saved, recompute_count))
        });
    });
    
    // Test 4: Gradient accumulation
    group.bench_function("gradient_accumulation_2000", |bencher| {
        let gradients: Vec<Vec<f32>> = (0..2000)
            .map(|i| {
                (0..1000)
                    .map(|j| ((i + j) as f32 * 0.0001 - 0.5).sin())
                    .collect()
            })
            .collect();
        
        let accumulation_steps = 8;
        
        bencher.iter(|| {
            let mut accumulated: Vec<f32> = vec![0.0; 1000];
            let mut update_count = 0;
            
            for (i, grad) in gradients.iter().enumerate() {
                // Accumulate
                for (acc, g) in accumulated.iter_mut().zip(grad.iter()) {
                    *acc += g;
                }
                
                if (i + 1) % accumulation_steps == 0 {
                    // Apply update (divide by steps)
                    for acc in accumulated.iter_mut() {
                        *acc /= accumulation_steps as f32;
                        *acc = 0.0; // Reset after "applying"
                    }
                    update_count += 1;
                }
            }
            
            std_black_box(update_count)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_batch_inference,
    bench_prompt_caching,
    bench_compression,
    bench_load_balancing,
    bench_token_optimization,
    bench_model_optimization,
);

criterion_main!(benches);
