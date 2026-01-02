//! Benchmarks for astraweave-blend caching operations.
//!
//! Tests cache entry creation, lookup, eviction, and persistence.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;
use sha2::{Digest, Sha256};

use astraweave_blend::cache::{CacheEntry, ConversionCache, CacheLookup, CacheMissReason, CacheManifest};
use astraweave_blend::version::BlenderVersion;
use astraweave_blend::options::CacheOptions;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Compute SHA-256 hash (same as ConversionCache::hash_file uses internally)
fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn create_test_entry(id: usize) -> CacheEntry {
    CacheEntry::new(
        compute_hash(format!("source_{}", id).as_bytes()),
        compute_hash(format!("options_{}", id).as_bytes()),
        BlenderVersion::new(4, 1, 0),
        PathBuf::from(format!("/cache/output_{}.glb", id)),
        PathBuf::from(format!("/models/model_{}.blend", id)),
        1024 * 1024,  // 1 MB output
        500,  // 500ms conversion
    )
}

// ============================================================================
// CACHE ENTRY BENCHMARKS
// ============================================================================

fn bench_cache_entry_creation(c: &mut Criterion) {
    c.bench_function("cache_entry_creation", |b| {
        let mut id = 0;
        b.iter(|| {
            id += 1;
            black_box(create_test_entry(black_box(id)))
        })
    });
}

fn bench_cache_entry_clone(c: &mut Criterion) {
    let entry = create_test_entry(0);
    
    c.bench_function("cache_entry_clone", |b| {
        b.iter(|| black_box(black_box(&entry).clone()))
    });
}

fn bench_cache_entry_touch(c: &mut Criterion) {
    let mut entry = create_test_entry(0);
    
    c.bench_function("cache_entry_touch", |b| {
        b.iter(|| {
            black_box(&mut entry).touch();
        })
    });
}

fn bench_cache_entry_age(c: &mut Criterion) {
    let entry = create_test_entry(0);
    
    c.bench_function("cache_entry_age", |b| {
        b.iter(|| black_box(black_box(&entry).age()))
    });
}

fn bench_cache_entry_time_since_access(c: &mut Criterion) {
    let entry = create_test_entry(0);
    
    c.bench_function("cache_entry_time_since_access", |b| {
        b.iter(|| black_box(black_box(&entry).time_since_access()))
    });
}

// ============================================================================
// CACHE OPTIONS BENCHMARKS
// ============================================================================

fn bench_cache_options_default(c: &mut Criterion) {
    c.bench_function("cache_options_default", |b| {
        b.iter(|| black_box(CacheOptions::default()))
    });
}

fn bench_cache_options_disabled(c: &mut Criterion) {
    c.bench_function("cache_options_disabled", |b| {
        b.iter(|| {
            black_box(CacheOptions {
                enabled: false,
                ..Default::default()
            })
        })
    });
}

fn bench_cache_options_full(c: &mut Criterion) {
    c.bench_function("cache_options_full", |b| {
        b.iter(|| {
            black_box(CacheOptions {
                enabled: true,
                cache_directory: Some(PathBuf::from("/cache")),
                max_cache_size: Some(10 * 1024 * 1024 * 1024),  // 10 GB
                max_age: Some(Duration::from_secs(86400 * 7)),  // 1 week
                validate_on_access: true,
            })
        })
    });
}

// ============================================================================
// CACHE MANIFEST BENCHMARKS
// ============================================================================

fn bench_cache_manifest_new(c: &mut Criterion) {
    c.bench_function("cache_manifest_new", |b| {
        b.iter(|| black_box(CacheManifest::new()))
    });
}

