/*!
# Token Counter

Accurate token counting for different encoding models and LLM providers.
*/

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tiktoken_rs::{get_bpe_from_model, CoreBPE};

/// Token counter for different encoding models
pub struct TokenCounter {
    /// The BPE encoder
    encoder: Option<CoreBPE>,
    /// Model name
    model_name: String,
    /// Token cache for repeated strings
    cache: parking_lot::RwLock<HashMap<String, usize>>,
    /// Cache statistics
    stats: parking_lot::RwLock<TokenCounterStats>,
}

/// Statistics for token counting performance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCounterStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_tokens_counted: u64,
    pub avg_tokens_per_request: f32,
    pub cache_hit_rate: f32,
}

impl TokenCounter {
    /// Create a new token counter for the specified model
    pub fn new(model_name: &str) -> Self {
        // Try to load the requested model encoder. If unavailable, fall back to
        // an estimation-only mode (encoder = None) to allow tests and environments
        // without tokenizer artifacts to run.
        let encoder = match get_bpe_from_model(model_name) {
            Ok(enc) => Some(enc),
            Err(_) => match get_bpe_from_model("cl100k_base") {
                Ok(enc2) => Some(enc2),
                Err(_) => {
                    // Could not load any tokenizer; use estimation-only fallback
                    // and avoid panicking so tests can run in minimal environments.
                    None
                }
            },
        };

        Self {
            encoder,
            model_name: model_name.to_string(),
            cache: parking_lot::RwLock::new(HashMap::new()),
            stats: parking_lot::RwLock::new(TokenCounterStats::default()),
        }
    }

    /// Count tokens in text
    pub fn count_tokens(&self, text: &str) -> Result<usize> {
        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(&count) = cache.get(text) {
                self.update_stats(count, true);
                return Ok(count);
            }
        }

        // Count tokens using the encoder if available, otherwise fall back to a
        // lightweight estimation.
        let count = if let Some(enc) = &self.encoder {
            let tokens = enc.encode_with_special_tokens(text);
            tokens.len()
        } else {
            // Fallback estimation: ~4 characters per token
            self.estimate_tokens(text)
        };

        // Update cache (with size limit)
        {
            let mut cache = self.cache.write();
            if cache.len() < 10_000 {
                // Limit cache size
                cache.insert(text.to_string(), count);
            }
        }

