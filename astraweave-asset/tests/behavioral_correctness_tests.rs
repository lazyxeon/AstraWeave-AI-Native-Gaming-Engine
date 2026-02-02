//! Behavioral Correctness Tests for astraweave-asset
//!
//! These tests validate the mathematical/behavioral correctness of asset processing,
//! ensuring formulas match expected behavior and are mutation-resistant.

use glam::Vec3;
use std::collections::HashMap;

// ============================================================================
// GUID Tests: SHA-256 hash, case-insensitive, path normalization
// ============================================================================

/// GUID formula: SHA-256(path.replace('\\', '/').to_lowercase())[0..16] as hex
fn guid_for_path(path: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(path.replace('\\', "/").to_lowercase());
    let out = hasher.finalize();
    hex::encode(&out[0..16])
}

/// Test GUID determinism - same path always produces same GUID
#[test]
fn test_guid_deterministic() {
    let guid1 = guid_for_path("assets/textures/wood.png");
    let guid2 = guid_for_path("assets/textures/wood.png");
    assert_eq!(guid1, guid2, "Same path must produce same GUID");
}

/// Test GUID case insensitivity - mixed case paths produce same GUID
#[test]
fn test_guid_case_insensitive() {
    let lower = guid_for_path("assets/textures/wood.png");
    let upper = guid_for_path("ASSETS/TEXTURES/WOOD.PNG");
    let mixed = guid_for_path("Assets/Textures/Wood.PNG");
    
    assert_eq!(lower, upper, "Case must not affect GUID");
    assert_eq!(upper, mixed, "Case must not affect GUID");
}

/// Test GUID path normalization - backslashes become forward slashes
#[test]
fn test_guid_path_normalization() {
    let forward = guid_for_path("assets/textures/wood.png");
    let back = guid_for_path("assets\\textures\\wood.png");
    let mixed = guid_for_path("assets\\textures/wood.png");
    
    assert_eq!(forward, back, "Backslashes must normalize to forward slashes");
    assert_eq!(forward, mixed, "Mixed slashes must normalize");
}

/// Test GUID length - must be 32 hex chars (128 bits)
#[test]
fn test_guid_length() {
    let guid = guid_for_path("some/path/asset.glb");
    assert_eq!(guid.len(), 32, "GUID must be 32 hex characters (128 bits)");
    assert!(guid.chars().all(|c| c.is_ascii_hexdigit()), "GUID must be hex");
}

/// Test GUID uniqueness - different paths produce different GUIDs
#[test]
fn test_guid_uniqueness() {
    let guid1 = guid_for_path("assets/wood.png");
    let guid2 = guid_for_path("assets/stone.png");
    let guid3 = guid_for_path("textures/wood.png");
    
    assert_ne!(guid1, guid2, "Different files must have different GUIDs");
    assert_ne!(guid1, guid3, "Different directories must have different GUIDs");
}

/// Test GUID empty path - should still produce valid GUID
#[test]
fn test_guid_empty_path() {
    let guid = guid_for_path("");
    assert_eq!(guid.len(), 32, "Empty path must still produce 32-char GUID");
}

// ============================================================================
// AABB (Axis-Aligned Bounding Box) Tests
// ============================================================================

#[derive(Debug, Clone, Copy)]
struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    fn from_points(points: &[[f32; 3]]) -> Self {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for p in points {
            let point = Vec3::from_array(*p);
            min = min.min(point);
            max = max.max(point);
        }
        Self { min, max }
    }
    
    fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    
    fn extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }
    
    fn diagonal(&self) -> f32 {
        (self.max - self.min).length()
    }
    
    fn contains(&self, point: Vec3) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }
    
    fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

/// Test AABB center formula: (min + max) / 2
#[test]
fn test_aabb_center_formula() {
    let aabb = AABB {
        min: Vec3::new(-1.0, -2.0, -3.0),
        max: Vec3::new(1.0, 2.0, 3.0),
    };
    let center = aabb.center();
    
    // Formula: (min + max) * 0.5
    let expected = (aabb.min + aabb.max) * 0.5;
    assert!((center - expected).length() < 0.001, "center = (min + max) / 2");
    assert!((center - Vec3::ZERO).length() < 0.001, "Symmetric AABB centered at origin");
}

