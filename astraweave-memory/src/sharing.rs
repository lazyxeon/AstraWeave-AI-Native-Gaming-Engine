//! Memory sharing and collaborative memory systems
//!
//! This module handles sharing memories between different agents/entities
//! while respecting privacy levels and sharing permissions.

use crate::memory_types::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for memory sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConfig {
    /// Default sharing permissions for new memories
    pub default_sharing_type: SharingType,
    /// Default privacy level
    pub default_privacy_level: PrivacyLevel,
    /// Whether to allow automatic sharing based on context
    pub auto_sharing_enabled: bool,
    /// Maximum number of entities that can access a shared memory
    pub max_authorized_entities: usize,
}

impl Default for SharingConfig {
    fn default() -> Self {
        Self {
            default_sharing_type: SharingType::Restricted,
            default_privacy_level: PrivacyLevel::Personal,
            auto_sharing_enabled: false,
            max_authorized_entities: 10,
        }
    }
}

/// Request to share a memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRequest {
    /// ID of the memory to share
    pub memory_id: String,
    /// Target entity to share with
    pub target_entity: String,
    /// Type of sharing requested
    pub sharing_type: SharingType,
    /// Reason for sharing (for audit trail)
    pub reason: String,
    /// Conditions for sharing
    pub conditions: Vec<String>,
}

/// Result of a sharing operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingResult {
    /// Whether the sharing was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
    /// Shared memory content (based on sharing type)
    pub shared_content: Option<SharedMemoryContent>,
    /// Sharing metadata
    pub sharing_metadata: SharingMetadata,
}

/// Content of a shared memory (filtered based on sharing type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemoryContent {
    /// Filtered memory content
    pub content: String,
    /// Memory type
    pub memory_type: MemoryType,
    /// Timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Importance level
    pub importance: f32,
    /// Associated entities (filtered)
    pub entities: Vec<String>,
}

/// Audit log entry for memory sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingAuditEntry {
    /// Unique ID for this sharing event
    pub event_id: String,
    /// Memory that was shared
    pub memory_id: String,
    /// Source entity (sharer)
    pub source_entity: String,
    /// Target entity (receiver)
    pub target_entity: String,
    /// Type of sharing
    pub sharing_type: SharingType,
    /// When the sharing occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Success status
    pub success: bool,
    /// Reason for sharing
    pub reason: String,
}

/// Memory sharing engine
#[derive(Debug)]
pub struct SharingEngine {
    config: SharingConfig,
    audit_log: Vec<SharingAuditEntry>,
}

impl SharingEngine {
    /// Create a new sharing engine
    pub fn new(config: SharingConfig) -> Self {
        Self {
            config,
            audit_log: Vec::new(),
        }
    }

    /// Share a memory with another entity
    pub fn share_memory(
        &mut self,
        request: &ShareRequest,
        memory: &Memory,
        requesting_entity: &str,
    ) -> Result<SharingResult> {
        // Check if the memory has sharing metadata
        let sharing_metadata = self.get_or_create_sharing_metadata(memory);

        // Validate sharing permissions
        if let Err(error) = self.validate_sharing_request(request, &sharing_metadata, requesting_entity) {
            let result = SharingResult {
                success: false,
                error_message: Some(error.to_string()),
                shared_content: None,
                sharing_metadata: sharing_metadata.clone(),
            };

            // Log the failed attempt
            self.log_sharing_event(request, requesting_entity, false, &error.to_string());

            return Ok(result);
        }

        // Generate shared content based on sharing type
        let shared_content = self.generate_shared_content(memory, &request.sharing_type)?;

        // Create successful result
        let mut updated_metadata = sharing_metadata.clone();
        if !updated_metadata.authorized_entities.contains(&request.target_entity) {
            updated_metadata.authorized_entities.push(request.target_entity.clone());
        }

        let result = SharingResult {
            success: true,
            error_message: None,
            shared_content: Some(shared_content),
            sharing_metadata: updated_metadata,
        };

        // Log successful sharing
        self.log_sharing_event(request, requesting_entity, true, "Success");

        Ok(result)
    }

