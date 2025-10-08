use crate::template::TemplateProcessor;
use crate::engine::PromptEngine;
use crate::context::{PromptContext, ContextValue};
use anyhow::Result;

/// Lightweight PromptTemplate expected by consumers
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub id: String,
    pub template: String,
}

impl PromptTemplate {
    pub fn new<S: Into<String>>(id: S, template: S) -> Self {
        Self { id: id.into(), template: template.into() }
    }
}

/// TemplateEngine wrapper that provides a minimal API expected by consumers
#[derive(Debug, Clone)]
pub struct TemplateEngine {
    engine: PromptEngine,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self { engine: PromptEngine::new(crate::engine::EngineConfig::default()) }
    }

    pub fn register_template(&mut self, name: &str, template: PromptTemplate) -> Result<()> {
        self.engine.register_template(name.to_string(), template.template)
    }

    pub fn render(&self, name: &str, ctx: &PromptContext) -> Result<String> {
        if let Some(t) = self.engine.get_template(name) {
            let vars = ctx.to_string_map();
            // Use TemplateProcessor for simple rendering
            let proc = TemplateProcessor::new(crate::template::ProcessorConfig::default());
            proc.process(t, &vars)
        } else {
            anyhow::bail!("template not found")
        }
    }
}

/// TemplateContext is an alias to PromptContext for compatibility
pub type TemplateContext = PromptContext;

// ContextValue is available via astraweave_prompts::ContextValue if needed
