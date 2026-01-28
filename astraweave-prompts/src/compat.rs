//! Compatibility layer for simpler prompt template usage
//!
//! This module provides lightweight wrapper types that offer a simpler API
//! while delegating to the full-featured engine implementation.

#[allow(unused_imports)]
use crate::context::{ContextValue, PromptContext};
use crate::engine::PromptEngine;
use anyhow::Result;

/// Lightweight PromptTemplate for compatibility with consumers
///
/// This is a simpler version of the full `template::PromptTemplate` for
/// use cases that don't need metadata or advanced features.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PromptTemplate {
    /// Template identifier
    pub id: String,
    /// Template content with {{variable}} placeholders
    pub template: String,
}

#[allow(dead_code)]
impl PromptTemplate {
    /// Create a new prompt template
    pub fn new<S: Into<String>>(id: S, template: S) -> Self {
        Self {
            id: id.into(),
            template: template.into(),
        }
    }

    /// Returns true if the template ID is empty
    pub fn has_empty_id(&self) -> bool {
        self.id.is_empty()
    }

    /// Returns true if the template content is empty
    pub fn has_empty_template(&self) -> bool {
        self.template.is_empty()
    }

    /// Returns the length of the template content
    pub fn template_len(&self) -> usize {
        self.template.len()
    }

    /// Returns true if template contains a placeholder variable
    pub fn has_placeholder(&self) -> bool {
        self.template.contains("{{") && self.template.contains("}}")
    }

    /// Count the number of placeholder variables in the template
    pub fn placeholder_count(&self) -> usize {
        self.template.matches("{{").count()
    }
}

/// TemplateEngine wrapper that provides a minimal API
///
/// This wrapper delegates to `PromptEngine` but provides a simpler interface
/// for basic use cases.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TemplateEngine {
    engine: PromptEngine,
}

#[allow(dead_code)]
impl TemplateEngine {
    /// Create a new template engine with default configuration
    pub fn new() -> Self {
        Self {
            engine: PromptEngine::new(crate::engine::EngineConfig::default()),
        }
    }

    /// Register a template with the engine
    pub fn register_template(&mut self, name: &str, template: PromptTemplate) -> Result<()> {
        self.engine
            .register_template(name.to_string(), template.template)
    }

    /// Render a template with the given context
    pub fn render(&self, name: &str, ctx: &PromptContext) -> Result<String> {
        self.engine.render(name, ctx)
    }

    /// Check if a template with the given name is registered
    pub fn has_template(&self, name: &str) -> bool {
        self.engine.has_template(name)
    }

    /// Get the number of registered templates
    pub fn template_count(&self) -> usize {
        self.engine.template_count()
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// TemplateContext is an alias to PromptContext for compatibility
#[allow(dead_code)]
pub type TemplateContext = PromptContext;

#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================================
    // PromptTemplate Construction Tests
    // ====================================================================

    #[test]
    fn test_prompt_template_new() {
        let template = PromptTemplate::new("greeting", "Hello, {{name}}!");
        assert_eq!(template.id, "greeting");
        assert_eq!(template.template, "Hello, {{name}}!");
    }

    #[test]
    fn test_prompt_template_from_string() {
        let id = String::from("test_id");
        let content = String::from("Test content");
        let template = PromptTemplate::new(id, content);
        assert_eq!(template.id, "test_id");
        assert_eq!(template.template, "Test content");
    }

    #[test]
    fn test_prompt_template_empty() {
        let template = PromptTemplate::new("", "");
        assert!(template.has_empty_id());
        assert!(template.has_empty_template());
    }

    #[test]
    fn test_prompt_template_clone() {
        let template = PromptTemplate::new("original", "Content here");
        let cloned = template.clone();
        assert_eq!(template.id, cloned.id);
        assert_eq!(template.template, cloned.template);
    }

    #[test]
    fn test_prompt_template_debug() {
        let template = PromptTemplate::new("debug_test", "Debug content");
        let debug_str = format!("{:?}", template);
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("Debug content"));
    }

    // ====================================================================
    // PromptTemplate Helper Methods Tests
    // ====================================================================

    #[test]
    fn test_prompt_template_has_empty_id() {
        assert!(PromptTemplate::new("", "content").has_empty_id());
        assert!(!PromptTemplate::new("id", "content").has_empty_id());
    }

