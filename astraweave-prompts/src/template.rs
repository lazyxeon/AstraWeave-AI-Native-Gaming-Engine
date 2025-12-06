//! Template management and processing functionality
//!
//! This module provides template management capabilities.

use crate::TemplateMetadata;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Public PromptTemplate used across the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub template: String,
    #[serde(default)]
    pub variables: Vec<String>,
    #[serde(default)]
    pub metadata: Option<TemplateMetadata>,
}

impl PromptTemplate {
    pub fn new<S1: Into<String>, S2: Into<String>>(id: S1, template: S2) -> Self {
        let template_s = template.into();
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let vars = processor.extract_variables(&template_s);
        Self {
            id: id.into(),
            template: template_s,
            variables: vars,
            metadata: None,
        }
    }

    /// Render the template using a PromptContext (compat alias: TemplateContext)
    /// Returns the rendered string or an anyhow::Error
    pub fn render(&self, ctx: &crate::context::PromptContext) -> anyhow::Result<String> {
        // Use to_json to preserve structure (objects, arrays)
        let vars = ctx.to_json();
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let rendered = processor.process_json(&self.template, &vars)?;
        Ok(rendered)
    }

    /// Backward-compatible render method accepting a simple HashMap
    pub fn render_map(
        &self,
        ctx: &std::collections::HashMap<String, String>,
    ) -> anyhow::Result<String> {
        // Convert HashMap into PromptContext and reuse render
        let mut pc = crate::context::PromptContext::new();
        for (k, v) in ctx.iter() {
            pc.set(k.clone(), v.clone().into());
        }
        self.render(&pc)
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn template(&self) -> &str {
        &self.template
    }
}

/// Template processor
#[derive(Debug, Clone)]
pub struct TemplateProcessor {
    /// Processing configuration
    #[allow(dead_code)]
    config: ProcessorConfig,
}

/// Processor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// Enable variable validation
    pub validate_variables: bool,
    /// Maximum processing time in milliseconds
    pub max_processing_time: u64,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            validate_variables: true,
            max_processing_time: 5000,
        }
    }
}

impl TemplateProcessor {
    /// Create a new template processor
    pub fn new(config: ProcessorConfig) -> Self {
        Self { config }
    }

    /// Process a template with variables (string map)
    pub fn process(&self, template: &str, variables: &HashMap<String, String>) -> Result<String> {
        let mut reg = handlebars::Handlebars::new();
        reg.set_strict_mode(self.config.validate_variables);
        Ok(reg.render_template(template, variables)?)
    }

    /// Process a template with JSON variables (preserves structure)
    pub fn process_json(&self, template: &str, variables: &serde_json::Value) -> Result<String> {
        let mut reg = handlebars::Handlebars::new();
        reg.set_strict_mode(self.config.validate_variables);
        Ok(reg.render_template(template, variables)?)
    }

    /// Validate template syntax
    pub fn validate_template(&self, template: &str) -> Result<()> {
        let mut reg = handlebars::Handlebars::new();
        reg.register_template_string("temp", template)?;
        Ok(())
    }

    /// Extract variables from template
    pub fn extract_variables(&self, template: &str) -> Vec<String> {
        // Simple regex extraction for Handlebars variables {{var}}
        // This is an approximation.
        let re = regex::Regex::new(r"\{\{\s*([a-zA-Z0-9_.]+)\s*\}\}").unwrap();
        let mut variables = Vec::new();
        
        for cap in re.captures_iter(template) {
            if let Some(m) = cap.get(1) {
                let var = m.as_str().to_string();
                if !variables.contains(&var) {
                    variables.push(var);
                }
            }
        }
        
        variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ContextValue, PromptContext};

    // PromptTemplate tests
    #[test]
    fn test_prompt_template_new() {
        let template = PromptTemplate::new("test_id", "Hello {{name}}!");
        
        assert_eq!(template.id(), "test_id");
        assert_eq!(template.template(), "Hello {{name}}!");
        assert!(template.variables.contains(&"name".to_string()));
    }

    #[test]
    fn test_prompt_template_render() {
        let template = PromptTemplate::new("greeting", "Hello {{name}}!");
        let mut ctx = PromptContext::new();
        ctx.set("name".to_string(), ContextValue::String("Alice".to_string()));
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_prompt_template_render_multiple_variables() {
        let template = PromptTemplate::new("intro", "{{greeting}}, {{name}}! Welcome to {{place}}.");
        let mut ctx = PromptContext::new();
        ctx.set("greeting".to_string(), ContextValue::String("Hello".to_string()));
        ctx.set("name".to_string(), ContextValue::String("Bob".to_string()));
        ctx.set("place".to_string(), ContextValue::String("Wonderland".to_string()));
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "Hello, Bob! Welcome to Wonderland.");
    }

