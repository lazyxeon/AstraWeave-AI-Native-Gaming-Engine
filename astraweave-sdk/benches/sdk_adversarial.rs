//! Adversarial benchmarks for astraweave-sdk
//!
//! Tests SDK/FFI interface under extreme conditions:
//! - C ABI function call overhead
//! - Data marshalling throughput
//! - String handling across FFI boundary
//! - Callback invocation performance
//! - Error propagation overhead
//! - Memory lifecycle management

#![allow(dead_code, unused_variables, clippy::type_complexity)]

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::hint::black_box;
use std::os::raw::{c_char, c_int, c_void};

// ============================================================================
// LOCAL TYPE DEFINITIONS (Standalone benchmark - no crate imports)
// ============================================================================

/// SDK error codes
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SdkError {
    Success = 0,
    InvalidParameter = 1,
    OutOfMemory = 2,
    NotInitialized = 3,
    AlreadyInitialized = 4,
    InvalidHandle = 5,
    OperationFailed = 6,
    BufferTooSmall = 7,
    Timeout = 8,
    NotFound = 9,
}

/// Opaque handle type
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Handle(u64);

impl Handle {
    fn new(id: u64) -> Self {
        Self(id)
    }

    fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// Vector3 for FFI
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }
}

/// Quaternion for FFI
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

/// Transform for FFI
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            rotation: Quat::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

/// Entity creation parameters
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct EntityParams {
    pub transform: Transform,
    pub flags: u32,
    pub layer: u32,
    pub parent: Handle,
}

impl Default for EntityParams {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
            flags: 0,
            layer: 0,
            parent: Handle(0),
        }
    }
}

/// Callback function types
type UpdateCallback = extern "C" fn(handle: Handle, delta_time: f32) -> c_int;
type EventCallback = extern "C" fn(event_type: c_int, data: *const c_void, data_len: usize) -> c_int;

/// SDK context (simulated)
struct SdkContext {
    initialized: bool,
    entities: HashMap<Handle, EntityData>,
    next_handle: u64,
    callbacks: Vec<Box<dyn Fn(Handle, f32) -> i32>>,
    string_cache: HashMap<String, CString>,
}

struct EntityData {
    transform: Transform,
    flags: u32,
    layer: u32,
    name: String,
}

impl SdkContext {
    fn new() -> Self {
        Self {
            initialized: false,
            entities: HashMap::new(),
            next_handle: 1,
            callbacks: Vec::new(),
            string_cache: HashMap::new(),
        }
    }

    fn initialize(&mut self) -> SdkError {
        if self.initialized {
            return SdkError::AlreadyInitialized;
        }
        self.initialized = true;
        SdkError::Success
    }

    fn create_entity(&mut self, params: &EntityParams) -> Result<Handle, SdkError> {
        if !self.initialized {
            return Err(SdkError::NotInitialized);
        }

        let handle = Handle::new(self.next_handle);
        self.next_handle += 1;

        self.entities.insert(
            handle,
            EntityData {
                transform: params.transform,
                flags: params.flags,
                layer: params.layer,
                name: String::new(),
            },
        );

        Ok(handle)
    }

    fn destroy_entity(&mut self, handle: Handle) -> SdkError {
        if !self.initialized {
            return SdkError::NotInitialized;
        }

        if self.entities.remove(&handle).is_some() {
            SdkError::Success
        } else {
            SdkError::InvalidHandle
        }
    }

    fn get_transform(&self, handle: Handle) -> Result<Transform, SdkError> {
        if !self.initialized {
            return Err(SdkError::NotInitialized);
        }

        self.entities
            .get(&handle)
            .map(|e| e.transform)
            .ok_or(SdkError::InvalidHandle)
    }

    fn set_transform(&mut self, handle: Handle, transform: Transform) -> SdkError {
        if !self.initialized {
            return SdkError::NotInitialized;
        }

        if let Some(entity) = self.entities.get_mut(&handle) {
            entity.transform = transform;
            SdkError::Success
        } else {
            SdkError::InvalidHandle
        }
    }

