//! Template management and processing functionality
//!
//! This module provides template management capabilities.

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
        }
    }

    /// Render the template using a PromptContext (compat alias: TemplateContext)
    /// Returns the rendered string or an anyhow::Error
    pub fn render(&self, ctx: &crate::context::PromptContext) -> anyhow::Result<String> {
        // Convert context to simple string map and use the processor for rendering
        let vars = ctx.to_string_map();
        let processor = TemplateProcessor::new(ProcessorConfig::default());
        let rendered = processor.process(&self.template, &vars)?;
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

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template name
    pub name: String,
    /// Template version
    pub version: String,
    /// Template description
    pub description: String,
    /// Template variables
    pub variables: Vec<String>,
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

    /// Process a template with variables
    pub fn process(&self, template: &str, variables: &HashMap<String, String>) -> Result<String> {
        // Simple variable substitution for now
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Validate template syntax
    pub fn validate_template(&self, template: &str) -> Result<()> {
        // Basic validation - check for balanced braces
        let mut brace_count = 0;
        for char in template.chars() {
            match char {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }
            if brace_count < 0 {
                anyhow::bail!("Unbalanced braces in template");
            }
        }

        if brace_count != 0 {
            anyhow::bail!("Unbalanced braces in template");
        }

        Ok(())
    }

    /// Extract variables from template
    pub fn extract_variables(&self, template: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = template.chars().peekable();

        while let Some(char) = chars.next() {
            if char == '{' {
                if chars.peek() == Some(&'{') {
                    // Skip double braces
                    chars.next();
                    continue;
                }

                let mut var_name = String::new();
                while let Some(char) = chars.next() {
                    if char == '}' {
                        break;
                    }
                    var_name.push(char);
                }

                if !var_name.is_empty() && !variables.contains(&var_name) {
                    variables.push(var_name);
                }
            }
        }

        variables
    }
}
