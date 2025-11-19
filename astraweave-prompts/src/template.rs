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
        let mut reg = handlebars::Handlebars::new();
        reg.set_strict_mode(self.config.validate_variables);
        
        // Handlebars renders {{var}}. 
        // If the input template uses {var} (single brace), we might need to convert or support both?
        // The lib.rs example uses {{var}}.
        // The tests used {var}.
        
        // Let's assume we are moving to Handlebars standard {{var}}.
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