    fn set_name(&mut self, handle: Handle, name: &str) -> SdkError {
        if !self.initialized {
            return SdkError::NotInitialized;
        }

        if let Some(entity) = self.entities.get_mut(&handle) {
            entity.name = name.to_string();
            SdkError::Success
        } else {
            SdkError::InvalidHandle
        }
    }

    fn get_name(&self, handle: Handle) -> Result<&str, SdkError> {
        if !self.initialized {
            return Err(SdkError::NotInitialized);
        }

        self.entities
            .get(&handle)
            .map(|e| e.name.as_str())
            .ok_or(SdkError::InvalidHandle)
    }
}

/// String marshalling helper
fn marshal_string_to_c(s: &str) -> CString {
    CString::new(s).unwrap_or_else(|_| CString::new("").unwrap())
}

fn marshal_string_from_c(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string()) }
}

/// Batch operation helper
struct BatchOperation {
    operations: Vec<BatchOpType>,
}

enum BatchOpType {
    CreateEntity(EntityParams),
    DestroyEntity(Handle),
    SetTransform(Handle, Transform),
    SetName(Handle, String),
}

impl BatchOperation {
    fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    fn add_create(&mut self, params: EntityParams) {
        self.operations.push(BatchOpType::CreateEntity(params));
    }

    fn add_destroy(&mut self, handle: Handle) {
        self.operations.push(BatchOpType::DestroyEntity(handle));
    }

    fn add_set_transform(&mut self, handle: Handle, transform: Transform) {
        self.operations.push(BatchOpType::SetTransform(handle, transform));
    }

    fn execute(&self, ctx: &mut SdkContext) -> Vec<SdkError> {
        self.operations
            .iter()
            .map(|op| match op {
                BatchOpType::CreateEntity(params) => {
                    ctx.create_entity(params).map(|_| SdkError::Success).unwrap_or_else(|e| e)
                }
                BatchOpType::DestroyEntity(handle) => ctx.destroy_entity(*handle),
                BatchOpType::SetTransform(handle, transform) => ctx.set_transform(*handle, *transform),
                BatchOpType::SetName(handle, name) => ctx.set_name(*handle, name),
            })
            .collect()
    }
}

/// Simulated callback registration
struct CallbackRegistry {
    update_callbacks: Vec<(Handle, Box<dyn Fn(f32) -> i32>)>,
    event_callbacks: HashMap<i32, Vec<Box<dyn Fn(*const c_void, usize) -> i32>>>,
}

impl CallbackRegistry {
    fn new() -> Self {
        Self {
            update_callbacks: Vec::new(),
            event_callbacks: HashMap::new(),
        }
    }

    fn register_update(&mut self, handle: Handle, callback: Box<dyn Fn(f32) -> i32>) {
        self.update_callbacks.push((handle, callback));
    }

    fn invoke_updates(&self, delta_time: f32) -> Vec<i32> {
        self.update_callbacks
            .iter()
            .map(|(_, cb)| cb(delta_time))
            .collect()
    }
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

fn bench_handle_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("handle_operations");

    // Handle creation
    group.bench_function("handle_creation_10000", |b| {
        b.iter(|| {
            let handles: Vec<Handle> = (1..=10000).map(Handle::new).collect();
            black_box(handles)
        })
    });

    // Handle validation
    let handles: Vec<Handle> = (0..10000).map(Handle::new).collect();
    group.bench_function("handle_validation_10000", |b| {
        b.iter(|| {
            let valid: Vec<bool> = handles.iter().map(|h| h.is_valid()).collect();
            black_box(valid)
        })
    });

    // Handle lookup in HashMap
    let mut map: HashMap<Handle, u32> = HashMap::new();
    for i in 0..10000u64 {
        map.insert(Handle::new(i + 1), i as u32);
    }

    group.bench_function("handle_lookup_10000", |b| {
        let lookup_handles: Vec<Handle> = (1..=10000).map(Handle::new).collect();
        b.iter(|| {
            let results: Vec<Option<&u32>> = lookup_handles.iter().map(|h| map.get(h)).collect();
            black_box(results)
        })
    });

    group.finish();
}

