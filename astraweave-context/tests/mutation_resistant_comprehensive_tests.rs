//! Mutation-resistant comprehensive tests for astraweave-context.
//! Targets exact default values, enum variants, serde roundtrips, token estimation,
//! format strings, and boundary conditions for 90%+ mutation kill rate.

use astraweave_context::*;
use std::collections::HashMap;

// ========================================================================
// CONTEXT CONFIG DEFAULTS
// ========================================================================

#[test]
fn context_config_default_max_tokens() {
    let c = ContextConfig::default();
    assert_eq!(c.max_tokens, 4096);
}

#[test]
fn context_config_default_sliding_window_size() {
    let c = ContextConfig::default();
    assert_eq!(c.sliding_window_size, 20);
}

#[test]
fn context_config_default_overflow_strategy() {
    let c = ContextConfig::default();
    assert_eq!(c.overflow_strategy, OverflowStrategy::SlidingWindow);
}

#[test]
fn context_config_default_enable_summarization() {
    let c = ContextConfig::default();
    assert!(c.enable_summarization);
}

#[test]
fn context_config_default_summarization_threshold() {
    let c = ContextConfig::default();
    assert_eq!(c.summarization_threshold, 50);
}

#[test]
fn context_config_default_encoding_model() {
    let c = ContextConfig::default();
    assert_eq!(c.encoding_model, "cl100k_base");
}

#[test]
fn context_config_default_preserve_system() {
    let c = ContextConfig::default();
    assert!(c.preserve_system_messages);
}

#[test]
fn context_config_serde_roundtrip() {
    let c = ContextConfig::default();
    let json = serde_json::to_string(&c).unwrap();
    let c2: ContextConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(c2.max_tokens, 4096);
    assert_eq!(c2.sliding_window_size, 20);
}

// ========================================================================
// SHARING CONFIG DEFAULTS
// ========================================================================

#[test]
fn sharing_config_default_allow_sharing() {
    let s = SharingConfig::default();
    assert!(!s.allow_sharing);
}

#[test]
fn sharing_config_default_max_shared_agents() {
    let s = SharingConfig::default();
    assert_eq!(s.max_shared_agents, 3);
}

#[test]
fn sharing_config_default_isolate_sensitive() {
    let s = SharingConfig::default();
    assert!(s.isolate_sensitive);
}

#[test]
fn sharing_config_default_tags_empty() {
    let s = SharingConfig::default();
    assert!(s.tags.is_empty());
}

// ========================================================================
// OVERFLOW STRATEGY ENUM
// ========================================================================

#[test]
fn overflow_strategy_all_variants() {
    let variants = [
        OverflowStrategy::SlidingWindow,
        OverflowStrategy::Summarization,
        OverflowStrategy::Hybrid,
        OverflowStrategy::TruncateStart,
        OverflowStrategy::TruncateMiddle,
    ];
    assert_eq!(variants.len(), 5);
}

#[test]
fn overflow_strategy_eq() {
    assert_eq!(
        OverflowStrategy::SlidingWindow,
        OverflowStrategy::SlidingWindow
    );
    assert_ne!(OverflowStrategy::SlidingWindow, OverflowStrategy::Hybrid);
    assert_ne!(
        OverflowStrategy::TruncateStart,
        OverflowStrategy::TruncateMiddle
    );
}

#[test]
fn overflow_strategy_copy() {
    let s = OverflowStrategy::Hybrid;
    let s2 = s;
    let _ = s;
    assert_eq!(s2, OverflowStrategy::Hybrid);
}

// ========================================================================
// ROLE ENUM
// ========================================================================

#[test]
fn role_system_as_str() {
    assert_eq!(Role::System.as_str(), "system");
}

#[test]
fn role_user_as_str() {
    assert_eq!(Role::User.as_str(), "user");
}

#[test]
fn role_assistant_as_str() {
    assert_eq!(Role::Assistant.as_str(), "assistant");
}

#[test]
fn role_function_as_str() {
    assert_eq!(Role::Function.as_str(), "function");
}

#[test]
fn role_agent_as_str() {
    assert_eq!(Role::Agent(42).as_str(), "agent");
}

#[test]
fn role_eq() {
    assert_eq!(Role::System, Role::System);
    assert_ne!(Role::System, Role::User);
    assert_eq!(Role::Agent(1), Role::Agent(1));
    assert_ne!(Role::Agent(1), Role::Agent(2));
}