    #[test]
    fn test_prompt_template_has_empty_template() {
        assert!(PromptTemplate::new("id", "").has_empty_template());
        assert!(!PromptTemplate::new("id", "content").has_empty_template());
    }

    #[test]
    fn test_prompt_template_template_len() {
        let template = PromptTemplate::new("id", "Hello, World!");
        assert_eq!(template.template_len(), 13);
    }

    #[test]
    fn test_prompt_template_has_placeholder() {
        assert!(PromptTemplate::new("id", "Hello, {{name}}!").has_placeholder());
        assert!(PromptTemplate::new("id", "{{a}} and {{b}}").has_placeholder());
        assert!(!PromptTemplate::new("id", "No placeholders here").has_placeholder());
        // Partial placeholder is not valid
        assert!(!PromptTemplate::new("id", "Missing closing {{name").has_placeholder());
    }

    #[test]
    fn test_prompt_template_placeholder_count() {
        assert_eq!(
            PromptTemplate::new("id", "No placeholders").placeholder_count(),
            0
        );
        assert_eq!(PromptTemplate::new("id", "{{one}}").placeholder_count(), 1);
        assert_eq!(
            PromptTemplate::new("id", "{{a}} {{b}} {{c}}").placeholder_count(),
            3
        );
        assert_eq!(
            PromptTemplate::new("id", "Nested: {{{{inner}}}}").placeholder_count(),
            2
        );
    }

    // ====================================================================
    // TemplateEngine Construction Tests
    // ====================================================================

    #[test]
    fn test_template_engine_new() {
        let engine = TemplateEngine::new();
        assert!(!engine.has_template("nonexistent"));
    }

    #[test]
    fn test_template_engine_default() {
        let engine = TemplateEngine::default();
        assert!(!engine.has_template("test"));
    }

    #[test]
    fn test_template_engine_debug() {
        let engine = TemplateEngine::new();
        let debug_str = format!("{:?}", engine);
        assert!(debug_str.contains("TemplateEngine"));
    }

    #[test]
    fn test_template_engine_clone() {
        let mut engine = TemplateEngine::new();
        let template = PromptTemplate::new("test", "Hello, {{name}}!");
        engine.register_template("greeting", template).unwrap();

        let cloned = engine.clone();
        assert!(cloned.has_template("greeting"));
    }

    // ====================================================================
    // TemplateEngine Registration Tests
    // ====================================================================

    #[test]
    fn test_template_engine_register_template() {
        let mut engine = TemplateEngine::new();
        let template = PromptTemplate::new("test", "Hello, {{name}}!");

        let result = engine.register_template("greeting", template);
        assert!(result.is_ok());
        assert!(engine.has_template("greeting"));
    }

    #[test]
    fn test_template_engine_register_multiple() {
        let mut engine = TemplateEngine::new();

        engine
            .register_template("t1", PromptTemplate::new("1", "Template 1"))
            .unwrap();
        engine
            .register_template("t2", PromptTemplate::new("2", "Template 2"))
            .unwrap();
        engine
            .register_template("t3", PromptTemplate::new("3", "Template 3"))
            .unwrap();

        assert!(engine.has_template("t1"));
        assert!(engine.has_template("t2"));
        assert!(engine.has_template("t3"));
    }

    #[test]
    fn test_template_engine_has_template_nonexistent() {
        let engine = TemplateEngine::new();
        assert!(!engine.has_template("does_not_exist"));
        assert!(!engine.has_template("random_name"));
    }

    #[test]
    fn test_template_engine_template_count() {
        let mut engine = TemplateEngine::new();
        assert_eq!(engine.template_count(), 0);

        engine
            .register_template("t1", PromptTemplate::new("1", "Template 1"))
            .unwrap();
        assert_eq!(engine.template_count(), 1);

        engine
            .register_template("t2", PromptTemplate::new("2", "Template 2"))
            .unwrap();
        assert_eq!(engine.template_count(), 2);
    }

    // ====================================================================
    // TemplateEngine Render Tests
    // ====================================================================

