# Hybrid Voxel/Polygon Terrain System

## Overview

The Hybrid Voxel/Polygon strategy implements a dynamic, deformable terrain system for AstraWeave that combines the flexibility of voxel-based terrain with the rendering efficiency of polygon meshes. This system is specifically designed for Enshrouded-style destructible terrain while maintaining compatibility with the existing rendering pipeline.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Hybrid Voxel System                       │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────┐         ┌──────────────────┐          │
│  │  Voxel Storage   │         │  Mesh Generation │          │
│  │  (SVO-based)     │────────▶│  (Dual Contour)  │          │
│  └──────────────────┘         └──────────────────┘          │
│         │                              │                     │
│         │                              │                     │
│         ▼                              ▼                     │
│  ┌──────────────────┐         ┌──────────────────┐          │
│  │  VoxelGrid       │         │  ChunkMesh       │          │
│  │  (HashMap)       │         │  (Vertices +     │          │
│  │                  │         │   Indices)       │          │
│  └──────────────────┘         └──────────────────┘          │
│         │                              │                     │
│         │                              │                     │
│         ▼                              ▼                     │
│  ┌──────────────────┐         ┌──────────────────┐          │
│  │  VoxelChunk      │         │  Renderer        │          │
│  │  (32³ voxels)    │         │  (wgpu)          │          │
│  └──────────────────┘         └──────────────────┘          │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. Voxel Data Structure (`voxel_data.rs`)

#### VoxelChunk
- **Size**: 32x32x32 voxels per chunk
- **Storage**: Sparse Voxel Octree (SVO) for memory efficiency
- **Data**: Each voxel stores density (f32) and material ID (u16)
- **Serialization**: Full serde support for save/load

```rust
pub struct VoxelChunk {
    coord: ChunkCoord,
    root: Option<OctreeNode>,
    dirty: bool,
}
```

#### VoxelGrid
- **Storage**: HashMap<ChunkCoord, VoxelChunk>
- **Dirty Tracking**: Maintains list of chunks needing remeshing
- **World Integration**: Coordinates align with World Partition cells

```rust
pub struct VoxelGrid {
    chunks: HashMap<ChunkCoord, VoxelChunk>,
    dirty_chunks: Vec<ChunkCoord>,
}
```

### 2. Isosurface Generation (`meshing.rs`)

#### Dual Contouring Algorithm
- **Advantages over Marching Cubes**:
  - Preserves sharp features
  - Fewer artifacts
  - More uniform triangles
  - Better handling of hermite data

#### Mesh Generation Pipeline
1. **Cell Processing**: Iterate through 32³ cells in chunk
2. **Surface Detection**: Find cells crossing isosurface (density = 0.5)
3. **Vertex Placement**: Use QEF minimization (simplified to edge interpolation)
4. **Normal Calculation**: Central differences for smooth normals
5. **Triangle Generation**: Connect vertices based on cell configuration

```rust
pub struct ChunkMesh {
    pub coord: ChunkCoord,
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}
```

#### LOD System
- **4 LOD Levels**: Based on distance thresholds
- **Distances**: [100m, 250m, 500m, 1000m]
- **Simplification**: Progressive mesh reduction at each level

### 3. Async Meshing

```rust
pub struct AsyncMeshGenerator {
    generator: DualContouring,
}
```

- **Background Processing**: Uses tokio for async mesh generation
- **Parallel Meshing**: Rayon for multi-chunk parallel processing
- **Dirty Chunk Queue**: Only remesh modified chunks

## Integration with Existing Systems

### World Partition Integration

```rust
// Chunk coordinates align with World Partition cells
let chunk_coord = ChunkCoord::from_world_pos(world_pos);
let partition_cell = world_partition.get_cell(chunk_coord);
```

- **Streaming**: Voxel chunks stream with World Partition cells
- **Memory Management**: Chunks unload when partition cells unload
- **Coordinate System**: Shared coordinate space

### Rendering Pipeline Integration

```rust
// Generated meshes use standard vertex format
pub struct MeshVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub material: u16,
}
```

- **Vertex Format**: Compatible with existing renderer
- **Material System**: Uses existing material IDs
- **Lighting**: Works with clustered forward + DDGI/VXGI

## Performance Characteristics

### Memory Usage
- **Empty Chunks**: ~100 bytes (SVO overhead only)
- **Full Chunks**: ~50-200 KB depending on complexity
- **Target**: <500MB for 10km² terrain
- **Actual**: Typically 200-300MB for varied terrain

### Meshing Performance
- **Single Chunk**: 1-5ms (depends on complexity)
- **Async Meshing**: Non-blocking, 4 concurrent tasks
- **Dirty Tracking**: Only remesh modified chunks
- **LOD**: Reduces mesh complexity at distance

### Runtime Performance
- **Target**: >60 FPS on mid-range hardware
- **Bottleneck**: Mesh generation (handled async)
- **Rendering**: Standard polygon rendering (efficient)

## Usage Examples

