/*!
# Conversation History

Core conversation history management with efficient storage and retrieval.
*/

use crate::{ContextConfig, ContextMetrics, Message, OverflowStrategy, Role, TokenCounter};
use anyhow::{anyhow, Result};
use astraweave_llm::LlmClient;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

/// Main conversation history manager
pub struct ConversationHistory {
    /// Configuration
    config: ContextConfig,

    /// Messages in chronological order
    messages: Arc<RwLock<VecDeque<Message>>>,

    /// Summary of older messages (if using summarization)
    summary: Arc<RwLock<Option<String>>>,

    /// Token counter
    token_counter: TokenCounter,

    /// Performance metrics
    metrics: Arc<RwLock<ContextMetrics>>,

    /// LLM client for summarization (optional)
    llm_client: Option<Arc<dyn LlmClient>>,
}

impl ConversationHistory {
    /// Create a new conversation history
    pub fn new(config: ContextConfig) -> Self {
        let token_counter = TokenCounter::new(&config.encoding_model);

        Self {
            config,
            messages: Arc::new(RwLock::new(VecDeque::new())),
            summary: Arc::new(RwLock::new(None)),
            token_counter,
            metrics: Arc::new(RwLock::new(ContextMetrics::default())),
            llm_client: None,
        }
    }

    /// Create with LLM client for summarization
    pub fn with_llm_client(config: ContextConfig, llm_client: Arc<dyn LlmClient>) -> Self {
        let token_counter = TokenCounter::new(&config.encoding_model);

        Self {
            config,
            messages: Arc::new(RwLock::new(VecDeque::new())),
            summary: Arc::new(RwLock::new(None)),
            token_counter,
            metrics: Arc::new(RwLock::new(ContextMetrics::default())),
            llm_client: Some(llm_client),
        }
    }

    /// Add a message to the conversation
    pub async fn add_message(&self, role: Role, content: String) -> Result<String> {
        let start_time = std::time::Instant::now();

        let mut message = Message::new(role, content);

        // Count tokens in the message
        message.token_count = self.token_counter.count_tokens(&message.content)?;

        let message_id = message.id.clone();

        // Add to messages
        {
            let mut messages = self.messages.write();
            messages.push_back(message);
        }

        // Check if we need to prune
        self.prune_if_needed().await?;

        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.total_messages = self.messages.read().len();
            metrics.processing_time_ms += start_time.elapsed().as_millis() as u64;
            self.update_metrics(&mut metrics);
        }

