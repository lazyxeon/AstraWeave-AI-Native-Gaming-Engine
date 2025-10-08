use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Core memory structure representing different types of memories in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub memory_type: MemoryType,
    pub content: MemoryContent,
    pub metadata: MemoryMetadata,
    pub associations: Vec<MemoryAssociation>,
    pub embedding: Option<Vec<f32>>,
}

/// Types of memories in the hierarchical memory system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    /// Short-term sensory impressions
    Sensory,
    /// Active working memory for current tasks
    Working,
    /// Episodic memories of specific events
    Episodic,
    /// Semantic knowledge and facts
    Semantic,
    /// Procedural knowledge (skills, habits)
    Procedural,
    /// Emotional memories with affective content
    Emotional,
    /// Social memories about relationships and interactions
    Social,
}

impl Default for MemoryType {
    fn default() -> Self {
        MemoryType::Working
    }
}

/// Content of a memory with structured data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContent {
    /// Primary text content
    pub text: String,
    /// Structured data associated with the memory
    pub data: serde_json::Value,
    /// Sensory information if applicable
    pub sensory_data: Option<SensoryData>,
    /// Emotional context
    pub emotional_context: Option<EmotionalContext>,
    /// Spatial/temporal context
    pub context: SpatialTemporalContext,
}

/// Sensory information associated with memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensoryData {
    pub visual: Option<String>,
    pub auditory: Option<String>,
    pub tactile: Option<String>,
    pub environmental: Option<String>,
}

/// Emotional context of a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalContext {
    pub primary_emotion: String,
    pub intensity: f32, // 0.0 to 1.0
    pub valence: f32,   // -1.0 (negative) to 1.0 (positive)
    pub arousal: f32,   // 0.0 (calm) to 1.0 (excited)
}

/// Spatial and temporal context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialTemporalContext {
    pub location: Option<String>,
    pub time_period: Option<String>,
    pub duration: Option<u64>, // milliseconds
    pub participants: Vec<String>,
    pub related_events: Vec<String>,
}

/// Metadata about a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// When the memory was created
    pub created_at: DateTime<Utc>,
    /// When the memory was last accessed
    pub last_accessed: DateTime<Utc>,
    /// How many times the memory has been accessed
    pub access_count: u32,
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
    /// Confidence in the memory's accuracy
    pub confidence: f32,
    /// Source of the memory (experience, learning, etc.)
    pub source: MemorySource,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Whether the memory can be forgotten
    pub permanent: bool,
    /// Strength of the memory (affects retrieval probability)
    pub strength: f32,
    /// Age-related decay factor
    pub decay_factor: f32,
}

/// Source of a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemorySource {
    DirectExperience,
    Conversation,
    Learning,
    Inference,
    SharedMemory,
    SystemGenerated,
}

/// Association between memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAssociation {
    /// ID of the associated memory
    pub memory_id: String,
    /// Type of association
    pub association_type: AssociationType,
    /// Strength of the association (0.0 to 1.0)
    pub strength: f32,
    /// When this association was formed
    pub formed_at: DateTime<Utc>,
}

/// Types of associations between memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssociationType {
    /// Temporal - happened at the same time
    Temporal,
    /// Spatial - happened in the same place
    Spatial,
    /// Causal - one caused the other
    Causal,
    /// Conceptual - related concepts
    Conceptual,
    /// Emotional - similar emotional content
    Emotional,
    /// Sequential - one followed the other
    Sequential,
    /// Contrast - opposite or contrasting
    Contrast,
}

/// Memory cluster grouping related memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCluster {
    pub id: String,
    pub name: String,
    pub cluster_type: ClusterType,
    pub memory_ids: Vec<String>,
    pub central_concept: String,
    pub importance: f32,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Types of memory clusters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterType {
    Event,      // Memories of a specific event
    Person,     // Memories related to a person
    Location,   // Memories tied to a location
    Concept,    // Memories about a concept
    Skill,      // Memories forming a skill
    Relationship, // Memories about a relationship
}

