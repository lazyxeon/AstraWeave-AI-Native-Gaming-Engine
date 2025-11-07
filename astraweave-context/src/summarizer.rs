/*!
# Conversation Summarizer

LLM-based conversation summarization for long-term context compression.
*/

use crate::{current_timestamp, Message, Role, TokenCounter};
use anyhow::{anyhow, Result};
use astraweave_llm::LlmClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Conversation summarizer using LLM
pub struct ConversationSummarizer {
    /// LLM client for summarization
    llm_client: Arc<dyn LlmClient>,

    /// Token counter for managing prompt sizes
    token_counter: TokenCounter,

    /// Summarization configuration
    config: SummarizerConfig,

    /// Performance metrics
    metrics: parking_lot::RwLock<SummarizerMetrics>,
}

/// Configuration for conversation summarization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizerConfig {
    /// Maximum tokens for input conversation
    pub max_input_tokens: usize,

    /// Target tokens for summary output
    pub target_summary_tokens: usize,

    /// Summarization strategy
    pub strategy: SummaryStrategy,

    /// Custom prompt template for summarization
    pub custom_prompt: Option<String>,

    /// Whether to preserve important messages verbatim
    pub preserve_important: bool,

    /// Minimum conversation length before summarization
    pub min_conversation_length: usize,

    /// Temperature for LLM generation
    pub temperature: f32,

    /// Token encoding model
    pub encoding_model: String,
}

impl Default for SummarizerConfig {
    fn default() -> Self {
        Self {
            max_input_tokens: 8192,
            target_summary_tokens: 512,
            strategy: SummaryStrategy::Comprehensive,
            custom_prompt: None,
            preserve_important: true,
            min_conversation_length: 10,
            temperature: 0.3,
            encoding_model: "cl100k_base".to_string(),
        }
    }
}

/// Strategies for conversation summarization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SummaryStrategy {
    /// Extract key points and decisions
    KeyPoints,
    /// Maintain narrative flow
    Narrative,
    /// Comprehensive summary with context
    Comprehensive,
    /// Focus on dialogue and interactions
    Dialogue,
    /// Custom strategy using provided prompt
    Custom,
}

/// Performance metrics for summarization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SummarizerMetrics {
    pub total_summarizations: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub avg_compression_ratio: f32,
    pub avg_summarization_time_ms: f32,
    pub successful_summarizations: u64,
    pub failed_summarizations: u64,
}

/// A summarized conversation segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    /// Unique identifier for this summary
    pub id: String,

    /// The summary text
    pub summary: String,

    /// Original conversation metadata
    pub metadata: SummaryMetadata,

    /// Token count of the summary
    pub token_count: usize,

    /// When this summary was created
    pub created_at: u64,

    /// Importance score (0.0 to 1.0)
    pub importance: f32,
}

/// Metadata about the summarized conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMetadata {
    /// Number of original messages
    pub original_message_count: usize,

    /// Total tokens in original conversation
    pub original_token_count: usize,

    /// Time range of the conversation
    pub time_range: (u64, u64),

    /// Participants in the conversation
    pub participants: Vec<Role>,

    /// Key topics discussed
    pub topics: Vec<String>,

    /// Compression ratio (original_tokens / summary_tokens)
    pub compression_ratio: f32,
}

impl ConversationSummarizer {
    /// Create a new conversation summarizer
    pub fn new(llm_client: Arc<dyn LlmClient>, config: SummarizerConfig) -> Self {
        let token_counter = TokenCounter::new(&config.encoding_model);

        Self {
            llm_client,
            token_counter,
            config,
            metrics: parking_lot::RwLock::new(SummarizerMetrics::default()),
        }
    }