fn bench_cache_manifest_recalculate_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_manifest_recalculate_size");
    
    for count in [10, 100, 1000].iter() {
        let mut manifest = CacheManifest::new();
        for i in 0..*count {
            manifest.entries.insert(
                format!("source_{}.blend", i),
                create_test_entry(i),
            );
        }
        
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &manifest,
            |b, manifest| {
                let mut m = manifest.clone();
                b.iter(|| {
                    black_box(&mut m).recalculate_size();
                })
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// CACHE LOOKUP RESULT BENCHMARKS
// ============================================================================

fn bench_cache_lookup_hit_creation(c: &mut Criterion) {
    let entry = create_test_entry(0);
    let output_path = PathBuf::from("/cache/output.glb");
    
    c.bench_function("cache_lookup_hit_creation", |b| {
        b.iter(|| {
            black_box(CacheLookup::Hit {
                output_path: black_box(&output_path).clone(),
                entry: black_box(&entry).clone(),
            })
        })
    });
}

fn bench_cache_lookup_miss_creation(c: &mut Criterion) {
    c.bench_function("cache_lookup_miss_creation", |b| {
        b.iter(|| {
            black_box(CacheLookup::Miss {
                reason: CacheMissReason::NotCached,
            })
        })
    });
}

fn bench_cache_miss_reason_display(c: &mut Criterion) {
    let reasons = vec![
        CacheMissReason::NotCached,
        CacheMissReason::SourceModified,
        CacheMissReason::OptionsChanged,
        CacheMissReason::BlenderVersionChanged,
        CacheMissReason::OutputMissing,
        CacheMissReason::Expired,
        CacheMissReason::ValidationFailed("test".to_string()),
    ];
    
    c.bench_function("cache_miss_reason_display", |b| {
        b.iter(|| {
            for reason in black_box(&reasons) {
                black_box(format!("{}", reason));
            }
        })
    });
}

// ============================================================================
// CACHE KEY/HASH BENCHMARKS
// ============================================================================

fn bench_cache_key_generation(c: &mut Criterion) {
    let source_hash = compute_hash(b"source content");
    let options_hash = compute_hash(b"options content");
    
    c.bench_function("cache_key_generation", |b| {
        b.iter(|| {
            // Simulate key generation
            let key = format!(
                "{}_{}", 
                black_box(&source_hash),
                black_box(&options_hash),
            );
            black_box(key)
        })
    });
}

fn bench_hash_for_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_for_cache");
    
    let sizes = vec![
        ("1kb", 1024),
        ("100kb", 100 * 1024),
        ("1mb", 1024 * 1024),
        ("10mb", 10 * 1024 * 1024),
    ];
    
    for (name, size) in sizes {
        let data = vec![0xABu8; size];
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &data,
            |b, data| b.iter(|| black_box(compute_hash(black_box(data)))),
        );
    }
    
    group.finish();
}

// ============================================================================
// CACHE INITIALIZATION BENCHMARKS
// ============================================================================

fn bench_conversion_cache_new(c: &mut Criterion) {
    c.bench_function("conversion_cache_new", |b| {
        b.iter(|| {
            let temp = tempdir().unwrap();
            let cache = ConversionCache::new(temp.path());
            black_box(cache)
        })
    });
}

fn bench_conversion_cache_for_project(c: &mut Criterion) {
    c.bench_function("conversion_cache_for_project", |b| {
        b.iter(|| {
            let temp = tempdir().unwrap();
            let cache = ConversionCache::for_project(temp.path());
            black_box(cache)
        })
    });
}

// ============================================================================
// CACHE ENTRY SERIALIZATION BENCHMARKS
// ============================================================================

fn bench_cache_entry_serialize_ron(c: &mut Criterion) {
    let entry = create_test_entry(0);
    
    c.bench_function("cache_entry_serialize_ron", |b| {
        b.iter(|| black_box(ron::to_string(black_box(&entry))))
    });
}

fn bench_cache_entry_deserialize_ron(c: &mut Criterion) {
    let entry = create_test_entry(0);
    let serialized = ron::to_string(&entry).unwrap();
    
    c.bench_function("cache_entry_deserialize_ron", |b| {
        b.iter(|| black_box(ron::from_str::<CacheEntry>(black_box(&serialized))))
    });
}

fn bench_cache_entry_serialize_json(c: &mut Criterion) {
    let entry = create_test_entry(0);
    
    c.bench_function("cache_entry_serialize_json", |b| {
        b.iter(|| black_box(serde_json::to_string(black_box(&entry))))
    });
}

fn bench_cache_entry_deserialize_json(c: &mut Criterion) {
    let entry = create_test_entry(0);
    let serialized = serde_json::to_string(&entry).unwrap();
    
    c.bench_function("cache_entry_deserialize_json", |b| {
        b.iter(|| black_box(serde_json::from_str::<CacheEntry>(black_box(&serialized))))
    });
}

// ============================================================================
// CACHE MANIFEST SERIALIZATION BENCHMARKS
// ============================================================================