#[test]
fn role_copy() {
    let r = Role::User;
    let r2 = r;
    let _ = r;
    assert_eq!(r2, Role::User);
}

#[test]
fn role_serde_roundtrip() {
    let r = Role::Agent(99);
    let json = serde_json::to_string(&r).unwrap();
    let r2: Role = serde_json::from_str(&json).unwrap();
    assert_eq!(r2, Role::Agent(99));
}

// ========================================================================
// MESSAGE
// ========================================================================

#[test]
fn message_new_preserve_false() {
    let m = Message::new(Role::User, "hello".to_string());
    assert!(!m.preserve);
}

#[test]
fn message_new_token_count_zero() {
    let m = Message::new(Role::User, "hello".to_string());
    assert_eq!(m.token_count, 0);
}

#[test]
fn message_new_preserved_true() {
    let m = Message::new_preserved(Role::System, "sys".to_string());
    assert!(m.preserve);
}

#[test]
fn message_new_preserved_token_count_zero() {
    let m = Message::new_preserved(Role::System, "sys".to_string());
    assert_eq!(m.token_count, 0);
}

#[test]
fn message_role_correct() {
    let m = Message::new(Role::Assistant, "reply".to_string());
    assert_eq!(m.role, Role::Assistant);
}

#[test]
fn message_content_correct() {
    let m = Message::new(Role::User, "test content".to_string());
    assert_eq!(m.content, "test content");
}

#[test]
fn message_metadata_empty_by_default() {
    let m = Message::new(Role::User, "x".to_string());
    assert!(m.metadata.is_empty());
}

#[test]
fn message_with_metadata_builder() {
    let m = Message::new(Role::User, "x".to_string())
        .with_metadata("key".to_string(), "val".to_string());
    assert_eq!(m.metadata.get("key").unwrap(), "val");
}

#[test]
fn message_id_non_empty() {
    let m = Message::new(Role::User, "x".to_string());
    assert!(!m.id.is_empty(), "UUID should generate non-empty ID");
}

#[test]
fn message_ids_unique() {
    let m1 = Message::new(Role::User, "a".to_string());
    let m2 = Message::new(Role::User, "b".to_string());
    assert_ne!(m1.id, m2.id, "two messages should have different UUIDs");
}

// ========================================================================
// MESSAGE FORMAT FOR PROMPT
// ========================================================================

#[test]
fn format_system() {
    let m = Message::new(Role::System, "setup".to_string());
    assert_eq!(m.format_for_prompt(), "SYSTEM: setup");
}

#[test]
fn format_user() {
    let m = Message::new(Role::User, "hello".to_string());
    assert_eq!(m.format_for_prompt(), "USER: hello");
}

#[test]
fn format_assistant() {
    let m = Message::new(Role::Assistant, "reply".to_string());
    assert_eq!(m.format_for_prompt(), "ASSISTANT: reply");
}

#[test]
fn format_function() {
    let m = Message::new(Role::Function, "result".to_string());
    assert_eq!(m.format_for_prompt(), "FUNCTION: result");
}

#[test]
fn format_agent() {
    let m = Message::new(Role::Agent(42), "action".to_string());
    assert_eq!(m.format_for_prompt(), "AGENT_42: action");
}

#[test]
fn format_agent_zero() {
    let m = Message::new(Role::Agent(0), "x".to_string());
    assert_eq!(m.format_for_prompt(), "AGENT_0: x");
}

// ========================================================================
// CONTEXT METRICS DEFAULTS
// ========================================================================

#[test]
fn context_metrics_default_zeroed() {
    let m = ContextMetrics::default();
    assert_eq!(m.total_messages, 0);
    assert_eq!(m.current_tokens, 0);
    assert_eq!(m.max_tokens, 0);
    assert!((m.utilization - 0.0).abs() < 1e-6);
    assert_eq!(m.prune_count, 0);
    assert_eq!(m.summarized_messages, 0);
    assert!((m.avg_message_tokens - 0.0).abs() < 1e-6);
    assert_eq!(m.processing_time_ms, 0);
}

// ========================================================================
// TOKEN ESTIMATOR
// ========================================================================

#[test]
fn estimate_english_empty() {
    assert_eq!(TokenEstimator::estimate_english(""), 0);
}

#[test]
fn estimate_english_one_char() {
    assert_eq!(TokenEstimator::estimate_english("a"), 1); // ceil(1/4.0) = 1
}

#[test]
fn estimate_english_four_chars() {
    assert_eq!(TokenEstimator::estimate_english("abcd"), 1); // ceil(4/4.0) = 1
}

