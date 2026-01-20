#![allow(dead_code)]

//! Adversarial Asset Loading Benchmarks
//!
//! Stress testing for asset loading, caching, hot reload, and validation.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-asset API)
// ============================================================================

#[derive(Clone, Debug)]
struct AssetHandle {
    id: u64,
    path: String,
    asset_type: AssetType,
    loaded: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum AssetType {
    Mesh,
    Texture,
    Audio,
    Material,
    Animation,
}

#[derive(Clone, Debug)]
struct MeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    vertex_count: usize,
}

#[derive(Clone, Debug)]
struct TextureData {
    width: u32,
    height: u32,
    format: TextureFormat,
    data_size: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TextureFormat {
    Rgba8,
    Bc7,
    Astc4x4,
}

struct AssetCache {
    meshes: HashMap<u64, MeshData>,
    textures: HashMap<u64, TextureData>,
    max_size_bytes: usize,
    current_size_bytes: usize,
}

impl AssetCache {
    fn new(max_size: usize) -> Self {
        Self {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            max_size_bytes: max_size,
            current_size_bytes: 0,
        }
    }

    fn insert_mesh(&mut self, id: u64, mesh: MeshData) -> bool {
        let size = mesh.positions.len() * 12 + mesh.indices.len() * 4;
        if self.current_size_bytes + size <= self.max_size_bytes {
            self.current_size_bytes += size;
            self.meshes.insert(id, mesh);
            true
        } else {
            false
        }
    }

    fn get_mesh(&self, id: u64) -> Option<&MeshData> {
        self.meshes.get(&id)
    }

    fn evict_lru(&mut self) {
        // Simple eviction: remove first entry
        if let Some(&id) = self.meshes.keys().next() {
            if let Some(mesh) = self.meshes.remove(&id) {
                self.current_size_bytes -= mesh.positions.len() * 12 + mesh.indices.len() * 4;
            }
        }
    }
}

fn generate_mock_mesh(vertex_count: usize) -> MeshData {
    MeshData {
        positions: vec![[0.0, 0.0, 0.0]; vertex_count],
        normals: vec![[0.0, 1.0, 0.0]; vertex_count],
        indices: (0..vertex_count as u32).collect(),
        vertex_count,
    }
}

fn compute_hash(data: &[u8]) -> u64 {
    let mut hash = 0u64;
    for (i, &byte) in data.iter().enumerate() {
        hash = hash.wrapping_add((byte as u64).wrapping_mul((i as u64).wrapping_add(1)));
    }
    hash
}

// ============================================================================
// CATEGORY 1: ASSET LOADING STRESS
// ============================================================================

fn bench_asset_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_adversarial/loading_stress");

