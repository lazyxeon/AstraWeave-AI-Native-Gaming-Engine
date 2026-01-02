use anyhow::Result;
use astraweave_embeddings::{Memory, MemoryCategory};
use astraweave_rag::retrieval::{RetrievalConfig, RetrievalEngine, RetrievalQuery};
use std::collections::HashMap;

fn create_test_memories() -> Vec<Memory> {
    vec![
        Memory {
            id: "1".to_string(),
            text: "The brave knight fought the dragon".to_string(),
            timestamp: 1000,
            importance: 0.8,
            valence: 0.0,
            category: MemoryCategory::Combat,
            entities: vec!["knight".to_string(), "dragon".to_string()],
            context: HashMap::new(),
        },
        Memory {
            id: "2".to_string(),
            text: "The merchant sold a potion".to_string(),
            timestamp: 2000,
            importance: 0.4,
            valence: 0.5,
            category: MemoryCategory::Social,
            entities: vec!["merchant".to_string(), "potion".to_string()],
            context: HashMap::new(),
        },
        Memory {
            id: "3".to_string(),
            text: "The knight rested at the inn".to_string(),
            timestamp: 3000,
            importance: 0.2,
            valence: 0.1,
            category: MemoryCategory::Social,
            entities: vec!["knight".to_string(), "inn".to_string()],
            context: HashMap::new(),
        },
        Memory {
            id: "4".to_string(),
            text: "A dragon flew over the castle".to_string(),
            timestamp: 4000,
            importance: 0.9,
            valence: -0.2,
            category: MemoryCategory::Gameplay,
            entities: vec!["dragon".to_string(), "castle".to_string()],
            context: HashMap::new(),
        },
    ]
}

#[test]
fn test_engine_initialization() {
    let config = RetrievalConfig::default();
    let _engine = RetrievalEngine::new(config);
    // Just checking it constructs
    assert!(true);
}

#[test]
fn test_basic_search() -> Result<()> {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    let memories = create_test_memories();

    let query = RetrievalQuery {
        text: "knight".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    
    // Should find memories 1 and 3
    assert!(results.len() >= 2);
    assert!(results.iter().any(|r| r.memory.id == "1"));
    assert!(results.iter().any(|r| r.memory.id == "3"));
    
    Ok(())
}

#[test]
fn test_category_filter() -> Result<()> {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    let memories = create_test_memories();

    let query = RetrievalQuery {
        text: "knight".to_string(), // Appears in Combat and Social
        categories: vec![MemoryCategory::Combat],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    
    // Should only find memory 1 (Combat), not 3 (Social)
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].memory.id, "1");
    
    Ok(())
}