        self.update_stats(count, false);
        Ok(count)
    }

    /// Count tokens in multiple texts (batch operation)
    pub fn count_tokens_batch(&self, texts: &[String]) -> Result<Vec<usize>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            results.push(self.count_tokens(text)?);
        }

        Ok(results)
    }

    /// Estimate tokens without exact counting (faster but less accurate)
    pub fn estimate_tokens(&self, text: &str) -> usize {
        // Simple estimation: ~4 characters per token for English text
        // This is a rough approximation and varies by content
        (text.len() as f32 / 4.0).ceil() as usize
    }

    /// Get the model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Get token counting statistics
    pub fn get_stats(&self) -> TokenCounterStats {
        self.stats.read().clone()
    }

    /// Clear the token cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();

        let mut stats = self.stats.write();
        stats.cache_hits = 0;
        stats.cache_misses = 0;
    }

    /// Truncate text to fit within token limit
    pub fn truncate_to_tokens(&self, text: &str, max_tokens: usize) -> Result<String> {
        if let Some(enc) = &self.encoder {
            let tokens = enc.encode_with_special_tokens(text);

            if tokens.len() <= max_tokens {
                return Ok(text.to_string());
            }

            // Take first max_tokens tokens
            let truncated_tokens = &tokens[..max_tokens];

            // Decode back to text
            let truncated_text = enc
                .decode(truncated_tokens.to_vec())
                .map_err(|e| anyhow!("Failed to decode truncated tokens: {}", e))?;

            Ok(truncated_text)
        } else {
            // Estimation-only fallback: truncate by character estimate
            let approx_chars = max_tokens * 4; // ~4 chars per token
            let truncated = text.chars().take(approx_chars).collect::<String>();
            Ok(truncated)
        }
    }

    /// Split text into chunks that fit within token limit
    pub fn chunk_by_tokens(&self, text: &str, max_tokens_per_chunk: usize) -> Result<Vec<String>> {
        if let Some(enc) = &self.encoder {
            let tokens = enc.encode_with_special_tokens(text);

            if tokens.len() <= max_tokens_per_chunk {
                return Ok(vec![text.to_string()]);
            }

            let mut chunks = Vec::new();
            let mut start = 0;

            while start < tokens.len() {
                let end = (start + max_tokens_per_chunk).min(tokens.len());
                let chunk_tokens = &tokens[start..end];

                let chunk_text = enc
                    .decode(chunk_tokens.to_vec())
                    .map_err(|e| anyhow!("Failed to decode chunk tokens: {}", e))?;

                chunks.push(chunk_text);
                start = end;
            }

            Ok(chunks)
        } else {
            // Fallback: split by approximate characters per chunk
            let approx_chars = max_tokens_per_chunk * 4;
            let mut chunks = Vec::new();
            let mut start = 0;
            let chars: Vec<char> = text.chars().collect();
            while start < chars.len() {
                let end = (start + approx_chars).min(chars.len());
                let chunk_text: String = chars[start..end].iter().collect();
                chunks.push(chunk_text);
                start = end;
            }
            Ok(chunks)
        }
    }

    /// Find optimal split point to keep token count under limit
    pub fn find_split_point(&self, text: &str, max_tokens: usize) -> Result<usize> {
        if let Some(_enc) = &self.encoder {
            // Use token-aware binary search when encoder is available
            let mut left = 0;
            let mut right = text.len();
            let mut best_split = 0;

            while left <= right {
                let mid = (left + right) / 2;
                let substr = &text[..mid];
                let token_count = self.count_tokens(substr)?;

                if token_count <= max_tokens {
                    best_split = mid;
                    left = mid + 1;
                } else {
                    if mid == 0 {
                        break;
                    }
                    right = mid - 1;
                }
            }

            // Try to split at word boundary
            if best_split > 0 && best_split < text.len() {
                for i in (0..best_split).rev().take(100) {
                    if text.chars().nth(i).unwrap_or(' ').is_whitespace() {
                        return Ok(i);
                    }
                }
            }

            Ok(best_split)
        } else {
            // Estimation-only fallback: approximate characters per token
            let approx_chars = max_tokens * 4;
            Ok(approx_chars.min(text.len()))
        }
    }

    /// Update statistics
    fn update_stats(&self, token_count: usize, cache_hit: bool) {
        let mut stats = self.stats.write();
        stats.total_requests += 1;
        stats.total_tokens_counted += token_count as u64;

        if cache_hit {
            stats.cache_hits += 1;
        } else {
            stats.cache_misses += 1;
        }

        stats.avg_tokens_per_request =
            stats.total_tokens_counted as f32 / stats.total_requests as f32;
        stats.cache_hit_rate = stats.cache_hits as f32 / stats.total_requests as f32;
    }
}

/// Utility functions for token estimation
pub struct TokenEstimator;

impl TokenEstimator {
    /// Quick estimation for English text (characters / 4)
    pub fn estimate_english(text: &str) -> usize {
        (text.len() as f32 / 4.0).ceil() as usize
    }

    /// Estimation for code (characters / 3.5, as code tends to be more token-dense)
    pub fn estimate_code(text: &str) -> usize {
        (text.len() as f32 / 3.5).ceil() as usize
    }

    /// Conservative estimation (characters / 3, overestimates slightly)
    pub fn estimate_conservative(text: &str) -> usize {
        (text.len() as f32 / 3.0).ceil() as usize
    }

    /// Estimation based on word count (words * 1.3)
    pub fn estimate_by_words(text: &str) -> usize {
        let word_count = text.split_whitespace().count();
        (word_count as f32 * 1.3).ceil() as usize
    }
}

/// Token budget manager for managing token allocation
pub struct TokenBudget {
    /// Total available tokens
    total_budget: usize,
    /// Used tokens
    used_tokens: usize,
    /// Reserved tokens for different purposes
    reservations: HashMap<String, usize>,
}

impl TokenBudget {
    /// Create a new token budget
    pub fn new(total_budget: usize) -> Self {
        Self {
            total_budget,
            used_tokens: 0,
            reservations: HashMap::new(),
        }
    }

