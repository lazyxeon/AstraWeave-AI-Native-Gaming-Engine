use anyhow::Result;
use astraweave_embeddings::{Memory, MemoryCategory};
use astraweave_rag::consolidation::{ConsolidationConfig, ConsolidationEngine};
use std::collections::HashMap;
use chrono::Utc;

fn create_memory(id: &str, text: &str, category: MemoryCategory) -> Memory {
    Memory {
        id: id.to_string(),
        text: text.to_string(),
        category,
        timestamp: Utc::now().timestamp() as u64,
        importance: 0.5,
        valence: 0.0,
        entities: vec![],
        context: HashMap::new(),
    }
}

#[test]
fn test_consolidation_merging() {
    let mut config = ConsolidationConfig::default();
    config.merge_similarity_threshold = 0.5; // Lower threshold for testing
    let engine = ConsolidationEngine::new(config);

    let memories = vec![
        create_memory("1", "the cat is sleeping", MemoryCategory::Gameplay),
        create_memory("2", "the cat is sleeping", MemoryCategory::Gameplay), // Identical
    ];

    let (consolidated, result) = engine.consolidate(memories).unwrap();
    
    // Should merge into 1
    assert_eq!(consolidated.len(), 1);
    assert_eq!(result.merged_count, 1);
    assert_eq!(result.removed_count, 1);
    
    // Check merged content
    assert!(consolidated[0].text.contains("the cat is sleeping"));
}

#[test]
fn test_consolidation_threshold() {
    let mut config = ConsolidationConfig::default();
    config.merge_similarity_threshold = 0.9; // High threshold
    let engine = ConsolidationEngine::new(config);

    let memories = vec![
        create_memory("1", "the cat is sleeping", MemoryCategory::Gameplay),
        create_memory("2", "the dog is barking", MemoryCategory::Gameplay), // Different
    ];

    let (consolidated, result) = engine.consolidate(memories).unwrap();
    
    // Should NOT merge
    assert_eq!(consolidated.len(), 2);
    assert_eq!(result.merged_count, 0);
}

#[test]
fn test_consolidation_category_mismatch() {
    let mut config = ConsolidationConfig::default();
    config.merge_similarity_threshold = 0.1; // Very low threshold
    let engine = ConsolidationEngine::new(config);

    let memories = vec![
        create_memory("1", "same text", MemoryCategory::Gameplay),
        create_memory("2", "same text", MemoryCategory::Combat), // Different category
    ];

    let (consolidated, result) = engine.consolidate(memories).unwrap();
    
    // Should NOT merge due to category mismatch
    assert_eq!(consolidated.len(), 2);
    assert_eq!(result.merged_count, 0);
}

#[test]
fn test_consolidation_context_merging() {
    let mut config = ConsolidationConfig::default();
    config.merge_similarity_threshold = 0.5;
    let engine = ConsolidationEngine::new(config);

    let mut m1 = create_memory("1", "test memory", MemoryCategory::Gameplay);
    m1.context.insert("key1".to_string(), "value1".to_string());

    let mut m2 = create_memory("2", "test memory", MemoryCategory::Gameplay);
    m2.context.insert("key2".to_string(), "value2".to_string());

    let memories = vec![m1, m2];

    let (consolidated, _) = engine.consolidate(memories).unwrap();
    
    assert_eq!(consolidated.len(), 1);
    let merged = &consolidated[0];
    
    assert_eq!(merged.context.get("key1").unwrap(), "value1");
    assert_eq!(merged.context.get("key2").unwrap(), "value2");
}