#[test]
fn estimate_english_five_chars() {
    assert_eq!(TokenEstimator::estimate_english("abcde"), 2); // ceil(5/4.0) = 2
}

#[test]
fn estimate_code_empty() {
    assert_eq!(TokenEstimator::estimate_code(""), 0);
}

#[test]
fn estimate_code_exact() {
    // ceil(7/3.5) = 2
    assert_eq!(TokenEstimator::estimate_code("abcdefg"), 2);
}

#[test]
fn estimate_conservative_empty() {
    assert_eq!(TokenEstimator::estimate_conservative(""), 0);
}

#[test]
fn estimate_conservative_three_chars() {
    // ceil(3/3.0) = 1
    assert_eq!(TokenEstimator::estimate_conservative("abc"), 1);
}

#[test]
fn estimate_conservative_four_chars() {
    // ceil(4/3.0) = 2
    assert_eq!(TokenEstimator::estimate_conservative("abcd"), 2);
}

#[test]
fn estimate_by_words_empty() {
    assert_eq!(TokenEstimator::estimate_by_words(""), 0);
}

#[test]
fn estimate_by_words_one_word() {
    // ceil(1 * 1.3) = 2
    assert_eq!(TokenEstimator::estimate_by_words("one"), 2);
}

#[test]
fn estimate_by_words_two_words() {
    // ceil(2 * 1.3) = 3
    assert_eq!(TokenEstimator::estimate_by_words("one two"), 3);
}

#[test]
fn estimate_by_words_ten_words() {
    // ceil(10 * 1.3) = 13
    assert_eq!(TokenEstimator::estimate_by_words("a b c d e f g h i j"), 13);
}

// ========================================================================
// TOKEN COUNTER
// ========================================================================

#[test]
fn token_counter_model_name() {
    let tc = TokenCounter::new("cl100k_base");
    assert_eq!(tc.model_name(), "cl100k_base");
}

#[test]
fn token_counter_estimate_tokens() {
    let tc = TokenCounter::new("cl100k_base");
    // Estimation: ceil(len / 4.0)
    assert_eq!(tc.estimate_tokens(""), 0);
    assert_eq!(tc.estimate_tokens("a"), 1);
    assert_eq!(tc.estimate_tokens("abcd"), 1);
    assert_eq!(tc.estimate_tokens("abcde"), 2);
}

#[test]
fn token_counter_count_non_negative() {
    let tc = TokenCounter::new("cl100k_base");
    let count = tc.count_tokens("Hello world").unwrap();
    assert!(count > 0, "non-empty text should have tokens");
}

#[test]
fn token_counter_stats_default() {
    let tc = TokenCounter::new("cl100k_base");
    let stats = tc.get_stats();
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
}

#[test]
fn token_counter_stats_after_count() {
    let tc = TokenCounter::new("cl100k_base");
    tc.count_tokens("test").unwrap();
    let stats = tc.get_stats();
    assert_eq!(stats.total_requests, 1);
}

#[test]
fn token_counter_cache_hit() {
    let tc = TokenCounter::new("cl100k_base");
    tc.count_tokens("identical text").unwrap();
    tc.count_tokens("identical text").unwrap();
    let stats = tc.get_stats();
    assert_eq!(stats.total_requests, 2);
    assert!(stats.cache_hits >= 1, "second call should be a cache hit");
}

#[test]
fn token_counter_clear_cache() {
    let tc = TokenCounter::new("cl100k_base");
    tc.count_tokens("something").unwrap();
    tc.clear_cache();
    let stats = tc.get_stats();
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
}

// ========================================================================
// TOKEN BUDGET
// ========================================================================

#[test]
fn token_budget_new() {
    let b = TokenBudget::new(1000);
    assert_eq!(b.total_budget(), 1000);
    assert_eq!(b.used_tokens(), 0);
    assert_eq!(b.available_tokens(), 1000);
}

#[test]
fn token_budget_use_tokens() {
    let mut b = TokenBudget::new(100);
    b.use_tokens(30).unwrap();
    assert_eq!(b.used_tokens(), 30);
    assert_eq!(b.available_tokens(), 70);
}

#[test]
fn token_budget_use_exceeds_fails() {
    let mut b = TokenBudget::new(100);
    let result = b.use_tokens(101);
    assert!(result.is_err());
}

