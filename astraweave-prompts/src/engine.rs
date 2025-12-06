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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ContextValue;

    // EngineConfig tests
    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert_eq!(config.max_template_size, 1024 * 1024);
        assert!(config.enable_caching);
    }

    #[test]
    fn test_engine_config_serialization() {
        let config = EngineConfig {
            max_template_size: 500,
            enable_caching: false,
        };
        
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: EngineConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.max_template_size, deserialized.max_template_size);
        assert_eq!(config.enable_caching, deserialized.enable_caching);
    }

    // PromptEngine tests
    #[test]
    fn test_prompt_engine_new() {
        let engine = PromptEngine::new(EngineConfig::default());
        assert!(engine.list_templates().is_empty());
    }

    #[test]
    fn test_prompt_engine_register_template() {
        let mut engine = PromptEngine::new(EngineConfig::default());
        let result = engine.register_template("greeting".to_string(), "Hello {{name}}!".to_string());
        
        assert!(result.is_ok());
        assert!(engine.list_templates().contains(&"greeting".to_string()));
    }

    #[test]
    fn test_prompt_engine_register_template_too_large() {
        let config = EngineConfig {
            max_template_size: 10, // Very small
            enable_caching: true,
        };
        let mut engine = PromptEngine::new(config);
        
        let result = engine.register_template("big".to_string(), "a".repeat(100));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_prompt_engine_register_partial() {
        let mut engine = PromptEngine::new(EngineConfig::default());
        let result = engine.register_partial("header".to_string(), "# {{title}}".to_string());
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_engine_render() {
        let mut engine = PromptEngine::new(EngineConfig::default());
        engine.register_template("test".to_string(), "Hello {{name}}!".to_string()).unwrap();
        
        let mut ctx = PromptContext::new();
        ctx.set("name".to_string(), ContextValue::String("World".to_string()));
        
        let result = engine.render("test", &ctx).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_prompt_engine_render_nonexistent() {
        let engine = PromptEngine::new(EngineConfig::default());
        let ctx = PromptContext::new();
        
        let result = engine.render("missing", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_engine_list_templates() {
        let mut engine = PromptEngine::new(EngineConfig::default());
        engine.register_template("a".to_string(), "{{x}}".to_string()).unwrap();
        engine.register_template("b".to_string(), "{{y}}".to_string()).unwrap();
        
        let templates = engine.list_templates();
        assert_eq!(templates.len(), 2);
        assert!(templates.contains(&"a".to_string()));
        assert!(templates.contains(&"b".to_string()));
    }

    #[test]
    fn test_prompt_engine_clear_templates() {
        let mut engine = PromptEngine::new(EngineConfig::default());
        engine.register_template("test".to_string(), "{{x}}".to_string()).unwrap();
        
        assert!(!engine.list_templates().is_empty());
        
        engine.clear_templates();
        assert!(engine.list_templates().is_empty());
    }

    // TemplateEngine tests
    #[test]
    fn test_template_engine_new() {
        let engine = TemplateEngine::new();
        // Should be empty initially (no templates registered)
        assert!(engine.list_templates().is_empty());
    }

    #[test]
    fn test_template_engine_register_template() {
        let mut engine = TemplateEngine::new();
        let template = PromptTemplate::new("greeting", "Hello {{name}}!");
        
        let result = engine.register_template("greeting", template);
        assert!(result.is_ok());
        assert!(engine.list_templates().contains(&"greeting".to_string()));
    }

    #[test]
    fn test_template_engine_register_partial() {
        let mut engine = TemplateEngine::new();
        let result = engine.register_partial("footer", "---\n{{text}}");
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_template_engine_render() {
        let mut engine = TemplateEngine::new();
        let template = PromptTemplate::new("test", "Value: {{value}}");
        engine.register_template("test", template).unwrap();
        
        let mut ctx = PromptContext::new();
        ctx.set("value".to_string(), ContextValue::String("42".to_string()));
        
        let result = engine.render("test", &ctx).unwrap();
        assert_eq!(result, "Value: 42");
    }

    #[test]
    fn test_template_engine_list_templates() {
        let mut engine = TemplateEngine::new();
        let t1 = PromptTemplate::new("first", "{{a}}");
        let t2 = PromptTemplate::new("second", "{{b}}");
        
        engine.register_template("first", t1).unwrap();
        engine.register_template("second", t2).unwrap();
        
        let list = engine.list_templates();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_template_engine_clear_templates() {
        let mut engine = TemplateEngine::new();
        let template = PromptTemplate::new("test", "{{x}}");
        engine.register_template("test", template).unwrap();
        
        engine.clear_templates();
        assert!(engine.list_templates().is_empty());
    }

    #[test]
    fn test_template_engine_with_partial() {
        let mut engine = TemplateEngine::new();
        
        // Register a partial
        engine.register_partial("sig", "-- Regards, {{name}}").unwrap();
        
        // Register a template that uses the partial
        let template = PromptTemplate::new("email", "Hello!\n{{> sig}}");
        engine.register_template("email", template).unwrap();
        
        let mut ctx = PromptContext::new();
        ctx.set("name".to_string(), ContextValue::String("Alice".to_string()));
        
        let result = engine.render("email", &ctx).unwrap();
        assert!(result.contains("Regards, Alice"));
    }

    #[test]
    fn test_template_engine_multiple_variables() {
        let mut engine = TemplateEngine::new();
        let template = PromptTemplate::new("intro", "{{greeting}}, {{name}}!");
        engine.register_template("intro", template).unwrap();
        
        let mut ctx = PromptContext::new();
        ctx.set("greeting".to_string(), ContextValue::String("Hi".to_string()));
        ctx.set("name".to_string(), ContextValue::String("Bob".to_string()));
        
        let result = engine.render("intro", &ctx).unwrap();
        assert_eq!(result, "Hi, Bob!");
    }
}