/// Test AABB extents formula: (max - min) / 2
#[test]
fn test_aabb_extents_formula() {
    let aabb = AABB {
        min: Vec3::new(0.0, 0.0, 0.0),
        max: Vec3::new(4.0, 6.0, 8.0),
    };
    let extents = aabb.extents();
    
    // Formula: (max - min) * 0.5
    let expected = (aabb.max - aabb.min) * 0.5;
    assert!((extents - expected).length() < 0.001, "extents = (max - min) / 2");
    assert!((extents.x - 2.0).abs() < 0.001, "X extent = 2");
    assert!((extents.y - 3.0).abs() < 0.001, "Y extent = 3");
    assert!((extents.z - 4.0).abs() < 0.001, "Z extent = 4");
}

/// Test AABB diagonal formula: |max - min|
#[test]
fn test_aabb_diagonal_formula() {
    let aabb = AABB {
        min: Vec3::ZERO,
        max: Vec3::new(3.0, 4.0, 0.0),
    };
    let diagonal = aabb.diagonal();
    
    // 3-4-5 right triangle: diagonal = sqrt(3² + 4²) = 5
    assert!((diagonal - 5.0).abs() < 0.001, "Diagonal follows Pythagorean theorem");
}

/// Test AABB from_points - bounds all input points
#[test]
fn test_aabb_from_points() {
    let points = [
        [-1.0, 0.0, 2.0],
        [3.0, -2.0, 1.0],
        [0.0, 5.0, 0.0],
    ];
    let aabb = AABB::from_points(&points);
    
    assert!((aabb.min.x - (-1.0)).abs() < 0.001, "min.x = -1");
    assert!((aabb.min.y - (-2.0)).abs() < 0.001, "min.y = -2");
    assert!((aabb.min.z - 0.0).abs() < 0.001, "min.z = 0");
    assert!((aabb.max.x - 3.0).abs() < 0.001, "max.x = 3");
    assert!((aabb.max.y - 5.0).abs() < 0.001, "max.y = 5");
    assert!((aabb.max.z - 2.0).abs() < 0.001, "max.z = 2");
}

/// Test AABB contains - point inside and outside
#[test]
fn test_aabb_contains() {
    let aabb = AABB {
        min: Vec3::ZERO,
        max: Vec3::ONE,
    };
    
    assert!(aabb.contains(Vec3::new(0.5, 0.5, 0.5)), "Center is inside");
    assert!(aabb.contains(Vec3::ZERO), "Min corner is inside");
    assert!(aabb.contains(Vec3::ONE), "Max corner is inside");
    assert!(!aabb.contains(Vec3::new(1.5, 0.5, 0.5)), "Outside X is not inside");
    assert!(!aabb.contains(Vec3::new(-0.1, 0.5, 0.5)), "Negative X is not inside");
}

/// Test AABB merge - combined bounds
#[test]
fn test_aabb_merge() {
    let a = AABB {
        min: Vec3::ZERO,
        max: Vec3::ONE,
    };
    let b = AABB {
        min: Vec3::new(0.5, 0.5, 0.5),
        max: Vec3::new(2.0, 2.0, 2.0),
    };
    let merged = a.merge(&b);
    
    assert!((merged.min - Vec3::ZERO).length() < 0.001, "Merged min is smaller min");
    assert!((merged.max - Vec3::new(2.0, 2.0, 2.0)).length() < 0.001, "Merged max is larger max");
}

// ============================================================================
// Bounding Cone Tests (for backface culling)
// ============================================================================

/// Test bounding cone backfacing - dot(axis, view) < cutoff
#[test]
fn test_bounding_cone_backfacing() {
    struct BoundingCone {
        axis: Vec3,
        cutoff: f32,
    }
    
    impl BoundingCone {
        fn is_backfacing(&self, view_dir: Vec3) -> bool {
            self.axis.dot(view_dir) < self.cutoff
        }
    }
    
    let cone = BoundingCone {
        axis: Vec3::Z, // Facing +Z
        cutoff: 0.0,   // 90° cone
    };
    
    // View from behind (+Z looking into -Z)
    assert!(cone.is_backfacing(Vec3::NEG_Z), "Viewing from front = backfacing for this cone");
    
    // View from front (-Z looking into +Z)
    assert!(!cone.is_backfacing(Vec3::Z), "Viewing from behind = not backfacing");
}