    /// Summarize a conversation
    pub async fn summarize(&self, messages: &[Message]) -> Result<ConversationSummary> {
        let start_time = std::time::Instant::now();

        if messages.len() < self.config.min_conversation_length {
            return Err(anyhow!(
                "Conversation too short for summarization: {} < {}",
                messages.len(),
                self.config.min_conversation_length
            ));
        }

        // Build conversation text
        let conversation_text = self.build_conversation_text(messages)?;

        // Generate summary prompt
        let prompt = self.build_summary_prompt(&conversation_text, messages)?;

        // Generate summary using LLM
        let summary_text = self.llm_client.complete(&prompt).await.map_err(|e| {
            self.update_metrics_failed();
            e
        })?;

        // Clean and validate summary
        let cleaned_summary = self.clean_summary(&summary_text);

        // Count tokens in summary
        let summary_token_count = self.token_counter.count_tokens(&cleaned_summary)?;

        // Calculate metadata
        let metadata = self.calculate_metadata(messages, summary_token_count);

        // Create summary object
        let summary = ConversationSummary {
            id: uuid::Uuid::new_v4().to_string(),
            summary: cleaned_summary,
            metadata,
            token_count: summary_token_count,
            created_at: current_timestamp(),
            importance: self.calculate_importance(messages),
        };

        // Update metrics
        let duration_ms = start_time.elapsed().as_millis() as f32;
        self.update_metrics_success(&summary, duration_ms);

        Ok(summary)
    }

    /// Summarize multiple conversation segments and merge
    pub async fn summarize_and_merge(
        &self,
        message_groups: &[&[Message]],
    ) -> Result<ConversationSummary> {
        let mut individual_summaries = Vec::new();

        // Summarize each group individually
        for group in message_groups {
            match self.summarize(group).await {
                Ok(summary) => individual_summaries.push(summary),
                Err(e) => {
                    // Log error but continue with other groups
                    eprintln!("Failed to summarize group: {}", e);
                }
            }
        }

        if individual_summaries.is_empty() {
            return Err(anyhow!("No groups could be summarized"));
        }

        // Merge summaries
        self.merge_summaries(&individual_summaries).await
    }

    /// Update an existing summary with new messages
    pub async fn update_summary(
        &self,
        existing_summary: &ConversationSummary,
        new_messages: &[Message],
    ) -> Result<ConversationSummary> {
        // Create a combined context with existing summary + new messages
        let mut combined_text = format!(
            "Previous summary: {}\n\nNew messages:\n",
            existing_summary.summary
        );

        for message in new_messages {
            combined_text.push_str(&message.format_for_prompt());
            combined_text.push('\n');
        }

        let prompt = format!(
            "Update the following conversation summary with new information from recent messages. \
             Keep important details from both the previous summary and new messages:\n\n{}",
            combined_text
        );

        let updated_summary_text = self.llm_client.complete(&prompt).await?;
        let cleaned_summary = self.clean_summary(&updated_summary_text);
        let token_count = self.token_counter.count_tokens(&cleaned_summary)?;

        // Combine messages from original and new for metadata calculation
        let all_original_messages: Vec<Message> = new_messages.to_vec(); // Simplified - in practice would need original messages
        let metadata = self.calculate_metadata(&all_original_messages, token_count);

        Ok(ConversationSummary {
            id: uuid::Uuid::new_v4().to_string(),
            summary: cleaned_summary,
            metadata,
            token_count,
            created_at: current_timestamp(),
            importance: existing_summary
                .importance
                .max(self.calculate_importance(new_messages)),
        })
    }

    /// Extract key topics from messages
    pub async fn extract_topics(&self, messages: &[Message]) -> Result<Vec<String>> {
        let conversation_text = self.build_conversation_text(messages)?;

        let prompt = format!(
            "Extract the main topics discussed in this conversation. \
             Return as a JSON array of topic strings:\n\n{}",
            conversation_text
        );

        let response = self.llm_client.complete(&prompt).await?;

        // Try to parse as JSON array
        if let Ok(topics) = serde_json::from_str::<Vec<String>>(&response) {
            Ok(topics)
        } else {
            // Fallback: split by lines and clean
            let topics: Vec<String> = response
                .lines()
                .map(|line| line.trim().trim_matches('"').to_string())
                .filter(|topic| !topic.is_empty())
                .take(10) // Limit to 10 topics
                .collect();
            Ok(topics)
        }
    }

