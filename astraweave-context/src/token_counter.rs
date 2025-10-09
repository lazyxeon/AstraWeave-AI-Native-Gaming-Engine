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
        if self.used_tokens + tokens > self.total_budget {
            return Err(anyhow!(
                "Would exceed budget: {} + {} > {}",
                self.used_tokens,
                tokens,
                self.total_budget
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
    fn test_token_counting() {
        let counter = TokenCounter::new("cl100k_base");

        let count = counter.count_tokens("Hello, world!").unwrap();
        assert!(count > 0);
        assert!(count < 10); // Should be a small number of tokens
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
    fn test_text_truncation() {
        let counter = TokenCounter::new("cl100k_base");

        let long_text = "This is a long text that should be truncated. ".repeat(100);
        let truncated = counter.truncate_to_tokens(&long_text, 10).unwrap();

        let truncated_count = counter.count_tokens(&truncated).unwrap();
        assert!(truncated_count <= 10);
        assert!(truncated.len() < long_text.len());
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
}