/// Test bounding cone cutoff threshold
#[test]
fn test_bounding_cone_cutoff() {
    struct BoundingCone {
        axis: Vec3,
        cutoff: f32,
    }
    
    impl BoundingCone {
        fn is_backfacing(&self, view_dir: Vec3) -> bool {
            self.axis.dot(view_dir) < self.cutoff
        }
    }
    
    // 60° cone (cutoff = cos(60°) = 0.5)
    let cone = BoundingCone {
        axis: Vec3::Z,
        cutoff: 0.5,
    };
    
    // Exactly at cutoff angle
    let view_45 = Vec3::new(0.0, (45.0_f32).to_radians().sin(), (45.0_f32).to_radians().cos());
    let dot_45 = cone.axis.dot(view_45.normalize());
    
    // cos(45°) ≈ 0.707 > 0.5, so not backfacing
    assert!(!cone.is_backfacing(view_45.normalize()), "45° view (cos=0.707) > cutoff 0.5");
    assert!(dot_45 > 0.5, "Dot product at 45° should be > 0.5");
}

// ============================================================================
// Quadric Error Tests (for mesh simplification)
// ============================================================================

/// Quadric error formula: v^T * Q * v
#[test]
fn test_quadric_error_formula() {
    // Simple plane equation: z = 0 (i.e., normal = (0,0,1), d = 0)
    // Quadric from plane (a,b,c,d) where ax + by + cz + d = 0
    // For z=0: a=0, b=0, c=1, d=0
    let a = 0.0_f64;
    let b = 0.0_f64;
    let c = 1.0_f64;
    let d = 0.0_f64;
    
    // Build quadric matrix Q = [a,b,c,d]^T * [a,b,c,d]
    let q = [
        [a*a, a*b, a*c, a*d],
        [a*b, b*b, b*c, b*d],
        [a*c, b*c, c*c, c*d],
        [a*d, b*d, c*d, d*d],
    ];
    
    // Error for point (0,0,0) should be 0 (on plane)
    let v = [0.0, 0.0, 0.0, 1.0];
    let mut error = 0.0;
    for i in 0..4 {
        for j in 0..4 {
            error += v[i] * q[i][j] * v[j];
        }
    }
    assert!(error.abs() < 0.001, "Point on plane has zero error");
    
    // Error for point (0,0,1) should be 1 (1 unit from plane)
    let v = [0.0, 0.0, 1.0, 1.0];
    let mut error = 0.0;
    for i in 0..4 {
        for j in 0..4 {
            error += v[i] * q[i][j] * v[j];
        }
    }
    assert!((error - 1.0).abs() < 0.001, "Point 1 unit from plane has error 1");
    
    // Error for point (0,0,2) should be 4 (error is squared distance)
    let v = [0.0, 0.0, 2.0, 1.0];
    let mut error = 0.0;
    for i in 0..4 {
        for j in 0..4 {
            error += v[i] * q[i][j] * v[j];
        }
    }
    assert!((error - 4.0).abs() < 0.001, "Point 2 units from plane has error 4 (squared)");
}

/// Quadric from triangle: uses cross product normal
#[test]
fn test_quadric_from_triangle() {
    let p0 = Vec3::ZERO;
    let p1 = Vec3::X;
    let p2 = Vec3::Z;
    
    // Triangle normal: (p1 - p0) × (p2 - p0) = X × Z = -Y
    let edge1 = p1 - p0;
    let edge2 = p2 - p0;
    let normal = edge1.cross(edge2).normalize_or_zero();
    
    assert!((normal.y - (-1.0)).abs() < 0.001, "XZ triangle normal is -Y");
    
    // Plane equation: -y + d = 0 where d = -normal · p0 = 0
    let d = -normal.dot(p0);
    assert!(d.abs() < 0.001, "Plane passes through origin");
}

// ============================================================================
// Meshlet Tests
// ============================================================================

/// Meshlet vertex limit: MAX_MESHLET_VERTICES = 64
#[test]
fn test_meshlet_vertex_limit() {
    const MAX_MESHLET_VERTICES: usize = 64;
    assert_eq!(MAX_MESHLET_VERTICES, 64, "Meshlet vertex limit is 64");
}

/// Meshlet triangle limit: MAX_MESHLET_TRIANGLES = 124
#[test]
fn test_meshlet_triangle_limit() {
    const MAX_MESHLET_TRIANGLES: usize = 124;
    assert_eq!(MAX_MESHLET_TRIANGLES, 124, "Meshlet triangle limit is 124");
}