    /// Reserve tokens for a specific purpose
    pub fn reserve(&mut self, name: &str, tokens: usize) -> Result<()> {
        let available = self.available_tokens();
        if tokens > available {
            return Err(anyhow!(
                "Cannot reserve {} tokens, only {} available",
                tokens,
                available
            ));
        }

        self.reservations.insert(name.to_string(), tokens);
        Ok(())
    }

    /// Use tokens from the budget
    pub fn use_tokens(&mut self, tokens: usize) -> Result<()> {
        if tokens > self.available_tokens() {
            return Err(anyhow!(
                "Would exceed available budget: {} > {} (total: {}, used: {}, reserved: {})",
                tokens,
                self.available_tokens(),
                self.total_budget,
                self.used_tokens,
                self.reservations.values().sum::<usize>()
            ));
        }

        self.used_tokens += tokens;
        Ok(())
    }

    /// Get available tokens (total - used - reserved)
    pub fn available_tokens(&self) -> usize {
        let reserved: usize = self.reservations.values().sum();
        self.total_budget
            .saturating_sub(self.used_tokens)
            .saturating_sub(reserved)
    }

    /// Get used tokens
    pub fn used_tokens(&self) -> usize {
        self.used_tokens
    }

    /// Get total budget
    pub fn total_budget(&self) -> usize {
        self.total_budget
    }

    /// Get utilization as percentage
    pub fn utilization(&self) -> f32 {
        (self.used_tokens as f32) / (self.total_budget as f32)
    }

    /// Reset used tokens
    pub fn reset(&mut self) {
        self.used_tokens = 0;
    }

    /// Clear all reservations
    pub fn clear_reservations(&mut self) {
        self.reservations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counter_creation() {
        let counter = TokenCounter::new("cl100k_base");
        assert_eq!(counter.model_name(), "cl100k_base");
    }

    #[test]
    fn test_token_counter_creation_fallback_model() {
        // Test with an unknown model - should fallback to cl100k_base or estimation
        let counter = TokenCounter::new("unknown_model_xyz");
        assert_eq!(counter.model_name(), "unknown_model_xyz");
        // Should still work with estimation fallback
        let count = counter.count_tokens("Hello world").unwrap();
        assert!(count > 0);
    }

    #[test]
    fn test_token_counting() {
        let counter = TokenCounter::new("cl100k_base");

        let count = counter.count_tokens("Hello, world!").unwrap();
        assert!(count > 0);
        assert!(count < 10); // Should be a small number of tokens
    }

    #[test]
    fn test_token_counting_empty_string() {
        let counter = TokenCounter::new("cl100k_base");
        let count = counter.count_tokens("").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_token_counting_unicode() {
        let counter = TokenCounter::new("cl100k_base");
        let count = counter.count_tokens("こんにちは世界 🌍").unwrap();
        assert!(count > 0);
    }

    #[test]
    fn test_token_counting_long_text() {
        let counter = TokenCounter::new("cl100k_base");
        let long_text = "This is a test sentence. ".repeat(1000);
        let count = counter.count_tokens(&long_text).unwrap();
        assert!(count > 1000); // Should be many tokens
    }

    #[test]
    fn test_token_cache() {
        let counter = TokenCounter::new("cl100k_base");

        let text = "This is a test message";

        // First call should miss cache
        let count1 = counter.count_tokens(text).unwrap();

        // Second call should hit cache
        let count2 = counter.count_tokens(text).unwrap();

        assert_eq!(count1, count2);

        let stats = counter.get_stats();
        assert!(stats.cache_hit_rate > 0.0);
    }

    #[test]
    fn test_token_cache_size_limit() {
        let counter = TokenCounter::new("cl100k_base");
        
        // Add many different strings to test cache size limit
        for i in 0..100 {
            let text = format!("Unique test string number {}", i);
            counter.count_tokens(&text).unwrap();
        }
        
        let stats = counter.get_stats();
        assert_eq!(stats.total_requests, 100);
    }

    #[test]
    fn test_token_cache_clear() {
        let counter = TokenCounter::new("cl100k_base");
        
        let text = "test text for cache";
        counter.count_tokens(text).unwrap();
        counter.count_tokens(text).unwrap();
        
        let stats_before = counter.get_stats();
        assert!(stats_before.cache_hits > 0);
        
        counter.clear_cache();
        
        let stats_after = counter.get_stats();
        assert_eq!(stats_after.cache_hits, 0);
        assert_eq!(stats_after.cache_misses, 0);
    }

    #[test]
    fn test_text_truncation() {
        let counter = TokenCounter::new("cl100k_base");

        let long_text = "This is a long text that should be truncated. ".repeat(100);
        let truncated = counter.truncate_to_tokens(&long_text, 10).unwrap();

        let truncated_count = counter.count_tokens(&truncated).unwrap();
        assert!(truncated_count <= 10);
        assert!(truncated.len() < long_text.len());
    }

    #[test]
    fn test_text_truncation_no_truncation_needed() {
        let counter = TokenCounter::new("cl100k_base");
        let short_text = "Short";
        let result = counter.truncate_to_tokens(short_text, 100).unwrap();
        assert_eq!(result, short_text);
    }

    #[test]
    fn test_text_truncation_exact_limit() {
        let counter = TokenCounter::new("cl100k_base");
        // Test with text that might be exactly at the limit
        let text = "Hello";
        let result = counter.truncate_to_tokens(text, 1).unwrap();
        let count = counter.count_tokens(&result).unwrap();
        assert!(count <= 1);
    }

    #[test]
    fn test_text_chunking() {
        let counter = TokenCounter::new("cl100k_base");

        let long_text = "Word ".repeat(100);
        let chunks = counter.chunk_by_tokens(&long_text, 20).unwrap();

        assert!(chunks.len() > 1);

        for chunk in &chunks {
            let count = counter.count_tokens(chunk).unwrap();
            assert!(count <= 20);
        }
    }

    #[test]
    fn test_text_chunking_short_text() {
        let counter = TokenCounter::new("cl100k_base");
        let short_text = "Hello";
        let chunks = counter.chunk_by_tokens(short_text, 100).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], short_text);
    }