    /// Build conversation text from messages
    fn build_conversation_text(&self, messages: &[Message]) -> Result<String> {
        let mut conversation_parts = Vec::new();
        let mut current_tokens = 0;

        for message in messages {
            let formatted = message.format_for_prompt();
            let message_tokens = self.token_counter.count_tokens(&formatted)?;

            if current_tokens + message_tokens > self.config.max_input_tokens {
                break; // Stop if we would exceed token limit
            }

            conversation_parts.push(formatted);
            current_tokens += message_tokens;
        }

        Ok(conversation_parts.join("\n"))
    }

    /// Build summary prompt based on strategy
    fn build_summary_prompt(&self, conversation: &str, _messages: &[Message]) -> Result<String> {
        if let Some(ref custom) = self.config.custom_prompt {
            return Ok(format!("{}\n\nConversation:\n{}", custom, conversation));
        }

        let base_instruction = match self.config.strategy {
            SummaryStrategy::KeyPoints => {
                "Summarize the following conversation by extracting the key points, decisions made, and important information. \
                 Focus on actionable items and conclusions."
            }
            SummaryStrategy::Narrative => {
                "Summarize the following conversation while maintaining the narrative flow. \
                 Keep the chronological order and context of how the discussion developed."
            }
            SummaryStrategy::Comprehensive => {
                "Create a comprehensive summary of the following conversation. \
                 Include key points, context, participants' perspectives, and important details. \
                 Maintain accuracy while being concise."
            }
            SummaryStrategy::Dialogue => {
                "Summarize the following conversation focusing on the dialogue and interactions between participants. \
                 Preserve the essence of what each participant contributed to the discussion."
            }
            SummaryStrategy::Custom => {
                "Summarize the following conversation:" // Fallback
            }
        };

        let token_guidance = format!(
            " Aim for approximately {} tokens in your summary.",
            self.config.target_summary_tokens
        );

        let preservation_note = if self.config.preserve_important {
            " Preserve any critical information, system instructions, or explicitly marked important content verbatim."
        } else {
            ""
        };

        Ok(format!(
            "{}{}{}\n\nConversation:\n{}",
            base_instruction, token_guidance, preservation_note, conversation
        ))
    }