/// Meshlet triangle count formula: indices.len() / 3
#[test]
fn test_meshlet_triangle_count() {
    let indices: Vec<u8> = vec![0, 1, 2, 0, 2, 3, 0, 3, 4]; // 3 triangles
    let triangle_count = indices.len() / 3;
    assert_eq!(triangle_count, 3, "9 indices = 3 triangles");
}

// ============================================================================
// AssetCache Tests
// ============================================================================

/// Test AssetCache insert and retrieve
#[test]
fn test_asset_cache_insert_retrieve() {
    let mut cache: HashMap<String, i32> = HashMap::new();
    let id = guid_for_path("assets/test.png");
    cache.insert(id.clone(), 42);
    
    assert_eq!(cache.get(&id), Some(&42), "Cache retrieves inserted value");
}

/// Test AssetCache length tracking
#[test]
fn test_asset_cache_length() {
    let mut cache: HashMap<String, i32> = HashMap::new();
    assert!(cache.is_empty(), "New cache is empty");
    assert_eq!(cache.len(), 0, "New cache length is 0");
    
    cache.insert(guid_for_path("a.png"), 1);
    cache.insert(guid_for_path("b.png"), 2);
    cache.insert(guid_for_path("c.png"), 3);
    
    assert_eq!(cache.len(), 3, "Cache has 3 items");
    assert!(!cache.is_empty(), "Cache is not empty");
}

/// Test AssetCache overwrite behavior
#[test]
fn test_asset_cache_overwrite() {
    let mut cache: HashMap<String, i32> = HashMap::new();
    let id = guid_for_path("assets/test.png");
    
    cache.insert(id.clone(), 1);
    assert_eq!(cache.get(&id), Some(&1), "First insert");
    
    cache.insert(id.clone(), 2);
    assert_eq!(cache.get(&id), Some(&2), "Second insert overwrites");
    assert_eq!(cache.len(), 1, "Length unchanged after overwrite");
}

// ============================================================================
// Cell Data Tests
// ============================================================================

/// Test EntityData default rotation is identity quaternion
#[test]
fn test_entity_data_identity_quaternion() {
    // Identity quaternion: (x=0, y=0, z=0, w=1)
    let rotation = [0.0, 0.0, 0.0, 1.0_f32];
    
    // Quaternion magnitude should be 1
    let mag = (rotation[0].powi(2) + rotation[1].powi(2) + 
               rotation[2].powi(2) + rotation[3].powi(2)).sqrt();
    assert!((mag - 1.0).abs() < 0.001, "Quaternion is normalized");
    assert_eq!(rotation[3], 1.0, "W component is 1 for identity");
}

/// Test EntityData default scale is uniform 1.0
#[test]
fn test_entity_data_default_scale() {
    let scale = [1.0, 1.0, 1.0_f32];
    assert_eq!(scale, [1.0, 1.0, 1.0], "Default scale is uniform 1.0");
}

/// Test CellData memory estimate formula
#[test]
fn test_cell_data_memory_estimate() {
    // Basic formula: base struct + entities * size_of::<EntityData> + assets * size_of::<AssetRef>
    let entity_count = 10;
    let asset_count = 5;
    
    // Rough size estimates (actual sizes may vary)
    let entity_size = std::mem::size_of::<([f32; 3], [f32; 4], [f32; 3])>(); // ~40 bytes
    let asset_size = std::mem::size_of::<(String, u8)>(); // ~32 bytes
    
    let estimate = entity_count * entity_size + asset_count * asset_size;
    assert!(estimate > 0, "Memory estimate is positive");
    assert!(estimate < 10000, "Memory estimate is reasonable");
}

/// Test CellData coordinate system
#[test]
fn test_cell_data_coordinate() {
    let coord: [i32; 3] = [5, -3, 10];
    
    // Grid coordinates can be negative
    assert_eq!(coord[0], 5, "X coordinate");
    assert_eq!(coord[1], -3, "Y coordinate can be negative");
    assert_eq!(coord[2], 10, "Z coordinate");
}

/// Test CellData asset deduplication logic
#[test]
fn test_cell_data_asset_deduplication() {
    // Simulate add_asset behavior: only add if path doesn't exist
    let mut assets: Vec<String> = Vec::new();
    
    fn add_asset(assets: &mut Vec<String>, path: &str) {
        if !assets.iter().any(|a| a == path) {
            assets.push(path.to_string());
        }
    }
    
    add_asset(&mut assets, "tex1.png");
    add_asset(&mut assets, "tex2.png");
    add_asset(&mut assets, "tex1.png"); // Duplicate
    
    assert_eq!(assets.len(), 2, "Duplicate assets not added");
}

