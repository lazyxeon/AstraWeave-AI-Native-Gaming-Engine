//! Adversarial benchmarks for astraweave-net-ecs
//!
//! Tests network ECS integration under extreme conditions:
//! - Entity state synchronization at scale
//! - Component replication throughput
//! - Network serialization performance
//! - Delta compression efficiency
//! - Packet batching optimization
//! - Interest management overhead

#![allow(dead_code, unused_imports)]

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use std::collections::{HashMap, HashSet};
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPE DEFINITIONS (Standalone benchmark - no crate imports)
// ============================================================================

/// Network entity identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NetEntityId(u64);

/// Network component types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum ComponentType {
    Transform = 0,
    Velocity = 1,
    Health = 2,
    Input = 3,
    Inventory = 4,
    Animation = 5,
    Physics = 6,
    AI = 7,
}

/// Transform component for network sync
#[derive(Clone, Copy, Debug, Default)]
struct NetworkTransform {
    x: f32,
    y: f32,
    z: f32,
    rot_x: f32,
    rot_y: f32,
    rot_z: f32,
    rot_w: f32,
    scale: f32,
}

impl NetworkTransform {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            rot_x: 0.0,
            rot_y: 0.0,
            rot_z: 0.0,
            rot_w: 1.0,
            scale: 1.0,
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32);
        bytes.extend_from_slice(&self.x.to_le_bytes());
        bytes.extend_from_slice(&self.y.to_le_bytes());
        bytes.extend_from_slice(&self.z.to_le_bytes());
        bytes.extend_from_slice(&self.rot_x.to_le_bytes());
        bytes.extend_from_slice(&self.rot_y.to_le_bytes());
        bytes.extend_from_slice(&self.rot_z.to_le_bytes());
        bytes.extend_from_slice(&self.rot_w.to_le_bytes());
        bytes.extend_from_slice(&self.scale.to_le_bytes());
        bytes
    }

    fn deserialize(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 32 {
            return None;
        }
        Some(Self {
            x: f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            y: f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            z: f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            rot_x: f32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            rot_y: f32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            rot_z: f32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            rot_w: f32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
            scale: f32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
        })
    }

    fn delta(&self, other: &Self) -> Option<TransformDelta> {
        let pos_diff = (self.x - other.x).abs()
            + (self.y - other.y).abs()
            + (self.z - other.z).abs();
        let rot_diff = (self.rot_x - other.rot_x).abs()
            + (self.rot_y - other.rot_y).abs()
            + (self.rot_z - other.rot_z).abs()
            + (self.rot_w - other.rot_w).abs();

        if pos_diff < 0.001 && rot_diff < 0.001 {
            return None;
        }

        Some(TransformDelta {
            has_position: pos_diff >= 0.001,
            has_rotation: rot_diff >= 0.001,
            position: if pos_diff >= 0.001 {
                Some((self.x, self.y, self.z))
            } else {
                None
            },
            rotation: if rot_diff >= 0.001 {
                Some((self.rot_x, self.rot_y, self.rot_z, self.rot_w))
            } else {
                None
            },
        })
    }
}

/// Delta-compressed transform
#[derive(Clone, Debug)]
struct TransformDelta {
    has_position: bool,
    has_rotation: bool,
    position: Option<(f32, f32, f32)>,
    rotation: Option<(f32, f32, f32, f32)>,
}

impl TransformDelta {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(29);
        let flags = (self.has_position as u8) | ((self.has_rotation as u8) << 1);
        bytes.push(flags);

        if let Some((x, y, z)) = self.position {
            bytes.extend_from_slice(&x.to_le_bytes());
            bytes.extend_from_slice(&y.to_le_bytes());
            bytes.extend_from_slice(&z.to_le_bytes());
        }

        if let Some((x, y, z, w)) = self.rotation {
            bytes.extend_from_slice(&x.to_le_bytes());
            bytes.extend_from_slice(&y.to_le_bytes());
            bytes.extend_from_slice(&z.to_le_bytes());
            bytes.extend_from_slice(&w.to_le_bytes());
        }

        bytes
    }
}

/// Velocity component
#[derive(Clone, Copy, Debug, Default)]
struct NetworkVelocity {
    vx: f32,
    vy: f32,
    vz: f32,
    angular_x: f32,
    angular_y: f32,
    angular_z: f32,
}