/// Memory consolidation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationState {
    /// Level of consolidation (0.0 = not consolidated, 1.0 = fully consolidated)
    pub consolidation_level: f32,
    /// Number of consolidation passes
    pub passes: u32,
    /// Last consolidation time
    pub last_consolidation: DateTime<Utc>,
    /// Whether consolidation is needed
    pub needs_consolidation: bool,
    /// Priority for consolidation
    pub priority: f32,
}

/// Forgetting curve parameters for a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingCurve {
    /// Initial strength of the memory
    pub initial_strength: f32,
    /// Decay rate constant
    pub decay_rate: f32,
    /// Half-life of the memory in days
    pub half_life: f32,
    /// Minimum strength threshold for retention
    pub retention_threshold: f32,
    /// Whether the memory is immune to forgetting
    pub immune: bool,
}

/// Memory retrieval context for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalContext {
    /// Query text
    pub query: String,
    /// Current emotional state
    pub emotional_state: Option<EmotionalContext>,
    /// Current location
    pub location: Option<String>,
    /// Recent memories to consider
    pub recent_memory_ids: Vec<String>,
    /// Preferred memory types
    pub preferred_types: Vec<MemoryType>,
    /// Time window for temporal filtering
    pub time_window: Option<TimeWindow>,
    /// Maximum number of memories to retrieve
    pub limit: usize,
}

/// Time window for memory filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Memory sharing permissions and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingMetadata {
    /// Whether this memory can be shared
    pub shareable: bool,
    /// List of agents/entities that can access this memory
    pub authorized_entities: Vec<String>,
    /// Type of sharing allowed
    pub sharing_type: SharingType,
    /// Privacy level
    pub privacy_level: PrivacyLevel,
    /// Conditions for sharing
    pub sharing_conditions: Vec<String>,
}

/// Types of memory sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingType {
    /// Full memory content can be shared
    Full,
    /// Only summary can be shared
    Summary,
    /// Only existence/metadata can be shared
    Metadata,
    /// Cannot be shared
    Restricted,
}

/// Privacy levels for memories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Public,     // Can be shared with anyone
    Group,      // Can be shared within group
    Personal,   // Only for this entity
    Secret,     // Highly protected
}

