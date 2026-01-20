//! Adversarial RAG Benchmarks
//!
//! Professional-grade stress testing for RAG pipeline:
//! retrieval, context injection, consolidation, forgetting, diversity.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-rag API)
// ============================================================================

#[derive(Clone, Debug)]
struct Memory {
    id: String,
    content: String,
    embedding: Vec<f32>,
    timestamp: u64,
    access_count: u32,
    importance: f32,
    category: MemoryCategory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum MemoryCategory {
    Dialogue,
    Event,
    Fact,
    Observation,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct RetrievalResult {
    memory: Memory,
    similarity: f32,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct RagConfig {
    max_retrieval_count: usize,
    min_similarity_score: f32,
    diversity_factor: f32,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            max_retrieval_count: 10,
            min_similarity_score: 0.3,
            diversity_factor: 0.2,
        }
    }
}

// Helper functions
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut mag_a = 0.0f32;
    let mut mag_b = 0.0f32;

    for i in 0..a.len() {
        dot += a[i] * b[i];
        mag_a += a[i] * a[i];
        mag_b += b[i] * b[i];
    }

    let denominator = (mag_a.sqrt() * mag_b.sqrt()).max(f32::EPSILON);
    dot / denominator
}

fn generate_mock_embedding(seed: usize, dim: usize) -> Vec<f32> {
    (0..dim)
        .map(|i| ((seed * 31337 + i) % 1000) as f32 / 1000.0 - 0.5)
        .collect()
}

// ============================================================================
// CATEGORY 1: RETRIEVAL STRESS
// ============================================================================