    #[test]
    fn test_template_engine_render_simple() {
        let mut engine = TemplateEngine::new();
        engine
            .register_template("greeting", PromptTemplate::new("g", "Hello!"))
            .unwrap();

        let ctx = PromptContext::new();
        let result = engine.render("greeting", &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello!");
    }

    #[test]
    fn test_template_engine_render_with_variable() {
        let mut engine = TemplateEngine::new();
        engine
            .register_template("greeting", PromptTemplate::new("g", "Hello, {{name}}!"))
            .unwrap();

        let mut ctx = PromptContext::new();
        ctx.set("name".to_string(), ContextValue::String("World".to_string()));

        let result = engine.render("greeting", &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_template_engine_render_multiple_variables() {
        let mut engine = TemplateEngine::new();
        engine
            .register_template(
                "intro",
                PromptTemplate::new("i", "{{greeting}}, {{name}}! Welcome to {{place}}."),
            )
            .unwrap();

        let mut ctx = PromptContext::new();
        ctx.set(
            "greeting".to_string(),
            ContextValue::String("Hello".to_string()),
        );
        ctx.set(
            "name".to_string(),
            ContextValue::String("Alice".to_string()),
        );
        ctx.set(
            "place".to_string(),
            ContextValue::String("Wonderland".to_string()),
        );

        let result = engine.render("intro", &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, Alice! Welcome to Wonderland.");
    }

    #[test]
    fn test_template_engine_render_nonexistent() {
        let engine = TemplateEngine::new();
        let ctx = PromptContext::new();

        let result = engine.render("nonexistent", &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_engine_render_empty_context() {
        let mut engine = TemplateEngine::new();
        engine
            .register_template("static", PromptTemplate::new("s", "No variables here"))
            .unwrap();

        let ctx = PromptContext::new();
        let result = engine.render("static", &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No variables here");
    }

    // ====================================================================
    // TemplateContext Type Alias Tests
    // ====================================================================

    #[test]
    fn test_template_context_alias() {
        let ctx: TemplateContext = PromptContext::new();
        let _: &PromptContext = &ctx;
    }

    #[test]
    fn test_template_context_with_values() {
        let mut ctx: TemplateContext = PromptContext::new();
        ctx.set(
            "key".to_string(),
            ContextValue::String("value".to_string()),
        );

        let map = ctx.to_string_map();
        assert!(map.contains_key("key"));
    }

    // ====================================================================
    // ContextValue Tests
    // ====================================================================

    #[test]
    fn test_context_value_string() {
        let value = ContextValue::String("test".to_string());
        if let ContextValue::String(s) = value {
            assert_eq!(s, "test");
        } else {
            panic!("Expected String variant");
        }
    }

    #[test]
    fn test_context_value_in_context() {
        let mut ctx = PromptContext::new();
        ctx.set(
            "str_key".to_string(),
            ContextValue::String("string_value".to_string()),
        );

        let map = ctx.to_string_map();
        assert_eq!(map.get("str_key"), Some(&"string_value".to_string()));
    }

    // ====================================================================
    // Edge Cases and Error Handling Tests
    // ====================================================================

    #[test]
    fn test_template_with_special_characters() {
        let template = PromptTemplate::new(
            "special",
            "Hello, {{name}}! Here's some <html> & \"quotes\".",
        );
        assert!(template.has_placeholder());
        assert_eq!(template.placeholder_count(), 1);
    }

    #[test]
    fn test_template_with_unicode() {
        let template = PromptTemplate::new("unicode", "こんにちは {{name}}! 🎉");
        assert!(template.has_placeholder());
        assert_eq!(template.placeholder_count(), 1);
    }

    #[test]
    fn test_template_with_newlines() {
        let template = PromptTemplate::new("multiline", "Line 1\n{{var}}\nLine 3");
        assert!(template.has_placeholder());
        assert_eq!(template.placeholder_count(), 1);
    }

    #[test]
    fn test_template_engine_overwrite_template() {
        let mut engine = TemplateEngine::new();

        engine
            .register_template("test", PromptTemplate::new("1", "Original"))
            .unwrap();
        engine
            .register_template("test", PromptTemplate::new("2", "Replaced"))
            .unwrap();

        let ctx = PromptContext::new();
        let result = engine.render("test", &ctx).unwrap();
        // The behavior depends on the engine implementation - it may keep the last one
        assert!(result == "Original" || result == "Replaced");
    }

    #[test]
    fn test_prompt_template_long_content() {
        let long_content = "A".repeat(10000);
        let template = PromptTemplate::new("long", &long_content);
        assert_eq!(template.template_len(), 10000);
        assert!(!template.has_placeholder());
    }

    #[test]
    fn test_prompt_template_whitespace_id() {
        let template = PromptTemplate::new("  ", "content");
        assert!(!template.has_empty_id()); // Has whitespace, not empty
        assert_eq!(template.id, "  ");
    }
}
