//! Benchmarks for astraweave-blend core operations.
//!
//! Tests performance of conversion options, builder pattern,
//! serialization, and other core operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;

use astraweave_blend::options::*;
use astraweave_blend::version::BlenderVersion;
use astraweave_blend::error::BlendError;

// ============================================================================
// VERSION BENCHMARKS
// ============================================================================

fn bench_version_creation(c: &mut Criterion) {
    c.bench_function("version_creation", |b| {
        b.iter(|| {
            black_box(BlenderVersion::new(
                black_box(4),
                black_box(1),
                black_box(0),
            ))
        })
    });
}

fn bench_version_comparison(c: &mut Criterion) {
    let v1 = BlenderVersion::new(3, 6, 5);
    let v2 = BlenderVersion::new(4, 0, 0);
    
    c.bench_function("version_comparison_less", |b| {
        b.iter(|| black_box(black_box(&v1) < black_box(&v2)))
    });
    
    c.bench_function("version_comparison_equal", |b| {
        b.iter(|| black_box(black_box(&v1) == black_box(&v1)))
    });
}

fn bench_version_meets_minimum(c: &mut Criterion) {
    let versions = vec![
        BlenderVersion::new(2, 79, 0),
        BlenderVersion::new(2, 93, 0),
        BlenderVersion::new(3, 6, 5),
        BlenderVersion::new(4, 0, 0),
    ];
    
    let mut group = c.benchmark_group("version_meets_minimum");
    for v in &versions {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}.{}.{}", v.major, v.minor, v.patch)),
            v,
            |b, v| b.iter(|| black_box(v.meets_minimum())),
        );
    }
    group.finish();
}

fn bench_version_display(c: &mut Criterion) {
    let v = BlenderVersion::new(4, 1, 0);
    
    c.bench_function("version_display", |b| {
        b.iter(|| black_box(format!("{}", black_box(&v))))
    });
}

// ============================================================================
// OPTIONS BENCHMARKS
// ============================================================================

fn bench_options_default(c: &mut Criterion) {
    c.bench_function("options_default", |b| {
        b.iter(|| black_box(ConversionOptions::default()))
    });
}

fn bench_options_presets(c: &mut Criterion) {
    let mut group = c.benchmark_group("options_presets");
    
    group.bench_function("game_runtime", |b| {
        b.iter(|| black_box(ConversionOptions::game_runtime()))
    });
    
    group.bench_function("editor_preview", |b| {
        b.iter(|| black_box(ConversionOptions::editor_preview()))
    });
    
    group.bench_function("archival_quality", |b| {
        b.iter(|| black_box(ConversionOptions::archival_quality()))
    });
    
    group.finish();
}

fn bench_options_builder_minimal(c: &mut Criterion) {
    c.bench_function("builder_minimal", |b| {
        b.iter(|| black_box(ConversionOptions::builder().build()))
    });
}

fn bench_options_builder_full(c: &mut Criterion) {
    c.bench_function("builder_full", |b| {
        b.iter(|| {
            black_box(
                ConversionOptions::builder()
                    .format(OutputFormat::GlbBinary)
                    .draco_compression(true)
                    .texture_format(TextureFormat::WebP)
                    .max_texture_resolution(Some(2048))
                    .export_animations(true)
                    .apply_modifiers(true)
                    .timeout(Duration::from_secs(300))
                    .linked_library_depth(10)
                    .cache_enabled(true)
                    .build()
            )
        })
    });
}

fn bench_options_clone(c: &mut Criterion) {
    let opts = ConversionOptions::game_runtime();
    
    c.bench_function("options_clone", |b| {
        b.iter(|| black_box(black_box(&opts).clone()))
    });
}

fn bench_nested_options_access(c: &mut Criterion) {
    let opts = ConversionOptions::game_runtime();
    
    let mut group = c.benchmark_group("nested_options_access");
    
    group.bench_function("gltf_draco", |b| {
        b.iter(|| black_box(black_box(&opts).gltf.draco_compression))
    });
    
    group.bench_function("texture_format", |b| {
        b.iter(|| black_box(black_box(&opts).textures.format))
    });
    
    group.bench_function("process_timeout", |b| {
        b.iter(|| black_box(black_box(&opts).process.timeout))
    });
    
    group.bench_function("linked_depth", |b| {
        b.iter(|| black_box(black_box(&opts).linked_libraries.max_recursion_depth))
    });
    
    group.finish();
}