    // Test 1: Sequential asset loading
    for count in [10, 50, 100] {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("sequential_load", count),
            &count,
            |bencher, &count| {
                bencher.iter(|| {
                    let handles: Vec<AssetHandle> = (0..count)
                        .map(|i| AssetHandle {
                            id: i as u64,
                            path: format!("assets/mesh_{}.glb", i),
                            asset_type: AssetType::Mesh,
                            loaded: true,
                        })
                        .collect();
                    std_black_box(handles.len())
                });
            },
        );
    }

    // Test 2: Mixed asset types
    group.bench_function("mixed_asset_types_100", |bencher| {
        bencher.iter(|| {
            let handles: Vec<AssetHandle> = (0..100)
                .map(|i| {
                    let asset_type = match i % 5 {
                        0 => AssetType::Mesh,
                        1 => AssetType::Texture,
                        2 => AssetType::Audio,
                        3 => AssetType::Material,
                        _ => AssetType::Animation,
                    };
                    AssetHandle {
                        id: i as u64,
                        path: format!("assets/asset_{}.bin", i),
                        asset_type,
                        loaded: false,
                    }
                })
                .collect();
            std_black_box(handles.len())
        });
    });

    // Test 3: Duplicate path detection
    group.bench_function("duplicate_detection_200", |bencher| {
        let paths: Vec<String> = (0..200)
            .map(|i| format!("assets/mesh_{}.glb", i % 50)) // 50 unique, rest duplicates
            .collect();

        bencher.iter(|| {
            let mut seen: HashMap<&str, usize> = HashMap::new();
            for path in &paths {
                *seen.entry(path.as_str()).or_insert(0) += 1;
            }
            let duplicates: usize = seen.values().filter(|&&c| c > 1).count();
            std_black_box(duplicates)
        });
    });

    // Test 4: Invalid path handling
    group.bench_function("invalid_path_validation", |bencher| {
        let paths = [
            "assets/valid.glb",
            "../../../etc/passwd",
            "assets/../secrets.txt",
            "C:\\Windows\\System32\\cmd.exe",
            "assets/valid/mesh.glb",
            "assets/./valid.glb",
        ];

        bencher.iter(|| {
            let valid: Vec<_> = paths
                .iter()
                .filter(|p| {
                    !p.contains("..") && !p.starts_with('/') && !p.contains(':')
                })
                .collect();
            std_black_box(valid.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 2: CACHE STRESS
// ============================================================================

fn bench_cache_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_adversarial/cache_stress");

    // Test 1: Cache insertion
    group.bench_function("cache_insert_100", |bencher| {
        bencher.iter(|| {
            let mut cache = AssetCache::new(100 * 1024 * 1024); // 100MB
            for i in 0..100 {
                let mesh = generate_mock_mesh(1000);
                cache.insert_mesh(i, mesh);
            }
            std_black_box(cache.meshes.len())
        });
    });

    // Test 2: Cache lookup
    group.bench_function("cache_lookup_1000", |bencher| {
        let mut cache = AssetCache::new(100 * 1024 * 1024);
        for i in 0..100 {
            let mesh = generate_mock_mesh(1000);
            cache.insert_mesh(i, mesh);
        }

        bencher.iter(|| {
            let mut hits = 0;
            for i in 0..1000 {
                if cache.get_mesh(i % 100).is_some() {
                    hits += 1;
                }
            }
            std_black_box(hits)
        });
    });

    // Test 3: Cache eviction under pressure
    group.bench_function("cache_eviction_pressure", |bencher| {
        bencher.iter(|| {
            let mut cache = AssetCache::new(1024 * 1024); // 1MB - will overflow
            for i in 0..200 {
                let mesh = generate_mock_mesh(1000); // ~12KB each
                if !cache.insert_mesh(i, mesh) {
                    cache.evict_lru();
                }
            }
            std_black_box(cache.meshes.len())
        });
    });

    // Test 4: Cache miss handling
    group.bench_function("cache_miss_handling_500", |bencher| {
        let cache = AssetCache::new(100 * 1024 * 1024);

        bencher.iter(|| {
            let mut misses = 0;
            for i in 0..500 {
                if cache.get_mesh(i).is_none() {
                    misses += 1;
                }
            }
            std_black_box(misses)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 3: MESH PROCESSING
// ============================================================================

fn bench_mesh_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_adversarial/mesh_processing");

    // Test 1: Vertex count validation
    for vertex_count in [1000, 10000, 100000] {
        group.throughput(Throughput::Elements(vertex_count as u64));

        group.bench_with_input(
            BenchmarkId::new("vertex_validation", vertex_count),
            &vertex_count,
            |bencher, &vertex_count| {
                let mesh = generate_mock_mesh(vertex_count);

                bencher.iter(|| {
                    let valid = mesh.positions.len() == mesh.normals.len()
                        && mesh.indices.iter().all(|&i| (i as usize) < mesh.vertex_count);
                    std_black_box(valid)
                });
            },
        );
    }

    // Test 2: Bounding box calculation
    group.bench_function("bounding_box_10000_verts", |bencher| {
        let mesh = MeshData {
            positions: (0..10000)
                .map(|i| [(i % 100) as f32, (i / 100 % 100) as f32, (i / 10000) as f32])
                .collect(),
            normals: vec![],
            indices: vec![],
            vertex_count: 10000,
        };

        bencher.iter(|| {
            let mut min = [f32::MAX; 3];
            let mut max = [f32::MIN; 3];
            for pos in &mesh.positions {
                for i in 0..3 {
                    min[i] = min[i].min(pos[i]);
                    max[i] = max[i].max(pos[i]);
                }
            }
            std_black_box((min, max))
        });
    });

    // Test 3: Index buffer validation
    group.bench_function("index_validation_30000", |bencher| {
        let indices: Vec<u32> = (0..30000).map(|i| (i % 10000) as u32).collect();
        let vertex_count = 10000u32;

        bencher.iter(|| {
            let valid = indices.iter().all(|&i| i < vertex_count);
            std_black_box(valid)
        });
    });

    // Test 4: Degenerate triangle detection
    group.bench_function("degenerate_triangle_check", |bencher| {
        let indices: Vec<u32> = (0..30000).collect();

        bencher.iter(|| {
            let mut degenerate_count = 0;
            for tri in indices.chunks(3) {
                if tri.len() == 3 && (tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2]) {
                    degenerate_count += 1;
                }
            }
            std_black_box(degenerate_count)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 4: TEXTURE PROCESSING
// ============================================================================

fn bench_texture_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_adversarial/texture_processing");

    // Test 1: Texture metadata validation
    group.bench_function("texture_metadata_validation", |bencher| {
        let textures: Vec<TextureData> = (0..100)
            .map(|i| TextureData {
                width: 2u32.pow((i % 12) as u32 + 1), // 2 to 4096
                height: 2u32.pow((i % 12) as u32 + 1),
                format: match i % 3 {
                    0 => TextureFormat::Rgba8,
                    1 => TextureFormat::Bc7,
                    _ => TextureFormat::Astc4x4,
                },
                data_size: 0,
            })
            .collect();

        bencher.iter(|| {
            let valid: Vec<_> = textures
                .iter()
                .filter(|t| {
                    t.width.is_power_of_two()
                        && t.height.is_power_of_two()
                        && t.width <= 4096
                        && t.height <= 4096
                })
                .collect();
            std_black_box(valid.len())
        });
    });

    // Test 2: Mipmap level calculation
    group.bench_function("mipmap_levels_100", |bencher| {
        let sizes: Vec<(u32, u32)> = (0..100)
            .map(|i| {
                let size = 2u32.pow((i % 12) as u32 + 1);
                (size, size)
            })
            .collect();

        bencher.iter(|| {
            let mip_counts: Vec<u32> = sizes
                .iter()
                .map(|(w, h)| {
                    let max_dim = (*w).max(*h);
                    (max_dim as f32).log2().floor() as u32 + 1
                })
                .collect();
            std_black_box(mip_counts.iter().sum::<u32>())
        });
    });

    // Test 3: Memory size estimation
    group.bench_function("memory_estimation_100", |bencher| {
        let textures: Vec<TextureData> = (0..100)
            .map(|i| {
                let size = 2u32.pow((i % 10) as u32 + 2);
                TextureData {
                    width: size,
                    height: size,
                    format: TextureFormat::Rgba8,
                    data_size: (size * size * 4) as usize,
                }
            })
            .collect();

        bencher.iter(|| {
            let total_bytes: usize = textures.iter().map(|t| t.data_size).sum();
            std_black_box(total_bytes)
        });
    });

    // Test 4: Format compatibility check
    group.bench_function("format_compatibility_check", |bencher| {
        let formats = [
            TextureFormat::Rgba8,
            TextureFormat::Bc7,
            TextureFormat::Astc4x4,
        ];
        let platform_supports_bc7 = true;
        let platform_supports_astc = false;

        bencher.iter(|| {
            let compatible: Vec<_> = formats
                .iter()
                .filter(|&&f| match f {
                    TextureFormat::Rgba8 => true,
                    TextureFormat::Bc7 => platform_supports_bc7,
                    TextureFormat::Astc4x4 => platform_supports_astc,
                })
                .collect();
            std_black_box(compatible.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 5: HOT RELOAD
// ============================================================================

fn bench_hot_reload(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_adversarial/hot_reload");

    // Test 1: File change detection
    group.bench_function("change_detection_1000", |bencher| {
        let file_times: HashMap<String, u64> = (0..1000)
            .map(|i| (format!("assets/file_{}.bin", i), i as u64 * 1000))
            .collect();

        let current_times: HashMap<String, u64> = (0..1000)
            .map(|i| {
                let time = if i % 10 == 0 { i as u64 * 1000 + 1 } else { i as u64 * 1000 };
                (format!("assets/file_{}.bin", i), time)
            })
            .collect();

        bencher.iter(|| {
            let changed: Vec<_> = file_times
                .iter()
                .filter(|(path, &old_time)| {
                    current_times
                        .get(*path)
                        .is_some_and(|&new_time| new_time > old_time)
                })
                .collect();
            std_black_box(changed.len())
        });
    });

    // Test 2: Dependency graph traversal
    group.bench_function("dependency_traversal_100", |bencher| {
        // Build dependency graph: material depends on textures
        let dependencies: HashMap<u64, Vec<u64>> = (0..100)
            .map(|i| (i, vec![i * 2, i * 2 + 1])) // Each asset depends on 2 others
            .collect();

        bencher.iter(|| {
            let mut to_reload = vec![50u64]; // Changed asset
            let mut reloaded = std::collections::HashSet::new();

            while let Some(id) = to_reload.pop() {
                if reloaded.insert(id) {
                    // Find dependents
                    for (&dependent, deps) in &dependencies {
                        if deps.contains(&id) && !reloaded.contains(&dependent) {
                            to_reload.push(dependent);
                        }
                    }
                }
            }
            std_black_box(reloaded.len())
        });
    });

    // Test 3: Reload queue prioritization
    group.bench_function("reload_queue_priority", |bencher| {
        let mut queue: Vec<(u64, u32)> = (0..200)
            .map(|i| (i as u64, (i % 5) as u32)) // id, priority
            .collect();

        bencher.iter(|| {
            queue.sort_by_key(|&(_, priority)| std::cmp::Reverse(priority));
            std_black_box(queue.first().cloned())
        });

        // Restore order
        queue.sort_by_key(|&(id, _)| id);
    });

    // Test 4: Hash-based change verification
    group.bench_function("hash_verification_100", |bencher| {
        let old_hashes: Vec<u64> = (0..100).map(|i| compute_hash(&[i as u8; 1024])).collect();
        let new_data: Vec<Vec<u8>> = (0..100)
            .map(|i| {
                if i % 10 == 0 {
                    vec![(i + 1) as u8; 1024] // Changed
                } else {
                    vec![i as u8; 1024] // Unchanged
                }
            })
            .collect();

        bencher.iter(|| {
            let changed: Vec<usize> = new_data
                .iter()
                .enumerate()
                .filter(|(i, data)| compute_hash(data) != old_hashes[*i])
                .map(|(i, _)| i)
                .collect();
            std_black_box(changed.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 6: STREAMING
// ============================================================================

fn bench_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_adversarial/streaming");

    // Test 1: Priority queue management
    group.bench_function("stream_priority_queue_500", |bencher| {
        let mut queue: Vec<(u64, f32)> = (0..500)
            .map(|i| (i as u64, (i % 100) as f32 / 100.0))
            .collect();

        bencher.iter(|| {
            queue.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let top_10: Vec<_> = queue.iter().take(10).collect();
            std_black_box(top_10.len())
        });
    });

    // Test 2: Distance-based prioritization
    group.bench_function("distance_priority_1000", |bencher| {
        let assets: Vec<(u64, [f32; 3])> = (0..1000)
            .map(|i| {
                (
                    i as u64,
                    [(i % 100) as f32, ((i / 100) % 10) as f32, (i / 1000) as f32],
                )
            })
            .collect();

        let camera_pos = [50.0f32, 5.0, 0.5];

        bencher.iter(|| {
            let mut priorities: Vec<(u64, f32)> = assets
                .iter()
                .map(|(id, pos)| {
                    let dist = ((pos[0] - camera_pos[0]).powi(2)
                        + (pos[1] - camera_pos[1]).powi(2)
                        + (pos[2] - camera_pos[2]).powi(2))
                    .sqrt();
                    (*id, 1.0 / (dist + 1.0)) // Inverse distance priority
                })
                .collect();

            priorities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            std_black_box(priorities.first().cloned())
        });
    });

    // Test 3: Budget-based streaming
    group.bench_function("budget_streaming_100", |bencher| {
        let assets: Vec<(u64, usize, f32)> = (0..100)
            .map(|i| (i as u64, (i + 1) * 1024, (100 - i) as f32)) // id, size, priority
            .collect();

        let budget_bytes = 50 * 1024usize;

        bencher.iter(|| {
            let mut sorted = assets.clone();
            sorted.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

            let mut loaded = Vec::new();
            let mut used = 0usize;

            for (id, size, _) in sorted {
                if used + size <= budget_bytes {
                    loaded.push(id);
                    used += size;
                }
            }
            std_black_box(loaded.len())
        });
    });

    // Test 4: LOD selection
    group.bench_function("lod_selection_500", |bencher| {
        let distances: Vec<f32> = (0..500).map(|i| i as f32 * 0.5).collect();
        let lod_thresholds = [10.0f32, 25.0, 50.0, 100.0]; // LOD 0, 1, 2, 3

        bencher.iter(|| {
            let lods: Vec<usize> = distances
                .iter()
                .map(|&dist| {
                    lod_thresholds
                        .iter()
                        .position(|&threshold| dist < threshold)
                        .unwrap_or(lod_thresholds.len())
                })
                .collect();
            std_black_box(lods.iter().sum::<usize>())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_asset_loading,
    bench_cache_stress,
    bench_mesh_processing,
    bench_texture_processing,
    bench_hot_reload,
    bench_streaming,
);

criterion_main!(benches);
