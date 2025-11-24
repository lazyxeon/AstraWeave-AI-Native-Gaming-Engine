/*!
# Context Windows

Advanced context window management with sliding windows, attention mechanisms,
and multi-agent context sharing.
*/

use crate::{current_timestamp, Message, Role, TokenCounter};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// A context window for managing conversation context
pub struct ContextWindow {
    /// Window configuration
    config: ContextWindowConfig,

    /// Messages in this window
    messages: VecDeque<Message>,

    /// Token counter
    token_counter: TokenCounter,

    /// Current token count
    current_tokens: usize,

    /// Attention weights for messages (for importance-based pruning)
    attention_weights: HashMap<String, f32>,

    /// Window statistics
    stats: ContextWindowStats,
}

/// Configuration for context windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindowConfig {
    /// Maximum tokens in this window
    pub max_tokens: usize,

    /// Maximum number of messages
    pub max_messages: usize,

    /// Window type
    pub window_type: WindowType,

    /// Attention mechanism settings
    pub attention_config: AttentionConfig,

    /// Token encoding model
    pub encoding_model: String,
}

impl Default for ContextWindowConfig {
    fn default() -> Self {
        Self {
            max_tokens: 2048,
            max_messages: 20,
            window_type: WindowType::Sliding,
            attention_config: AttentionConfig::default(),
            encoding_model: "cl100k_base".to_string(),
        }
    }
}

/// Types of context windows
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WindowType {
    /// Simple sliding window (FIFO)
    Sliding,
    /// Attention-based (keep important messages)
    Attention,
    /// Fixed window (no automatic pruning)
    Fixed,
    /// Hierarchical (multiple levels of context)
    Hierarchical,
}

/// Attention mechanism configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionConfig {
    /// Enable recency bias (newer messages get higher attention)
    pub recency_bias: f32,

    /// Enable role-based attention (system messages get higher attention)
    pub role_weights: HashMap<String, f32>,

    /// Enable content-based attention (keywords get higher attention)
    pub content_keywords: Vec<String>,

    /// Minimum attention score to keep a message
    pub min_attention_score: f32,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        let mut role_weights = HashMap::new();
        role_weights.insert("system".to_string(), 1.0);
        role_weights.insert("user".to_string(), 0.8);
        role_weights.insert("assistant".to_string(), 0.6);
        role_weights.insert("function".to_string(), 0.7);

        Self {
            recency_bias: 0.1,
            role_weights,
            content_keywords: Vec::new(),
            min_attention_score: 0.1,
        }
    }
}

/// Statistics for context window performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextWindowStats {
    pub total_messages_added: u64,
    pub messages_pruned: u64,
    pub total_tokens_processed: u64,
    pub avg_message_tokens: f32,
    pub window_utilization: f32,
    pub attention_computations: u64,
}

impl ContextWindow {
    /// Create a new context window
    pub fn new(config: ContextWindowConfig) -> Self {
        let token_counter = TokenCounter::new(&config.encoding_model);

        Self {
            config,
            messages: VecDeque::new(),
            token_counter,
            current_tokens: 0,
            attention_weights: HashMap::new(),
            stats: ContextWindowStats::default(),
        }
    }

    /// Add a message to the window
    pub fn add_message(&mut self, mut message: Message) -> Result<()> {
        // Count tokens in the message
        message.token_count = self.token_counter.count_tokens(&message.content)?;

        // Calculate attention weight
        let attention_weight = self.calculate_attention_weight(&message);
        self.attention_weights
            .insert(message.id.clone(), attention_weight);

        // Add to window
        self.messages.push_back(message.clone());
        self.current_tokens += message.token_count;

        // Update statistics
        self.stats.total_messages_added += 1;
        self.stats.total_tokens_processed += message.token_count as u64;
        self.update_stats();

        // Prune if necessary
        self.prune_if_needed()?;

        Ok(())
    }

    /// Get all messages in the window
    pub fn get_messages(&self) -> &VecDeque<Message> {
        &self.messages
    }