fn bench_cache_manifest_serialize_ron(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_manifest_serialize_ron");
    
    for count in [10, 100, 1000].iter() {
        let mut manifest = CacheManifest::new();
        for i in 0..*count {
            manifest.entries.insert(
                format!("source_{}.blend", i),
                create_test_entry(i),
            );
        }
        manifest.recalculate_size();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &manifest,
            |b, manifest| b.iter(|| black_box(ron::to_string(black_box(manifest)))),
        );
    }
    
    group.finish();
}

fn bench_cache_manifest_deserialize_ron(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_manifest_deserialize_ron");
    
    for count in [10, 100, 1000].iter() {
        let mut manifest = CacheManifest::new();
        for i in 0..*count {
            manifest.entries.insert(
                format!("source_{}.blend", i),
                create_test_entry(i),
            );
        }
        manifest.recalculate_size();
        let serialized = ron::to_string(&manifest).unwrap();
        
        group.throughput(Throughput::Bytes(serialized.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &serialized,
            |b, serialized| b.iter(|| black_box(ron::from_str::<CacheManifest>(black_box(serialized)))),
        );
    }
    
    group.finish();
}

// ============================================================================
// SCALED BENCHMARKS
// ============================================================================

fn bench_many_cache_entries_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_cache_entries_creation");
    
    for count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.iter(|| {
                    let entries: Vec<_> = (0..count)
                        .map(|i| create_test_entry(i))
                        .collect();
                    black_box(entries)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_cache_hashmap_operations(c: &mut Criterion) {
    use std::collections::HashMap;
    
    let mut group = c.benchmark_group("cache_hashmap_operations");
    
    for count in [100, 1000, 10000].iter() {
        let entries: HashMap<String, CacheEntry> = (0..*count)
            .map(|i| (format!("source_{}.blend", i), create_test_entry(i)))
            .collect();
        
        // Benchmark lookup
        group.bench_with_input(
            BenchmarkId::new("lookup", count),
            &entries,
            |b, entries| {
                let mut i = 0;
                b.iter(|| {
                    i = (i + 1) % *count;
                    let key = format!("source_{}.blend", i);
                    black_box(entries.get(black_box(&key)))
                })
            },
        );
        
        // Benchmark insert
        group.bench_with_input(
            BenchmarkId::new("insert", count),
            &entries,
            |b, _| {
                let mut map: HashMap<String, CacheEntry> = HashMap::new();
                let mut i = 0;
                b.iter(|| {
                    i += 1;
                    let key = format!("source_{}.blend", i);
                    let entry = create_test_entry(i);
                    map.insert(black_box(key), black_box(entry));
                })
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// CRITERION GROUPS
// ============================================================================

criterion_group!(
    entry_benches,
    bench_cache_entry_creation,
    bench_cache_entry_clone,
    bench_cache_entry_touch,
    bench_cache_entry_age,
    bench_cache_entry_time_since_access,
);

criterion_group!(
    options_benches,
    bench_cache_options_default,
    bench_cache_options_disabled,
    bench_cache_options_full,
);

criterion_group!(
    manifest_benches,
    bench_cache_manifest_new,
    bench_cache_manifest_recalculate_size,
);

criterion_group!(
    lookup_benches,
    bench_cache_lookup_hit_creation,
    bench_cache_lookup_miss_creation,
    bench_cache_miss_reason_display,
    bench_cache_key_generation,
    bench_hash_for_cache,
);

criterion_group!(
    init_benches,
    bench_conversion_cache_new,
    bench_conversion_cache_for_project,
);

criterion_group!(
    serialization_benches,
    bench_cache_entry_serialize_ron,
    bench_cache_entry_deserialize_ron,
    bench_cache_entry_serialize_json,
    bench_cache_entry_deserialize_json,
);

criterion_group!(
    manifest_serialization_benches,
    bench_cache_manifest_serialize_ron,
    bench_cache_manifest_deserialize_ron,
);

criterion_group!(
    scaled_benches,
    bench_many_cache_entries_creation,
    bench_cache_hashmap_operations,
);

criterion_main!(
    entry_benches,
    options_benches,
    manifest_benches,
    lookup_benches,
    init_benches,
    serialization_benches,
    manifest_serialization_benches,
    scaled_benches,
);