    /// Validate a sharing request
    fn validate_sharing_request(
        &self,
        request: &ShareRequest,
        metadata: &SharingMetadata,
        requesting_entity: &str,
    ) -> Result<()> {
        // Check if sharing is allowed at all
        if !metadata.shareable {
            return Err(anyhow!("Memory is not shareable"));
        }

        // Check privacy level
        match metadata.privacy_level {
            PrivacyLevel::Secret => {
                return Err(anyhow!("Memory has secret privacy level and cannot be shared"));
            }
            PrivacyLevel::Personal => {
                if !metadata.authorized_entities.contains(&requesting_entity.to_string()) {
                    return Err(anyhow!("Requesting entity not authorized for personal memory"));
                }
            }
            PrivacyLevel::Group => {
                // For group level, check if both entities are in authorized list
                if !metadata.authorized_entities.contains(&requesting_entity.to_string())
                    || !metadata.authorized_entities.contains(&request.target_entity)
                {
                    return Err(anyhow!("Entity not in authorized group"));
                }
            }
            PrivacyLevel::Public => {
                // Public memories can be shared by anyone
            }
        }

        // Check sharing type compatibility
        match (&metadata.sharing_type, &request.sharing_type) {
            (SharingType::Restricted, _) => {
                return Err(anyhow!("Memory has restricted sharing type"));
            }
            (SharingType::Metadata, SharingType::Full | SharingType::Summary) => {
                return Err(anyhow!("Memory only allows metadata sharing"));
            }
            (SharingType::Summary, SharingType::Full) => {
                return Err(anyhow!("Memory only allows summary sharing"));
            }
            _ => {} // Compatible sharing types
        }

        // Check entity limit
        if metadata.authorized_entities.len() >= self.config.max_authorized_entities {
            return Err(anyhow!("Maximum authorized entities limit reached"));
        }

        Ok(())
    }

    /// Generate shared content based on sharing type
    fn generate_shared_content(
        &self,
        memory: &Memory,
        sharing_type: &SharingType,
    ) -> Result<SharedMemoryContent> {
        let content = match sharing_type {
            SharingType::Full => memory.content.text.clone(),
            SharingType::Summary => self.generate_summary(&memory.content.text),
            SharingType::Metadata => format!("Memory of type {:?} created on {}",
                memory.memory_type,
                memory.metadata.created_at.format("%Y-%m-%d")
            ),
            SharingType::Restricted => {
                return Err(anyhow!("Cannot generate content for restricted sharing type"));
            }
        };

        // Filter entities based on sharing type
        let entities = match sharing_type {
            SharingType::Full => memory.content.context.participants.clone(),
            SharingType::Summary | SharingType::Metadata => {
                // Only include non-sensitive entities
                memory
                    .content
                    .context
                    .participants
                    .iter()
                    .filter(|entity| !entity.starts_with("private:"))
                    .cloned()
                    .collect()
            }
            SharingType::Restricted => Vec::new(),
        };

        Ok(SharedMemoryContent {
            content,
            memory_type: memory.memory_type.clone(),
            created_at: memory.metadata.created_at,
            importance: memory.metadata.importance,
            entities,
        })
    }

    /// Generate a summary of memory content
    fn generate_summary(&self, content: &str) -> String {
        // Simple summarization (in practice, use LLM)
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.len() <= 20 {
            return content.to_string();
        }

        let summary_length = (words.len() / 3).max(10);
        let mut summary_words = Vec::new();
        summary_words.extend_from_slice(&words[..summary_length]);
        summary_words.push("[...]");

        summary_words.join(" ")
    }

    /// Get or create sharing metadata for a memory
    fn get_or_create_sharing_metadata(&self, _memory: &Memory) -> SharingMetadata {
        // In practice, this would be stored with the memory
        // For now, create default metadata
        SharingMetadata {
            shareable: true,
            authorized_entities: vec!["owner".to_string()],
            sharing_type: self.config.default_sharing_type.clone(),
            privacy_level: self.config.default_privacy_level.clone(),
            sharing_conditions: Vec::new(),
        }
    }

    /// Log a sharing event
    fn log_sharing_event(
        &mut self,
        request: &ShareRequest,
        requesting_entity: &str,
        success: bool,
        message: &str,
    ) {
        let entry = SharingAuditEntry {
            event_id: uuid::Uuid::new_v4().to_string(),
            memory_id: request.memory_id.clone(),
            source_entity: requesting_entity.to_string(),
            target_entity: request.target_entity.clone(),
            sharing_type: request.sharing_type.clone(),
            timestamp: chrono::Utc::now(),
            success,
            reason: if success {
                request.reason.clone()
            } else {
                message.to_string()
            },
        };

        self.audit_log.push(entry);
    }

    /// Get sharing audit log for a memory
    pub fn get_audit_log(&self, memory_id: &str) -> Vec<&SharingAuditEntry> {
        self.audit_log
            .iter()
            .filter(|entry| entry.memory_id == memory_id)
            .collect()
    }

