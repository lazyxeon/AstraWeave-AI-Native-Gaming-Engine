//! aw-save Benchmarks
//!
//! Measures performance of save/load system:
//! - Serialization (postcard encoding/decoding)
//! - Compression (LZ4 compress/decompress)
//! - File I/O (atomic writes, fsync)
//! - Full save/load cycle
//! - Index operations
//! - Migration (V1 → V2 schema)
//!
//! Performance targets:
//! - Serialization: <10 ms for 1MB save (100 MB/s throughput)
//! - Compression: <20 ms for 1MB data (50 MB/s throughput)
//! - Full save cycle: <100 ms (sub-frame @ 60 FPS for background saves)
//! - Full load cycle: <50 ms (fast enough for level transitions)
//! - Migration: <150 ms for old saves

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;
use tempfile::TempDir;
use time::OffsetDateTime;
use uuid::Uuid;

use aw_save::{
    CompanionProfile, ItemStack, PlayerInventory, SaveBundleV2, SaveManager, WorldState,
    SAVE_SCHEMA_VERSION,
};

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_bundle(ecs_blob_size: usize) -> SaveBundleV2 {
    SaveBundleV2 {
        schema: SAVE_SCHEMA_VERSION,
        save_id: Uuid::new_v4(),
        created_at: OffsetDateTime::now_utc(),
        player_id: "test_player_12345".to_string(),
        slot: 0,
        world: WorldState {
            tick: 12345,
            ecs_blob: vec![0x42; ecs_blob_size],
            state_hash: 0xDEADBEEF,
        },
        companions: vec![
            CompanionProfile {
                id: "companion_1".to_string(),
                name: "Alex".to_string(),
                level: 10,
                skills: vec!["combat".to_string(), "stealth".to_string()],
                facts: vec!["likes coffee".to_string()],
                episodes_summarized: vec!["Episode 1 summary".to_string()],
            },
            CompanionProfile {
                id: "companion_2".to_string(),
                name: "Beth".to_string(),
                level: 8,
                skills: vec!["healing".to_string()],
                facts: vec!["afraid of heights".to_string()],
                episodes_summarized: vec![],
            },
        ],
        inventory: PlayerInventory {
            credits: 10000,
            items: vec![
                ItemStack {
                    kind: "health_potion".to_string(),
                    qty: 5,
                    attrs: {
                        let mut map = HashMap::new();
                        map.insert("rarity".to_string(), 1);
                        map
                    },
                },
                ItemStack {
                    kind: "iron_sword".to_string(),
                    qty: 1,
                    attrs: {
                        let mut map = HashMap::new();
                        map.insert("damage".to_string(), 25);
                        map.insert("durability".to_string(), 100);
                        map
                    },
                },
            ],
        },
        meta: {
            let mut map = HashMap::new();
            map.insert("difficulty".to_string(), "normal".to_string());
            map.insert("playtime_seconds".to_string(), "3600".to_string());
            map
        },
    }
}

