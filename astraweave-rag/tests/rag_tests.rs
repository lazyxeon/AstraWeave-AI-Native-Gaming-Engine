//! Retrieval Engine Tests - Sprint 1 Day 4-5
//! Simplified tests for retrieval core functionality

#![allow(clippy::field_reassign_with_default)]

use astraweave_embeddings::{Memory, MemoryCategory};
use astraweave_rag::{RetrievalConfig, RetrievalEngine, RetrievalQuery};
use std::collections::HashMap;

#[test]
fn test_retrieval_basic_search() {
    let mut config = RetrievalConfig::default();
    config.similarity_threshold = 0.0; // Accept all matches
    let engine = RetrievalEngine::new(config);

    let memories = vec![
        create_memory("Dragon combat", MemoryCategory::Combat, 0.8),
        create_memory("Found potion", MemoryCategory::Exploration, 0.5),
    ];

    let query = RetrievalQuery {
        text: "dragon fight".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(3),
    };

    let results = engine.search(&query, &memories).unwrap();
    assert!(!results.is_empty());
    
    if results.len() >= 2 {
        assert!(results[0].score >= results[1].score);
    }
}

#[test]
fn test_retrieval_empty_memories() {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);

    let memories: Vec<Memory> = vec![];
    let query = RetrievalQuery {
        text: "test".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(5),
    };

    let results = engine.search(&query, &memories).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_retrieval_limit_enforcement() {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);

    let memories = vec![
        create_memory("Mem 1", MemoryCategory::Exploration, 0.8),
        create_memory("Mem 2", MemoryCategory::Exploration, 0.8),
        create_memory("Mem 3", MemoryCategory::Exploration, 0.8),
        create_memory("Mem 4", MemoryCategory::Exploration, 0.8),
        create_memory("Mem 5", MemoryCategory::Exploration, 0.8),
    ];

    let query = RetrievalQuery {
        text: "memory".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(2),
    };

    let results = engine.search(&query, &memories).unwrap();
    assert!(results.len() <= 2);
}

#[test]
fn test_retrieval_category_filter() {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);

    let memories = vec![
        create_memory("Combat event", MemoryCategory::Combat, 0.8),
        create_memory("Dialogue event", MemoryCategory::Dialogue, 0.8),
        create_memory("Combat 2", MemoryCategory::Combat, 0.8),
    ];

    let query = RetrievalQuery {
        text: "event".to_string(),
        categories: vec![MemoryCategory::Combat],
        filters: HashMap::new(),
        limit: Some(5),
    };

    let results = engine.search(&query, &memories).unwrap();

    for result in &results {
        assert_eq!(result.memory.category, MemoryCategory::Combat);
    }
}

#[test]
fn test_retrieval_multiple_categories() {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);

    let memories = vec![
        create_memory("Combat", MemoryCategory::Combat, 0.8),
        create_memory("Dialogue", MemoryCategory::Dialogue, 0.8),
        create_memory("Explore", MemoryCategory::Exploration, 0.8),
    ];

    let query = RetrievalQuery {
        text: "test".to_string(),
        categories: vec![MemoryCategory::Combat, MemoryCategory::Dialogue],
        filters: HashMap::new(),
        limit: Some(5),
    };

    let results = engine.search(&query, &memories).unwrap();

    for result in &results {
        assert!(result.memory.category == MemoryCategory::Combat || result.memory.category == MemoryCategory::Dialogue);
    }
}

#[test]
fn test_retrieval_no_category_filter() {
    let mut config = RetrievalConfig::default();
    config.similarity_threshold = 0.0; // Accept all
    let engine = RetrievalEngine::new(config);

    let memories = vec![
        create_memory("Combat", MemoryCategory::Combat, 0.8),
        create_memory("Dialogue", MemoryCategory::Dialogue, 0.8),
        create_memory("Quest", MemoryCategory::Quest, 0.8),
    ];

    let query = RetrievalQuery {
        text: "test".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(10),
    };

    let results = engine.search(&query, &memories).unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_retrieval_high_threshold() {
    let mut config = RetrievalConfig::default();
    config.similarity_threshold = 0.99;

    let engine = RetrievalEngine::new(config);

    let memories = vec![create_memory("Test", MemoryCategory::Exploration, 0.5)];

    let query = RetrievalQuery {
        text: "completely different".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(5),
    };

    let results = engine.search(&query, &memories).unwrap();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_retrieval_large_set() {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);

    let memories: Vec<Memory> = (0..100)
        .map(|i| create_memory(&format!("Content {}", i), MemoryCategory::Exploration, 0.5))
        .collect();

    let query = RetrievalQuery {
        text: "content".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(10),
    };

    let start = std::time::Instant::now();
    let results = engine.search(&query, &memories).unwrap();
    let duration = start.elapsed();

    assert!(results.len() <= 10);
    assert!(duration.as_millis() < 100);
}

#[test]
fn test_retrieval_empty_query() {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);

    let memories = vec![create_memory("Test", MemoryCategory::Exploration, 0.5)];

    let query = RetrievalQuery {
        text: "".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(5),
    };

    let result = engine.search(&query, &memories);
    assert!(result.is_ok());
}

#[test]
fn test_retrieval_ordering() {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);

    let memories = vec![
        create_memory("Dragon", MemoryCategory::Combat, 0.5),
        create_memory("Dragon fight epic", MemoryCategory::Combat, 0.9),
        create_memory("Other", MemoryCategory::Exploration, 0.3),
    ];

    let query = RetrievalQuery {
        text: "dragon fight".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(3),
    };

    let results = engine.search(&query, &memories).unwrap();

    for i in 1..results.len() {
        assert!(results[i - 1].score >= results[i].score);
    }
}

fn create_memory(text: &str, category: MemoryCategory, importance: f32) -> Memory {
    Memory {
        id: uuid::Uuid::new_v4().to_string(),
        text: text.to_string(),
        timestamp: 1234567890,
        importance,
        valence: 0.0,
        category,
        entities: vec![],
        context: HashMap::new(),
    }
}