    #[test]
    fn test_prompt_template_render_map() {
        let template = PromptTemplate::new("simple", "Value: {{value}}");
        let mut map = HashMap::new();
        map.insert("value".to_string(), "42".to_string());
        
        let result = template.render_map(&map).unwrap();
        assert_eq!(result, "Value: 42");
    }

    #[test]
    fn test_prompt_template_render_with_object() {
        let template = PromptTemplate::new("nested", "User: {{user.name}}");
        let mut ctx = PromptContext::new();
        ctx.set_path("user.name", ContextValue::String("Charlie".to_string()));
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "User: Charlie");
    }

    #[test]
    fn test_prompt_template_serialization() {
        let template = PromptTemplate::new("test", "Hello {{name}}!");
        
        let serialized = serde_json::to_string(&template).unwrap();
        let deserialized: PromptTemplate = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(template.id, deserialized.id);
        assert_eq!(template.template, deserialized.template);
    }

    // TemplateProcessor tests
    #[test]
    fn test_processor_new() {
        let config = ProcessorConfig::default();
        let processor = TemplateProcessor::new(config);
        
        // Just verify it creates successfully
        assert!(processor.config.validate_variables);
    }

    #[test]
    fn test_processor_process() {
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        
        let result = processor.process("Hello {{name}}!", &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_processor_process_json() {
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let vars = serde_json::json!({
            "name": "Test",
            "count": 5
        });
        
        let result = processor.process_json("Name: {{name}}, Count: {{count}}", &vars).unwrap();
        assert_eq!(result, "Name: Test, Count: 5");
    }

    #[test]
    fn test_processor_validate_template_valid() {
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let result = processor.validate_template("Hello {{name}}!");
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_processor_validate_template_invalid() {
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let result = processor.validate_template("Hello {{#if}}!");
        
        // Should fail - incomplete if block
        assert!(result.is_err());
    }

    #[test]
    fn test_processor_extract_variables() {
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let vars = processor.extract_variables("Hello {{name}}! Your {{item}} is {{status}}.");
        
        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"item".to_string()));
        assert!(vars.contains(&"status".to_string()));
    }

    #[test]
    fn test_processor_extract_variables_dotted() {
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let vars = processor.extract_variables("User: {{user.name}}, ID: {{user.id}}");
        
        assert!(vars.contains(&"user.name".to_string()));
        assert!(vars.contains(&"user.id".to_string()));
    }

    #[test]
    fn test_processor_extract_variables_no_duplicates() {
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let vars = processor.extract_variables("{{name}} meets {{name}}");
        
        assert_eq!(vars.len(), 1);
    }

    #[test]
    fn test_processor_extract_variables_empty() {
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let vars = processor.extract_variables("No variables here!");
        
        assert!(vars.is_empty());
    }

    // ProcessorConfig tests
    #[test]
    fn test_processor_config_default() {
        let config = ProcessorConfig::default();
        
        assert!(config.validate_variables);
        assert_eq!(config.max_processing_time, 5000);
    }

    #[test]
    fn test_processor_config_serialization() {
        let config = ProcessorConfig {
            validate_variables: false,
            max_processing_time: 1000,
        };
        
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: ProcessorConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.validate_variables, deserialized.validate_variables);
        assert_eq!(config.max_processing_time, deserialized.max_processing_time);
    }

    #[test]
    fn test_processor_strict_mode_on() {
        let config = ProcessorConfig {
            validate_variables: true,
            ..Default::default()
        };
        let processor = TemplateProcessor::new(config);
        let vars = HashMap::new(); // Empty - missing variables
        
        // With strict mode ON, missing variables should fail
        let result = processor.process("Hello {{name}}!", &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_processor_strict_mode_off() {
        let config = ProcessorConfig {
            validate_variables: false,
            ..Default::default()
        };
        let processor = TemplateProcessor::new(config);
        let vars = HashMap::new(); // Empty - missing variables
        
        // With strict mode OFF, missing variables become empty
        let result = processor.process("Hello {{name}}!", &vars);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello !");
    }

    // Edge cases
    #[test]
    fn test_template_with_whitespace_in_variables() {
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let vars = processor.extract_variables("{{ name }} and {{  value  }}");
        
        // Should handle whitespace inside braces
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"value".to_string()));
    }

    #[test]
    fn test_template_empty() {
        let template = PromptTemplate::new("empty", "");
        let ctx = PromptContext::new();
        
        let result = template.render(&ctx).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_template_no_variables() {
        let template = PromptTemplate::new("static", "This is static text.");
        let ctx = PromptContext::new();
        
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "This is static text.");
    }

    #[test]
    fn test_template_clone() {
        let template = PromptTemplate::new("original", "Hello {{name}}!");
        let cloned = template.clone();
        
        assert_eq!(template.id, cloned.id);
        assert_eq!(template.template, cloned.template);
    }
}