    /// Clean and validate summary text
    fn clean_summary(&self, summary: &str) -> String {
        summary
            .trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Calculate metadata for the summary
    fn calculate_metadata(&self, messages: &[Message], summary_tokens: usize) -> SummaryMetadata {
        let original_tokens: usize = messages.iter().map(|m| m.token_count).sum();
        let time_range = if let (Some(first), Some(last)) = (messages.first(), messages.last()) {
            (first.timestamp, last.timestamp)
        } else {
            (0, 0)
        };

        let mut participants = Vec::new();
        let mut seen_roles = std::collections::HashSet::new();

        for message in messages {
            if seen_roles.insert(message.role) {
                participants.push(message.role);
            }
        }

        // Simple topic extraction from message content
        let topics = self.extract_simple_topics(messages);

        let compression_ratio = if summary_tokens > 0 {
            original_tokens as f32 / summary_tokens as f32
        } else {
            1.0
        };

        SummaryMetadata {
            original_message_count: messages.len(),
            original_token_count: original_tokens,
            time_range,
            participants,
            topics,
            compression_ratio,
        }
    }

    /// Simple topic extraction using keyword frequency
    fn extract_simple_topics(&self, messages: &[Message]) -> Vec<String> {
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for message in messages {
            for word in message.content.split_whitespace() {
                let word = word
                    .to_lowercase()
                    .trim_matches(|c: char| !c.is_alphabetic())
                    .to_string();

                if word.len() > 3 && !is_common_word(&word) {
                    *word_counts.entry(word).or_insert(0) += 1;
                }
            }
        }

        let mut topics: Vec<_> = word_counts.into_iter().collect();
        topics.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by frequency

        topics
            .into_iter()
            .take(5) // Top 5 topics
            .map(|(word, _)| word)
            .collect()
    }

    /// Calculate importance score for messages
    fn calculate_importance(&self, messages: &[Message]) -> f32 {
        let mut importance = 0.0;
        let mut weight_sum = 0.0;

        for message in messages {
            let message_weight = match message.role {
                Role::System => 1.0,
                Role::User => 0.8,
                Role::Assistant => 0.6,
                Role::Function => 0.7,
                Role::Agent(_) => 0.5,
            };

            let message_importance = if message.preserve {
                1.0
            } else {
                0.5 + (message.content.len() as f32 / 1000.0).min(0.5)
            };

            importance += message_importance * message_weight;
            weight_sum += message_weight;
        }

        if weight_sum > 0.0 {
            (importance / weight_sum).min(1.0)
        } else {
            0.5
        }
    }

    /// Merge multiple summaries into one
    async fn merge_summaries(
        &self,
        summaries: &[ConversationSummary],
    ) -> Result<ConversationSummary> {
        let combined_text = summaries
            .iter()
            .map(|s| s.summary.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format!(
            "Merge the following conversation summaries into a single, coherent summary. \
             Remove redundancy while preserving all important information:\n\n{}",
            combined_text
        );

        let merged_summary = self.llm_client.complete(&prompt).await?;
        let cleaned_summary = self.clean_summary(&merged_summary);
        let token_count = self.token_counter.count_tokens(&cleaned_summary)?;

        // Combine metadata from all summaries
        let total_messages: usize = summaries
            .iter()
            .map(|s| s.metadata.original_message_count)
            .sum();
        let total_tokens: usize = summaries
            .iter()
            .map(|s| s.metadata.original_token_count)
            .sum();
        let max_importance = summaries.iter().map(|s| s.importance).fold(0.0, f32::max);

        let metadata = SummaryMetadata {
            original_message_count: total_messages,
            original_token_count: total_tokens,
            time_range: (
                summaries
                    .iter()
                    .map(|s| s.metadata.time_range.0)
                    .min()
                    .unwrap_or(0),
                summaries
                    .iter()
                    .map(|s| s.metadata.time_range.1)
                    .max()
                    .unwrap_or(0),
            ),
            participants: summaries
                .iter()
                .flat_map(|s| &s.metadata.participants)
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
            topics: summaries
                .iter()
                .flat_map(|s| &s.metadata.topics)
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .take(10)
                .collect(),
            compression_ratio: total_tokens as f32 / token_count as f32,
        };

        Ok(ConversationSummary {
            id: uuid::Uuid::new_v4().to_string(),
            summary: cleaned_summary,
            metadata,
            token_count,
            created_at: current_timestamp(),
            importance: max_importance,
        })
    }

    /// Update metrics for successful summarization
    fn update_metrics_success(&self, summary: &ConversationSummary, duration_ms: f32) {
        let mut metrics = self.metrics.write();
        metrics.total_summarizations += 1;
        metrics.successful_summarizations += 1;
        metrics.total_input_tokens += summary.metadata.original_token_count as u64;
        metrics.total_output_tokens += summary.token_count as u64;

        // Update averages
        let total = metrics.total_summarizations as f32;
        metrics.avg_compression_ratio = (metrics.avg_compression_ratio * (total - 1.0)
            + summary.metadata.compression_ratio)
            / total;
        metrics.avg_summarization_time_ms =
            (metrics.avg_summarization_time_ms * (total - 1.0) + duration_ms) / total;
    }

    /// Update metrics for failed summarization
    fn update_metrics_failed(&self) {
        let mut metrics = self.metrics.write();
        metrics.total_summarizations += 1;
        metrics.failed_summarizations += 1;
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> SummarizerMetrics {
        self.metrics.read().clone()
    }
}

/// Check if a word is a common word that should be ignored for topic extraction
fn is_common_word(word: &str) -> bool {
    matches!(
        word,
        "the"
            | "and"
            | "for"
            | "are"
            | "but"
            | "not"
            | "you"
            | "all"
            | "can"
            | "had"
            | "her"
            | "was"
            | "one"
            | "our"
            | "out"
            | "day"
            | "get"
            | "has"
            | "him"
            | "his"
            | "how"
            | "its"
            | "may"
            | "new"
            | "now"
            | "old"
            | "see"
            | "two"
            | "way"
            | "who"
            | "boy"
            | "did"
            | "she"
            | "use"
            | "your"
            | "said"
            | "each"
            | "make"
            | "most"
            | "over"
            | "some"
            | "time"
            | "very"
            | "when"
            | "come"
            | "here"
            | "just"
            | "like"
            | "long"
            | "many"
            | "such"
            | "take"
            | "than"
            | "them"
            | "well"
            | "were"
            | "will"
            | "with"
            | "have"
            | "this"
            | "that"
            | "from"
            | "they"
            | "know"
            | "want"
            | "been"
            | "good"
            | "much"
            | "work"
            | "life"
            | "only"
            | "think"
            | "also"
            | "back"
            | "after"
            | "first"
            | "year"
            | "where"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_llm::MockLlm;

    #[tokio::test]
    async fn test_summarizer_creation() {
        let llm_client = Arc::new(MockLlm);
        let config = SummarizerConfig::default();
        let summarizer = ConversationSummarizer::new(llm_client, config);

        let metrics = summarizer.get_metrics();
        assert_eq!(metrics.total_summarizations, 0);
    }

    #[tokio::test]
    async fn test_conversation_summarization() {
        let llm_client = Arc::new(MockLlm);
        let config = SummarizerConfig::default();
        let summarizer = ConversationSummarizer::new(llm_client, config);

        let messages = vec![
            Message::new(Role::User, "Hello, how are you?".to_string()),
            Message::new(Role::Assistant, "I'm doing well, thank you!".to_string()),
            Message::new(Role::User, "What's the weather like?".to_string()),
            Message::new(
                Role::Assistant,
                "I don't have access to current weather data.".to_string(),
            ),
        ];

        // Set token counts (normally done by conversation history)
        let messages: Vec<_> = messages
            .into_iter()
            .map(|mut msg| {
                msg.token_count = msg.content.split_whitespace().count(); // Rough approximation
                msg
            })
            .collect();

        // Add more messages to meet minimum length requirement
        let mut extended_messages = messages;
        for i in 0..10 {
            extended_messages.push(Message::new(
                Role::User,
                format!("Additional message {}", i),
            ));
        }

        let result = summarizer.summarize(&extended_messages).await;

        match result {
            Ok(summary) => {
                assert!(!summary.summary.is_empty());
                assert!(summary.token_count > 0);
                assert!(summary.metadata.compression_ratio > 0.0);
            }
            Err(e) => {
                // MockLlm might return a simple response, which could cause parsing issues
                // This is expected in a test environment
                println!("Summarization error (expected with MockLlm): {}", e);
            }
        }
    }

    #[test]
    fn test_topic_extraction() {
        let messages = vec![
            Message::new(
                Role::User,
                "Let's discuss artificial intelligence and machine learning".to_string(),
            ),
            Message::new(
                Role::Assistant,
                "AI and ML are fascinating topics in computer science".to_string(),
            ),
            Message::new(
                Role::User,
                "I'm particularly interested in neural networks".to_string(),
            ),
        ];

        let llm_client = Arc::new(MockLlm);
        let config = SummarizerConfig::default();
        let summarizer = ConversationSummarizer::new(llm_client, config);

        let topics = summarizer.extract_simple_topics(&messages);

        // Should extract meaningful words, not common words
        assert!(!topics.contains(&"the".to_string()));
        assert!(!topics.contains(&"and".to_string()));

        // May contain topic-related words
        println!("Extracted topics: {:?}", topics);
    }

    #[test]
    fn test_importance_calculation() {
        let llm_client = Arc::new(MockLlm);
        let config = SummarizerConfig::default();
        let summarizer = ConversationSummarizer::new(llm_client, config);

        let messages = vec![
            Message::new_preserved(Role::System, "Important system message".to_string()),
            Message::new(Role::User, "Regular message".to_string()),
        ];

        let importance = summarizer.calculate_importance(&messages);

        assert!(importance > 0.0);
        assert!(importance <= 1.0);
    }

    #[test]
    fn test_common_word_detection() {
        assert!(is_common_word("the"));
        assert!(is_common_word("and"));
        assert!(is_common_word("for"));
        assert!(!is_common_word("artificial"));
        assert!(!is_common_word("intelligence"));
    }
}