fn bench_retrieval_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("rag_adversarial/retrieval_stress");

    // Test 1: Empty store retrieval
    group.bench_function("empty_store_retrieval", |bencher| {
        let memories: Vec<Memory> = Vec::new();
        let query_embedding = generate_mock_embedding(42, 384);

        bencher.iter(|| {
            // Retrieve from empty store
            let results: Vec<RetrievalResult> = memories
                .iter()
                .map(|m| RetrievalResult {
                    similarity: cosine_similarity(&query_embedding, &m.embedding),
                    memory: m.clone(),
                })
                .filter(|r| r.similarity > 0.3)
                .collect();

            std_black_box(results.len())
        });
    });

    // Test 2: Large store retrieval
    for count in [100, 500, 1000] {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("store_retrieval", count),
            &count,
            |bencher, &count| {
                let memories: Vec<Memory> = (0..count)
                    .map(|i| Memory {
                        id: format!("mem_{}", i),
                        content: format!("Memory content {}", i),
                        embedding: generate_mock_embedding(i, 384),
                        timestamp: i as u64 * 1000,
                        access_count: 0,
                        importance: 0.5,
                        category: MemoryCategory::Event,
                    })
                    .collect();

                let query_embedding = generate_mock_embedding(500, 384);

                bencher.iter(|| {
                    let mut results: Vec<RetrievalResult> = memories
                        .iter()
                        .map(|m| RetrievalResult {
                            similarity: cosine_similarity(&query_embedding, &m.embedding),
                            memory: m.clone(),
                        })
                        .filter(|r| r.similarity > 0.3)
                        .collect();

                    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
                    results.truncate(10);

                    std_black_box(results.len())
                });
            },
        );
    }

    // Test 3: Zero similarity threshold (return all)
    group.bench_function("zero_threshold_100", |bencher| {
        let memories: Vec<Memory> = (0..100)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Content {}", i),
                embedding: generate_mock_embedding(i, 384),
                timestamp: i as u64 * 1000,
                access_count: 0,
                importance: 0.5,
                category: MemoryCategory::Fact,
            })
            .collect();

        let query_embedding = generate_mock_embedding(50, 384);

        bencher.iter(|| {
            // No filtering - return all
            let results: Vec<RetrievalResult> = memories
                .iter()
                .map(|m| RetrievalResult {
                    similarity: cosine_similarity(&query_embedding, &m.embedding),
                    memory: m.clone(),
                })
                .collect();

            std_black_box(results.len())
        });
    });

    // Test 4: High similarity threshold (return none)
    group.bench_function("high_threshold_100", |bencher| {
        let memories: Vec<Memory> = (0..100)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Content {}", i),
                embedding: generate_mock_embedding(i, 384),
                timestamp: i as u64 * 1000,
                access_count: 0,
                importance: 0.5,
                category: MemoryCategory::Observation,
            })
            .collect();

        let query_embedding = generate_mock_embedding(9999, 384); // Very different

        bencher.iter(|| {
            let results: Vec<RetrievalResult> = memories
                .iter()
                .map(|m| RetrievalResult {
                    similarity: cosine_similarity(&query_embedding, &m.embedding),
                    memory: m.clone(),
                })
                .filter(|r| r.similarity > 0.99) // Very high threshold
                .collect();

            std_black_box(results.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 2: CONTEXT INJECTION
// ============================================================================

fn bench_context_injection(c: &mut Criterion) {
    let mut group = c.benchmark_group("rag_adversarial/context_injection");

    // Test 1: Single memory injection
    group.bench_function("single_memory_injection", |bencher| {
        let memory = Memory {
            id: "mem_1".to_string(),
            content: "The player defeated the dragon.".to_string(),
            embedding: vec![],
            timestamp: 1000,
            access_count: 5,
            importance: 0.8,
            category: MemoryCategory::Event,
        };

        let template = "Relevant context:\n{memories}\n\nNow respond to: {query}";
        let query = "What happened with the dragon?";

        bencher.iter(|| {
            let injected = template
                .replace("{memories}", &memory.content)
                .replace("{query}", query);
            std_black_box(injected.len())
        });
    });

    // Test 2: Multiple memories injection
    group.bench_function("multiple_memories_10", |bencher| {
        let memories: Vec<Memory> = (0..10)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Memory {} about game event.", i),
                embedding: vec![],
                timestamp: i as u64 * 1000,
                access_count: i as u32,
                importance: (i as f32 * 0.1).min(1.0),
                category: MemoryCategory::Event,
            })
            .collect();

        let template = "Relevant context:\n{memories}\n\nQuery: {query}";
        let query = "What happened?";

        bencher.iter(|| {
            let memory_text: String = memories
                .iter()
                .enumerate()
                .map(|(i, m)| format!("{}. {}", i + 1, m.content))
                .collect::<Vec<_>>()
                .join("\n");

            let injected = template
                .replace("{memories}", &memory_text)
                .replace("{query}", query);

            std_black_box(injected.len())
        });
    });

    // Test 3: Token limit enforcement
    group.bench_function("token_limit_enforcement", |bencher| {
        let memories: Vec<Memory> = (0..50)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("This is a detailed memory {} with lots of content.", i),
                embedding: vec![],
                timestamp: i as u64 * 1000,
                access_count: 0,
                importance: 0.5,
                category: MemoryCategory::Dialogue,
            })
            .collect();

        let max_tokens = 500; // Approximate token limit

        bencher.iter(|| {
            let mut result = String::new();
            let mut total_len = 0;

            for mem in &memories {
                let mem_len = mem.content.len();
                if total_len + mem_len > max_tokens * 4 {
                    // Rough chars-to-tokens ratio
                    break;
                }
                result.push_str(&mem.content);
                result.push('\n');
                total_len += mem_len + 1;
            }

            std_black_box(result.len())
        });
    });

    // Test 4: Empty memories injection
    group.bench_function("empty_memories_injection", |bencher| {
        let memories: Vec<Memory> = Vec::new();
        let template = "Context:\n{memories}\n\nQuery: {query}";
        let query = "What happened?";

        bencher.iter(|| {
            let memory_text = if memories.is_empty() {
                "No relevant memories found.".to_string()
            } else {
                memories.iter().map(|m| &m.content).cloned().collect::<Vec<_>>().join("\n")
            };

            let injected = template
                .replace("{memories}", &memory_text)
                .replace("{query}", query);

            std_black_box(injected.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 3: MEMORY CONSOLIDATION
// ============================================================================

fn bench_memory_consolidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rag_adversarial/memory_consolidation");

    // Test 1: Similar memories consolidation
    group.bench_function("consolidate_similar_20", |bencher| {
        let memories: Vec<Memory> = (0..20)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("The player entered room {} and found treasure.", i % 5),
                embedding: generate_mock_embedding(i % 5, 384), // Some will be similar
                timestamp: i as u64 * 1000,
                access_count: (i % 3) as u32,
                importance: 0.5,
                category: MemoryCategory::Event,
            })
            .collect();

        bencher.iter(|| {
            // Group by similarity
            let mut groups: Vec<Vec<&Memory>> = Vec::new();

            for mem in &memories {
                let mut found_group = false;
                for group in &mut groups {
                    if let Some(representative) = group.first() {
                        let sim = cosine_similarity(&mem.embedding, &representative.embedding);
                        if sim > 0.9 {
                            group.push(mem);
                            found_group = true;
                            break;
                        }
                    }
                }
                if !found_group {
                    groups.push(vec![mem]);
                }
            }

            // Count consolidated groups
            let consolidated_count = groups.len();
            std_black_box(consolidated_count)
        });
    });

    // Test 2: Consolidation priority calculation
    group.bench_function("consolidation_priority_100", |bencher| {
        let memories: Vec<Memory> = (0..100)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Memory {}", i),
                embedding: vec![],
                timestamp: i as u64 * 1000,
                access_count: (i * 2) as u32,
                importance: (i as f32 / 100.0),
                category: MemoryCategory::Fact,
            })
            .collect();

        bencher.iter(|| {
            let mut priorities: Vec<(String, f32)> = memories
                .iter()
                .map(|m| {
                    let recency = 1.0 / (1.0 + (100000 - m.timestamp) as f32 / 10000.0);
                    let priority = m.importance * 0.5 + m.access_count as f32 * 0.3 + recency * 0.2;
                    (m.id.clone(), priority)
                })
                .collect();

            priorities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            std_black_box(priorities.first().cloned())
        });
    });

    // Test 3: No consolidation needed
    group.bench_function("no_consolidation_unique_50", |bencher| {
        // All unique embeddings
        let memories: Vec<Memory> = (0..50)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Unique memory {}", i),
                embedding: generate_mock_embedding(i * 1000, 384), // Very different
                timestamp: i as u64 * 1000,
                access_count: 0,
                importance: 0.5,
                category: MemoryCategory::Observation,
            })
            .collect();

        bencher.iter(|| {
            // Try to find consolidation candidates (should find none)
            let mut candidates = 0;
            for i in 0..memories.len() {
                for j in (i + 1)..memories.len() {
                    let sim = cosine_similarity(&memories[i].embedding, &memories[j].embedding);
                    if sim > 0.95 {
                        candidates += 1;
                    }
                }
            }
            std_black_box(candidates)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 4: FORGETTING MECHANISMS
// ============================================================================

fn bench_forgetting_mechanisms(c: &mut Criterion) {
    let mut group = c.benchmark_group("rag_adversarial/forgetting_mechanisms");

    // Test 1: Importance-based forgetting
    group.bench_function("importance_forgetting_100", |bencher| {
        let memories: Vec<Memory> = (0..100)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Memory {}", i),
                embedding: vec![],
                timestamp: i as u64 * 1000,
                access_count: (i % 10) as u32,
                importance: (i as f32 / 100.0),
                category: MemoryCategory::Event,
            })
            .collect();

        let importance_threshold = 0.3;

        bencher.iter(|| {
            let retained: Vec<_> = memories
                .iter()
                .filter(|m| m.importance >= importance_threshold)
                .collect();

            let forgotten = memories.len() - retained.len();
            std_black_box((retained.len(), forgotten))
        });
    });

    // Test 2: Time-based decay
    group.bench_function("time_decay_100", |bencher| {
        let current_time = 100000u64;
        let decay_rate = 0.0001f32;

        let memories: Vec<Memory> = (0..100)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Memory {}", i),
                embedding: vec![],
                timestamp: i as u64 * 1000,
                access_count: 0,
                importance: 0.8,
                category: MemoryCategory::Dialogue,
            })
            .collect();

        bencher.iter(|| {
            let decayed: Vec<(String, f32)> = memories
                .iter()
                .map(|m| {
                    let age = current_time - m.timestamp;
                    let decayed_importance = m.importance * (-decay_rate * age as f32).exp();
                    (m.id.clone(), decayed_importance)
                })
                .collect();

            let should_forget: Vec<_> = decayed.iter().filter(|(_, imp)| *imp < 0.1).collect();
            std_black_box(should_forget.len())
        });
    });

    // Test 3: Access count threshold
    group.bench_function("access_count_forgetting", |bencher| {
        let memories: Vec<Memory> = (0..100)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Memory {}", i),
                embedding: vec![],
                timestamp: i as u64 * 1000,
                access_count: (i % 20) as u32,
                importance: 0.5,
                category: MemoryCategory::Fact,
            })
            .collect();

        let min_access_count = 5u32;

        bencher.iter(|| {
            let retained: Vec<_> = memories
                .iter()
                .filter(|m| m.access_count >= min_access_count)
                .collect();

            std_black_box(retained.len())
        });
    });

    // Test 4: Combined forgetting policy
    group.bench_function("combined_forgetting_policy", |bencher| {
        let current_time = 100000u64;
        let memories: Vec<Memory> = (0..100)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Memory {}", i),
                embedding: vec![],
                timestamp: i as u64 * 1000,
                access_count: (i % 15) as u32,
                importance: (i as f32 / 100.0),
                category: MemoryCategory::Event,
            })
            .collect();

        bencher.iter(|| {
            let retained: Vec<_> = memories
                .iter()
                .filter(|m| {
                    let age = current_time - m.timestamp;
                    let recency_score = 1.0 / (1.0 + age as f32 / 10000.0);

                    // Combined score
                    let retention_score = m.importance * 0.4
                        + m.access_count as f32 / 15.0 * 0.3
                        + recency_score * 0.3;

                    retention_score > 0.3
                })
                .collect();

            std_black_box(retained.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 5: DIVERSITY SAMPLING
// ============================================================================

fn bench_diversity_sampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("rag_adversarial/diversity_sampling");

    // Test 1: Category diversity
    group.bench_function("category_diversity_100", |bencher| {
        let memories: Vec<Memory> = (0..100)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Memory {}", i),
                embedding: generate_mock_embedding(i, 384),
                timestamp: i as u64 * 1000,
                access_count: 0,
                importance: 0.5,
                category: match i % 4 {
                    0 => MemoryCategory::Dialogue,
                    1 => MemoryCategory::Event,
                    2 => MemoryCategory::Fact,
                    _ => MemoryCategory::Observation,
                },
            })
            .collect();

        let target_per_category = 3;

        bencher.iter(|| {
            let mut by_category: HashMap<MemoryCategory, Vec<&Memory>> = HashMap::new();

            for mem in &memories {
                by_category.entry(mem.category).or_default().push(mem);
            }

            // Take top N from each category
            let diverse: Vec<&Memory> = by_category
                .values()
                .flat_map(|mems| mems.iter().take(target_per_category).copied())
                .collect();

            std_black_box(diverse.len())
        });
    });

    // Test 2: Maximal marginal relevance (MMR)
    group.bench_function("mmr_sampling_50", |bencher| {
        let memories: Vec<Memory> = (0..50)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Memory {}", i),
                embedding: generate_mock_embedding(i, 128), // Smaller for speed
                timestamp: i as u64 * 1000,
                access_count: 0,
                importance: 0.5,
                category: MemoryCategory::Event,
            })
            .collect();

        let query_embedding = generate_mock_embedding(25, 128);
        let lambda = 0.5f32; // Balance relevance and diversity
        let k = 5;

        bencher.iter(|| {
            let mut selected: Vec<usize> = Vec::new();
            let mut remaining: Vec<usize> = (0..memories.len()).collect();

            while selected.len() < k && !remaining.is_empty() {
                let mut best_idx = 0;
                let mut best_score = f32::NEG_INFINITY;

                for (i, &mem_idx) in remaining.iter().enumerate() {
                    let relevance = cosine_similarity(&query_embedding, &memories[mem_idx].embedding);

                    let max_sim_to_selected = selected
                        .iter()
                        .map(|&s| cosine_similarity(&memories[mem_idx].embedding, &memories[s].embedding))
                        .fold(0.0f32, |a, b| a.max(b));

                    let mmr_score = lambda * relevance - (1.0 - lambda) * max_sim_to_selected;

                    if mmr_score > best_score {
                        best_score = mmr_score;
                        best_idx = i;
                    }
                }

                let chosen = remaining.remove(best_idx);
                selected.push(chosen);
            }

            std_black_box(selected.len())
        });
    });

    // Test 3: Temporal diversity
    group.bench_function("temporal_diversity_100", |bencher| {
        let memories: Vec<Memory> = (0..100)
            .map(|i| Memory {
                id: format!("mem_{}", i),
                content: format!("Memory {}", i),
                embedding: vec![],
                timestamp: (i / 10) as u64 * 10000, // Group into time buckets
                access_count: 0,
                importance: 0.5,
                category: MemoryCategory::Event,
            })
            .collect();

        let time_buckets = 5;

        bencher.iter(|| {
            let mut by_time: HashMap<u64, Vec<&Memory>> = HashMap::new();

            for mem in &memories {
                let bucket = mem.timestamp / 20000; // Bucket by time period
                by_time.entry(bucket).or_default().push(mem);
            }

            // Take from different time periods
            let diverse: Vec<&Memory> = by_time
                .values()
                .take(time_buckets)
                .flat_map(|mems| mems.first().copied())
                .collect();

            std_black_box(diverse.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 6: QUERY PROCESSING
// ============================================================================

fn bench_query_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("rag_adversarial/query_processing");

    // Test 1: Empty query
    group.bench_function("empty_query_handling", |bencher| {
        let query = String::new();

        bencher.iter(|| {
            let processed = if query.is_empty() {
                None
            } else {
                Some(query.to_lowercase())
            };
            std_black_box(processed.is_none())
        });
    });

    // Test 2: Very long query
    group.bench_function("long_query_5k_chars", |bencher| {
        let query = "word ".repeat(1000);

        bencher.iter(|| {
            // Truncate and process
            let max_len = 500;
            let truncated = if query.len() > max_len {
                &query[..max_len]
            } else {
                &query
            };

            let processed = truncated.to_lowercase();
            std_black_box(processed.len())
        });
    });

    // Test 3: Query expansion
    group.bench_function("query_expansion", |bencher| {
        let query = "dragon fight combat";
        let synonyms: HashMap<&str, Vec<&str>> = [
            ("fight", vec!["battle", "combat", "clash"]),
            ("dragon", vec!["beast", "creature", "monster"]),
            ("combat", vec!["fight", "battle", "warfare"]),
        ]
        .into_iter()
        .collect();

        bencher.iter(|| {
            let words: Vec<&str> = query.split_whitespace().collect();
            let mut expanded: Vec<&str> = words.clone();

            for word in &words {
                if let Some(syns) = synonyms.get(word) {
                    expanded.extend(syns.iter());
                }
            }

            std_black_box(expanded.len())
        });
    });

    // Test 4: Query normalization
    group.bench_function("query_normalization", |bencher| {
        let query = "  What   happened   to the    DRAGON?!  ";

        bencher.iter(|| {
            let normalized: String = query
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect();

            std_black_box(normalized.len())
        });
    });

    // Test 5: Multi-query batching
    group.bench_function("multi_query_batch_10", |bencher| {
        let queries = [
            "dragon battle",
            "treasure found",
            "player died",
            "level complete",
            "boss defeated",
            "item crafted",
            "quest accepted",
            "npc dialogue",
            "map explored",
            "achievement unlocked",
        ];

        bencher.iter(|| {
            let processed: Vec<String> = queries
                .iter()
                .map(|q| q.to_lowercase())
                .collect();

            std_black_box(processed.len())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_retrieval_stress,
    bench_context_injection,
    bench_memory_consolidation,
    bench_forgetting_mechanisms,
    bench_diversity_sampling,
    bench_query_processing,
);

criterion_main!(benches);