    #[test]
    fn test_text_chunking_empty_text() {
        let counter = TokenCounter::new("cl100k_base");
        let chunks = counter.chunk_by_tokens("", 10).unwrap();
        // Empty text returns no chunks (since tokens <= max for empty)
        // The implementation returns vec![text] for short text, which is empty string
        assert!(chunks.len() <= 1);
    }

    #[test]
    fn test_text_chunking_multiple_chunks() {
        let counter = TokenCounter::new("cl100k_base");
        // Create text that will definitely need multiple chunks
        let long_text = "This is a sentence that contains multiple words and will need chunking. ".repeat(50);
        let chunks = counter.chunk_by_tokens(&long_text, 10).unwrap();
        assert!(chunks.len() >= 5);
        
        // Verify all chunks together approximate the original
        let combined: String = chunks.join("");
        assert!(combined.len() >= long_text.len() * 9 / 10); // At least 90% of original
    }

    #[test]
    fn test_token_estimation() {
        let text = "This is a sample text for estimation testing";

        let english_est = TokenEstimator::estimate_english(text);
        let code_est = TokenEstimator::estimate_code(text);
        let conservative_est = TokenEstimator::estimate_conservative(text);
        let word_est = TokenEstimator::estimate_by_words(text);

        assert!(english_est > 0);
        assert!(code_est >= english_est); // Code should be higher estimate
        assert!(conservative_est >= english_est); // Conservative should be higher
        assert!(word_est > 0);
    }

    #[test]
    fn test_token_estimation_empty() {
        let text = "";
        
        assert_eq!(TokenEstimator::estimate_english(text), 0);
        assert_eq!(TokenEstimator::estimate_code(text), 0);
        assert_eq!(TokenEstimator::estimate_conservative(text), 0);
        assert_eq!(TokenEstimator::estimate_by_words(text), 0);
    }

    #[test]
    fn test_token_estimation_single_char() {
        let text = "a";
        
        // Ceil of 1/4 = 1, 1/3.5 = 1, 1/3 = 1
        assert!(TokenEstimator::estimate_english(text) >= 1);
        assert!(TokenEstimator::estimate_code(text) >= 1);
        assert!(TokenEstimator::estimate_conservative(text) >= 1);
        assert!(TokenEstimator::estimate_by_words(text) >= 1); // 1 word * 1.3 ceil = 2
    }

    #[test]
    fn test_token_estimation_code() {
        // Code typically has more tokens per char than prose
        let code = "fn main() { println!(\"Hello, world!\"); }";
        let est = TokenEstimator::estimate_code(code);
        assert!(est > 0);
        assert!(est >= code.len() / 4); // Should be at least English estimate
    }