    /// Get all memories that an entity has access to
    pub fn get_accessible_memories(
        &self,
        entity_id: &str,
        all_memories: &[Memory],
    ) -> Vec<String> {
        let mut accessible = Vec::new();

        for memory in all_memories {
            let metadata = self.get_or_create_sharing_metadata(memory);

            // Check if entity has access
            let has_access = match metadata.privacy_level {
                PrivacyLevel::Public => true,
                PrivacyLevel::Group | PrivacyLevel::Personal => {
                    metadata.authorized_entities.contains(&entity_id.to_string())
                }
                PrivacyLevel::Secret => false,
            };

            if has_access {
                accessible.push(memory.id.clone());
            }
        }

        accessible
    }

    /// Create a shared memory cluster
    pub fn create_shared_cluster(
        &mut self,
        cluster_name: String,
        memory_ids: Vec<String>,
        _authorized_entities: Vec<String>,
        _privacy_level: PrivacyLevel,
    ) -> Result<MemoryCluster> {
        // Validate that all memories can be shared
        // In practice, you would check each memory's sharing permissions

        let mut cluster = MemoryCluster::new(
            cluster_name,
            ClusterType::Concept, // Default cluster type for shared clusters
            "shared_cluster".to_string(),
        );

        // Add memories to cluster
        for memory_id in memory_ids {
            cluster.add_memory(memory_id);
        }

        // Set cluster importance based on included memories
        // This would be calculated based on the actual memories
        cluster.importance = 0.7;

        Ok(cluster)
    }

    /// Revoke sharing access for an entity
    pub fn revoke_access(
        &mut self,
        memory_id: &str,
        target_entity: &str,
        revoking_entity: &str,
    ) -> Result<()> {
        // Log the revocation
        let entry = SharingAuditEntry {
            event_id: uuid::Uuid::new_v4().to_string(),
            memory_id: memory_id.to_string(),
            source_entity: revoking_entity.to_string(),
            target_entity: target_entity.to_string(),
            sharing_type: SharingType::Restricted,
            timestamp: chrono::Utc::now(),
            success: true,
            reason: "Access revoked".to_string(),
        };

        self.audit_log.push(entry);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharing_engine_creation() {
        let config = SharingConfig::default();
        let engine = SharingEngine::new(config);
        assert_eq!(engine.config.max_authorized_entities, 10);
    }

    #[test]
    fn test_memory_sharing() {
        let mut engine = SharingEngine::new(SharingConfig::default());
        let memory = Memory::episodic(
            "Met with the team today".to_string(),
            vec!["Alice".to_string(), "Bob".to_string()],
            Some("office".to_string()),
        );

        let request = ShareRequest {
            memory_id: memory.id.clone(),
            target_entity: "colleague".to_string(),
            sharing_type: SharingType::Summary,
            reason: "Project collaboration".to_string(),
            conditions: Vec::new(),
        };

        let result = engine.share_memory(&request, &memory, "owner").unwrap();
        assert!(result.success);
        assert!(result.shared_content.is_some());
    }

    #[test]
    fn test_sharing_validation() {
        let engine = SharingEngine::new(SharingConfig::default());
        let metadata = SharingMetadata {
            shareable: false,
            authorized_entities: vec!["owner".to_string()],
            sharing_type: SharingType::Restricted,
            privacy_level: PrivacyLevel::Personal,
            sharing_conditions: Vec::new(),
        };

        let request = ShareRequest {
            memory_id: "test".to_string(),
            target_entity: "other".to_string(),
            sharing_type: SharingType::Full,
            reason: "test".to_string(),
            conditions: Vec::new(),
        };

        let result = engine.validate_sharing_request(&request, &metadata, "owner");
        assert!(result.is_err());
    }

    #[test]
    fn test_audit_logging() {
        let mut engine = SharingEngine::new(SharingConfig::default());
        let memory = Memory::semantic("Test knowledge".to_string(), "test".to_string());

        let request = ShareRequest {
            memory_id: memory.id.clone(),
            target_entity: "researcher".to_string(),
            sharing_type: SharingType::Full,
            reason: "Research collaboration".to_string(),
            conditions: Vec::new(),
        };

        engine.share_memory(&request, &memory, "owner").unwrap();

        let audit_entries = engine.get_audit_log(&memory.id);
        assert_eq!(audit_entries.len(), 1);
        assert!(audit_entries[0].success);
    }
}