impl NetworkVelocity {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(24);
        bytes.extend_from_slice(&self.vx.to_le_bytes());
        bytes.extend_from_slice(&self.vy.to_le_bytes());
        bytes.extend_from_slice(&self.vz.to_le_bytes());
        bytes.extend_from_slice(&self.angular_x.to_le_bytes());
        bytes.extend_from_slice(&self.angular_y.to_le_bytes());
        bytes.extend_from_slice(&self.angular_z.to_le_bytes());
        bytes
    }
}

/// Health component
#[derive(Clone, Copy, Debug, Default)]
struct NetworkHealth {
    current: f32,
    max: f32,
    regeneration: f32,
    shield: f32,
}

impl NetworkHealth {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&self.current.to_le_bytes());
        bytes.extend_from_slice(&self.max.to_le_bytes());
        bytes.extend_from_slice(&self.regeneration.to_le_bytes());
        bytes.extend_from_slice(&self.shield.to_le_bytes());
        bytes
    }
}

/// Network entity with all components
#[derive(Clone, Debug)]
struct NetworkEntity {
    id: NetEntityId,
    owner_id: u32,
    transform: NetworkTransform,
    velocity: NetworkVelocity,
    health: NetworkHealth,
    dirty_components: HashSet<ComponentType>,
    last_update_tick: u64,
}

impl NetworkEntity {
    fn new(id: u64, owner: u32) -> Self {
        Self {
            id: NetEntityId(id),
            owner_id: owner,
            transform: NetworkTransform::default(),
            velocity: NetworkVelocity::default(),
            health: NetworkHealth {
                current: 100.0,
                max: 100.0,
                regeneration: 0.5,
                shield: 0.0,
            },
            dirty_components: HashSet::new(),
            last_update_tick: 0,
        }
    }

    fn mark_dirty(&mut self, component: ComponentType) {
        self.dirty_components.insert(component);
    }

    fn clear_dirty(&mut self) {
        self.dirty_components.clear();
    }

    fn serialize_dirty(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.0.to_le_bytes());
        bytes.push(self.dirty_components.len() as u8);

        for &component in &self.dirty_components {
            bytes.push(component as u8);
            match component {
                ComponentType::Transform => bytes.extend(self.transform.serialize()),
                ComponentType::Velocity => bytes.extend(self.velocity.serialize()),
                ComponentType::Health => bytes.extend(self.health.serialize()),
                _ => {}
            }
        }

        bytes
    }

    fn serialize_full(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(128);
        bytes.extend_from_slice(&self.id.0.to_le_bytes());
        bytes.extend_from_slice(&self.owner_id.to_le_bytes());
        bytes.extend(self.transform.serialize());
        bytes.extend(self.velocity.serialize());
        bytes.extend(self.health.serialize());
        bytes.extend_from_slice(&self.last_update_tick.to_le_bytes());
        bytes
    }
}

/// Network packet for entity updates
#[derive(Clone, Debug)]
struct EntityUpdatePacket {
    sequence: u32,
    tick: u64,
    entity_updates: Vec<Vec<u8>>,
    compressed: bool,
}

impl EntityUpdatePacket {
    fn new(sequence: u32, tick: u64) -> Self {
        Self {
            sequence,
            tick,
            entity_updates: Vec::new(),
            compressed: false,
        }
    }

    fn add_entity_update(&mut self, entity: &NetworkEntity) {
        self.entity_updates.push(entity.serialize_dirty());
    }

    fn add_full_entity(&mut self, entity: &NetworkEntity) {
        self.entity_updates.push(entity.serialize_full());
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes.extend_from_slice(&self.tick.to_le_bytes());
        bytes.push(self.compressed as u8);
        bytes.extend_from_slice(&(self.entity_updates.len() as u32).to_le_bytes());

        for update in &self.entity_updates {
            bytes.extend_from_slice(&(update.len() as u32).to_le_bytes());
            bytes.extend(update);
        }

        bytes
    }

    fn byte_size(&self) -> usize {
        self.serialize().len()
    }
}

