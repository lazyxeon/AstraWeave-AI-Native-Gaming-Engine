//! Prompt optimization and performance utilities
//!
//! This module provides optimization features for prompt processing.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Prompt optimization engine
#[derive(Debug, Clone)]
pub struct OptimizationEngine {
    /// Optimization configuration
    config: OptimizationConfig,
    /// Performance metrics
    metrics: PerformanceMetrics,
}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Enable template caching
    pub enable_caching: bool,
    /// Cache size limit
    pub cache_size_limit: usize,
    /// Enable prompt compression
    pub enable_compression: bool,
    /// Maximum prompt length before optimization
    pub max_prompt_length: usize,
}

/// Performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total templates processed
    pub templates_processed: u64,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    /// Average processing time
    pub avg_processing_time_ms: f64,
    /// Total processing time
    pub total_processing_time: Duration,
}

/// Template cache
#[derive(Debug, Clone)]
pub struct TemplateCache {
    /// Cached templates
    cache: HashMap<String, CachedTemplate>,
    /// Cache configuration
    config: CacheConfig,
}

/// Cached template
#[derive(Debug, Clone)]
pub struct CachedTemplate {
    /// Compiled template
    template: String,
    /// Cache timestamp
    created_at: Instant,
    /// Access count
    access_count: u64,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum cache size
    pub max_size: usize,
    /// TTL for cached items
    pub ttl_seconds: u64,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_size_limit: 1000,
            enable_compression: true,
            max_prompt_length: 4000,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            ttl_seconds: 3600, // 1 hour
        }
    }
}

impl OptimizationEngine {
    /// Create a new optimization engine
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            config,
            metrics: PerformanceMetrics::default(),
        }
    }

    /// Optimize a prompt for processing
    pub fn optimize_prompt(&mut self, prompt: &str) -> Result<String> {
        let start_time = Instant::now();

        let mut optimized = prompt.to_string();

        // Apply optimization techniques
        if self.config.enable_compression && prompt.len() > self.config.max_prompt_length {
            optimized = self.compress_prompt(&optimized)?;
        }

        // Update metrics
        let processing_time = start_time.elapsed();
        self.update_metrics(processing_time);

        Ok(optimized)
    }

    /// Compress a prompt to reduce token count
    fn compress_prompt(&self, prompt: &str) -> Result<String> {
        // Simple compression - remove extra whitespace
        let compressed = prompt
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(compressed)
    }

    /// Update performance metrics
    fn update_metrics(&mut self, processing_time: Duration) {
        self.metrics.templates_processed += 1;
        self.metrics.total_processing_time += processing_time;

        // Calculate average processing time
        let total_ms = self.metrics.total_processing_time.as_millis() as f64;
        self.metrics.avg_processing_time_ms = total_ms / self.metrics.templates_processed as f64;
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Reset metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = PerformanceMetrics::default();
    }
}

impl TemplateCache {
    /// Create a new template cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: HashMap::new(),
            config,
        }
    }

    /// Get a cached template
    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(cached) = self.cache.get_mut(key) {
            // Check TTL
            if cached.created_at.elapsed().as_secs() > self.config.ttl_seconds {
                self.cache.remove(key);
                return None;
            }

            cached.access_count += 1;
            Some(cached.template.clone())
        } else {
            None
        }
    }

    /// Put a template in cache
    pub fn put(&mut self, key: String, template: String) {
        // Check cache size limit
        if self.cache.len() >= self.config.max_size {
            self.evict_lru();
        }

        let cached = CachedTemplate {
            template,
            created_at: Instant::now(),
            access_count: 1,
        };

        self.cache.insert(key, cached);
    }

    /// Evict least recently used item
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self.find_lru_key() {
            self.cache.remove(&lru_key);
        }
    }

    /// Find the least recently used cache key
    fn find_lru_key(&self) -> Option<String> {
        self.cache
            .iter()
            .min_by_key(|(_, cached)| cached.access_count)
            .map(|(key, _)| key.clone())
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            max_size: self.config.max_size,
            hit_count: self.cache.values().map(|c| c.access_count).sum(),
        }
    }
}

/// A/B Testing Engine
#[derive(Debug, Clone)]
pub struct ABTestingEngine {
    variants: HashMap<String, Vec<String>>,
    metrics: HashMap<String, ABMetrics>,
}

#[derive(Debug, Clone, Default)]
pub struct ABMetrics {
    pub selections: HashMap<String, u64>,
    pub successes: HashMap<String, u64>,
}

impl ABTestingEngine {
    pub fn new() -> Self {
        Self {
            variants: HashMap::new(),
            metrics: HashMap::new(),
        }
    }

    pub fn register_variant(&mut self, test_name: String, variant_name: String) {
        self.variants
            .entry(test_name.clone())
            .or_default()
            .push(variant_name);
        self.metrics.entry(test_name).or_default();
    }

