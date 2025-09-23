use criterion::{criterion_group, criterion_main, Criterion};
use astraweave_terrain::*;
use std::hint::black_box;

fn benchmark_heightmap_generation(c: &mut Criterion) {
    let config = NoiseConfig::default();
    let noise = TerrainNoise::new(&config, 12345);
    
    c.bench_function("heightmap_generation_64x64", |b| {
        b.iter(|| {
            let chunk_id = ChunkId::new(black_box(0), black_box(0));
            noise.generate_heightmap(black_box(chunk_id), black_box(256.0), black_box(64))
        })
    });
    
    c.bench_function("heightmap_generation_128x128", |b| {
        b.iter(|| {
            let chunk_id = ChunkId::new(black_box(0), black_box(0));
            noise.generate_heightmap(black_box(chunk_id), black_box(256.0), black_box(128))
        })
    });
}

fn benchmark_climate_sampling(c: &mut Criterion) {
    let config = ClimateConfig::default();
    let climate = ClimateMap::new(&config, 12345);
    
    c.bench_function("climate_sampling", |b| {
        b.iter(|| {
            climate.sample_climate(black_box(100.0), black_box(200.0), black_box(10.0))
        })
    });
    
    c.bench_function("chunk_climate_sampling", |b| {
        b.iter(|| {
            let chunk_id = ChunkId::new(black_box(0), black_box(0));
            climate.sample_chunk(black_box(chunk_id), black_box(256.0), black_box(64))
        })
    });
}

fn benchmark_world_generation(c: &mut Criterion) {
    let config = WorldConfig::default();
    let mut generator = WorldGenerator::new(config);
    
    c.bench_function("world_chunk_generation", |b| {
        b.iter(|| {
            let chunk_id = ChunkId::new(black_box(1), black_box(1));
            generator.generate_chunk(black_box(chunk_id))
        })
    });
}

criterion_group!(
    benches,
    benchmark_heightmap_generation,
    benchmark_climate_sampling,
    benchmark_world_generation
);
criterion_main!(benches);