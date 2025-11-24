use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "rag")]
use crate::rag::{RagConfig, RagDocument, RagPipeline};
#[cfg(feature = "rag")]
use astraweave_embeddings::EmbeddingClient;

/// Configuration for a persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaConfig {
    pub name: String,
    pub description: String,
    pub traits: Vec<String>,
    pub voice_style: String,
    #[cfg(feature = "rag")]
    pub rag_config: RagConfig,
}

impl Default for PersonaConfig {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            description: "A helpful AI assistant.".to_string(),
            traits: vec!["helpful".to_string()],
            voice_style: "neutral".to_string(),
            #[cfg(feature = "rag")]
            rag_config: RagConfig::default(),
        }
    }
}

/// Manager for LLM personas and their context
pub struct LlmPersonaManager {
    config: PersonaConfig,
    #[cfg(feature = "rag")]
    rag: Arc<RagPipeline>,
    // In-memory short-term memory or other state could go here
}

impl LlmPersonaManager {
    #[cfg(feature = "rag")]
    pub fn new(
        config: PersonaConfig,
        client: Arc<dyn EmbeddingClient + Send + Sync>,
    ) -> Self {
        let rag = Arc::new(RagPipeline::new(config.rag_config.clone(), client));
        Self { config, rag }
    }

    #[cfg(not(feature = "rag"))]
    pub fn new(config: PersonaConfig) -> Self {
        Self { config }
    }

    /// Add a memory/document to the persona's knowledge base
    #[cfg(feature = "rag")]
    pub async fn add_memory(&self, content: String, importance: f32) -> Result<()> {
        let doc = RagDocument {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            metadata: HashMap::new(),
            importance,
        };
        self.rag.add_document(doc).await
    }

    /// Retrieve relevant context for a query
    #[cfg(feature = "rag")]
    pub async fn get_context(&self, query: &str) -> Result<String> {
        let context = self.rag.retrieve(query).await?;
        Ok(context.to_string())
    }

    #[cfg(not(feature = "rag"))]
    pub async fn get_context(&self, _query: &str) -> Result<String> {
        Ok(String::new())
    }

    /// Inject context into a prompt
    #[cfg(feature = "rag")]
    pub async fn inject_context(&self, prompt: &str) -> Result<String> {
        // Retrieve context first
        let context = self.rag.retrieve(prompt).await?;
        Ok(self.rag.inject_context(prompt, &context))
    }

    #[cfg(not(feature = "rag"))]
    pub async fn inject_context(&self, prompt: &str) -> Result<String> {
        Ok(prompt.to_string())
    }

    /// Perform maintenance tasks (pruning, optimization)
    pub fn maintenance(&self) -> Result<()> {
        #[cfg(feature = "rag")]
        self.rag.maintenance()?;
        Ok(())
    }

    /// Get the system prompt for this persona, optionally including context
    pub fn get_system_prompt(&self, context: Option<&str>) -> String {
        let traits = self.config.traits.join(", ");
        let base = format!(
            "You are {}, {}. Your traits are: {}. Speak in a {} style.",
            self.config.name, self.config.description, traits, self.config.voice_style
        );

        if let Some(ctx) = context {
            format!("{}\n\nContext:\n{}", base, ctx)
        } else {
            base
        }
    }

    /// Get the number of stored memories
    #[cfg(feature = "rag")]
    pub fn document_count(&self) -> usize {
        self.rag.document_count()
    }

    #[cfg(not(feature = "rag"))]
    pub fn document_count(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "rag")]
    use astraweave_embeddings::MockEmbeddingClient;

    #[test]
    fn test_persona_config_default() {
        let config = PersonaConfig::default();
        assert_eq!(config.name, "Default");
    }

    #[cfg(feature = "rag")]
    #[tokio::test]
    async fn test_persona_manager_rag() {
        let client = Arc::new(MockEmbeddingClient::new());
        let config = PersonaConfig::default();
        let manager = LlmPersonaManager::new(config, client);

        manager.add_memory("I like apples.".to_string(), 0.8).await.unwrap();
        let ctx = manager.get_context("What do I like?").await.unwrap();
        
        // Mock client behavior is random/deterministic but we check it runs
        assert!(!ctx.is_empty() || ctx.contains("No relevant context"));
    }
}