// ============================================================================
// OUTPUT FORMAT BENCHMARKS
// ============================================================================

fn bench_output_format_extension(c: &mut Criterion) {
    let formats = [
        OutputFormat::GlbBinary,
        OutputFormat::GltfEmbedded,
        OutputFormat::GltfSeparate,
    ];
    
    let mut group = c.benchmark_group("output_format_extension");
    for format in &formats {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", format)),
            format,
            |b, f| b.iter(|| black_box(f.extension())),
        );
    }
    group.finish();
}

fn bench_output_format_blender_format(c: &mut Criterion) {
    let formats = [
        OutputFormat::GlbBinary,
        OutputFormat::GltfEmbedded,
        OutputFormat::GltfSeparate,
    ];
    
    let mut group = c.benchmark_group("output_format_blender_format");
    for format in &formats {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", format)),
            format,
            |b, f| b.iter(|| black_box(f.blender_format())),
        );
    }
    group.finish();
}

// ============================================================================
// TEXTURE FORMAT BENCHMARKS
// ============================================================================

fn bench_texture_format_extension(c: &mut Criterion) {
    let formats = [
        TextureFormat::Png,
        TextureFormat::Jpeg,
        TextureFormat::WebP,
        TextureFormat::Original,
    ];
    
    let mut group = c.benchmark_group("texture_format_extension");
    for format in &formats {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", format)),
            format,
            |b, f| b.iter(|| black_box(f.extension())),
        );
    }
    group.finish();
}

// ============================================================================
// SERIALIZATION BENCHMARKS
// ============================================================================

fn bench_version_serialize_ron(c: &mut Criterion) {
    let v = BlenderVersion::new(4, 1, 0);
    
    c.bench_function("version_serialize_ron", |b| {
        b.iter(|| black_box(ron::to_string(black_box(&v))))
    });
}

fn bench_version_deserialize_ron(c: &mut Criterion) {
    let v = BlenderVersion::new(4, 1, 0);
    let serialized = ron::to_string(&v).unwrap();
    
    c.bench_function("version_deserialize_ron", |b| {
        b.iter(|| black_box(ron::from_str::<BlenderVersion>(black_box(&serialized))))
    });
}

fn bench_options_serialize_ron(c: &mut Criterion) {
    let opts = ConversionOptions::game_runtime();
    
    c.bench_function("options_serialize_ron", |b| {
        b.iter(|| black_box(ron::to_string(black_box(&opts))))
    });
}

fn bench_options_deserialize_ron(c: &mut Criterion) {
    let opts = ConversionOptions::game_runtime();
    let serialized = ron::to_string(&opts).unwrap();
    
    c.bench_function("options_deserialize_ron", |b| {
        b.iter(|| black_box(ron::from_str::<ConversionOptions>(black_box(&serialized))))
    });
}

fn bench_options_serialize_json(c: &mut Criterion) {
    let opts = ConversionOptions::game_runtime();
    
    c.bench_function("options_serialize_json", |b| {
        b.iter(|| black_box(serde_json::to_string(black_box(&opts))))
    });
}

fn bench_options_deserialize_json(c: &mut Criterion) {
    let opts = ConversionOptions::game_runtime();
    let serialized = serde_json::to_string(&opts).unwrap();
    
    c.bench_function("options_deserialize_json", |b| {
        b.iter(|| black_box(serde_json::from_str::<ConversionOptions>(black_box(&serialized))))
    });
}

// ============================================================================
// ERROR BENCHMARKS
// ============================================================================

fn bench_error_creation(c: &mut Criterion) {
    use std::path::PathBuf;
    
    let mut group = c.benchmark_group("error_creation");
    
    group.bench_function("blender_not_found", |b| {
        b.iter(|| black_box(BlendError::BlenderNotFound {
            searched_paths: vec![PathBuf::from("/usr/bin"), PathBuf::from("/opt/blender")],
        }))
    });
    
    group.bench_function("invalid_blend_file", |b| {
        b.iter(|| {
            black_box(BlendError::InvalidBlendFile {
                path: PathBuf::from("/test/file.blend"),
                message: "Invalid magic bytes".to_string(),
            })
        })
    });
    
    group.bench_function("conversion_failed", |b| {
        b.iter(|| {
            black_box(BlendError::ConversionFailed {
                message: "Test error message".to_string(),
                exit_code: Some(1),
                stderr: "Error output".to_string(),
                blender_output: Some("Full output".to_string()),
            })
        })
    });
    
    group.bench_function("timeout", |b| {
        b.iter(|| black_box(BlendError::Timeout {
            operation: "conversion".to_string(),
            duration: Duration::from_secs(60),
            path: PathBuf::from("/test/file.blend"),
            timeout_secs: 60,
        }))
    });
    
    group.finish();
}