fn bench_data_marshalling(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_marshalling");

    // Vec3 marshalling
    for count in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("vec3_round_trip", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let vectors: Vec<Vec3> = (0..count)
                        .map(|i| Vec3::new(i as f32, (i as f32).sin(), (i as f32).cos()))
                        .collect();

                    // Simulate FFI boundary (copy to raw bytes and back)
                    let bytes: Vec<[u8; 12]> = vectors
                        .iter()
                        .map(|v| {
                            let mut buf = [0u8; 12];
                            buf[0..4].copy_from_slice(&v.x.to_le_bytes());
                            buf[4..8].copy_from_slice(&v.y.to_le_bytes());
                            buf[8..12].copy_from_slice(&v.z.to_le_bytes());
                            buf
                        })
                        .collect();

                    let reconstructed: Vec<Vec3> = bytes
                        .iter()
                        .map(|b| Vec3 {
                            x: f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                            y: f32::from_le_bytes([b[4], b[5], b[6], b[7]]),
                            z: f32::from_le_bytes([b[8], b[9], b[10], b[11]]),
                        })
                        .collect();

                    black_box(reconstructed)
                })
            },
        );
    }

    // Transform marshalling
    group.bench_function("transform_round_trip_1000", |b| {
        b.iter(|| {
            let transforms: Vec<Transform> =
                (0..1000).map(|i| Transform::new(i as f32, 0.0, i as f32 * 0.5)).collect();

            // Simulate FFI boundary (48 bytes per Transform)
            let bytes: Vec<[u8; 48]> = transforms
                .iter()
                .map(|t| {
                    let mut buf = [0u8; 48];
                    buf[0..4].copy_from_slice(&t.position.x.to_le_bytes());
                    buf[4..8].copy_from_slice(&t.position.y.to_le_bytes());
                    buf[8..12].copy_from_slice(&t.position.z.to_le_bytes());
                    buf[12..16].copy_from_slice(&t.rotation.x.to_le_bytes());
                    buf[16..20].copy_from_slice(&t.rotation.y.to_le_bytes());
                    buf[20..24].copy_from_slice(&t.rotation.z.to_le_bytes());
                    buf[24..28].copy_from_slice(&t.rotation.w.to_le_bytes());
                    buf[28..32].copy_from_slice(&t.scale.x.to_le_bytes());
                    buf[32..36].copy_from_slice(&t.scale.y.to_le_bytes());
                    buf[36..40].copy_from_slice(&t.scale.z.to_le_bytes());
                    // Padding for alignment
                    buf
                })
                .collect();

            black_box(bytes)
        })
    });

    group.finish();
}

fn bench_string_marshalling(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_marshalling");

    // String to CString conversion
    let strings: Vec<String> = (0..1000)
        .map(|i| format!("Entity_{}_{}", i, "test_name"))
        .collect();

    group.throughput(Throughput::Elements(1000));

    group.bench_function("string_to_cstring_1000", |b| {
        b.iter(|| {
            let cstrings: Vec<CString> = strings.iter().map(|s| marshal_string_to_c(s)).collect();
            black_box(cstrings)
        })
    });

    // CString to String conversion
    let cstrings: Vec<CString> = strings.iter().map(|s| marshal_string_to_c(s)).collect();
    let ptrs: Vec<*const c_char> = cstrings.iter().map(|cs| cs.as_ptr()).collect();

    group.bench_function("cstring_to_string_1000", |b| {
        b.iter(|| {
            let results: Vec<Option<String>> =
                ptrs.iter().map(|&p| marshal_string_from_c(p)).collect();
            black_box(results)
        })
    });

    // Long string handling
    let long_strings: Vec<String> = (0..100)
        .map(|i| format!("{}_{}", "x".repeat(1000), i))
        .collect();

    group.bench_function("long_string_marshal_100", |b| {
        b.iter(|| {
            let cstrings: Vec<CString> =
                long_strings.iter().map(|s| marshal_string_to_c(s)).collect();
            black_box(cstrings)
        })
    });

    // String cache lookup simulation
    let mut cache: HashMap<String, CString> = HashMap::new();
    for s in &strings {
        cache.insert(s.clone(), marshal_string_to_c(s));
    }

    group.bench_function("cached_string_lookup_1000", |b| {
        b.iter(|| {
            let results: Vec<Option<&CString>> = strings.iter().map(|s| cache.get(s)).collect();
            black_box(results)
        })
    });

    group.finish();
}