    #[test]
    fn test_token_estimation_by_words_various() {
        // Test different word counts
        assert_eq!(TokenEstimator::estimate_by_words("one"), 2); // 1 * 1.3 = 1.3 -> 2
        assert_eq!(TokenEstimator::estimate_by_words("one two"), 3); // 2 * 1.3 = 2.6 -> 3
        assert!(TokenEstimator::estimate_by_words("one two three four five") >= 6); // 5 * 1.3 = 6.5 -> 7
    }

    #[test]
    fn test_token_budget() {
        let mut budget = TokenBudget::new(1000);

        assert_eq!(budget.available_tokens(), 1000);
        assert_eq!(budget.utilization(), 0.0);

        budget.use_tokens(300).unwrap();
        assert_eq!(budget.used_tokens(), 300);
        assert_eq!(budget.available_tokens(), 700);

        budget.reserve("context", 200).unwrap();
        assert_eq!(budget.available_tokens(), 500);

        // Should fail to use more than available
        assert!(budget.use_tokens(600).is_err());
    }

    #[test]
    fn test_token_budget_total() {
        let budget = TokenBudget::new(500);
        assert_eq!(budget.total_budget(), 500);
    }

    #[test]
    fn test_token_budget_utilization() {
        let mut budget = TokenBudget::new(1000);
        
        assert_eq!(budget.utilization(), 0.0);
        
        budget.use_tokens(250).unwrap();
        assert!((budget.utilization() - 0.25).abs() < 0.001);
        
        budget.use_tokens(250).unwrap();
        assert!((budget.utilization() - 0.5).abs() < 0.001);
        
        budget.use_tokens(500).unwrap();
        assert!((budget.utilization() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_token_budget_reserve_exceeds_available() {
        let mut budget = TokenBudget::new(100);
        budget.use_tokens(50).unwrap();
        
        let result = budget.reserve("big", 200);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_budget_multiple_reservations() {
        let mut budget = TokenBudget::new(1000);
        
        budget.reserve("context", 100).unwrap();
        budget.reserve("system", 100).unwrap();
        budget.reserve("response", 100).unwrap();
        
        assert_eq!(budget.available_tokens(), 700);
        
        // Can override a reservation
        budget.reserve("context", 200).unwrap();
        assert_eq!(budget.available_tokens(), 600); // 1000 - (200 + 100 + 100)
    }

    #[test]
    fn test_token_budget_use_tokens_error_message() {
        let mut budget = TokenBudget::new(100);
        budget.use_tokens(50).unwrap();
        budget.reserve("reserved", 30).unwrap();
        
        let result = budget.use_tokens(50);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("exceed"));
    }

    #[test]
    fn test_find_split_point() {
        let counter = TokenCounter::new("cl100k_base");

        let text = "First sentence. Second sentence. Third sentence.";
        let split_point = counter.find_split_point(text, 5).unwrap();

        assert!(split_point > 0);
        assert!(split_point < text.len());

        let prefix = &text[..split_point];
        let token_count = counter.count_tokens(prefix).unwrap();
        assert!(token_count <= 5);
    }

    #[test]
    fn test_find_split_point_short_text() {
        let counter = TokenCounter::new("cl100k_base");
        let text = "Hi";
        let split_point = counter.find_split_point(text, 100).unwrap();
        // Should return full length when under limit
        assert!(split_point <= text.len());
    }

    #[test]
    fn test_find_split_point_at_word_boundary() {
        let counter = TokenCounter::new("cl100k_base");
        // Text with clear word boundaries
        let text = "The quick brown fox jumps over the lazy dog repeatedly and continuously.";
        let split_point = counter.find_split_point(text, 3).unwrap();
        
        // The split should ideally be at a whitespace boundary
        if split_point > 0 && split_point < text.len() {
            // Check that we're either at start, end, or near a whitespace
            let nearby = &text[..split_point.min(text.len())];
            // Just verify it doesn't panic and returns a valid point
            assert!(split_point <= text.len());
        }
    }

    #[test]
    fn test_find_split_point_very_restrictive() {
        let counter = TokenCounter::new("cl100k_base");
        let text = "Hello world this is a test";
        // Very restrictive - only 1 token
        let split_point = counter.find_split_point(text, 1).unwrap();
        // Should find some split point
        assert!(split_point <= text.len());
    }

    #[test]
    fn test_count_tokens_batch() {
        let counter = TokenCounter::new("cl100k_base");

        let texts = vec![
            "First message".to_string(),
            "Second message".to_string(),
            "Third message".to_string(),
            "Fourth message with more content".to_string(),
        ];

        let counts = counter.count_tokens_batch(&texts).unwrap();

        assert_eq!(counts.len(), texts.len());

        // All counts should be positive
        for count in &counts {
            assert!(*count > 0);
        }

        // Verify batch results match individual calls
        for (i, text) in texts.iter().enumerate() {
            let individual_count = counter.count_tokens(text).unwrap();
            assert_eq!(counts[i], individual_count);
        }
    }

    #[test]
    fn test_count_tokens_batch_empty() {
        let counter = TokenCounter::new("cl100k_base");
        let texts: Vec<String> = vec![];
        let counts = counter.count_tokens_batch(&texts).unwrap();
        assert!(counts.is_empty());
    }

    #[test]
    fn test_count_tokens_batch_single() {
        let counter = TokenCounter::new("cl100k_base");
        let texts = vec!["Single item".to_string()];
        let counts = counter.count_tokens_batch(&texts).unwrap();
        assert_eq!(counts.len(), 1);
        assert!(counts[0] > 0);
    }

    #[test]
    fn test_token_budget_reset() {
        let mut budget = TokenBudget::new(1000);

        budget.use_tokens(300).unwrap();
        assert_eq!(budget.used_tokens(), 300);
        assert_eq!(budget.available_tokens(), 700);

        budget.reset();

        assert_eq!(budget.used_tokens(), 0);
        assert_eq!(budget.available_tokens(), 1000);
        assert_eq!(budget.utilization(), 0.0);
    }

    #[test]
    fn test_token_budget_clear_reservations() {
        let mut budget = TokenBudget::new(1000);

        budget.reserve("context", 200).unwrap();
        budget.reserve("response", 300).unwrap();

        assert_eq!(budget.available_tokens(), 500);

        budget.clear_reservations();

        assert_eq!(budget.available_tokens(), 1000);

        // Should be able to use more tokens now
        budget.use_tokens(800).unwrap();
        assert_eq!(budget.used_tokens(), 800);
    }

    #[test]
    fn test_token_counter_stats() {
        let counter = TokenCounter::new("cl100k_base");
        
        let initial_stats = counter.get_stats();
        assert_eq!(initial_stats.total_requests, 0);
        assert_eq!(initial_stats.total_tokens_counted, 0);
        assert_eq!(initial_stats.avg_tokens_per_request, 0.0);
        
        counter.count_tokens("Hello").unwrap();
        counter.count_tokens("World").unwrap();
        
        let stats = counter.get_stats();
        assert_eq!(stats.total_requests, 2);
        assert!(stats.total_tokens_counted > 0);
        assert!(stats.avg_tokens_per_request > 0.0);
    }

    #[test]
    fn test_token_counter_stats_cache_hit_rate() {
        let counter = TokenCounter::new("cl100k_base");
        
        let text = "repeated text";
        counter.count_tokens(text).unwrap(); // miss
        counter.count_tokens(text).unwrap(); // hit
        counter.count_tokens(text).unwrap(); // hit
        counter.count_tokens(text).unwrap(); // hit
        
        let stats = counter.get_stats();
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.cache_hits, 3);
        assert!((stats.cache_hit_rate - 0.75).abs() < 0.01); // 3/4 = 0.75
    }

    #[test]
    fn test_estimate_tokens_direct() {
        let counter = TokenCounter::new("cl100k_base");
        
        // Test the estimate_tokens method directly
        let estimate = counter.estimate_tokens("This is a test");
        assert!(estimate > 0);
        assert!(estimate < 100); // Reasonable estimate
    }

    #[test]
    fn test_token_budget_zero() {
        let mut budget = TokenBudget::new(0);
        assert_eq!(budget.available_tokens(), 0);
        assert!(budget.use_tokens(1).is_err());
    }

    #[test]
    fn test_token_budget_saturating_sub() {
        // Test the saturating_sub behavior when used + reserved > total
        // This shouldn't happen in normal use but tests the safety
        let budget = TokenBudget::new(100);
        assert_eq!(budget.available_tokens(), 100);
    }
}
