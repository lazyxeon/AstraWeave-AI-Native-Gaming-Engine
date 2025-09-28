use astraweave_materials::{Graph, MaterialPackage, Node};
use astraweave_render as _; // ensure crate compiles
use criterion::{criterion_group, criterion_main, Criterion};
use glam::Vec3;

fn bench_material_compile(c: &mut Criterion) {
    c.bench_function("material_compile_64_nodes", |b| {
        b.iter(|| {
            let mut nodes = std::collections::BTreeMap::new();
            nodes.insert(
                "uv".into(),
                Node::Constant3 {
                    value: [0.0, 0.0, 0.0],
                },
            );
            for i in 0..32 {
                nodes.insert(
                    format!("t{}", i),
                    Node::Texture2D {
                        id: format!("tex{}", i),
                        uv: "uv".into(),
                    },
                );
            }
            nodes.insert(
                "a".into(),
                Node::Add {
                    a: "t0".into(),
                    b: "t1".into(),
                },
            );
            nodes.insert(
                "b".into(),
                Node::Multiply {
                    a: "a".into(),
                    b: "t2".into(),
                },
            );
            let g = Graph {
                nodes,
                base_color: "b".into(),
                mr: None,
                normal: None,
                clearcoat: None,
                anisotropy: None,
                transmission: None,
            };
            let pkg = MaterialPackage::from_graph(&g).unwrap();
            std::hint::black_box(pkg)
        })
    });
}

fn bench_cpu_cluster_binning(c: &mut Criterion) {
    use astraweave_render::clustered::{bin_lights_cpu, ClusterDims, CpuLight};
    c.bench_function("cpu_cluster_binning_1k_lights", |b| {
        let dims = ClusterDims { x: 16, y: 9, z: 24 };
        let mut lights = Vec::new();
        for i in 0..1000 {
            let f = i as f32 * 0.01;
            lights.push(CpuLight {
                pos: Vec3::new(
                    f.sin() * 10.0,
                    f.cos() * 5.0,
                    (f * 10.0).fract() * 50.0 + 1.0,
                ),
                radius: 1.0 + (f * 3.0).fract(),
            });
        }
        b.iter(|| {
            let (c, _i, _o) = bin_lights_cpu(
                &lights,
                dims,
                (1920, 1080),
                0.1,
                100.0,
                std::f32::consts::FRAC_PI_3,
            );
            std::hint::black_box(c)
        })
    });
}

criterion_group!(benches, bench_material_compile, bench_cpu_cluster_binning);
criterion_main!(benches);