        Ok(message_id)
    }

    /// Add a message with metadata
    pub async fn add_message_with_metadata(
        &self,
        role: Role,
        content: String,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<String> {
        let mut message = Message::new(role, content);
        message.metadata = metadata;
        message.token_count = self.token_counter.count_tokens(&message.content)?;

        let message_id = message.id.clone();

        {
            let mut messages = self.messages.write();
            messages.push_back(message);
        }

        self.prune_if_needed().await?;

        Ok(message_id)
    }

    /// Get context for LLM prompt within token limit
    pub async fn get_context(&self, max_tokens: usize) -> Result<String> {
        let start_time = std::time::Instant::now();

        let context = self.build_context_string(max_tokens).await?;

        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.processing_time_ms += start_time.elapsed().as_millis() as u64;
        }

        Ok(context)
    }

    /// Get recent messages up to a limit
    pub fn get_recent_messages(&self, limit: usize) -> Vec<Message> {
        let messages = self.messages.read();
        messages.iter().rev().take(limit).rev().cloned().collect()
    }

    /// Get messages by role
    pub fn get_messages_by_role(&self, role: Role) -> Vec<Message> {
        let messages = self.messages.read();
        messages
            .iter()
            .filter(|m| m.role == role)
            .cloned()
            .collect()
    }

    /// Get total token count
    pub fn get_total_tokens(&self) -> usize {
        let messages = self.messages.read();
        let summary_tokens = if let Some(ref summary) = *self.summary.read() {
            self.token_counter.count_tokens(summary).unwrap_or(0)
        } else {
            0
        };

        let message_tokens: usize = messages.iter().map(|m| m.token_count).sum();
        message_tokens + summary_tokens
    }

    /// Get conversation metrics
    pub fn get_metrics(&self) -> ContextMetrics {
        let metrics = self.metrics.read();
        metrics.clone()
    }

    /// Clear all messages and summary
    pub fn clear(&self) {
        {
            let mut messages = self.messages.write();
            messages.clear();
        }

        {
            let mut summary = self.summary.write();
            *summary = None;
        }

        {
            let mut metrics = self.metrics.write();
            *metrics = ContextMetrics::default();
        }
    }

    /// Build context string within token limit
    async fn build_context_string(&self, max_tokens: usize) -> Result<String> {
        let mut context_parts = Vec::new();
        let mut current_tokens = 0;

        // Add summary if available
        if let Some(ref summary) = *self.summary.read() {
            let summary_tokens = self.token_counter.count_tokens(summary)?;
            if summary_tokens <= max_tokens {
                context_parts.push(format!("SUMMARY: {}", summary));
                current_tokens += summary_tokens;
            }
        }

        // Add messages from most recent backwards
        let messages = self.messages.read();
        let mut selected_messages = Vec::new();

        for message in messages.iter().rev() {
            let message_text = message.format_for_prompt();
            let message_tokens = self.token_counter.count_tokens(&message_text)?;

            if current_tokens + message_tokens > max_tokens {
                break;
            }

            selected_messages.push(message_text);
            current_tokens += message_tokens;
        }

        // Reverse to get chronological order
        selected_messages.reverse();
        context_parts.extend(selected_messages);

        Ok(context_parts.join("\n"))
    }

    /// Prune messages if needed based on overflow strategy
    async fn prune_if_needed(&self) -> Result<()> {
        let total_tokens = self.get_total_tokens();
        let message_count = self.messages.read().len();

        // Prune if either token limit OR message count limit exceeded
        let needs_pruning = total_tokens > self.config.max_tokens
            || (self.config.overflow_strategy == OverflowStrategy::SlidingWindow
                && message_count > self.config.sliding_window_size);

        if !needs_pruning {
            return Ok(());
        }

        match self.config.overflow_strategy {
            OverflowStrategy::SlidingWindow => self.prune_sliding_window().await,
            OverflowStrategy::Summarization => self.prune_with_summarization().await,
            OverflowStrategy::Hybrid => self.prune_hybrid().await,
            OverflowStrategy::TruncateStart => self.prune_truncate_start().await,
            OverflowStrategy::TruncateMiddle => self.prune_truncate_middle().await,
        }
    }

    /// Prune using sliding window (FIFO)
    async fn prune_sliding_window(&self) -> Result<()> {
        let mut messages = self.messages.write();
        let mut removed_count = 0;

        while messages.len() > self.config.sliding_window_size {
            if let Some(removed) = messages.pop_front() {
                // Don't remove preserved messages
                if removed.preserve {
                    messages.push_front(removed);
                    break;
                }
                removed_count += 1;
            } else {
                break;
            }
        }

        if removed_count > 0 {
            let mut metrics = self.metrics.write();
            metrics.prune_count += 1;
        }

        Ok(())
    }

    /// Prune with summarization
    async fn prune_with_summarization(&self) -> Result<()> {
        if !self.config.enable_summarization || self.llm_client.is_none() {
            return self.prune_sliding_window().await;
        }

        let messages_to_summarize = {
            let messages = self.messages.read();
            if messages.len() < self.config.summarization_threshold {
                return Ok(());
            }

            // Take first half of messages for summarization
            let split_point = messages.len() / 2;
            messages
                .iter()
                .take(split_point)
                .filter(|m| !m.preserve) // Don't summarize preserved messages
                .cloned()
                .collect::<Vec<_>>()
        };

        if messages_to_summarize.is_empty() {
            return Ok(());
        }

        // Create summary
        let new_summary = self.create_summary(&messages_to_summarize).await?;

        // Update summary and remove summarized messages
        {
            let mut summary = self.summary.write();
            if let Some(existing) = summary.as_ref() {
                *summary = Some(format!("{}\n{}", existing, new_summary));
            } else {
                *summary = Some(new_summary);
            }
        }

        // Remove summarized messages
        {
            let mut messages = self.messages.write();
            let mut new_messages = VecDeque::new();

            // Keep preserved messages and messages after split point
            let split_point = messages_to_summarize.len();
            let mut skipped = 0;

            for message in messages.drain(..) {
                if message.preserve || skipped >= split_point {
                    new_messages.push_back(message);
                } else {
                    skipped += 1;
                }
            }

            *messages = new_messages;
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.prune_count += 1;
            metrics.summarized_messages += messages_to_summarize.len();
        }

        Ok(())
    }

    /// Hybrid pruning (summarize old, sliding window for recent)
    async fn prune_hybrid(&self) -> Result<()> {
        // First try summarization
        if self.config.enable_summarization && self.llm_client.is_some() {
            self.prune_with_summarization().await?;
        }

        // Then apply sliding window if still over limit
        let total_tokens = self.get_total_tokens();
        if total_tokens > self.config.max_tokens {
            self.prune_sliding_window().await?;
        }

        Ok(())
    }

    /// Truncate from start
    async fn prune_truncate_start(&self) -> Result<()> {
        let target_tokens = self.config.max_tokens * 3 / 4; // Target 75% of max
        let mut current_tokens = self.get_total_tokens();

        let mut messages = self.messages.write();

        while current_tokens > target_tokens && !messages.is_empty() {
            if let Some(removed) = messages.pop_front() {
                if removed.preserve {
                    messages.push_front(removed);
                    break;
                }
                current_tokens -= removed.token_count;
            }
        }

        Ok(())
    }

    /// Truncate from middle (keep start and end)
    async fn prune_truncate_middle(&self) -> Result<()> {
        let messages = self.messages.read().clone();
        if messages.len() <= 4 {
            return Ok(()); // Need at least 4 messages to truncate middle
        }

        let keep_start = 2;
        let keep_end = 2;
        let target_size = self.config.sliding_window_size.max(keep_start + keep_end);

        if messages.len() <= target_size {
            return Ok(());
        }

        drop(messages); // Release read lock

        let mut messages = self.messages.write();
        let mut new_messages = VecDeque::new();

        // Keep first messages
        for _ in 0..keep_start {
            if let Some(msg) = messages.pop_front() {
                new_messages.push_back(msg);
            }
        }

        // Skip middle messages (except preserved ones)
        let skip_count = messages.len().saturating_sub(keep_end);
        let mut preserved_middle = Vec::new();

        for _ in 0..skip_count {
            if let Some(msg) = messages.pop_front() {
                if msg.preserve {
                    preserved_middle.push(msg);
                }
            }
        }

        // Add back preserved messages from middle
        for msg in preserved_middle {
            new_messages.push_back(msg);
        }

        // Keep last messages
        while let Some(msg) = messages.pop_front() {
            new_messages.push_back(msg);
        }

        *messages = new_messages;

        Ok(())
    }

    /// Create summary of messages using LLM
    async fn create_summary(&self, messages: &[Message]) -> Result<String> {
        let llm_client = self
            .llm_client
            .as_ref()
            .ok_or_else(|| anyhow!("LLM client required for summarization"))?;

        // Build conversation text
        let conversation = messages
            .iter()
            .map(|m| m.format_for_prompt())
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Summarize the following conversation in 2-3 sentences, preserving key information:\n\n{}",
            conversation
        );

        let summary = llm_client.complete(&prompt).await?;
        Ok(summary)
    }

    /// Update internal metrics
    fn update_metrics(&self, metrics: &mut ContextMetrics) {
        let messages = self.messages.read();
        let total_tokens = self.get_total_tokens();

        metrics.current_tokens = total_tokens;
        metrics.max_tokens = self.config.max_tokens;
        metrics.utilization = (total_tokens as f32) / (self.config.max_tokens as f32);

        if !messages.is_empty() {
            metrics.avg_message_tokens = (total_tokens as f32) / (messages.len() as f32);
        }
    }
}