fn bench_entity_lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_lifecycle");

    for count in [100, 500, 1000] {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("create_destroy_cycle", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let mut ctx = SdkContext::new();
                    ctx.initialize();

                    // Create entities
                    let handles: Vec<Handle> = (0..count)
                        .filter_map(|i| {
                            let params = EntityParams {
                                transform: Transform::new(i as f32, 0.0, 0.0),
                                ..Default::default()
                            };
                            ctx.create_entity(&params).ok()
                        })
                        .collect();

                    // Destroy all
                    for handle in &handles {
                        ctx.destroy_entity(*handle);
                    }

                    black_box(handles.len())
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("transform_update_cycle", count),
            &count,
            |b, &count| {
                let mut ctx = SdkContext::new();
                ctx.initialize();

                let handles: Vec<Handle> = (0..count)
                    .filter_map(|i| {
                        let params = EntityParams {
                            transform: Transform::new(i as f32, 0.0, 0.0),
                            ..Default::default()
                        };
                        ctx.create_entity(&params).ok()
                    })
                    .collect();

                b.iter(|| {
                    // Update all transforms
                    for (i, handle) in handles.iter().enumerate() {
                        let transform = Transform::new(i as f32 + 0.1, 0.5, 0.0);
                        ctx.set_transform(*handle, transform);
                    }

                    // Read all transforms
                    let transforms: Vec<Transform> =
                        handles.iter().filter_map(|h| ctx.get_transform(*h).ok()).collect();

                    black_box(transforms)
                })
            },
        );
    }

    group.finish();
}

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    for batch_size in [10, 50, 100, 500] {
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_with_input(
            BenchmarkId::new("batched_create", batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    let mut ctx = SdkContext::new();
                    ctx.initialize();

                    let mut batch = BatchOperation::new();
                    for i in 0..size {
                        batch.add_create(EntityParams {
                            transform: Transform::new(i as f32, 0.0, 0.0),
                            ..Default::default()
                        });
                    }

                    let results = batch.execute(&mut ctx);
                    black_box(results)
                })
            },
        );

        // Mixed batch operations
        group.bench_with_input(
            BenchmarkId::new("mixed_batch", batch_size),
            &batch_size,
            |b, &size| {
                let mut ctx = SdkContext::new();
                ctx.initialize();

                // Pre-create entities
                let handles: Vec<Handle> = (0..size)
                    .filter_map(|i| {
                        ctx.create_entity(&EntityParams {
                            transform: Transform::new(i as f32, 0.0, 0.0),
                            ..Default::default()
                        })
                        .ok()
                    })
                    .collect();

                b.iter(|| {
                    let mut batch = BatchOperation::new();

                    // Mix of operations
                    for (i, handle) in handles.iter().enumerate() {
                        if i % 3 == 0 {
                            batch.add_set_transform(*handle, Transform::new(i as f32 + 1.0, 0.0, 0.0));
                        } else if i % 3 == 1 {
                            batch.add_create(EntityParams::default());
                        }
                        // i % 3 == 2: no-op
                    }

                    let results = batch.execute(&mut ctx);
                    black_box(results)
                })
            },
        );
    }

    group.finish();
}