    /// Get messages with attention scores above threshold
    pub fn get_important_messages(&self, threshold: f32) -> Vec<(Message, f32)> {
        self.messages
            .iter()
            .filter_map(|msg| {
                if let Some(&weight) = self.attention_weights.get(&msg.id) {
                    if weight >= threshold {
                        Some((msg.clone(), weight))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get recent messages up to a limit
    pub fn get_recent_messages(&self, limit: usize) -> Vec<Message> {
        self.messages
            .iter()
            .rev()
            .take(limit)
            .rev()
            .cloned()
            .collect()
    }

    /// Get messages by role
    pub fn get_messages_by_role(&self, role: Role) -> Vec<Message> {
        self.messages
            .iter()
            .filter(|m| m.role == role)
            .cloned()
            .collect()
    }

    /// Get current token count
    pub fn current_tokens(&self) -> usize {
        self.current_tokens
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if window is full
    pub fn is_full(&self) -> bool {
        self.current_tokens >= self.config.max_tokens
            || self.messages.len() > self.config.max_messages
    }

    /// Get window utilization (0.0 to 1.0)
    pub fn utilization(&self) -> f32 {
        (self.current_tokens as f32) / (self.config.max_tokens as f32)
    }

    /// Format window content for LLM prompt
    pub fn format_for_prompt(&self) -> String {
        self.messages
            .iter()
            .map(|m| m.format_for_prompt())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format with attention weights (for debugging)
    pub fn format_with_attention(&self) -> String {
        self.messages
            .iter()
            .map(|m| {
                let weight = self.attention_weights.get(&m.id).unwrap_or(&0.0);
                format!("[{:.2}] {}", weight, m.format_for_prompt())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Clear the window
    pub fn clear(&mut self) {
        self.messages.clear();
        self.attention_weights.clear();
        self.current_tokens = 0;
        self.stats = ContextWindowStats::default();
    }

    /// Calculate attention weight for a message
    fn calculate_attention_weight(&self, message: &Message) -> f32 {
        let mut weight = 1.0;

        // Recency bias (newer messages get higher weight)
        if self.config.attention_config.recency_bias > 0.0 {
            let age_hours = (current_timestamp() - message.timestamp) as f32 / 3600.0;
            let recency_factor = (-age_hours * self.config.attention_config.recency_bias).exp();
            weight *= recency_factor;
        }

        // Role-based weighting
        if let Some(&role_weight) = self
            .config
            .attention_config
            .role_weights
            .get(message.role.as_str())
        {
            weight *= role_weight;
        }

        // Content-based weighting (keyword matching)
        if !self.config.attention_config.content_keywords.is_empty() {
            let content_lower = message.content.to_lowercase();
            let keyword_matches = self
                .config
                .attention_config
                .content_keywords
                .iter()
                .filter(|keyword| content_lower.contains(&keyword.to_lowercase()))
                .count();

            if keyword_matches > 0 {
                weight *= 1.0 + (keyword_matches as f32 * 0.2); // 20% boost per keyword
            }
        }

        // Preserve flag gives maximum weight
        if message.preserve {
            weight = 10.0; // High weight to prevent pruning
        }

        weight.max(0.001) // Minimum weight to prevent division by zero
    }

    /// Prune messages if window is over capacity
    fn prune_if_needed(&mut self) -> Result<()> {
        if !self.is_full() {
            return Ok(());
        }

        match self.config.window_type {
            WindowType::Sliding => self.prune_sliding(),
            WindowType::Attention => self.prune_by_attention(),
            WindowType::Fixed => Ok(()), // No pruning for fixed windows
            WindowType::Hierarchical => self.prune_hierarchical(),
        }
    }

    /// Prune using sliding window (remove oldest)
    fn prune_sliding(&mut self) -> Result<()> {
        while self.is_full() && !self.messages.is_empty() {
            if let Some(removed) = self.messages.pop_front() {
                // Don't remove preserved messages
                if removed.preserve {
                    self.messages.push_front(removed);
                    break;
                }

                self.current_tokens = self.current_tokens.saturating_sub(removed.token_count);
                self.attention_weights.remove(&removed.id);
                self.stats.messages_pruned += 1;
            }
        }

        self.update_stats();
        Ok(())
    }

    /// Prune using attention weights (remove least important)
    fn prune_by_attention(&mut self) -> Result<()> {
        while self.is_full() && self.messages.len() > 1 {
            // Find message with lowest attention weight (excluding preserved)
            let mut lowest_weight = f32::MAX;
            let mut lowest_index = None;

            for (i, message) in self.messages.iter().enumerate() {
                if message.preserve {
                    continue; // Skip preserved messages
                }

                if let Some(&weight) = self.attention_weights.get(&message.id) {
                    if weight < lowest_weight {
                        lowest_weight = weight;
                        lowest_index = Some(i);
                    }
                }
            }

            if let Some(index) = lowest_index {
                if let Some(removed) = self.messages.remove(index) {
                    self.current_tokens = self.current_tokens.saturating_sub(removed.token_count);
                    self.attention_weights.remove(&removed.id);
                    self.stats.messages_pruned += 1;
                }
            } else {
                break; // All remaining messages are preserved
            }
        }

        self.update_stats();
        Ok(())
    }

    /// Prune using hierarchical strategy
    fn prune_hierarchical(&mut self) -> Result<()> {
        // First try attention-based pruning
        self.prune_by_attention()?;

        // If still over capacity, fall back to sliding window
        if self.is_full() {
            self.prune_sliding()?;
        }

        Ok(())
    }

    /// Update internal statistics
    fn update_stats(&mut self) {
        self.stats.window_utilization = self.utilization();

        if self.stats.total_messages_added > 0 {
            self.stats.avg_message_tokens = (self.stats.total_tokens_processed as f32)
                / (self.stats.total_messages_added as f32);
        }
    }

    /// Get window statistics
    pub fn get_stats(&self) -> ContextWindowStats {
        self.stats.clone()
    }

    /// Export to serializable format
    pub fn export(&self) -> SerializableContextWindow {
        SerializableContextWindow {
            config: self.config.clone(),
            messages: self.messages.iter().cloned().collect(),
            attention_weights: self.attention_weights.clone(),
            stats: self.stats.clone(),
        }
    }

    /// Import from serializable format
    pub fn import(data: SerializableContextWindow) -> Self {
        let token_counter = TokenCounter::new(&data.config.encoding_model);
        let current_tokens = data.messages.iter().map(|m| m.token_count).sum();

        Self {
            config: data.config,
            messages: data.messages.into(),
            token_counter,
            current_tokens,
            attention_weights: data.attention_weights,
            stats: data.stats,
        }
    }
}

/// Serializable representation of a context window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableContextWindow {
    pub config: ContextWindowConfig,
    pub messages: Vec<Message>,
    pub attention_weights: HashMap<String, f32>,
    pub stats: ContextWindowStats,
}

/// Multi-agent context manager
pub struct MultiAgentContextManager {
    /// Individual context windows for each agent
    agent_windows: HashMap<String, ContextWindow>,

    /// Shared context window
    shared_window: Option<ContextWindow>,

    /// Cross-agent message routing
    routing_config: RoutingConfig,
}

/// Configuration for multi-agent context routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Whether to enable message sharing between agents
    pub enable_sharing: bool,

    /// Maximum agents that can share context
    pub max_shared_agents: usize,

    /// Rules for message routing
    pub routing_rules: Vec<RoutingRule>,
}

/// A rule for routing messages between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Source agent pattern (regex or exact match)
    pub source_pattern: String,

    /// Target agents
    pub targets: Vec<String>,

    /// Message content filter (optional)
    pub content_filter: Option<String>,

    /// Whether to copy or move the message
    pub copy_message: bool,
}

impl MultiAgentContextManager {
    /// Create a new multi-agent context manager
    pub fn new(routing_config: RoutingConfig) -> Self {
        Self {
            agent_windows: HashMap::new(),
            shared_window: None,
            routing_config,
        }
    }

    /// Create a context window for an agent
    pub fn create_agent_window(&mut self, agent_id: &str, config: ContextWindowConfig) {
        let window = ContextWindow::new(config);
        self.agent_windows.insert(agent_id.to_string(), window);
    }

    /// Create shared context window
    pub fn create_shared_window(&mut self, config: ContextWindowConfig) {
        self.shared_window = Some(ContextWindow::new(config));
    }

    /// Add message to specific agent's window
    pub fn add_message_to_agent(&mut self, agent_id: &str, message: Message) -> Result<()> {
        if let Some(window) = self.agent_windows.get_mut(agent_id) {
            window.add_message(message.clone())?;

            // Route message to other agents if needed
            if self.routing_config.enable_sharing {
                self.route_message(agent_id, &message)?;
            }
        }

        Ok(())
    }

    /// Add message to shared window
    pub fn add_message_to_shared(&mut self, message: Message) -> Result<()> {
        if let Some(shared) = &mut self.shared_window {
            shared.add_message(message)?;
        }

        Ok(())
    }

    /// Get agent's context window
    pub fn get_agent_window(&self, agent_id: &str) -> Option<&ContextWindow> {
        self.agent_windows.get(agent_id)
    }

    /// Get shared context window
    pub fn get_shared_window(&self) -> Option<&ContextWindow> {
        self.shared_window.as_ref()
    }

    /// Get combined context for an agent (own + shared + relevant from others)
    pub fn get_combined_context(&self, agent_id: &str, max_tokens: usize) -> Result<String> {
        let mut context_parts = Vec::new();
        let mut token_count = 0;

        // Add shared context first
        if let Some(shared) = &self.shared_window {
            let shared_context = shared.format_for_prompt();
            let shared_tokens = shared.current_tokens();

            if token_count + shared_tokens <= max_tokens {
                context_parts.push(format!("SHARED CONTEXT:\n{}", shared_context));
                token_count += shared_tokens;
            }
        }

        // Add agent's own context
        if let Some(window) = self.agent_windows.get(agent_id) {
            let agent_context = window.format_for_prompt();
            let agent_tokens = window.current_tokens();

            if token_count + agent_tokens <= max_tokens {
                context_parts.push(format!("AGENT CONTEXT:\n{}", agent_context));
                let _ = token_count.saturating_add(agent_tokens);
            }
        }

        Ok(context_parts.join("\n\n"))
    }

    /// Route message to other agents based on routing rules
    fn route_message(&mut self, source_agent: &str, message: &Message) -> Result<()> {
        for rule in &self.routing_config.routing_rules {
            if self.matches_rule(source_agent, message, rule) {
                for target in &rule.targets {
                    if target != source_agent {
                        if let Some(target_window) = self.agent_windows.get_mut(target) {
                            if rule.copy_message {
                                target_window.add_message(message.clone())?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if message matches routing rule
    fn matches_rule(&self, source_agent: &str, message: &Message, rule: &RoutingRule) -> bool {
        // Check source pattern
        if rule.source_pattern != "*" && rule.source_pattern != source_agent {
            // Could implement regex matching here
            return false;
        }

        // Check content filter
        if let Some(ref filter) = rule.content_filter {
            if !message
                .content
                .to_lowercase()
                .contains(&filter.to_lowercase())
            {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Role;

    #[test]
    fn test_context_window_creation() {
        let config = ContextWindowConfig::default();
        let window = ContextWindow::new(config);

        assert_eq!(window.message_count(), 0);
        assert_eq!(window.current_tokens(), 0);
        assert!(!window.is_full());
    }

    #[test]
    fn test_add_message() {
        let config = ContextWindowConfig::default();
        let mut window = ContextWindow::new(config);

        let message = Message::new(Role::User, "Hello world".to_string());
        window.add_message(message).unwrap();

        assert_eq!(window.message_count(), 1);
        assert!(window.current_tokens() > 0);
    }

    #[test]
    fn test_sliding_window_pruning() {
        let mut config = ContextWindowConfig::default();
        config.max_messages = 2;
        config.window_type = WindowType::Sliding;

        let mut window = ContextWindow::new(config);

        // Add more messages than capacity
        window
            .add_message(Message::new(Role::User, "Message 1".to_string()))
            .unwrap();
        window
            .add_message(Message::new(Role::User, "Message 2".to_string()))
            .unwrap();
        window
            .add_message(Message::new(Role::User, "Message 3".to_string()))
            .unwrap();

        assert_eq!(window.message_count(), 2);

        // Should keep most recent messages
        let messages = window.get_messages();
        assert!(messages.iter().any(|m| m.content.contains("Message 3")));
        assert!(messages.iter().any(|m| m.content.contains("Message 2")));
        assert!(!messages.iter().any(|m| m.content.contains("Message 1")));
    }

    #[test]
    fn test_attention_based_pruning() {
        let mut config = ContextWindowConfig::default();
        config.max_messages = 2;
        config.window_type = WindowType::Attention;

        let mut window = ContextWindow::new(config);

        // Add messages with different importance
        let mut important_msg = Message::new(Role::System, "Important system message".to_string());
        important_msg.preserve = true;

        window
            .add_message(Message::new(Role::User, "Regular message 1".to_string()))
            .unwrap();
        window.add_message(important_msg).unwrap();
        window
            .add_message(Message::new(Role::User, "Regular message 2".to_string()))
            .unwrap();

        assert_eq!(window.message_count(), 2);

        // Should keep important message
        let messages = window.get_messages();
        assert!(messages
            .iter()
            .any(|m| m.content.contains("Important system message")));
    }

    #[test]
    fn test_multi_agent_context_manager() {
        let routing_config = RoutingConfig {
            enable_sharing: false,
            max_shared_agents: 3,
            routing_rules: vec![],
        };

        let mut manager = MultiAgentContextManager::new(routing_config);

        // Create agent windows
        manager.create_agent_window("agent1", ContextWindowConfig::default());
        manager.create_agent_window("agent2", ContextWindowConfig::default());

        // Add messages
        manager
            .add_message_to_agent(
                "agent1",
                Message::new(Role::User, "Hello from agent1".to_string()),
            )
            .unwrap();

        // Check agent windows
        let agent1_window = manager.get_agent_window("agent1").unwrap();
        assert_eq!(agent1_window.message_count(), 1);

        let agent2_window = manager.get_agent_window("agent2").unwrap();
        assert_eq!(agent2_window.message_count(), 0); // No sharing enabled
    }

    #[test]
    fn test_context_formatting() {
        let config = ContextWindowConfig::default();
        let mut window = ContextWindow::new(config);

        window
            .add_message(Message::new(Role::User, "Hello".to_string()))
            .unwrap();
        window
            .add_message(Message::new(Role::Assistant, "Hi there".to_string()))
            .unwrap();

        let formatted = window.format_for_prompt();
        assert!(formatted.contains("USER: Hello"));
        assert!(formatted.contains("ASSISTANT: Hi there"));
    }
}