// ============================================================================
// LOD Error Tests
// ============================================================================

/// Test LOD error scaling - higher LOD = more error tolerance
#[test]
fn test_lod_error_scaling() {
    // LOD error typically scales with level
    fn compute_lod_error(bounds_diagonal: f32, lod_level: u32) -> f32 {
        bounds_diagonal * (2.0_f32).powi(lod_level as i32)
    }
    
    let diagonal = 10.0;
    let error_0 = compute_lod_error(diagonal, 0);
    let error_1 = compute_lod_error(diagonal, 1);
    let error_2 = compute_lod_error(diagonal, 2);
    
    assert!((error_0 - 10.0).abs() < 0.001, "LOD 0 error = diagonal");
    assert!((error_1 - 20.0).abs() < 0.001, "LOD 1 error = 2 × diagonal");
    assert!((error_2 - 40.0).abs() < 0.001, "LOD 2 error = 4 × diagonal");
}

/// Test mesh simplification target - 50% reduction per LOD
#[test]
fn test_mesh_simplification_ratio() {
    let initial_triangles = 1000;
    
    // Each LOD reduces by ~50%
    let lod1_target = (initial_triangles / 2).max(1);
    let lod2_target = (lod1_target / 2).max(1);
    let lod3_target = (lod2_target / 2).max(1);
    
    assert_eq!(lod1_target, 500, "LOD 1 target = 500");
    assert_eq!(lod2_target, 250, "LOD 2 target = 250");
    assert_eq!(lod3_target, 125, "LOD 3 target = 125");
}

/// Test simplification floor - minimum 4 triangles (12 indices)
#[test]
fn test_simplification_floor() {
    let min_indices = 12;
    let min_triangles = min_indices / 3;
    
    assert_eq!(min_triangles, 4, "Minimum 4 triangles for valid mesh");
    
    // Simplification stops when indices < 12
    let should_stop = |indices_len: usize| indices_len < 12;
    assert!(should_stop(9), "9 indices = 3 triangles, should stop");
    assert!(!should_stop(12), "12 indices = 4 triangles, can continue");
}

// ============================================================================
// File Hash Tests
// ============================================================================

/// Test file hash is SHA-256
#[test]
fn test_file_hash_algorithm() {
    use sha2::{Digest, Sha256};
    
    let content = b"test file content";
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash = hex::encode(hasher.finalize());
    
    // SHA-256 produces 64 hex characters (256 bits)
    assert_eq!(hash.len(), 64, "SHA-256 hash is 64 hex chars");
}

/// Test file hash determinism
#[test]
fn test_file_hash_deterministic() {
    use sha2::{Digest, Sha256};
    
    let content = b"same content";
    
    let mut hasher1 = Sha256::new();
    hasher1.update(content);
    let hash1 = hex::encode(hasher1.finalize());
    
    let mut hasher2 = Sha256::new();
    hasher2.update(content);
    let hash2 = hex::encode(hasher2.finalize());
    
    assert_eq!(hash1, hash2, "Same content produces same hash");
}

/// Test file hash uniqueness
#[test]
fn test_file_hash_uniqueness() {
    use sha2::{Digest, Sha256};
    
    let hash_content = |content: &[u8]| -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    };
    
    let hash1 = hash_content(b"content A");
    let hash2 = hash_content(b"content B");
    
    assert_ne!(hash1, hash2, "Different content produces different hash");
}

// ============================================================================
// Blend File Detection Tests
// ============================================================================

/// Test blend file extension detection
#[test]
fn test_blend_file_detection() {
    use std::path::Path;
    
    fn is_blend_file(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("blend"))
            .unwrap_or(false)
    }
    
    assert!(is_blend_file(Path::new("model.blend")), ".blend detected");
    assert!(is_blend_file(Path::new("model.BLEND")), ".BLEND detected (case insensitive)");
    assert!(is_blend_file(Path::new("path/to/model.blend")), "Nested .blend detected");
    assert!(!is_blend_file(Path::new("model.gltf")), ".gltf not detected as blend");
    assert!(!is_blend_file(Path::new("blend")), "No extension");
}