impl Memory {
    /// Create a new memory
    pub fn new(memory_type: MemoryType, content: MemoryContent) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            memory_type,
            content,
            metadata: MemoryMetadata {
                created_at: Utc::now(),
                last_accessed: Utc::now(),
                access_count: 0,
                importance: 0.5,
                confidence: 1.0,
                source: MemorySource::DirectExperience,
                tags: Vec::new(),
                permanent: false,
                strength: 1.0,
                decay_factor: 1.0,
            },
            associations: Vec::new(),
            embedding: None,
        }
    }

    /// Create a sensory memory
    pub fn sensory(text: String, sensory_data: Option<SensoryData>) -> Self {
        let content = MemoryContent {
            text,
            data: serde_json::Value::Null,
            sensory_data,
            emotional_context: None,
            context: SpatialTemporalContext {
                location: None,
                time_period: None,
                duration: None,
                participants: Vec::new(),
                related_events: Vec::new(),
            },
        };

        let mut memory = Self::new(MemoryType::Sensory, content);
        memory.metadata.importance = 0.2; // Sensory memories start with low importance
        memory.metadata.decay_factor = 2.0; // Decay faster
        memory
    }

    /// Create an episodic memory
    pub fn episodic(text: String, participants: Vec<String>, location: Option<String>) -> Self {
        let content = MemoryContent {
            text,
            data: serde_json::Value::Null,
            sensory_data: None,
            emotional_context: None,
            context: SpatialTemporalContext {
                location,
                time_period: Some("current".to_string()),
                duration: None,
                participants,
                related_events: Vec::new(),
            },
        };

        let mut memory = Self::new(MemoryType::Episodic, content);
        memory.metadata.importance = 0.7; // Episodic memories are more important
        memory
    }

    /// Create a semantic memory
    pub fn semantic(text: String, concept: String) -> Self {
        let mut data = HashMap::new();
        data.insert("concept", serde_json::json!(concept));

        let content = MemoryContent {
            text,
            data: serde_json::to_value(data).unwrap_or_default(),
            sensory_data: None,
            emotional_context: None,
            context: SpatialTemporalContext {
                location: None,
                time_period: None,
                duration: None,
                participants: Vec::new(),
                related_events: Vec::new(),
            },
        };

        let mut memory = Self::new(MemoryType::Semantic, content);
        memory.metadata.importance = 0.8; // Semantic memories are very important
        memory.metadata.permanent = true; // Facts should be permanent
        memory
    }

    /// Update access information
    pub fn accessed(&mut self) {
        self.metadata.last_accessed = Utc::now();
        self.metadata.access_count += 1;
        
        // Strengthen memory through access (spaced repetition effect)
        self.metadata.strength = (self.metadata.strength + 0.1).min(1.0);
    }

    /// Check if memory should be forgotten based on current strength
    pub fn should_forget(&self, threshold: f32) -> bool {
        if self.metadata.permanent {
            return false;
        }
        
        let current_strength = self.calculate_current_strength();
        current_strength < threshold
    }

    /// Calculate current memory strength based on age and access
    pub fn calculate_current_strength(&self) -> f32 {
        let age_days = (Utc::now() - self.metadata.created_at).num_days() as f32;
        let time_since_access = (Utc::now() - self.metadata.last_accessed).num_days() as f32;
        
        // Apply forgetting curve: strength = initial * e^(-decay * time)
        let base_strength = self.metadata.strength * (-0.1 * age_days * self.metadata.decay_factor).exp();
        
        // Boost from recent access
        let access_boost = if time_since_access < 1.0 {
            0.2
        } else {
            0.0
        };
        
        (base_strength + access_boost).min(1.0).max(0.0)
    }

    /// Add an association to another memory
    pub fn add_association(&mut self, memory_id: String, association_type: AssociationType, strength: f32) {
        let association = MemoryAssociation {
            memory_id,
            association_type,
            strength: strength.max(0.0).min(1.0),
            formed_at: Utc::now(),
        };
        
        self.associations.push(association);
    }

    /// Get memories strongly associated with this one
    pub fn get_strong_associations(&self, min_strength: f32) -> Vec<&MemoryAssociation> {
        self.associations
            .iter()
            .filter(|assoc| assoc.strength >= min_strength)
            .collect()
    }

    /// Check if memory matches retrieval context
    pub fn matches_context(&self, context: &RetrievalContext) -> bool {
        // Check memory type preference
        if !context.preferred_types.is_empty() && !context.preferred_types.contains(&self.memory_type) {
            return false;
        }
        
        // Check time window
        if let Some(window) = &context.time_window {
            if self.metadata.created_at < window.start || self.metadata.created_at > window.end {
                return false;
            }
        }
        
        // Check location
        if let Some(query_location) = &context.location {
            if let Some(memory_location) = &self.content.context.location {
                if memory_location != query_location {
                    return false;
                }
            }
        }
        
        true
    }

    /// Calculate relevance score for a query
    pub fn calculate_relevance(&self, context: &RetrievalContext) -> f32 {
        let mut relevance = 0.0;
        
        // Text similarity (simplified - would use embeddings in practice)
        let query_words: Vec<&str> = context.query.split_whitespace().collect();
        let memory_words: Vec<&str> = self.content.text.split_whitespace().collect();
        
        let common_words = query_words
            .iter()
            .filter(|word| memory_words.contains(word))
            .count();
        
        let text_similarity = if !query_words.is_empty() {
            common_words as f32 / query_words.len() as f32
        } else {
            0.0
        };
        
        relevance += text_similarity * 0.4;
        
        // Importance and strength
        relevance += self.metadata.importance * 0.3;
        relevance += self.calculate_current_strength() * 0.2;
        
        // Recency bonus
        let age_days = (Utc::now() - self.metadata.created_at).num_days() as f32;
        let recency_bonus = if age_days < 7.0 {
            0.1 * (7.0 - age_days) / 7.0
        } else {
            0.0
        };
        relevance += recency_bonus;
        
        relevance.min(1.0)
    }
}