// ============================================================================
// Benchmark 1: Serialization (postcard)
// ============================================================================

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    // Benchmark: Small save (10 KB ECS blob)
    group.bench_function("serialize_small_10kb", |b| {
        let bundle = create_test_bundle(10 * 1024);

        b.iter(|| {
            let bytes = postcard::to_allocvec(&bundle).unwrap();
            std_black_box(bytes)
        })
    });

    // Benchmark: Medium save (100 KB ECS blob)
    group.bench_function("serialize_medium_100kb", |b| {
        let bundle = create_test_bundle(100 * 1024);

        b.iter(|| {
            let bytes = postcard::to_allocvec(&bundle).unwrap();
            std_black_box(bytes)
        })
    });

    // Benchmark: Large save (1 MB ECS blob)
    group.bench_function("serialize_large_1mb", |b| {
        let bundle = create_test_bundle(1024 * 1024);

        b.iter(|| {
            let bytes = postcard::to_allocvec(&bundle).unwrap();
            std_black_box(bytes)
        })
    });

    // Benchmark: Deserialization (small)
    group.bench_function("deserialize_small_10kb", |b| {
        let bundle = create_test_bundle(10 * 1024);
        let bytes = postcard::to_allocvec(&bundle).unwrap();

        b.iter(|| {
            let decoded: SaveBundleV2 = postcard::from_bytes(&bytes).unwrap();
            std_black_box(decoded)
        })
    });

    // Benchmark: Deserialization (large)
    group.bench_function("deserialize_large_1mb", |b| {
        let bundle = create_test_bundle(1024 * 1024);
        let bytes = postcard::to_allocvec(&bundle).unwrap();

        b.iter(|| {
            let decoded: SaveBundleV2 = postcard::from_bytes(&bytes).unwrap();
            std_black_box(decoded)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 2: Compression (LZ4)
// ============================================================================

fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    // Benchmark: LZ4 compress (10 KB)
    group.throughput(Throughput::Bytes(10 * 1024));
    group.bench_function("lz4_compress_10kb", |b| {
        let bundle = create_test_bundle(10 * 1024);
        let bytes = postcard::to_allocvec(&bundle).unwrap();

        b.iter(|| {
            let compressed = lz4_flex::compress_prepend_size(&bytes);
            std_black_box(compressed)
        })
    });

    // Benchmark: LZ4 compress (100 KB)
    group.throughput(Throughput::Bytes(100 * 1024));
    group.bench_function("lz4_compress_100kb", |b| {
        let bundle = create_test_bundle(100 * 1024);
        let bytes = postcard::to_allocvec(&bundle).unwrap();

        b.iter(|| {
            let compressed = lz4_flex::compress_prepend_size(&bytes);
            std_black_box(compressed)
        })
    });

    // Benchmark: LZ4 compress (1 MB)
    group.throughput(Throughput::Bytes(1024 * 1024));
    group.bench_function("lz4_compress_1mb", |b| {
        let bundle = create_test_bundle(1024 * 1024);
        let bytes = postcard::to_allocvec(&bundle).unwrap();

        b.iter(|| {
            let compressed = lz4_flex::compress_prepend_size(&bytes);
            std_black_box(compressed)
        })
    });

    // Benchmark: LZ4 decompress (10 KB)
    group.throughput(Throughput::Bytes(10 * 1024));
    group.bench_function("lz4_decompress_10kb", |b| {
        let bundle = create_test_bundle(10 * 1024);
        let bytes = postcard::to_allocvec(&bundle).unwrap();
        let compressed = lz4_flex::compress_prepend_size(&bytes);

        b.iter(|| {
            let decompressed = lz4_flex::decompress_size_prepended(&compressed).unwrap();
            std_black_box(decompressed)
        })
    });

    // Benchmark: LZ4 decompress (1 MB)
    group.throughput(Throughput::Bytes(1024 * 1024));
    group.bench_function("lz4_decompress_1mb", |b| {
        let bundle = create_test_bundle(1024 * 1024);
        let bytes = postcard::to_allocvec(&bundle).unwrap();
        let compressed = lz4_flex::compress_prepend_size(&bytes);

        b.iter(|| {
            let decompressed = lz4_flex::decompress_size_prepended(&compressed).unwrap();
            std_black_box(decompressed)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 3: Checksum (CRC32)
// ============================================================================

fn bench_checksum(c: &mut Criterion) {
    let mut group = c.benchmark_group("checksum");

    // Benchmark: CRC32 (10 KB)
    group.throughput(Throughput::Bytes(10 * 1024));
    group.bench_function("crc32_10kb", |b| {
        let data = vec![0x42; 10 * 1024];

        b.iter(|| {
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&data);
            let crc = hasher.finalize();
            std_black_box(crc)
        })
    });

    // Benchmark: CRC32 (100 KB)
    group.throughput(Throughput::Bytes(100 * 1024));
    group.bench_function("crc32_100kb", |b| {
        let data = vec![0x42; 100 * 1024];

        b.iter(|| {
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&data);
            let crc = hasher.finalize();
            std_black_box(crc)
        })
    });

    // Benchmark: CRC32 (1 MB)
    group.throughput(Throughput::Bytes(1024 * 1024));
    group.bench_function("crc32_1mb", |b| {
        let data = vec![0x42; 1024 * 1024];

        b.iter(|| {
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&data);
            let crc = hasher.finalize();
            std_black_box(crc)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 4: Full Save/Load Cycle
// ============================================================================

fn bench_save_load_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_load_cycle");

    // Benchmark: Full save (small 10 KB)
    group.bench_function("full_save_small_10kb", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SaveManager::new(temp_dir.path());
        let bundle = create_test_bundle(10 * 1024);

        b.iter(|| {
            let path = mgr.save("player1", 0, bundle.clone()).unwrap();
            std_black_box(path)
        })
    });

    // Benchmark: Full save (medium 100 KB)
    group.bench_function("full_save_medium_100kb", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SaveManager::new(temp_dir.path());
        let bundle = create_test_bundle(100 * 1024);

        b.iter(|| {
            let path = mgr.save("player1", 0, bundle.clone()).unwrap();
            std_black_box(path)
        })
    });

    // Benchmark: Full save (large 1 MB)
    group.bench_function("full_save_large_1mb", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SaveManager::new(temp_dir.path());
        let bundle = create_test_bundle(1024 * 1024);

        b.iter(|| {
            let path = mgr.save("player1", 0, bundle.clone()).unwrap();
            std_black_box(path)
        })
    });

    // Benchmark: Full load (small 10 KB)
    group.bench_function("full_load_small_10kb", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SaveManager::new(temp_dir.path());
        let bundle = create_test_bundle(10 * 1024);
        mgr.save("player1", 0, bundle.clone()).unwrap();

        b.iter(|| {
            let (loaded, _path) = mgr.load_latest_slot("player1", 0).unwrap();
            std_black_box(loaded)
        })
    });

    // Benchmark: Full load (large 1 MB)
    group.bench_function("full_load_large_1mb", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SaveManager::new(temp_dir.path());
        let bundle = create_test_bundle(1024 * 1024);
        mgr.save("player1", 0, bundle.clone()).unwrap();

        b.iter(|| {
            let (loaded, _path) = mgr.load_latest_slot("player1", 0).unwrap();
            std_black_box(loaded)
        })
    });

    // Benchmark: Round-trip (save + load)
    group.bench_function("round_trip_100kb", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SaveManager::new(temp_dir.path());
        let bundle = create_test_bundle(100 * 1024);

        b.iter(|| {
            let _save_path = mgr.save("player1", 0, bundle.clone()).unwrap();
            let (loaded, _load_path) = mgr.load_latest_slot("player1", 0).unwrap();
            std_black_box(loaded)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 5: Index Operations
// ============================================================================

fn bench_index_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("index_operations");

    // Benchmark: List saves (empty)
    group.bench_function("list_saves_empty", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SaveManager::new(temp_dir.path());

        b.iter(|| {
            let list = mgr.list_saves("player1").unwrap_or_default();
            std_black_box(list)
        })
    });

    // Benchmark: List saves (10 saves)
    group.bench_function("list_saves_10_saves", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SaveManager::new(temp_dir.path());
        
        // Create 10 saves
        for i in 0..10 {
            let bundle = create_test_bundle(10 * 1024);
            mgr.save("player1", i as u8, bundle).unwrap();
        }

        b.iter(|| {
            let list = mgr.list_saves("player1").unwrap();
            std_black_box(list)
        })
    });

    // Benchmark: List saves (100 saves - stress test)
    group.bench_function("list_saves_100_saves", |b| {
        let temp_dir = TempDir::new().unwrap();
        let mgr = SaveManager::new(temp_dir.path());
        
        // Create 100 saves across 10 slots
        for i in 0..100 {
            let bundle = create_test_bundle(1024); // Small to speed up setup
            mgr.save("player1", (i % 10) as u8, bundle).unwrap();
        }

        b.iter(|| {
            let list = mgr.list_saves("player1").unwrap();
            std_black_box(list)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 6: Scaling with Save Size
// ============================================================================

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    for size_kb in [1, 10, 100, 500, 1000, 5000].iter() {
        let size_bytes = size_kb * 1024;
        
        group.throughput(Throughput::Bytes(size_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("full_save_cycle", format!("{}kb", size_kb)),
            size_kb,
            |b, &kb| {
                let temp_dir = TempDir::new().unwrap();
                let mgr = SaveManager::new(temp_dir.path());
                let bundle = create_test_bundle(kb * 1024);

                b.iter(|| {
                    let path = mgr.save("player1", 0, bundle.clone()).unwrap();
                    std_black_box(path)
                })
            },
        );

        group.throughput(Throughput::Bytes(size_bytes as u64));
        group.bench_with_input(
            BenchmarkId::new("full_load_cycle", format!("{}kb", size_kb)),
            size_kb,
            |b, &kb| {
                let temp_dir = TempDir::new().unwrap();
                let mgr = SaveManager::new(temp_dir.path());
                let bundle = create_test_bundle(kb * 1024);
                mgr.save("player1", 0, bundle.clone()).unwrap();

                b.iter(|| {
                    let (loaded, _path) = mgr.load_latest_slot("player1", 0).unwrap();
                    std_black_box(loaded)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_serialization,
    bench_compression,
    bench_checksum,
    bench_save_load_cycle,
    bench_index_operations,
    bench_scaling,
);
criterion_main!(benches);