/// Conversation history that can be serialized/deserialized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableHistory {
    pub config: ContextConfig,
    pub messages: Vec<Message>,
    pub summary: Option<String>,
    pub metrics: ContextMetrics,
}

impl ConversationHistory {
    /// Export to serializable format
    pub fn export(&self) -> SerializableHistory {
        let messages = self.messages.read();
        let summary = self.summary.read();
        let metrics = self.metrics.read();

        SerializableHistory {
            config: self.config.clone(),
            messages: messages.iter().cloned().collect(),
            summary: summary.clone(),
            metrics: metrics.clone(),
        }
    }

    /// Import from serializable format
    pub fn import(data: SerializableHistory, llm_client: Option<Arc<dyn LlmClient>>) -> Self {
        let token_counter = TokenCounter::new(&data.config.encoding_model);

        let history = Self {
            config: data.config,
            messages: Arc::new(RwLock::new(data.messages.into())),
            summary: Arc::new(RwLock::new(data.summary)),
            token_counter,
            metrics: Arc::new(RwLock::new(data.metrics)),
            llm_client,
        };

        history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Role;

    #[tokio::test]
    async fn test_conversation_history_creation() {
        let config = ContextConfig::default();
        let history = ConversationHistory::new(config);

        assert_eq!(history.get_recent_messages(10).len(), 0);
        assert_eq!(history.get_total_tokens(), 0);
    }

    #[tokio::test]
    async fn test_add_message() {
        let config = ContextConfig::default();
        let history = ConversationHistory::new(config);

        let message_id = history
            .add_message(Role::User, "Hello world".to_string())
            .await
            .unwrap();

        assert!(!message_id.is_empty());
        assert_eq!(history.get_recent_messages(10).len(), 1);
        assert!(history.get_total_tokens() > 0);
    }

    #[tokio::test]
    async fn test_get_context() {
        let config = ContextConfig::default();
        let history = ConversationHistory::new(config);

        history
            .add_message(Role::User, "Hello".to_string())
            .await
            .unwrap();
        history
            .add_message(Role::Assistant, "Hi there".to_string())
            .await
            .unwrap();

        let context = history.get_context(1000).await.unwrap();
        assert!(context.contains("Hello"));
        assert!(context.contains("Hi there"));
    }

    #[tokio::test]
    async fn test_sliding_window_pruning() {
        let mut config = ContextConfig::default();
        config.sliding_window_size = 2;
        config.overflow_strategy = OverflowStrategy::SlidingWindow;

        let history = ConversationHistory::new(config);

        // Add more messages than window size
        history
            .add_message(Role::User, "Message 1".to_string())
            .await
            .unwrap();
        history
            .add_message(Role::User, "Message 2".to_string())
            .await
            .unwrap();
        history
            .add_message(Role::User, "Message 3".to_string())
            .await
            .unwrap();

        let messages = history.get_recent_messages(10);
        assert!(messages.len() <= 2);

        // Should keep most recent
        assert!(messages.iter().any(|m| m.content.contains("Message 3")));
    }

    #[tokio::test]
    async fn test_preserved_messages() {
        let mut config = ContextConfig::default();
        config.sliding_window_size = 1;

        let history = ConversationHistory::new(config);

        // Add preserved message
        let preserved_msg =
            Message::new_preserved(Role::System, "Important system message".to_string());
        {
            let mut messages = history.messages.write();
            messages.push_back(preserved_msg);
        }

        // Add regular message that should trigger pruning
        history
            .add_message(Role::User, "Regular message".to_string())
            .await
            .unwrap();

        let messages = history.get_recent_messages(10);

        // Should still have preserved message
        assert!(messages
            .iter()
            .any(|m| m.content.contains("Important system message")));
    }

    #[tokio::test]
    async fn test_export_import() {
        let config = ContextConfig::default();
        let history1 = ConversationHistory::new(config);

        history1
            .add_message(Role::User, "Hello".to_string())
            .await
            .unwrap();
        history1
            .add_message(Role::Assistant, "Hi".to_string())
            .await
            .unwrap();

        let exported = history1.export();
        assert_eq!(exported.messages.len(), 2);

        let history2 = ConversationHistory::import(exported, None);
        assert_eq!(history2.get_recent_messages(10).len(), 2);
    }
}