    pub fn select_variant(&mut self, test_name: &str) -> Option<String> {
        if let Some(variants) = self.variants.get(test_name) {
            if variants.is_empty() {
                return None;
            }
            // Simple round-robin or random selection
            // For determinism in tests, we'll pick based on total selections
            let metrics = self.metrics.get_mut(test_name).unwrap();
            let total_selections: u64 = metrics.selections.values().sum();
            let index = (total_selections as usize) % variants.len();
            let selected = variants[index].clone();

            *metrics.selections.entry(selected.clone()).or_default() += 1;
            Some(selected)
        } else {
            None
        }
    }

    pub fn record_success(&mut self, test_name: &str, variant_name: &str) {
        if let Some(metrics) = self.metrics.get_mut(test_name) {
            *metrics.successes.entry(variant_name.to_string()).or_default() += 1;
        }
    }

    pub fn get_metrics(&self, test_name: &str) -> Option<&ABMetrics> {
        self.metrics.get(test_name)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Current cache size
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Total cache hits
    pub hit_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config_default() {
        let config = OptimizationConfig::default();
        assert!(config.enable_caching);
        assert_eq!(config.cache_size_limit, 1000);
        assert!(config.enable_compression);
        assert_eq!(config.max_prompt_length, 4000);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_size, 1000);
        assert_eq!(config.ttl_seconds, 3600);
    }

    #[test]
    fn test_optimization_engine_new() {
        let config = OptimizationConfig::default();
        let engine = OptimizationEngine::new(config);
        let metrics = engine.get_metrics();
        assert_eq!(metrics.templates_processed, 0);
        assert_eq!(metrics.avg_processing_time_ms, 0.0);
    }

    #[test]
    fn test_optimize_prompt_short() {
        let config = OptimizationConfig::default();
        let mut engine = OptimizationEngine::new(config);
        
        let prompt = "Hello world";
        let result = engine.optimize_prompt(prompt).unwrap();
        assert_eq!(result, prompt);
        
        let metrics = engine.get_metrics();
        assert_eq!(metrics.templates_processed, 1);
    }

    #[test]
    fn test_optimize_prompt_long_with_compression() {
        let mut config = OptimizationConfig::default();
        config.max_prompt_length = 50; // Low threshold
        config.enable_compression = true;
        let mut engine = OptimizationEngine::new(config);
        
        let prompt = "  Line 1  \n\n  Line 2  \n  Line 3  \n  More content here for testing  ";
        let result = engine.optimize_prompt(prompt).unwrap();
        
        // Compression should join lines with single spaces
        assert!(!result.contains("\n"));
        assert!(!result.contains("  ")); // No double spaces
    }

    #[test]
    fn test_optimize_prompt_no_compression_when_disabled() {
        let mut config = OptimizationConfig::default();
        config.enable_compression = false;
        config.max_prompt_length = 10;
        let mut engine = OptimizationEngine::new(config);
        
        let prompt = "  Line 1  \n\n  Line 2  ";
        let result = engine.optimize_prompt(prompt).unwrap();
        
        // Without compression, should return as-is
        assert_eq!(result, prompt);
    }

    #[test]
    fn test_metrics_update() {
        let config = OptimizationConfig::default();
        let mut engine = OptimizationEngine::new(config);
        
        // Process multiple prompts
        for i in 0..5 {
            engine.optimize_prompt(&format!("Prompt {}", i)).unwrap();
        }
        
        let metrics = engine.get_metrics();
        assert_eq!(metrics.templates_processed, 5);
        assert!(metrics.avg_processing_time_ms >= 0.0);
    }

    #[test]
    fn test_reset_metrics() {
        let config = OptimizationConfig::default();
        let mut engine = OptimizationEngine::new(config);
        
        engine.optimize_prompt("test").unwrap();
        assert_eq!(engine.get_metrics().templates_processed, 1);
        
        engine.reset_metrics();
        assert_eq!(engine.get_metrics().templates_processed, 0);
        assert_eq!(engine.get_metrics().avg_processing_time_ms, 0.0);
    }

    #[test]
    fn test_template_cache_new() {
        let config = CacheConfig::default();
        let cache = TemplateCache::new(config);
        let stats = cache.stats();
        assert_eq!(stats.size, 0);
        assert_eq!(stats.max_size, 1000);
    }

    #[test]
    fn test_template_cache_put_get() {
        let config = CacheConfig::default();
        let mut cache = TemplateCache::new(config);
        
        cache.put("key1".to_string(), "template1".to_string());
        
        let result = cache.get("key1");
        assert_eq!(result, Some("template1".to_string()));
        
        // Access count should increase
        let _ = cache.get("key1");
        let stats = cache.stats();
        assert!(stats.hit_count >= 2);
    }

