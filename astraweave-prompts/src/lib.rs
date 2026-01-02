/*!
# AstraWeave Prompt Templates

Advanced prompt templating and engineering for AI-native gaming. This crate provides:

- **Template Engine**: Handlebars-based templating with game-specific helpers
- **Persona Integration**: Automatic persona-specific prompt generation
- **Few-Shot Examples**: Dynamic example selection and management
- **Prompt Optimization**: Automatic prompt tuning and A/B testing
- **Template Library**: Hot-reloadable template collections

## Quick Start

```rust
use astraweave_prompts::{PromptTemplate, TemplateEngine, PromptContext};

fn main() -> anyhow::Result<()> {
    // Create template engine
    let mut engine = TemplateEngine::new();

    // Register a template
    let template = PromptTemplate::new(
        "dialogue",
        "You are {{character_name}}, a {{character_role}}. \
         Your personality is {{character_personality}}. \
         Respond to: {{user_input}}"
    );

    engine.register_template("dialogue", template)?;

    // Create context with variables
    let mut context = PromptContext::new();
    context.set("character_name".to_string(), "Elena".into());
    context.set("character_role".to_string(), "wise mage".into());
    context.set("character_personality".to_string(), "mysterious and helpful".into());
    context.set("user_input".to_string(), "What magic can you teach me?".into());

    // Render the prompt
    let prompt = engine.render("dialogue", &context)?;
    println!("Generated prompt: {}", prompt);

    Ok(())
}
```
*/

pub mod context;
pub mod engine;
pub mod helpers;
pub mod library;
pub mod loader;
pub mod optimization;
pub mod sanitize;
pub mod template;
pub mod terrain_prompts; // Phase 10: AI-orchestrated terrain generation

pub use context::*;
pub use engine::*;
pub use helpers::*;
pub use library::*;
pub use loader::*;
pub use optimization::*;
pub use sanitize::*;
pub use template::*;
pub use terrain_prompts::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the prompt templating system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsConfig {
    /// Directory containing template files
    pub templates_dir: String,

    /// Whether to enable hot reloading of templates
    pub hot_reload: bool,

    /// Default template format
    pub default_format: TemplateFormat,

    /// Maximum template size in bytes
    pub max_template_size: usize,

    /// Cache configuration
    pub cache_config: CacheConfig,

    /// Validation settings
    pub validation: ValidationConfig,
}

impl Default for PromptsConfig {
    fn default() -> Self {
        Self {
            templates_dir: "templates".to_string(),
            hot_reload: true,
            default_format: TemplateFormat::Handlebars,
            max_template_size: 1024 * 1024, // 1MB
            cache_config: CacheConfig::default(),
            validation: ValidationConfig::default(),
        }
    }
}

/// Supported template formats
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TemplateFormat {
    /// Handlebars templating
    Handlebars,
    /// Simple string interpolation
    Simple,
    /// Jinja2-style templating
    Jinja2,
}

/// Cache configuration for templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable template caching
    pub enabled: bool,

    /// Maximum cached templates
    pub max_templates: usize,

    /// Cache TTL in seconds
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_templates: 1000,
            ttl_seconds: 3600, // 1 hour
        }
    }
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enable template validation
    pub enabled: bool,

    /// Require all variables to be defined
    pub strict_variables: bool,

    /// Maximum recursion depth
    pub max_recursion_depth: usize,

    /// Schema validation for contexts
    pub schema_validation: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strict_variables: false,
            max_recursion_depth: 10,
            schema_validation: false,
        }
    }
}

