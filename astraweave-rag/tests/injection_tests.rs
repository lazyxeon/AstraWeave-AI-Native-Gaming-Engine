use anyhow::Result;
use astraweave_embeddings::{Memory, MemoryCategory};
use astraweave_rag::injection::{InjectionConfig, InjectionContext, InjectionEngine};
use std::collections::HashMap;
use chrono::Utc;

fn create_memory(id: &str, text: &str, category: MemoryCategory, age_hours: i64) -> Memory {
    let timestamp = Utc::now().timestamp() - (age_hours * 3600);
    Memory {
        id: id.to_string(),
        text: text.to_string(),
        category,
        timestamp: timestamp as u64,
        importance: 0.5,
        valence: 0.0,
        entities: vec![],
        context: HashMap::new(),
    }
}

#[test]
fn test_injection_relevance_threshold() {
    let mut config = InjectionConfig::default();
    config.relevance_threshold = 0.5; // High threshold
    let engine = InjectionEngine::new(config);

    let context = InjectionContext {
        query: "cats".to_string(),
        conversation_history: vec![],
        metadata: HashMap::new(),
        preferred_categories: vec![],
    };

    let memories = vec![
        create_memory("1", "cats are great", MemoryCategory::Gameplay, 0), // Relevant
        create_memory("2", "dogs are loyal", MemoryCategory::Gameplay, 0), // Irrelevant
    ];

    let result = engine.inject(&context, &memories).unwrap();
    
    // "cats are great" has "cats" (1/3 match) -> 0.33 * 0.5 = 0.165
    // Plus base score 0.1 = 0.265.
    // Wait, the naive similarity is: common_words / words1.len().
    // query="cats" (len 1). memory="cats are great".
    // similarity("cats", "cats are great") -> common=1, len=1 -> 1.0.
    // relevance = 1.0 * 0.5 + 0.1 = 0.6.
    
    // "dogs are loyal". query="cats".
    // similarity("cats", "dogs are loyal") -> common=0.
    // relevance = 0.0 * 0.5 + 0.1 = 0.1.
    
    // So "cats" should pass (0.6 >= 0.5), "dogs" should fail (0.1 < 0.5).
    
    assert_eq!(result.injected_memories.len(), 1);
    assert_eq!(result.injected_memories[0].id, "1");
}

#[test]
fn test_injection_recency_prioritization() {
    let mut config = InjectionConfig::default();
    config.prioritize_recent = true;
    config.max_memories = 1;
    let engine = InjectionEngine::new(config);

    let context = InjectionContext {
        query: "news".to_string(),
        conversation_history: vec![],
        metadata: HashMap::new(),
        preferred_categories: vec![],
    };

    // Two identical memories, one recent, one old
    let memories = vec![
        create_memory("old", "breaking news", MemoryCategory::Gameplay, 24), // 24 hours old
        create_memory("new", "breaking news", MemoryCategory::Gameplay, 0),  // Now
    ];

    let result = engine.inject(&context, &memories).unwrap();
    
    assert_eq!(result.injected_memories.len(), 1);
    assert_eq!(result.injected_memories[0].id, "new");
}

#[test]
fn test_injection_category_preference() {
    let config = InjectionConfig::default();
    let engine = InjectionEngine::new(config);

    let context = InjectionContext {
        query: "something".to_string(),
        conversation_history: vec![],
        metadata: HashMap::new(),
        preferred_categories: vec![MemoryCategory::Combat],
    };

    let memories = vec![
        create_memory("1", "something happened", MemoryCategory::Gameplay, 0),
        create_memory("2", "something happened", MemoryCategory::Combat, 0),
    ];

    let result = engine.inject(&context, &memories).unwrap();
    
    // Both have same text similarity.
    // Memory 2 matches preferred category -> +0.3 boost.
    // Memory 1 does not -> +0.0 (and base score logic might differ).
    // calculate_relevance:
    // if preferred matches: +0.3.
    // else: +0.1.
    // So Combat gets +0.3, Gameplay gets +0.1.
    // Combat should be first.
    
    assert!(result.injected_memories.len() >= 2);
    assert_eq!(result.injected_memories[0].id, "2");
}

#[test]
fn test_injection_max_memories() {
    let mut config = InjectionConfig::default();
    config.max_memories = 2;
    let engine = InjectionEngine::new(config);

    let context = InjectionContext {
        query: "test".to_string(),
        conversation_history: vec![],
        metadata: HashMap::new(),
        preferred_categories: vec![],
    };

    let memories = vec![
        create_memory("1", "test 1", MemoryCategory::Gameplay, 0),
        create_memory("2", "test 2", MemoryCategory::Gameplay, 0),
        create_memory("3", "test 3", MemoryCategory::Gameplay, 0),
    ];

    let result = engine.inject(&context, &memories).unwrap();
    
    assert_eq!(result.injected_memories.len(), 2);
}