/// Interest management for relevancy filtering
struct InterestManager {
    view_distance: f32,
    max_entities_per_client: usize,
    entity_positions: HashMap<NetEntityId, (f32, f32, f32)>,
    client_positions: HashMap<u32, (f32, f32, f32)>,
}

impl InterestManager {
    fn new(view_distance: f32, max_entities: usize) -> Self {
        Self {
            view_distance,
            max_entities_per_client: max_entities,
            entity_positions: HashMap::new(),
            client_positions: HashMap::new(),
        }
    }

    fn update_entity_position(&mut self, id: NetEntityId, x: f32, y: f32, z: f32) {
        self.entity_positions.insert(id, (x, y, z));
    }

    fn update_client_position(&mut self, client_id: u32, x: f32, y: f32, z: f32) {
        self.client_positions.insert(client_id, (x, y, z));
    }

    fn get_relevant_entities(&self, client_id: u32) -> Vec<NetEntityId> {
        let Some(&(cx, cy, cz)) = self.client_positions.get(&client_id) else {
            return Vec::new();
        };

        let mut relevant: Vec<(NetEntityId, f32)> = self
            .entity_positions
            .iter()
            .filter_map(|(&id, &(ex, ey, ez))| {
                let dx = ex - cx;
                let dy = ey - cy;
                let dz = ez - cz;
                let dist_sq = dx * dx + dy * dy + dz * dz;

                if dist_sq <= self.view_distance * self.view_distance {
                    Some((id, dist_sq))
                } else {
                    None
                }
            })
            .collect();

        // Sort by distance and limit
        relevant.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        relevant
            .into_iter()
            .take(self.max_entities_per_client)
            .map(|(id, _)| id)
            .collect()
    }
}

/// Snapshot interpolation buffer
struct SnapshotBuffer {
    snapshots: Vec<(u64, HashMap<NetEntityId, NetworkTransform>)>,
    max_snapshots: usize,
    interpolation_delay: u64,
}

impl SnapshotBuffer {
    fn new(max_snapshots: usize, delay: u64) -> Self {
        Self {
            snapshots: Vec::with_capacity(max_snapshots),
            max_snapshots,
            interpolation_delay: delay,
        }
    }

    fn add_snapshot(&mut self, tick: u64, transforms: HashMap<NetEntityId, NetworkTransform>) {
        self.snapshots.push((tick, transforms));
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    fn interpolate(&self, current_tick: u64, entity_id: NetEntityId) -> Option<NetworkTransform> {
        let target_tick = current_tick.saturating_sub(self.interpolation_delay);

        // Find surrounding snapshots
        let mut before: Option<(u64, &NetworkTransform)> = None;
        let mut after: Option<(u64, &NetworkTransform)> = None;

        for (tick, transforms) in &self.snapshots {
            if let Some(transform) = transforms.get(&entity_id) {
                if *tick <= target_tick {
                    before = Some((*tick, transform));
                } else if after.is_none() {
                    after = Some((*tick, transform));
                    break;
                }
            }
        }

        match (before, after) {
            (Some((t0, tr0)), Some((t1, tr1))) => {
                let alpha = if t1 != t0 {
                    (target_tick - t0) as f32 / (t1 - t0) as f32
                } else {
                    0.0
                };
                Some(NetworkTransform {
                    x: tr0.x + (tr1.x - tr0.x) * alpha,
                    y: tr0.y + (tr1.y - tr0.y) * alpha,
                    z: tr0.z + (tr1.z - tr0.z) * alpha,
                    rot_x: tr0.rot_x + (tr1.rot_x - tr0.rot_x) * alpha,
                    rot_y: tr0.rot_y + (tr1.rot_y - tr0.rot_y) * alpha,
                    rot_z: tr0.rot_z + (tr1.rot_z - tr0.rot_z) * alpha,
                    rot_w: tr0.rot_w + (tr1.rot_w - tr0.rot_w) * alpha,
                    scale: tr0.scale + (tr1.scale - tr0.scale) * alpha,
                })
            }
            (Some((_, tr)), None) | (None, Some((_, tr))) => Some(*tr),
            (None, None) => None,
        }
    }
}

/// Simple LZ4-style compression for benchmarking
fn compress_simple(data: &[u8]) -> Vec<u8> {
    // Very simple RLE compression for benchmarking
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let mut run_length: u8 = 1;
        while i + (run_length as usize) < data.len()
            && data[i] == data[i + run_length as usize]
            && run_length < 255
        {
            run_length += 1;
        }

        if run_length > 3 {
            result.push(0xFF); // Escape byte
            result.push(run_length);
            result.push(data[i]);
            i += run_length as usize;
        } else {
            if data[i] == 0xFF {
                result.push(0xFF);
                result.push(1);
            }
            result.push(data[i]);
            i += 1;
        }
    }