/// Template metadata for categorization and management
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateMetadata {
    /// Template name/identifier
    pub name: String,

    /// Human-readable description
    #[serde(default)]
    pub description: String,

    /// Template category
    #[serde(default)]
    pub category: TemplateCategory,

    /// Author information
    #[serde(default)]
    pub author: Option<String>,

    /// Template version
    #[serde(default = "default_version")]
    pub version: String,

    /// Creation timestamp
    #[serde(default = "current_timestamp")]
    pub created_at: u64,

    /// Last modified timestamp
    #[serde(default = "current_timestamp")]
    pub updated_at: u64,

    /// Tags for searching and filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Required variables
    #[serde(default)]
    pub required_variables: Vec<String>,

    /// Optional variables with defaults
    #[serde(default)]
    pub optional_variables: HashMap<String, serde_json::Value>,

    /// Template usage statistics
    #[serde(default)]
    pub usage_stats: UsageStats,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Categories for organizing templates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
pub enum TemplateCategory {
    /// Character dialogue templates
    Dialogue,

    /// NPC behavior and actions
    Behavior,

    /// Quest and story generation
    Narrative,

    /// Combat and tactics
    Combat,

    /// System instructions
    System,

    /// General conversation
    Conversation,

    /// World building and descriptions
    WorldBuilding,

    /// Terrain generation and modification (Phase 10)
    TerrainGeneration,

    /// Custom category
    #[default]
    Custom,
}

/// Usage statistics for templates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total times this template was used
    pub usage_count: u64,

    /// Average render time in milliseconds
    pub avg_render_time_ms: f32,

    /// Success rate (successful renders / total attempts)
    pub success_rate: f32,

    /// Last used timestamp
    pub last_used: Option<u64>,

    /// Performance score (0.0 to 1.0)
    pub performance_score: f32,
}

/// Template rendering performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderMetrics {
    /// Total templates rendered
    pub total_renders: u64,

    /// Successful renders
    pub successful_renders: u64,

    /// Failed renders
    pub failed_renders: u64,

    /// Average render time
    pub avg_render_time_ms: f32,

    /// Cache hit rate
    pub cache_hit_rate: f32,

    /// Total rendering time
    pub total_render_time_ms: u64,
}

/// Error types for template operations
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {name}")]
    TemplateNotFound { name: String },

    #[error("Template syntax error: {message}")]
    SyntaxError { message: String },

    #[error("Variable not defined: {variable}")]
    UndefinedVariable { variable: String },

    #[error("Circular template dependency: {templates:?}")]
    CircularDependency { templates: Vec<String> },

    #[error("Template too large: {size} bytes > {max} bytes")]
    TemplateTooLarge { size: usize, max: usize },

    #[error("Invalid template format: {format:?}")]
    InvalidFormat { format: TemplateFormat },

    #[error("Validation failed: {message}")]
    ValidationFailed { message: String },
}

/// Get current Unix timestamp in seconds
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_config() {
        let config = PromptsConfig::default();
        assert_eq!(config.templates_dir, "templates");
        assert!(config.hot_reload);
        assert!(config.cache_config.enabled);
    }

    #[test]
    fn test_template_metadata() {
        let metadata = TemplateMetadata {
            name: "test_template".to_string(),
            version: "1.0.0".to_string(),
            description: "A test template".to_string(),
            category: TemplateCategory::Dialogue,
            author: Some("Test Author".to_string()),
            created_at: 0,
            updated_at: 0,
            tags: vec![],
            required_variables: vec!["character_name".to_string()],
            optional_variables: std::collections::HashMap::new(),
            usage_stats: UsageStats::default(),
        };

        assert_eq!(metadata.name, "test_template");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.required_variables.len(), 1);
    }

    #[test]
    fn test_template_categories() {
        let categories = vec![
            TemplateCategory::Dialogue,
            TemplateCategory::Behavior,
            TemplateCategory::Narrative,
            TemplateCategory::Combat,
            TemplateCategory::System,
        ];

        assert_eq!(categories.len(), 5);
    }

    #[test]
    fn test_render_metrics() {
        let mut metrics = RenderMetrics::default();
        assert_eq!(metrics.total_renders, 0);
        assert_eq!(metrics.successful_renders, 0);

        metrics.total_renders = 100;
        metrics.successful_renders = 95;
        metrics.failed_renders = 5;

        let success_rate = metrics.successful_renders as f32 / metrics.total_renders as f32;
        assert_eq!(success_rate, 0.95);
    }
}