#[test]
fn token_budget_reserve() {
    let mut b = TokenBudget::new(100);
    b.reserve("system", 20).unwrap();
    assert_eq!(b.available_tokens(), 80);
}

#[test]
fn token_budget_reserve_exceeds_fails() {
    let mut b = TokenBudget::new(100);
    let result = b.reserve("big", 101);
    assert!(result.is_err());
}

#[test]
fn token_budget_utilization_zero() {
    let b = TokenBudget::new(100);
    assert!((b.utilization() - 0.0).abs() < 1e-6);
}

#[test]
fn token_budget_utilization_half() {
    let mut b = TokenBudget::new(100);
    b.use_tokens(50).unwrap();
    assert!((b.utilization() - 0.5).abs() < 1e-4);
}

#[test]
fn token_budget_reset_keeps_reservations() {
    let mut b = TokenBudget::new(100);
    b.reserve("sys", 20).unwrap();
    b.use_tokens(30).unwrap();
    b.reset();
    assert_eq!(b.used_tokens(), 0);
    assert_eq!(b.available_tokens(), 80); // reservation preserved
}

#[test]
fn token_budget_clear_reservations() {
    let mut b = TokenBudget::new(100);
    b.reserve("sys", 20).unwrap();
    b.clear_reservations();
    assert_eq!(b.available_tokens(), 100);
}

// ========================================================================
// CONTEXT WINDOW CONFIG DEFAULTS
// ========================================================================

#[test]
fn window_config_default_max_tokens() {
    let c = ContextWindowConfig::default();
    assert_eq!(c.max_tokens, 2048);
}

#[test]
fn window_config_default_max_messages() {
    let c = ContextWindowConfig::default();
    assert_eq!(c.max_messages, 20);
}

// ========================================================================
// ATTENTION CONFIG DEFAULTS
// ========================================================================

#[test]
fn attention_config_default_recency_bias() {
    let a = AttentionConfig::default();
    assert!((a.recency_bias - 0.1).abs() < 1e-6);
}

#[test]
fn attention_config_default_min_score() {
    let a = AttentionConfig::default();
    assert!((a.min_attention_score - 0.1).abs() < 1e-6);
}

#[test]
fn attention_config_default_role_weights() {
    let a = AttentionConfig::default();
    assert!((a.role_weights["system"] - 1.0).abs() < 1e-6);
    assert!((a.role_weights["user"] - 0.8).abs() < 1e-6);
    assert!((a.role_weights["assistant"] - 0.6).abs() < 1e-6);
    assert!((a.role_weights["function"] - 0.7).abs() < 1e-6);
}

#[test]
fn attention_config_default_keywords_empty() {
    let a = AttentionConfig::default();
    assert!(a.content_keywords.is_empty());
}

// ========================================================================
// CONTEXT WINDOW — basic operations
// ========================================================================

#[test]
fn context_window_new_empty() {
    let w = ContextWindow::new(ContextWindowConfig::default());
    assert_eq!(w.message_count(), 0);
    assert_eq!(w.current_tokens(), 0);
}

#[test]
fn context_window_add_message() {
    let mut w = ContextWindow::new(ContextWindowConfig::default());
    let m = Message::new(Role::User, "hello".to_string());
    w.add_message(m).unwrap();
    assert_eq!(w.message_count(), 1);
    assert!(w.current_tokens() > 0);
}

#[test]
fn context_window_utilization_zero_when_empty() {
    let w = ContextWindow::new(ContextWindowConfig::default());
    assert!((w.utilization() - 0.0).abs() < 1e-6);
}

#[test]
fn context_window_clear() {
    let mut w = ContextWindow::new(ContextWindowConfig::default());
    w.add_message(Message::new(Role::User, "test".to_string()))
        .unwrap();
    w.clear();
    assert_eq!(w.message_count(), 0);
    assert_eq!(w.current_tokens(), 0);
}

#[test]
fn context_window_get_recent() {
    let mut w = ContextWindow::new(ContextWindowConfig::default());
    w.add_message(Message::new(Role::User, "first".to_string()))
        .unwrap();
    w.add_message(Message::new(Role::Assistant, "second".to_string()))
        .unwrap();
    let recent = w.get_recent_messages(1);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].content, "second");
}

#[test]
fn context_window_get_by_role() {
    let mut w = ContextWindow::new(ContextWindowConfig::default());
    w.add_message(Message::new(Role::User, "q".to_string()))
        .unwrap();
    w.add_message(Message::new(Role::Assistant, "a".to_string()))
        .unwrap();
    w.add_message(Message::new(Role::User, "q2".to_string()))
        .unwrap();
    let user_msgs = w.get_messages_by_role(Role::User);
    assert_eq!(user_msgs.len(), 2);
}