    result
}

fn decompress_simple(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < data.len() {
        if data[i] == 0xFF && i + 1 < data.len() {
            let count = data[i + 1];
            if count == 1 && i + 2 >= data.len() {
                // Escaped 0xFF
                result.push(0xFF);
                i += 2;
            } else if i + 2 < data.len() {
                // RLE
                let byte = data[i + 2];
                for _ in 0..count {
                    result.push(byte);
                }
                i += 3;
            } else {
                break;
            }
        } else {
            result.push(data[i]);
            i += 1;
        }
    }

    result
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

fn bench_entity_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_serialization");

    for entity_count in [10, 100, 1000, 10000] {
        group.throughput(Throughput::Elements(entity_count as u64));

        let entities: Vec<NetworkEntity> = (0..entity_count)
            .map(|i| {
                let mut entity = NetworkEntity::new(i as u64, (i % 100) as u32);
                entity.transform = NetworkTransform::new(
                    (i as f32) * 1.5,
                    ((i * 7) as f32).sin() * 100.0,
                    (i as f32) * 0.5,
                );
                entity.velocity = NetworkVelocity {
                    vx: (i as f32).cos(),
                    vy: 0.0,
                    vz: (i as f32).sin(),
                    angular_x: 0.0,
                    angular_y: (i as f32) * 0.01,
                    angular_z: 0.0,
                };
                entity.mark_dirty(ComponentType::Transform);
                entity.mark_dirty(ComponentType::Velocity);
                entity
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("full_serialize", entity_count),
            &entities,
            |b, entities| {
                b.iter(|| {
                    let bytes: Vec<Vec<u8>> =
                        entities.iter().map(|e| e.serialize_full()).collect();
                    std_black_box(bytes)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("dirty_serialize", entity_count),
            &entities,
            |b, entities| {
                b.iter(|| {
                    let bytes: Vec<Vec<u8>> =
                        entities.iter().map(|e| e.serialize_dirty()).collect();
                    std_black_box(bytes)
                })
            },
        );
    }

    // Transform delta compression
    group.bench_function("delta_compression_1000", |b| {
        let transforms_old: Vec<NetworkTransform> = (0..1000)
            .map(|i| NetworkTransform::new(i as f32, 0.0, 0.0))
            .collect();
        let transforms_new: Vec<NetworkTransform> = (0..1000)
            .map(|i| NetworkTransform::new(i as f32 + 0.1, 0.5, 0.0))
            .collect();

        b.iter(|| {
            let deltas: Vec<Option<TransformDelta>> = transforms_old
                .iter()
                .zip(transforms_new.iter())
                .map(|(old, new)| new.delta(old))
                .collect();
            std_black_box(deltas)
        })
    });

    group.finish();
}

fn bench_packet_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_batching");

    for batch_size in [10, 50, 100, 500] {
        group.throughput(Throughput::Elements(batch_size as u64));

        let entities: Vec<NetworkEntity> = (0..batch_size)
            .map(|i| {
                let mut entity = NetworkEntity::new(i as u64, 0);
                entity.transform = NetworkTransform::new(i as f32, 0.0, 0.0);
                entity.mark_dirty(ComponentType::Transform);
                entity
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("create_packet", batch_size),
            &entities,
            |b, entities| {
                b.iter(|| {
                    let mut packet = EntityUpdatePacket::new(0, 0);
                    for entity in entities {
                        packet.add_entity_update(entity);
                    }
                    std_black_box(packet.serialize())
                })
            },
        );
    }

    // Multiple packets per frame simulation
    group.bench_function("multi_packet_frame_100_entities_4_clients", |b| {
        let entities: Vec<NetworkEntity> = (0..100)
            .map(|i| {
                let mut entity = NetworkEntity::new(i as u64, (i % 4) as u32);
                entity.transform = NetworkTransform::new(i as f32, 0.0, 0.0);
                entity.mark_dirty(ComponentType::Transform);
                entity
            })
            .collect();

        b.iter(|| {
            let packets: Vec<Vec<u8>> = (0..4)
                .map(|client_id| {
                    let mut packet = EntityUpdatePacket::new(0, 0);
                    for entity in entities.iter().filter(|e| e.owner_id != client_id) {
                        packet.add_entity_update(entity);
                    }
                    packet.serialize()
                })
                .collect();
            std_black_box(packets)
        })
    });

    group.finish();
}

fn bench_interest_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("interest_management");

    for entity_count in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(entity_count as u64));

        let mut manager = InterestManager::new(100.0, 50);

        // Setup entities in a grid
        for i in 0..entity_count {
            let x = (i % 100) as f32 * 10.0;
            let z = (i / 100) as f32 * 10.0;
            manager.update_entity_position(NetEntityId(i as u64), x, 0.0, z);
        }

        // Setup client at center
        manager.update_client_position(0, 500.0, 0.0, 500.0);

        group.bench_with_input(
            BenchmarkId::new("get_relevant", entity_count),
            &manager,
            |b, manager| {
                b.iter(|| std_black_box(manager.get_relevant_entities(0)))
            },
        );
    }

    // Multi-client interest calculation
    group.bench_function("multi_client_interest_1000_entities_16_clients", |b| {
        let mut manager = InterestManager::new(100.0, 50);

        for i in 0..1000 {
            let x = (i % 32) as f32 * 10.0;
            let z = (i / 32) as f32 * 10.0;
            manager.update_entity_position(NetEntityId(i as u64), x, 0.0, z);
        }

        for client_id in 0..16u32 {
            let x = (client_id % 4) as f32 * 80.0 + 40.0;
            let z = (client_id / 4) as f32 * 80.0 + 40.0;
            manager.update_client_position(client_id, x, 0.0, z);
        }

        b.iter(|| {
            let relevance: Vec<Vec<NetEntityId>> = (0..16)
                .map(|client_id| manager.get_relevant_entities(client_id))
                .collect();
            std_black_box(relevance)
        })
    });

    group.finish();
}

fn bench_snapshot_interpolation(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_interpolation");

    for snapshot_count in [10, 32, 64, 128] {
        let mut buffer = SnapshotBuffer::new(snapshot_count, 3);

        // Fill buffer with snapshots
        for tick in 0..snapshot_count as u64 {
            let transforms: HashMap<NetEntityId, NetworkTransform> = (0..100)
                .map(|i| {
                    let id = NetEntityId(i);
                    let transform = NetworkTransform::new(
                        i as f32 + tick as f32 * 0.1,
                        (tick as f32).sin() * 10.0,
                        i as f32 * 0.5,
                    );
                    (id, transform)
                })
                .collect();
            buffer.add_snapshot(tick, transforms);
        }

        group.bench_with_input(
            BenchmarkId::new("interpolate_100_entities", snapshot_count),
            &buffer,
            |b, buffer| {
                let current_tick = snapshot_count as u64 - 1;
                b.iter(|| {
                    let interpolated: Vec<Option<NetworkTransform>> = (0..100)
                        .map(|i| buffer.interpolate(current_tick, NetEntityId(i)))
                        .collect();
                    std_black_box(interpolated)
                })
            },
        );
    }

    group.finish();
}

fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    // Generate typical network data
    let entity_data: Vec<Vec<u8>> = (0..100)
        .map(|i| {
            let entity = NetworkEntity::new(i as u64, 0);
            entity.serialize_full()
        })
        .collect();

    let combined_data: Vec<u8> = entity_data.iter().flatten().copied().collect();

    group.throughput(Throughput::Bytes(combined_data.len() as u64));

    group.bench_function("compress_entity_batch", |b| {
        b.iter(|| std_black_box(compress_simple(&combined_data)))
    });

    let compressed = compress_simple(&combined_data);
    group.bench_function("decompress_entity_batch", |b| {
        b.iter(|| std_black_box(decompress_simple(&compressed)))
    });

    // Compression ratio test
    group.bench_function("compression_ratio_analysis", |b| {
        b.iter(|| {
            let compressed = compress_simple(&combined_data);
            let ratio = combined_data.len() as f32 / compressed.len() as f32;
            std_black_box(ratio)
        })
    });

    // Delta + compression combined
    group.bench_function("delta_then_compress_100", |b| {
        let transforms_old: Vec<NetworkTransform> = (0..100)
            .map(|i| NetworkTransform::new(i as f32, 0.0, 0.0))
            .collect();
        let transforms_new: Vec<NetworkTransform> = (0..100)
            .map(|i| NetworkTransform::new(i as f32 + 0.05, 0.1, 0.0))
            .collect();

        b.iter(|| {
            let deltas: Vec<u8> = transforms_old
                .iter()
                .zip(transforms_new.iter())
                .filter_map(|(old, new)| new.delta(old))
                .flat_map(|d| d.serialize())
                .collect();

            let compressed = compress_simple(&deltas);
            std_black_box(compressed)
        })
    });

    group.finish();
}

