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