fn bench_error_display(c: &mut Criterion) {
    use std::path::PathBuf;
    
    let errors: Vec<BlendError> = vec![
        BlendError::BlenderNotFound {
            searched_paths: vec![PathBuf::from("/usr/bin")],
        },
        BlendError::InvalidBlendFile {
            path: PathBuf::from("/test/file.blend"),
            message: "Invalid".to_string(),
        },
        BlendError::ConversionFailed {
            message: "Test error".to_string(),
            exit_code: Some(1),
            stderr: "Error".to_string(),
            blender_output: None,
        },
        BlendError::Timeout {
            operation: "conversion".to_string(),
            duration: Duration::from_secs(60),
            path: PathBuf::from("/test/file.blend"),
            timeout_secs: 60,
        },
    ];
    
    let mut group = c.benchmark_group("error_display");
    for (i, err) in errors.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::from_parameter(i),
            err,
            |b, e| b.iter(|| black_box(format!("{}", e))),
        );
    }
    group.finish();
}

// ============================================================================
// THROUGHPUT BENCHMARKS
// ============================================================================

fn bench_builder_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder_throughput");
    
    for count in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                b.iter(|| {
                    for _ in 0..count {
                        black_box(ConversionOptions::builder().build());
                    }
                })
            },
        );
    }
    
    group.finish();
}

fn bench_version_comparison_throughput(c: &mut Criterion) {
    let versions: Vec<_> = (0..100)
        .map(|i| BlenderVersion::new(2 + i / 40, (80 + i) % 100, i % 20))
        .collect();
    
    let mut group = c.benchmark_group("version_comparison_throughput");
    group.throughput(Throughput::Elements(versions.len() as u64));
    
    group.bench_function("all_pairs", |b| {
        b.iter(|| {
            for v1 in &versions {
                for v2 in &versions {
                    black_box(v1.cmp(v2));
                }
            }
        })
    });
    
    group.finish();
}

fn bench_options_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("options_memory");
    
    group.bench_function("single_allocation", |b| {
        b.iter(|| {
            let opts = Box::new(ConversionOptions::game_runtime());
            black_box(opts)
        })
    });
    
    group.bench_function("vec_allocation_10", |b| {
        b.iter(|| {
            let opts: Vec<ConversionOptions> = (0..10)
                .map(|_| ConversionOptions::game_runtime())
                .collect();
            black_box(opts)
        })
    });
    
    group.bench_function("vec_allocation_100", |b| {
        b.iter(|| {
            let opts: Vec<ConversionOptions> = (0..100)
                .map(|_| ConversionOptions::game_runtime())
                .collect();
            black_box(opts)
        })
    });
    
    group.finish();
}

// ============================================================================
// CRITERION GROUPS
// ============================================================================

criterion_group!(
    version_benches,
    bench_version_creation,
    bench_version_comparison,
    bench_version_meets_minimum,
    bench_version_display,
);

criterion_group!(
    options_benches,
    bench_options_default,
    bench_options_presets,
    bench_options_builder_minimal,
    bench_options_builder_full,
    bench_options_clone,
    bench_nested_options_access,
);

criterion_group!(
    format_benches,
    bench_output_format_extension,
    bench_output_format_blender_format,
    bench_texture_format_extension,
);

criterion_group!(
    serialization_benches,
    bench_version_serialize_ron,
    bench_version_deserialize_ron,
    bench_options_serialize_ron,
    bench_options_deserialize_ron,
    bench_options_serialize_json,
    bench_options_deserialize_json,
);

criterion_group!(
    error_benches,
    bench_error_creation,
    bench_error_display,
);

criterion_group!(
    throughput_benches,
    bench_builder_throughput,
    bench_version_comparison_throughput,
    bench_options_memory_allocation,
);

criterion_main!(
    version_benches,
    options_benches,
    format_benches,
    serialization_benches,
    error_benches,
    throughput_benches,
);