fn bench_state_synchronization(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_synchronization");

    // Full world state sync
    for entity_count in [100, 500, 1000] {
        group.throughput(Throughput::Elements(entity_count as u64));

        let entities: Vec<NetworkEntity> = (0..entity_count)
            .map(|i| {
                let mut entity = NetworkEntity::new(i as u64, (i % 32) as u32);
                entity.transform = NetworkTransform::new(
                    (i as f32) * 1.5,
                    ((i * 7) as f32).sin() * 50.0,
                    (i as f32) * 0.5,
                );
                entity
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("full_world_sync", entity_count),
            &entities,
            |b, entities| {
                b.iter(|| {
                    let mut packet = EntityUpdatePacket::new(0, 0);
                    for entity in entities {
                        packet.add_full_entity(entity);
                    }
                    let bytes = packet.serialize();
                    let compressed = compress_simple(&bytes);
                    std_black_box(compressed)
                })
            },
        );

        // Partial update (20% dirty)
        let dirty_entities: Vec<NetworkEntity> = entities
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 5 == 0)
            .map(|(_, e)| {
                let mut entity = e.clone();
                entity.mark_dirty(ComponentType::Transform);
                entity
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("partial_update_20pct", entity_count),
            &dirty_entities,
            |b, entities| {
                b.iter(|| {
                    let mut packet = EntityUpdatePacket::new(0, 0);
                    for entity in entities {
                        packet.add_entity_update(entity);
                    }
                    let bytes = packet.serialize();
                    std_black_box(bytes)
                })
            },
        );
    }

    group.finish();
}

fn bench_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialization");

    // Pre-serialize transforms
    let serialized_transforms: Vec<Vec<u8>> = (0..1000)
        .map(|i| {
            NetworkTransform::new(i as f32, (i as f32).sin() * 10.0, i as f32 * 0.5).serialize()
        })
        .collect();

    group.throughput(Throughput::Elements(1000));

    group.bench_function("deserialize_transforms_1000", |b| {
        b.iter(|| {
            let transforms: Vec<Option<NetworkTransform>> = serialized_transforms
                .iter()
                .map(|bytes| NetworkTransform::deserialize(bytes))
                .collect();
            std_black_box(transforms)
        })
    });

    // Packet deserialization simulation
    let packet_bytes: Vec<u8> = {
        let mut packet = EntityUpdatePacket::new(12345, 67890);
        for i in 0..100 {
            let entity = NetworkEntity::new(i, 0);
            packet.add_full_entity(&entity);
        }
        packet.serialize()
    };

    group.bench_function("parse_packet_header", |b| {
        b.iter(|| {
            let sequence = u32::from_le_bytes([
                packet_bytes[0],
                packet_bytes[1],
                packet_bytes[2],
                packet_bytes[3],
            ]);
            let tick = u64::from_le_bytes([
                packet_bytes[4],
                packet_bytes[5],
                packet_bytes[6],
                packet_bytes[7],
                packet_bytes[8],
                packet_bytes[9],
                packet_bytes[10],
                packet_bytes[11],
            ]);
            std_black_box((sequence, tick))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_entity_serialization,
    bench_packet_batching,
    bench_interest_management,
    bench_snapshot_interpolation,
    bench_compression,
    bench_state_synchronization,
    bench_deserialization,
);

criterion_main!(benches);