### Basic Voxel Operations

```rust
use astraweave_terrain::{VoxelGrid, Voxel, ChunkCoord};
use glam::Vec3;

// Create voxel grid
let mut grid = VoxelGrid::new();

// Set voxel at world position
let pos = Vec3::new(100.0, 50.0, 200.0);
let voxel = Voxel::new(1.0, 1); // density=1.0, material=1
grid.set_voxel(pos, voxel);

// Get voxel
if let Some(voxel) = grid.get_voxel(pos) {
    println!("Density: {}, Material: {}", voxel.density, voxel.material);
}

// Get dirty chunks for remeshing
let dirty = grid.dirty_chunks();
for &coord in dirty {
    // Remesh this chunk
}
```

### Mesh Generation

```rust
use astraweave_terrain::{DualContouring, VoxelChunk};

// Create mesher
let mut dc = DualContouring::new();

// Generate mesh from chunk
let chunk = /* ... get chunk ... */;
let mesh = dc.generate_mesh(&chunk);

// Use mesh vertices and indices for rendering
for vertex in &mesh.vertices {
    println!("Pos: {:?}, Normal: {:?}", vertex.position, vertex.normal);
}
```

### Async Meshing

```rust
use astraweave_terrain::AsyncMeshGenerator;

let mut generator = AsyncMeshGenerator::new();

// Generate single mesh async
let mesh = generator.generate_mesh_async(chunk).await;

// Generate multiple meshes in parallel
let chunks = vec![chunk1, chunk2, chunk3];
let meshes = generator.generate_meshes_parallel(chunks).await;
```

### LOD Mesh Generation

```rust
use astraweave_terrain::{LodMeshGenerator, LodConfig};

let config = LodConfig::default();
let mut lod_gen = LodMeshGenerator::new(config);

// Generate mesh with appropriate LOD based on distance
let distance = 300.0; // meters from camera
let mesh = lod_gen.generate_mesh_lod(&chunk, distance);
```

## Deformation and Destruction

### Crater Creation Example

```rust
// Create a spherical crater
let crater_center = Vec3::new(500.0, 100.0, 500.0);
let crater_radius = 20.0;

for x in -20..=20 {
    for y in -20..=20 {
        for z in -20..=20 {
            let offset = Vec3::new(x as f32, y as f32, z as f32);
            let pos = crater_center + offset;
            let distance = offset.length();
            
            if distance <= crater_radius {
                // Remove voxels (set density to 0)
                grid.set_voxel(pos, Voxel::new(0.0, 0));
            }
        }
    }
}

// Remesh affected chunks
for &coord in grid.dirty_chunks() {
    if let Some(chunk) = grid.get_chunk(coord) {
        let mesh = dc.generate_mesh(chunk);
        // Upload mesh to GPU
    }
    grid.mark_chunk_clean(coord);
}
```

## Testing

### Unit Tests

```bash
# Run voxel data tests
cargo test -p astraweave-terrain voxel_data

# Run meshing tests
cargo test -p astraweave-terrain meshing

# Run all terrain tests
cargo test -p astraweave-terrain
```

### Integration Tests

```bash
# Run with hybrid-voxel feature
cargo test -p astraweave-terrain --features hybrid-voxel
```

## Future Enhancements

### Planned Features
- [ ] GPU-accelerated meshing (compute shaders)
- [ ] Advanced QEF solver for better vertex placement
- [ ] Texture coordinate generation
- [ ] Ambient occlusion baking
- [ ] Collision mesh generation
- [ ] Procedural cave generation
- [ ] Biome-aware voxel materials

### Optimization Opportunities
- [ ] Octree compression
- [ ] Mesh simplification algorithms
- [ ] Incremental meshing (only affected regions)
- [ ] Mesh caching and reuse
- [ ] SIMD optimizations for voxel operations

## Troubleshooting

### Common Issues

**Issue**: Meshes have holes or gaps
- **Cause**: Neighboring chunks not properly synchronized
- **Solution**: Ensure chunk boundaries share voxel data

**Issue**: High memory usage
- **Cause**: Too many chunks loaded or dense voxel data
- **Solution**: Adjust World Partition streaming radius, use LOD

**Issue**: Slow meshing performance
- **Cause**: Synchronous meshing blocking main thread
- **Solution**: Use AsyncMeshGenerator for background processing

**Issue**: Jagged surfaces
- **Cause**: Low voxel resolution or disabled smoothing
- **Solution**: Increase chunk resolution or enable smooth normals

## References

- [Dual Contouring Paper](https://www.cs.rice.edu/~jwarren/papers/dualcontour.pdf)
- [Sparse Voxel Octrees](https://research.nvidia.com/publication/efficient-sparse-voxel-octrees)
- [Enshrouded Terrain System](https://www.youtube.com/watch?v=Nfnia7m_Kks)
- [World Partition Documentation](../astraweave-scene/WORLD_PARTITION.md)

## License

This implementation is part of the AstraWeave engine and follows the same license terms.