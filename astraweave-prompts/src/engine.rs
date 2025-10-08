//! Prompt templating engine core functionality
//!
//! This module provides the core templating engine for AstraWeave prompt management.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core prompt templating engine
#[derive(Debug, Clone)]
pub struct PromptEngine {
    /// Template registry
    templates: HashMap<String, String>,
    /// Configuration
    config: EngineConfig,
}

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Maximum template size in bytes
    pub max_template_size: usize,
    /// Enable template caching
    pub enable_caching: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_template_size: 1024 * 1024, // 1MB
            enable_caching: true,
        }
    }
}

impl PromptEngine {
    /// Create a new prompt engine
    pub fn new(config: EngineConfig) -> Self {
        Self {
            templates: HashMap::new(),
            config,
        }
    }

    /// Register a template
    pub fn register_template(&mut self, name: String, template: String) -> Result<()> {
        if template.len() > self.config.max_template_size {
            anyhow::bail!("Template too large: {} bytes > {} bytes",
                         template.len(), self.config.max_template_size);
        }

        self.templates.insert(name, template);
        Ok(())
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&String> {
        self.templates.get(name)
    }

    /// List all registered templates
    pub fn list_templates(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }
}