fn bench_callback_invocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("callback_invocation");

    for callback_count in [10, 100, 1000] {
        group.throughput(Throughput::Elements(callback_count as u64));

        group.bench_with_input(
            BenchmarkId::new("invoke_update_callbacks", callback_count),
            &callback_count,
            |b, &count| {
                let mut registry = CallbackRegistry::new();

                for i in 0..count {
                    let handle = Handle::new(i as u64);
                    registry.register_update(
                        handle,
                        Box::new(move |dt| {
                            // Simulate simple callback work
                            let _ = black_box(dt * i as f32);
                            0
                        }),
                    );
                }

                b.iter(|| {
                    let results = registry.invoke_updates(0.016);
                    black_box(results)
                })
            },
        );
    }

    // Heavy callback work simulation
    group.bench_function("heavy_callback_100", |b| {
        let mut registry = CallbackRegistry::new();

        for i in 0..100 {
            let handle = Handle::new(i as u64);
            registry.register_update(
                handle,
                Box::new(move |dt| {
                    // Heavier work in callback
                    let mut sum = 0.0f32;
                    for j in 0..100 {
                        sum += (j as f32 * dt).sin();
                    }
                    let _ = black_box(sum);
                    0
                }),
            );
        }

        b.iter(|| {
            let results = registry.invoke_updates(0.016);
            black_box(results)
        })
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    // Error propagation overhead
    group.bench_function("error_propagation_chain", |b| {
        fn level_3() -> Result<i32, SdkError> {
            Ok(42)
        }

        fn level_2() -> Result<i32, SdkError> {
            level_3()
        }

        fn level_1() -> Result<i32, SdkError> {
            level_2()
        }

        b.iter(|| {
            let results: Vec<Result<i32, SdkError>> = (0..1000).map(|_| level_1()).collect();
            black_box(results)
        })
    });

    // Error vs success comparison
    group.bench_function("error_vs_success_1000", |b| {
        let mut ctx = SdkContext::new();
        ctx.initialize();

        // Create some valid handles
        let valid_handles: Vec<Handle> = (0..500)
            .filter_map(|_| ctx.create_entity(&EntityParams::default()).ok())
            .collect();

        // Mix of valid and invalid handles
        let mixed_handles: Vec<Handle> = (0..1000)
            .map(|i| {
                if i < 500 {
                    valid_handles[i]
                } else {
                    Handle::new(99999 + i as u64)
                }
            })
            .collect();

        b.iter(|| {
            let results: Vec<Result<Transform, SdkError>> =
                mixed_handles.iter().map(|h| ctx.get_transform(*h)).collect();
            black_box(results)
        })
    });

    // Error code conversion
    group.bench_function("error_code_to_string_1000", |b| {
        let errors = [
            SdkError::Success,
            SdkError::InvalidParameter,
            SdkError::OutOfMemory,
            SdkError::NotInitialized,
            SdkError::InvalidHandle,
            SdkError::OperationFailed,
        ];

        b.iter(|| {
            let strings: Vec<&str> = (0..1000)
                .map(|i| match errors[i % errors.len()] {
                    SdkError::Success => "Success",
                    SdkError::InvalidParameter => "Invalid Parameter",
                    SdkError::OutOfMemory => "Out of Memory",
                    SdkError::NotInitialized => "Not Initialized",
                    SdkError::AlreadyInitialized => "Already Initialized",
                    SdkError::InvalidHandle => "Invalid Handle",
                    SdkError::OperationFailed => "Operation Failed",
                    SdkError::BufferTooSmall => "Buffer Too Small",
                    SdkError::Timeout => "Timeout",
                    SdkError::NotFound => "Not Found",
                })
                .collect();
            black_box(strings)
        })
    });

    group.finish();
}

fn bench_vector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_operations");

    // These would typically be exposed as SDK utility functions
    let vectors: Vec<Vec3> = (0..10000)
        .map(|i| Vec3::new(i as f32, (i as f32).sin() * 100.0, (i as f32).cos() * 100.0))
        .collect();

    group.throughput(Throughput::Elements(10000));

    group.bench_function("vec3_length_10000", |b| {
        b.iter(|| {
            let lengths: Vec<f32> = vectors.iter().map(|v| v.length()).collect();
            black_box(lengths)
        })
    });

    group.bench_function("vec3_normalize_10000", |b| {
        b.iter(|| {
            let normalized: Vec<Vec3> = vectors.iter().map(|v| v.normalize()).collect();
            black_box(normalized)
        })
    });

    // Cross-product simulation
    group.bench_function("vec3_operations_combined_10000", |b| {
        b.iter(|| {
            let results: Vec<(f32, Vec3)> = vectors
                .iter()
                .map(|v| {
                    let len = v.length();
                    let norm = v.normalize();
                    (len, norm)
                })
                .collect();
            black_box(results)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_handle_operations,
    bench_data_marshalling,
    bench_string_marshalling,
    bench_entity_lifecycle,
    bench_batch_operations,
    bench_callback_invocation,
    bench_error_handling,
    bench_vector_operations,
);

criterion_main!(benches);