impl MemoryCluster {
    pub fn new(name: String, cluster_type: ClusterType, central_concept: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            cluster_type,
            memory_ids: Vec::new(),
            central_concept,
            importance: 0.5,
            created_at: Utc::now(),
            last_updated: Utc::now(),
        }
    }

    /// Add a memory to this cluster
    pub fn add_memory(&mut self, memory_id: String) {
        if !self.memory_ids.contains(&memory_id) {
            self.memory_ids.push(memory_id);
            self.last_updated = Utc::now();
        }
    }

    /// Remove a memory from this cluster
    pub fn remove_memory(&mut self, memory_id: &str) {
        self.memory_ids.retain(|id| id != memory_id);
        self.last_updated = Utc::now();
    }

    /// Calculate cluster importance based on member memories
    pub fn calculate_importance(&self, memories: &[&Memory]) -> f32 {
        if memories.is_empty() {
            return 0.0;
        }

        let total_importance: f32 = memories.iter().map(|m| m.metadata.importance).sum();
        let avg_importance = total_importance / memories.len() as f32;
        
        // Boost importance for larger clusters
        let size_bonus = (memories.len() as f32 / 10.0).min(0.2);
        
        (avg_importance + size_bonus).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::sensory("I see a red apple".to_string(), None);
        assert_eq!(memory.memory_type, MemoryType::Sensory);
        assert_eq!(memory.content.text, "I see a red apple");
        assert!(memory.metadata.importance < 0.5);
    }

    #[test]
    fn test_memory_strength_calculation() {
        let mut memory = Memory::sensory("Test memory".to_string(), None);
        let initial_strength = memory.calculate_current_strength();
        
        // Simulate aging
        memory.metadata.created_at = Utc::now() - chrono::Duration::days(10);
        let aged_strength = memory.calculate_current_strength();
        
        assert!(aged_strength < initial_strength);
    }

    #[test]
    fn test_memory_associations() {
        let mut memory = Memory::episodic(
            "Met John at the market".to_string(),
            vec!["John".to_string()],
            Some("market".to_string()),
        );

        memory.add_association("other_memory_id".to_string(), AssociationType::Temporal, 0.8);

        assert_eq!(memory.associations.len(), 1);
        assert_eq!(memory.associations[0].association_type, AssociationType::Temporal);
    }

    #[test]
    fn test_cluster_management() {
        let mut cluster = MemoryCluster::new(
            "John Memories".to_string(),
            ClusterType::Person,
            "John".to_string(),
        );

        cluster.add_memory("memory1".to_string());
        cluster.add_memory("memory2".to_string());

        assert_eq!(cluster.memory_ids.len(), 2);

        cluster.remove_memory("memory1");
        assert_eq!(cluster.memory_ids.len(), 1);
    }

    #[test]
    fn test_retrieval_context_matching() {
        let memory = Memory::episodic(
            "Visited the library".to_string(),
            vec!["librarian".to_string()],
            Some("library".to_string()),
        );

        let context = RetrievalContext {
            query: "library".to_string(),
            emotional_state: None,
            location: Some("library".to_string()),
            recent_memory_ids: Vec::new(),
            preferred_types: vec![MemoryType::Episodic],
            time_window: None,
            limit: 10,
        };

        assert!(memory.matches_context(&context));

        let relevance = memory.calculate_relevance(&context);
        assert!(relevance > 0.0);
    }
}