#[test]
fn test_similarity_ranking() -> Result<()> {
    let mut config = RetrievalConfig::default();
    config.similarity_threshold = 0.4; // Lower threshold to include partial matches
    let engine = RetrievalEngine::new(config);
    let memories = create_test_memories();

    // "dragon" appears in 1 and 4. 
    // Memory 1: "The brave knight fought the dragon" (6 words)
    // Memory 4: "A dragon flew over the castle" (6 words)
    // Query: "dragon"
    // Both have 1 matching word. Score = 1/1 = 1.0? 
    // Wait, calculate_similarity implementation:
    // common_words / query_words.len()
    // Query "dragon" -> 1 word.
    // Memory 1 has "dragon" -> 1 common. Score 1.0.
    // Memory 4 has "dragon" -> 1 common. Score 1.0.
    
    // Let's try a query with more words to differentiate.
    // Query: "knight dragon"
    // Memory 1: has both. Score 2/2 = 1.0.
    // Memory 3: has "knight". Score 1/2 = 0.5.
    // Memory 4: has "dragon". Score 1/2 = 0.5.
    
    let query = RetrievalQuery {
        text: "knight dragon".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    
    assert!(!results.is_empty());
    assert_eq!(results[0].memory.id, "1"); // Should be highest score
    
    // With threshold 0.4, we should get all 3
    if results.len() >= 3 {
        assert!(results[0].score > results[1].score || results[0].score == results[1].score);
    }
    
    Ok(())
}

#[test]
fn test_result_limit() -> Result<()> {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    let memories = create_test_memories();

    let query = RetrievalQuery {
        text: "the".to_string(), // Appears in all
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(2),
    };

    let results = engine.search(&query, &memories)?;
    
    assert_eq!(results.len(), 2);
    
    Ok(())
}

#[test]
fn test_empty_query() -> Result<()> {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    let memories = create_test_memories();

    let query = RetrievalQuery {
        text: "".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    
    // Empty query words -> score 0.0 -> filtered out by threshold 0.7 default
    assert!(results.is_empty());
    
    Ok(())
}

#[test]
fn test_no_matches() -> Result<()> {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    let memories = create_test_memories();

    let query = RetrievalQuery {
        text: "spaceship".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    
    assert!(results.is_empty());
    
    Ok(())
}

#[test]
fn test_custom_threshold() -> Result<()> {
    let mut config = RetrievalConfig::default();
    config.similarity_threshold = 0.1; // Low threshold
    let engine = RetrievalEngine::new(config);
    let memories = create_test_memories();

    // "knight" matches 1 and 3.
    // Query "knight spaceship" -> 2 words.
    // Memory 1: has "knight". Score 0.5.
    // Memory 3: has "knight". Score 0.5.
    // Default threshold is 0.7, so these would be filtered out.
    // With 0.1, they should be included.
    
    let query = RetrievalQuery {
        text: "knight spaceship".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    
    assert!(!results.is_empty());
    
    Ok(())
}

#[test]
fn test_multiple_categories() -> Result<()> {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    let memories = create_test_memories();

    let query = RetrievalQuery {
        text: "The".to_string(), // Capitalized to match memories 1, 2, 3
        categories: vec![MemoryCategory::Combat, MemoryCategory::Social],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    
    // Should find 1 (Combat), 2 (Social), 3 (Social). Not 4 (Gameplay).
    assert!(results.iter().any(|r| r.memory.id == "1"));
    assert!(results.iter().any(|r| r.memory.id == "2"));
    assert!(results.iter().any(|r| r.memory.id == "3"));
    assert!(!results.iter().any(|r| r.memory.id == "4"));
    
    Ok(())
}

#[test]
fn test_retrieval_result_structure() -> Result<()> {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    let memories = create_test_memories();

    let query = RetrievalQuery {
        text: "knight".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: Some(1),
    };

    let results = engine.search(&query, &memories)?;
    
    assert_eq!(results.len(), 1);
    let result = &results[0];
    
    assert!(result.score > 0.0);
    assert_eq!(result.rank, 0);
    assert_eq!(result.memory.id, "1"); // Or 3, but 1 comes first in list and sort is stable-ish or equal
    
    Ok(())
}

#[test]
fn test_case_sensitivity() -> Result<()> {
    // The current implementation splits by whitespace and checks exact match.
    // "Knight" vs "knight".
    // Let's check if it handles case.
    // The implementation in retrieval.rs:
    // common_words = query_words.iter().filter(|word| content_words.contains(word)).count()
    // It does NOT lowercase. So it is case sensitive.
    
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    
    let memories = vec![
        Memory {
            id: "1".to_string(),
            text: "The Knight fought".to_string(),
            timestamp: 1000,
            importance: 0.5,
            valence: 0.0,
            category: MemoryCategory::Combat,
            entities: vec![],
            context: HashMap::new(),
        }
    ];

    let query = RetrievalQuery {
        text: "knight".to_string(), // lowercase
        categories: vec![],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    
    // Expect empty because "Knight" != "knight" in current implementation
    assert!(results.is_empty());
    
    Ok(())
}

#[test]
fn test_exact_word_matching() -> Result<()> {
    // "knights" vs "knight"
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    
    let memories = vec![
        Memory {
            id: "1".to_string(),
            text: "The knights fought".to_string(),
            timestamp: 1000,
            importance: 0.5,
            valence: 0.0,
            category: MemoryCategory::Combat,
            entities: vec![],
            context: HashMap::new(),
        }
    ];

    let query = RetrievalQuery {
        text: "knight".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    
    // Expect empty because "knights" != "knight"
    assert!(results.is_empty());
    
    Ok(())
}

#[test]
fn test_config_defaults() {
    let config = RetrievalConfig::default();
    assert_eq!(config.max_results, 10);
    assert_eq!(config.similarity_threshold, 0.7);
    assert!(config.use_semantic_search);
}

#[test]
fn test_query_construction() {
    let query = RetrievalQuery {
        text: "test".to_string(),
        categories: vec![MemoryCategory::Gameplay],
        filters: HashMap::new(),
        limit: Some(5),
    };
    
    assert_eq!(query.text, "test");
    assert_eq!(query.categories.len(), 1);
    assert_eq!(query.limit, Some(5));
}

#[test]
fn test_empty_memories() -> Result<()> {
    let config = RetrievalConfig::default();
    let engine = RetrievalEngine::new(config);
    let memories = vec![];

    let query = RetrievalQuery {
        text: "test".to_string(),
        categories: vec![],
        filters: HashMap::new(),
        limit: None,
    };

    let results = engine.search(&query, &memories)?;
    assert!(results.is_empty());
    
    Ok(())
}