    #[test]
    fn test_template_cache_miss() {
        let config = CacheConfig::default();
        let mut cache = TemplateCache::new(config);
        
        let result = cache.get("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_template_cache_clear() {
        let config = CacheConfig::default();
        let mut cache = TemplateCache::new(config);
        
        cache.put("key1".to_string(), "template1".to_string());
        cache.put("key2".to_string(), "template2".to_string());
        
        assert_eq!(cache.stats().size, 2);
        
        cache.clear();
        
        assert_eq!(cache.stats().size, 0);
    }

    #[test]
    fn test_template_cache_eviction() {
        let mut config = CacheConfig::default();
        config.max_size = 2; // Small limit
        let mut cache = TemplateCache::new(config);
        
        cache.put("key1".to_string(), "template1".to_string());
        cache.put("key2".to_string(), "template2".to_string());
        
        // Access key2 more to increase its access_count
        cache.get("key2");
        cache.get("key2");
        
        // Adding key3 should evict LRU (key1)
        cache.put("key3".to_string(), "template3".to_string());
        
        // key1 should be evicted (LRU)
        assert!(cache.get("key1").is_none());
        // key2 and key3 should still exist
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_ab_testing_engine_new() {
        let engine = ABTestingEngine::new();
        assert!(engine.variants.is_empty());
        assert!(engine.metrics.is_empty());
    }

    #[test]
    fn test_ab_testing_register_variant() {
        let mut engine = ABTestingEngine::new();
        
        engine.register_variant("test1".to_string(), "variant_a".to_string());
        engine.register_variant("test1".to_string(), "variant_b".to_string());
        
        assert!(engine.variants.contains_key("test1"));
        assert_eq!(engine.variants["test1"].len(), 2);
    }

    #[test]
    fn test_ab_testing_select_variant() {
        let mut engine = ABTestingEngine::new();
        
        engine.register_variant("test1".to_string(), "variant_a".to_string());
        engine.register_variant("test1".to_string(), "variant_b".to_string());
        
        // First selection should be variant_a (index 0)
        let selected1 = engine.select_variant("test1");
        assert_eq!(selected1, Some("variant_a".to_string()));
        
        // Second selection should be variant_b (index 1)
        let selected2 = engine.select_variant("test1");
        assert_eq!(selected2, Some("variant_b".to_string()));
        
        // Third selection should cycle back to variant_a
        let selected3 = engine.select_variant("test1");
        assert_eq!(selected3, Some("variant_a".to_string()));
    }

    #[test]
    fn test_ab_testing_select_variant_nonexistent() {
        let mut engine = ABTestingEngine::new();
        
        let result = engine.select_variant("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_ab_testing_select_variant_empty() {
        let mut engine = ABTestingEngine::new();
        
        // Register but with no variants pushed
        engine.variants.insert("empty_test".to_string(), Vec::new());
        engine.metrics.insert("empty_test".to_string(), ABMetrics::default());
        
        let result = engine.select_variant("empty_test");
        assert!(result.is_none());
    }

    #[test]
    fn test_ab_testing_record_success() {
        let mut engine = ABTestingEngine::new();
        
        engine.register_variant("test1".to_string(), "variant_a".to_string());
        engine.select_variant("test1");
        
        engine.record_success("test1", "variant_a");
        engine.record_success("test1", "variant_a");
        
        let metrics = engine.get_metrics("test1").unwrap();
        assert_eq!(metrics.successes["variant_a"], 2);
    }

    #[test]
    fn test_ab_testing_record_success_no_test() {
        let mut engine = ABTestingEngine::new();
        
        // Should not panic on nonexistent test
        engine.record_success("nonexistent", "variant");
    }

    #[test]
    fn test_ab_testing_get_metrics_none() {
        let engine = ABTestingEngine::new();
        
        let metrics = engine.get_metrics("nonexistent");
        assert!(metrics.is_none());
    }

    #[test]
    fn test_cache_stats_serialization() {
        let stats = CacheStats {
            size: 10,
            max_size: 100,
            hit_count: 50,
        };
        
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: CacheStats = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.size, 10);
        assert_eq!(deserialized.max_size, 100);
        assert_eq!(deserialized.hit_count, 50);
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.templates_processed, 0);
        assert_eq!(metrics.cache_hit_ratio, 0.0);
        assert_eq!(metrics.avg_processing_time_ms, 0.0);
    }

    #[test]
    fn test_compress_prompt_empty_lines() {
        let mut config = OptimizationConfig::default();
        config.max_prompt_length = 10;
        config.enable_compression = true;
        let mut engine = OptimizationEngine::new(config);
        
        let prompt = "\n\n\n\nLine 1\n\n\nLine 2\n\n\n";
        let result = engine.optimize_prompt(prompt).unwrap();
        
        // Should remove empty lines and trim
        assert!(!result.is_empty());
        assert!(!result.starts_with('\n'));
        assert!(!result.ends_with('\n'));
    }
}