#[test]
fn context_window_format_for_prompt() {
    let mut w = ContextWindow::new(ContextWindowConfig::default());
    w.add_message(Message::new(Role::System, "sys".to_string()))
        .unwrap();
    w.add_message(Message::new(Role::User, "hi".to_string()))
        .unwrap();
    let prompt = w.format_for_prompt();
    assert!(prompt.contains("SYSTEM: sys"));
    assert!(prompt.contains("USER: hi"));
}

#[test]
fn context_window_export_import() {
    let mut w = ContextWindow::new(ContextWindowConfig::default());
    w.add_message(Message::new(Role::User, "test".to_string()))
        .unwrap();
    let exported = w.export();
    let imported = ContextWindow::import(exported);
    assert_eq!(imported.message_count(), 1);
}

// ========================================================================
// WINDOW TYPE ENUM
// ========================================================================

#[test]
fn window_type_all_variants() {
    let types = [
        WindowType::Sliding,
        WindowType::Attention,
        WindowType::Fixed,
        WindowType::Hierarchical,
    ];
    assert_eq!(types.len(), 4);
}

// ========================================================================
// CONTEXT OPERATION ENUM
// ========================================================================

#[test]
fn context_operation_all_variants() {
    let ops = [
        ContextOperation::AddMessage,
        ContextOperation::GetContext,
        ContextOperation::Prune,
        ContextOperation::Summarize,
        ContextOperation::TokenCount,
    ];
    assert_eq!(ops.len(), 5);
}

// ========================================================================
// CONTEXT WINDOW STATS DEFAULTS
// ========================================================================

#[test]
fn window_stats_default_zeroed() {
    let s = ContextWindowStats::default();
    assert_eq!(s.total_messages_added, 0);
    assert_eq!(s.messages_pruned, 0);
    assert_eq!(s.total_tokens_processed, 0);
    assert!((s.avg_message_tokens - 0.0).abs() < 1e-6);
    assert!((s.window_utilization - 0.0).abs() < 1e-6);
    assert_eq!(s.attention_computations, 0);
}

// ========================================================================
// SUMMARIZER CONFIG DEFAULTS
// ========================================================================

#[test]
fn summarizer_config_default_max_input_tokens() {
    let c = SummarizerConfig::default();
    assert_eq!(c.max_input_tokens, 8192);
}

#[test]
fn summarizer_config_default_target_summary_tokens() {
    let c = SummarizerConfig::default();
    assert_eq!(c.target_summary_tokens, 512);
}

#[test]
fn summarizer_config_default_custom_prompt_none() {
    let c = SummarizerConfig::default();
    assert!(c.custom_prompt.is_none());
}

#[test]
fn summarizer_config_default_preserve_important() {
    let c = SummarizerConfig::default();
    assert!(c.preserve_important);
}

#[test]
fn summarizer_config_default_min_conversation_length() {
    let c = SummarizerConfig::default();
    assert_eq!(c.min_conversation_length, 10);
}

#[test]
fn summarizer_config_default_temperature() {
    let c = SummarizerConfig::default();
    assert!((c.temperature - 0.3).abs() < 1e-6);
}

// ========================================================================
// SUMMARY STRATEGY ENUM
// ========================================================================

#[test]
fn summary_strategy_all_variants() {
    let strats = [
        SummaryStrategy::KeyPoints,
        SummaryStrategy::Narrative,
        SummaryStrategy::Comprehensive,
        SummaryStrategy::Dialogue,
        SummaryStrategy::Custom,
    ];
    assert_eq!(strats.len(), 5);
}

// ========================================================================
// SUMMARIZER METRICS DEFAULTS
// ========================================================================

#[test]
fn summarizer_metrics_default_zeroed() {
    let m = SummarizerMetrics::default();
    assert_eq!(m.total_summarizations, 0);
    assert_eq!(m.total_input_tokens, 0);
    assert_eq!(m.total_output_tokens, 0);
    assert!((m.avg_compression_ratio - 0.0).abs() < 1e-6);
    assert_eq!(m.successful_summarizations, 0);
    assert_eq!(m.failed_summarizations, 0);
}

// ========================================================================
// MULTI-AGENT CONTEXT MANAGER
// ========================================================================

