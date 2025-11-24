//! Prompt templating engine core functionality
//!
//! This module provides the core templating engine for AstraWeave prompt management.

use crate::context::PromptContext;
use crate::helpers::register_default_helpers;
use crate::template::PromptTemplate;
use anyhow::Result;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Core prompt templating engine
#[derive(Debug, Clone)]
pub struct PromptEngine {
    /// Handlebars registry
    registry: Handlebars<'static>,
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
        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);
        
        Self {
            registry,
            config,
        }
    }

    /// Register a template
    pub fn register_template(&mut self, name: String, template: String) -> Result<()> {
        if template.len() > self.config.max_template_size {
            anyhow::bail!(
                "Template too large: {} bytes > {} bytes",
                template.len(),
                self.config.max_template_size
            );
        }

        self.registry.register_template_string(&name, template)?;
        Ok(())
    }

    /// Register a partial
    pub fn register_partial(&mut self, name: String, partial: String) -> Result<()> {
        self.registry.register_partial(&name, partial)?;
        Ok(())
    }

    /// Register a helper
    pub fn register_helper(
        &mut self,
        name: &str,
        helper: Box<dyn handlebars::HelperDef + Send + Sync>,
    ) {
        self.registry.register_helper(name, helper);
    }

    /// Render a template
    pub fn render(&self, name: &str, ctx: &PromptContext) -> Result<String> {
        // Convert context to serde_json::Value for Handlebars
        let vars = ctx.to_json();
        Ok(self.registry.render(name, &vars)?)
    }

    /// List all registered templates
    pub fn list_templates(&self) -> Vec<String> {
        self.registry.get_templates().keys().cloned().collect()
    }

    /// Clear all registered templates
    pub fn clear_templates(&mut self) {
        self.registry.clear_templates();
    }
}

/// Public TemplateEngine expected by consumers
#[derive(Debug, Clone)]
pub struct TemplateEngine {
    inner: PromptEngine,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            inner: PromptEngine::new(EngineConfig::default()),
        };
        register_default_helpers(&mut engine);
        engine
    }

    pub fn register_template(&mut self, name: &str, template: PromptTemplate) -> Result<()> {
        self.inner
            .register_template(name.to_string(), template.template)
    }

    pub fn register_partial(&mut self, name: &str, partial: &str) -> Result<()> {
        self.inner.register_partial(name.to_string(), partial.to_string())
    }

    pub fn register_helper(
        &mut self,
        name: &str,
        helper: Box<dyn handlebars::HelperDef + Send + Sync>,
    ) {
        self.inner.register_helper(name, helper);
    }

    pub fn load_templates_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<()> {
        let loader = crate::loader::PromptLoader::new();
        let templates = loader.load_from_dir(dir)?;
        for template in templates {
            self.register_template(&template.id.clone(), template)?;
        }
        Ok(())
    }

    pub fn render(&self, name: &str, ctx: &PromptContext) -> Result<String> {
        self.inner.render(name, ctx)
    }

    pub fn list_templates(&self) -> Vec<String> {
        self.inner.list_templates()
    }

    pub fn clear_templates(&mut self) {
        self.inner.clear_templates();
    }
}