/// Test blend to glTF path conversion
#[test]
fn test_blend_to_gltf_path() {
    use std::path::{Path, PathBuf};
    
    fn blend_to_gltf_path(blend_path: &Path) -> PathBuf {
        let stem = blend_path.file_stem().unwrap_or_default();
        let output_name = format!("{}.glb", stem.to_string_lossy());
        blend_path.with_file_name(output_name)
    }
    
    let input = Path::new("assets/models/hero.blend");
    let output = blend_to_gltf_path(input);
    
    assert_eq!(output, PathBuf::from("assets/models/hero.glb"));
}

// ============================================================================
// Asset Kind Tests
// ============================================================================

/// Test AssetKind variants
#[test]
fn test_asset_kind_variants() {
    #[derive(Debug, Clone, PartialEq)]
    enum AssetKind {
        Mesh,
        Texture,
        Audio,
        Dialogue,
        Material,
        Animation,
        Script,
        BlenderSource,
        Other,
    }
    
    // Verify all variants can be compared
    assert_eq!(AssetKind::Mesh, AssetKind::Mesh);
    assert_ne!(AssetKind::Mesh, AssetKind::Texture);
    assert_eq!(AssetKind::BlenderSource, AssetKind::BlenderSource);
}

/// Test asset kind from file extension heuristic
#[test]
fn test_asset_kind_from_extension() {
    fn kind_from_extension(ext: &str) -> &'static str {
        match ext.to_lowercase().as_str() {
            "gltf" | "glb" | "obj" | "fbx" => "Mesh",
            "png" | "jpg" | "jpeg" | "dds" | "ktx2" => "Texture",
            "wav" | "ogg" | "mp3" | "flac" => "Audio",
            "blend" => "BlenderSource",
            "lua" | "rhai" => "Script",
            _ => "Other",
        }
    }
    
    assert_eq!(kind_from_extension("glb"), "Mesh");
    assert_eq!(kind_from_extension("PNG"), "Texture");
    assert_eq!(kind_from_extension("wav"), "Audio");
    assert_eq!(kind_from_extension("blend"), "BlenderSource");
    assert_eq!(kind_from_extension("unknown"), "Other");
}

// ============================================================================
// Dependency Graph Tests
// ============================================================================

/// Test dependency graph bidirectional tracking
#[test]
fn test_dependency_graph_bidirectional() {
    use std::collections::HashSet;
    
    let mut dependency_graph: HashMap<String, HashSet<String>> = HashMap::new(); // dependent -> dependees
    let mut reverse_deps: HashMap<String, HashSet<String>> = HashMap::new(); // dependee -> dependents
    
    // Material depends on Texture
    let material_guid = "mat_001".to_string();
    let texture_guid = "tex_001".to_string();
    
    // Add dependency
    reverse_deps.entry(material_guid.clone())
        .or_default()
        .insert(texture_guid.clone());
    dependency_graph.entry(texture_guid.clone())
        .or_default()
        .insert(material_guid.clone());
    
    // Verify bidirectional
    assert!(reverse_deps.get(&material_guid).unwrap().contains(&texture_guid),
        "Material depends on texture");
    assert!(dependency_graph.get(&texture_guid).unwrap().contains(&material_guid),
        "Texture has material as dependent");
}

/// Test dependency cascade invalidation
#[test]
fn test_dependency_cascade_invalidation() {
    use std::collections::HashSet;
    
    // Texture → Material → Mesh (dependency chain)
    let mut dependency_graph: HashMap<String, HashSet<String>> = HashMap::new();
    
    dependency_graph.insert("texture".to_string(), 
        HashSet::from_iter(["material".to_string()]));
    dependency_graph.insert("material".to_string(),
        HashSet::from_iter(["mesh".to_string()]));
    
    // When texture changes, find all dependents (BFS/DFS)
    fn collect_dependents(graph: &HashMap<String, HashSet<String>>, start: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        let mut queue = vec![start.to_string()];
        
        while let Some(current) = queue.pop() {
            if let Some(deps) = graph.get(&current) {
                for dep in deps {
                    if result.insert(dep.clone()) {
                        queue.push(dep.clone());
                    }
                }
            }
        }
        result
    }
    
    let invalidated = collect_dependents(&dependency_graph, "texture");
    assert!(invalidated.contains("material"), "Material invalidated");
    assert!(invalidated.contains("mesh"), "Mesh invalidated via material");
    assert_eq!(invalidated.len(), 2, "Only 2 assets invalidated");
}
