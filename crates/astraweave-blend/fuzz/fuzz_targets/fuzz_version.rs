#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

use astraweave_blend::version::BlenderVersion;

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    major: u32,
    minor: u32,
    patch: u32,
    other_major: u32,
    other_minor: u32,
    other_patch: u32,
}

fuzz_target!(|input: FuzzInput| {
    // Test version creation
    let v1 = BlenderVersion::new(input.major, input.minor, input.patch);
    let v2 = BlenderVersion::new(input.other_major, input.other_minor, input.other_patch);
    
    // Test accessors (should not panic)
    let _ = v1.major();
    let _ = v1.minor();
    let _ = v1.patch();
    
    // Test comparison operations (should satisfy mathematical properties)
    // Reflexivity
    assert!(v1 == v1);
    assert!(v2 == v2);
    
    // Symmetry
    if v1 == v2 {
        assert!(v2 == v1);
    }
    
    // Transitivity (check with self)
    assert!(v1 <= v1);
    assert!(v1 >= v1);
    
    // Trichotomy: exactly one of <, ==, > holds
    let lt = v1 < v2;
    let eq = v1 == v2;
    let gt = v1 > v2;
    
    let count = [lt, eq, gt].iter().filter(|&&x| x).count();
    assert_eq!(count, 1, "Trichotomy violated: lt={}, eq={}, gt={}", lt, eq, gt);
    
    // Test display (should not panic)
    let _ = format!("{}", v1);
    let _ = format!("{:?}", v1);
    
    // Test hash (should be deterministic)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut h1 = DefaultHasher::new();
    v1.hash(&mut h1);
    let hash1 = h1.finish();
    
    let mut h2 = DefaultHasher::new();
    v1.hash(&mut h2);
    let hash2 = h2.finish();
    
    assert_eq!(hash1, hash2, "Hash is not deterministic");
    
    // If equal, hashes must be equal
    if v1 == v2 {
        let mut h3 = DefaultHasher::new();
        let mut h4 = DefaultHasher::new();
        v1.hash(&mut h3);
        v2.hash(&mut h4);
        assert_eq!(h3.finish(), h4.finish(), "Equal values have different hashes");
    }
    
    // Test clone (should be equal to original)
    let v1_clone = v1.clone();
    assert_eq!(v1, v1_clone);
    
    // Test serialization roundtrip
    if let Ok(serialized) = ron::to_string(&v1) {
        if let Ok(deserialized) = ron::from_str::<BlenderVersion>(&serialized) {
            assert_eq!(v1, deserialized, "RON roundtrip failed");
        }
    }
    
    if let Ok(serialized) = serde_json::to_string(&v1) {
        if let Ok(deserialized) = serde_json::from_str::<BlenderVersion>(&serialized) {
            assert_eq!(v1, deserialized, "JSON roundtrip failed");
        }
    }
});