#[test]
fn multi_agent_manager_new_empty() {
    let config = RoutingConfig {
        enable_sharing: false,
        max_shared_agents: 3,
        routing_rules: vec![],
    };
    let mgr = MultiAgentContextManager::new(config);
    assert!(mgr.get_agent_window("agent_1").is_none());
    assert!(mgr.get_shared_window().is_none());
}

#[test]
fn multi_agent_manager_create_agent_window() {
    let config = RoutingConfig {
        enable_sharing: false,
        max_shared_agents: 3,
        routing_rules: vec![],
    };
    let mut mgr = MultiAgentContextManager::new(config);
    mgr.create_agent_window("agent_1", ContextWindowConfig::default());
    assert!(mgr.get_agent_window("agent_1").is_some());
    assert!(mgr.get_agent_window("agent_2").is_none());
}

#[test]
fn multi_agent_manager_create_shared_window() {
    let config = RoutingConfig {
        enable_sharing: true,
        max_shared_agents: 3,
        routing_rules: vec![],
    };
    let mut mgr = MultiAgentContextManager::new(config);
    mgr.create_shared_window(ContextWindowConfig::default());
    assert!(mgr.get_shared_window().is_some());
}

#[test]
fn multi_agent_add_to_nonexistent_returns_ok() {
    let config = RoutingConfig {
        enable_sharing: false,
        max_shared_agents: 3,
        routing_rules: vec![],
    };
    let mut mgr = MultiAgentContextManager::new(config);
    // Silent no-op for nonexistent agent
    let result = mgr.add_message_to_agent("ghost", Message::new(Role::User, "x".to_string()));
    assert!(result.is_ok());
}

// ========================================================================
// CONVERSATION HISTORY
// ========================================================================

#[tokio::test]
async fn history_add_and_retrieve() {
    let history = ConversationHistory::new(ContextConfig::default());
    let id = history
        .add_message(Role::User, "hello".to_string())
        .await
        .unwrap();
    assert!(!id.is_empty());
    let msg = history.get_message(&id);
    assert!(msg.is_some());
    assert_eq!(msg.unwrap().content, "hello");
}

#[tokio::test]
async fn history_get_recent() {
    let history = ConversationHistory::new(ContextConfig::default());
    history
        .add_message(Role::User, "first".to_string())
        .await
        .unwrap();
    history
        .add_message(Role::Assistant, "second".to_string())
        .await
        .unwrap();
    let recent = history.get_recent_messages(1);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].content, "second");
}

#[tokio::test]
async fn history_get_by_role() {
    let history = ConversationHistory::new(ContextConfig::default());
    history
        .add_message(Role::User, "q1".to_string())
        .await
        .unwrap();
    history
        .add_message(Role::Assistant, "a1".to_string())
        .await
        .unwrap();
    history
        .add_message(Role::User, "q2".to_string())
        .await
        .unwrap();
    let user = history.get_messages_by_role(Role::User);
    assert_eq!(user.len(), 2);
}

#[tokio::test]
async fn history_clear() {
    let history = ConversationHistory::new(ContextConfig::default());
    history
        .add_message(Role::User, "hello".to_string())
        .await
        .unwrap();
    history.clear();
    assert_eq!(history.get_recent_messages(10).len(), 0);
}

#[tokio::test]
async fn history_metrics() {
    let history = ConversationHistory::new(ContextConfig::default());
    history
        .add_message(Role::User, "test".to_string())
        .await
        .unwrap();
    let m = history.get_metrics();
    assert_eq!(m.total_messages, 1);
}

#[tokio::test]
async fn history_export_import() {
    let history = ConversationHistory::new(ContextConfig::default());
    history
        .add_message(Role::User, "persist".to_string())
        .await
        .unwrap();
    let exported = history.export();
    assert_eq!(exported.messages.len(), 1);
    let imported = ConversationHistory::import(exported, None);
    assert_eq!(imported.get_recent_messages(10).len(), 1);
}

// ========================================================================
// CURRENT_TIMESTAMP
// ========================================================================

#[test]
fn current_timestamp_positive() {
    let ts = current_timestamp();
    assert!(ts > 0, "timestamp should be positive");
}

#[test]
fn current_timestamp_reasonable() {
    let ts = current_timestamp();
    // Should be after 2020 (1577836800) and before 2100 (4102444800)
    assert!(ts > 1_577_836_800, "timestamp should be after 2020");
    assert!(ts < 4_102_444_800, "timestamp should be before 2100");
}
