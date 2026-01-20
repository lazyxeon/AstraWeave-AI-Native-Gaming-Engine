//! Fuzz target for asset database operations.
//!
//! Tests hash computation and metadata structures.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Arbitrary)]
struct FuzzAssetMeta {
    path: String,
    content: Vec<u8>,
    dependencies: Vec<String>,
    tags: Vec<String>,
}

fn compute_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.into()
}

fn encode_hash_hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

fuzz_target!(|input: FuzzAssetMeta| {
    // Compute content hash
    let hash = compute_hash(&input.content);
    let hash_hex = encode_hash_hex(&hash);
    
    // Verify hash properties
    assert_eq!(hash_hex.len(), 64); // SHA256 = 32 bytes = 64 hex chars
    assert!(hash_hex.chars().all(|c| c.is_ascii_hexdigit()));
    
    // Same content should produce same hash
    let hash2 = compute_hash(&input.content);
    assert_eq!(hash, hash2);
    
    // Build dependency graph (no cycles allowed)
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    graph.insert(input.path.clone(), input.dependencies.clone());
    
    // Simulate topological sort detection
    let mut visited = std::collections::HashSet::new();
    let mut in_progress = std::collections::HashSet::new();
    
    fn has_cycle(
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        in_progress: &mut std::collections::HashSet<String>,
    ) -> bool {
        if in_progress.contains(node) {
            return true; // Cycle detected
        }
        if visited.contains(node) {
            return false;
        }
        in_progress.insert(node.to_string());
        
        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if has_cycle(dep, graph, visited, in_progress) {
                    return true;
                }
            }
        }
        
        in_progress.remove(node);
        visited.insert(node.to_string());
        false
    }
    
    let _ = has_cycle(&input.path, &graph, &mut visited, &mut in_progress);
    
    // Tag filtering
    for tag in &input.tags {
        let normalized = tag.trim().to_lowercase();
        let _ = normalized.len();
    }
});
