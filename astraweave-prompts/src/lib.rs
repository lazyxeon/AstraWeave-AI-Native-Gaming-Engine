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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateFormat {
    /// Handlebars templating
    Handlebars,
    /// Simple string interpolation
    Simple,
    /// Jinja2-style templating
    Jinja2,
}

impl TemplateFormat {
    /// Returns all template format variants.
    pub fn all() -> &'static [TemplateFormat] {
        &[
            TemplateFormat::Handlebars,
            TemplateFormat::Simple,
            TemplateFormat::Jinja2,
        ]
    }

    /// Returns the display name for this format.
    pub fn name(self) -> &'static str {
        match self {
            TemplateFormat::Handlebars => "Handlebars",
            TemplateFormat::Simple => "Simple",
            TemplateFormat::Jinja2 => "Jinja2",
        }
    }

    /// Returns the file extension commonly used for this format.
    pub fn extension(self) -> &'static str {
        match self {
            TemplateFormat::Handlebars => ".hbs",
            TemplateFormat::Simple => ".txt",
            TemplateFormat::Jinja2 => ".j2",
        }
    }

    /// Returns an icon character for this format.
    pub fn icon(self) -> char {
        match self {
            TemplateFormat::Handlebars => '🔧',
            TemplateFormat::Simple => '📝',
            TemplateFormat::Jinja2 => '🐍',
        }
    }

    /// Returns a description for this format.
    pub fn description(self) -> &'static str {
        match self {
            TemplateFormat::Handlebars => "Mustache-style templating with helpers and partials",
            TemplateFormat::Simple => "Basic string interpolation with {{variable}} syntax",
            TemplateFormat::Jinja2 => "Python-style templating with filters and blocks",
        }
    }

    /// Returns true if this is the Handlebars format.
    pub fn is_handlebars(self) -> bool {
        matches!(self, TemplateFormat::Handlebars)
    }

    /// Returns true if this is the Simple format.
    pub fn is_simple(self) -> bool {
        matches!(self, TemplateFormat::Simple)
    }

    /// Returns true if this is the Jinja2 format.
    pub fn is_jinja2(self) -> bool {
        matches!(self, TemplateFormat::Jinja2)
    }

    /// Returns true if this format supports helpers/filters.
    pub fn supports_helpers(self) -> bool {
        matches!(self, TemplateFormat::Handlebars | TemplateFormat::Jinja2)
    }

    /// Returns true if this format supports partials/includes.
    pub fn supports_partials(self) -> bool {
        matches!(self, TemplateFormat::Handlebars | TemplateFormat::Jinja2)
    }
}

impl std::fmt::Display for TemplateFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
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

impl CacheConfig {
    /// Creates a new cache configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a disabled cache configuration.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            max_templates: 0,
            ttl_seconds: 0,
        }
    }

    /// Returns true if caching is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the TTL in human-readable format.
    pub fn ttl_display(&self) -> String {
        if self.ttl_seconds >= 3600 {
            format!("{:.1}h", self.ttl_seconds as f64 / 3600.0)
        } else if self.ttl_seconds >= 60 {
            format!("{:.1}m", self.ttl_seconds as f64 / 60.0)
        } else {
            format!("{}s", self.ttl_seconds)
        }
    }

    /// Returns a summary of the cache configuration.
    pub fn summary(&self) -> String {
        if self.enabled {
            format!(
                "Enabled: {} templates, TTL: {}",
                self.max_templates,
                self.ttl_display()
            )
        } else {
            "Disabled".to_string()
        }
    }

    /// Returns true if the configuration is valid.
    pub fn is_valid(&self) -> bool {
        if self.enabled {
            self.max_templates > 0 && self.ttl_seconds > 0
        } else {
            true
        }
    }
}

impl std::fmt::Display for CacheConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
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

impl ValidationConfig {
    /// Creates a new validation configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a strict validation configuration.
    pub fn strict() -> Self {
        Self {
            enabled: true,
            strict_variables: true,
            max_recursion_depth: 10,
            schema_validation: true,
        }
    }

    /// Creates a permissive (disabled) validation configuration.
    pub fn permissive() -> Self {
        Self {
            enabled: false,
            strict_variables: false,
            max_recursion_depth: 100,
            schema_validation: false,
        }
    }

    /// Returns true if validation is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns true if strict variable checking is enabled.
    pub fn is_strict(&self) -> bool {
        self.strict_variables
    }

    /// Returns true if schema validation is enabled.
    pub fn has_schema_validation(&self) -> bool {
        self.schema_validation
    }

    /// Returns a summary of the validation configuration.
    pub fn summary(&self) -> String {
        if !self.enabled {
            return "Disabled".to_string();
        }

        let depth_str = format!("depth: {}", self.max_recursion_depth);
        let mut features: Vec<&str> = Vec::new();
        if self.strict_variables {
            features.push("strict vars");
        }
        if self.schema_validation {
            features.push("schema");
        }
        features.push(&depth_str);

        format!("Enabled ({})", features.join(", "))
    }

    /// Returns the strictness level as a string.
    pub fn strictness_level(&self) -> &'static str {
        match (self.enabled, self.strict_variables, self.schema_validation) {
            (false, _, _) => "None",
            (true, false, false) => "Low",
            (true, true, false) | (true, false, true) => "Medium",
            (true, true, true) => "High",
        }
    }
}

impl std::fmt::Display for ValidationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation: {}", self.strictness_level())
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

impl TemplateMetadata {
    /// Creates new template metadata with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Creates template metadata with name and description.
    pub fn with_description(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            ..Default::default()
        }
    }

    /// Returns true if this template has an author.
    pub fn has_author(&self) -> bool {
        self.author.is_some()
    }

    /// Returns true if this template has tags.
    pub fn has_tags(&self) -> bool {
        !self.tags.is_empty()
    }

    /// Returns true if this template has required variables.
    pub fn has_required_variables(&self) -> bool {
        !self.required_variables.is_empty()
    }

    /// Returns true if this template has optional variables.
    pub fn has_optional_variables(&self) -> bool {
        !self.optional_variables.is_empty()
    }

    /// Returns the total number of variables (required + optional).
    pub fn total_variables(&self) -> usize {
        self.required_variables.len() + self.optional_variables.len()
    }

    /// Returns true if the given variable is required.
    pub fn is_required_variable(&self, name: &str) -> bool {
        self.required_variables.iter().any(|v| v == name)
    }

    /// Returns true if the given variable is optional.
    pub fn is_optional_variable(&self, name: &str) -> bool {
        self.optional_variables.contains_key(name)
    }

    /// Returns the default value for an optional variable.
    pub fn get_default(&self, name: &str) -> Option<&serde_json::Value> {
        self.optional_variables.get(name)
    }

    /// Returns true if the template has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
    }

    /// Returns the age of this template in seconds since creation.
    pub fn age_seconds(&self) -> u64 {
        current_timestamp().saturating_sub(self.created_at)
    }

    /// Returns the age of this template in human-readable format.
    pub fn age_display(&self) -> String {
        let secs = self.age_seconds();
        if secs < 60 {
            format!("{}s ago", secs)
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    }

    /// Returns true if this template was recently updated (within 24 hours).
    pub fn is_recently_updated(&self) -> bool {
        current_timestamp().saturating_sub(self.updated_at) < 86400
    }

    /// Returns a summary of the template metadata.
    pub fn summary(&self) -> String {
        let vars = if self.required_variables.is_empty() {
            "no required vars".to_string()
        } else {
            format!("{} required vars", self.required_variables.len())
        };

        format!(
            "{} ({}) - {} [v{}]",
            self.name,
            self.category.name(),
            vars,
            self.version
        )
    }

    /// Marks the template as updated with the current timestamp.
    pub fn touch(&mut self) {
        self.updated_at = current_timestamp();
    }
}

impl std::fmt::Display for TemplateMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
    }
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Categories for organizing templates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
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

impl TemplateCategory {
    /// Returns all template category variants.
    pub fn all() -> &'static [TemplateCategory] {
        &[
            TemplateCategory::Dialogue,
            TemplateCategory::Behavior,
            TemplateCategory::Narrative,
            TemplateCategory::Combat,
            TemplateCategory::System,
            TemplateCategory::Conversation,
            TemplateCategory::WorldBuilding,
            TemplateCategory::TerrainGeneration,
            TemplateCategory::Custom,
        ]
    }

    /// Returns the display name for this category.
    pub fn name(self) -> &'static str {
        match self {
            TemplateCategory::Dialogue => "Dialogue",
            TemplateCategory::Behavior => "Behavior",
            TemplateCategory::Narrative => "Narrative",
            TemplateCategory::Combat => "Combat",
            TemplateCategory::System => "System",
            TemplateCategory::Conversation => "Conversation",
            TemplateCategory::WorldBuilding => "World Building",
            TemplateCategory::TerrainGeneration => "Terrain Generation",
            TemplateCategory::Custom => "Custom",
        }
    }

    /// Returns an icon string for this category.
    pub fn icon(self) -> &'static str {
        match self {
            TemplateCategory::Dialogue => "💬",
            TemplateCategory::Behavior => "🤖",
            TemplateCategory::Narrative => "📖",
            TemplateCategory::Combat => "⚔",
            TemplateCategory::System => "⚙",
            TemplateCategory::Conversation => "🗣",
            TemplateCategory::WorldBuilding => "🌍",
            TemplateCategory::TerrainGeneration => "🏔",
            TemplateCategory::Custom => "🔧",
        }
    }

    /// Returns a description for this category.
    pub fn description(self) -> &'static str {
        match self {
            TemplateCategory::Dialogue => "Character dialogue and NPC conversations",
            TemplateCategory::Behavior => "NPC behavior patterns and action sequences",
            TemplateCategory::Narrative => "Quest generation and story progression",
            TemplateCategory::Combat => "Combat tactics and battle descriptions",
            TemplateCategory::System => "System prompts and AI instructions",
            TemplateCategory::Conversation => "General conversational exchanges",
            TemplateCategory::WorldBuilding => "World lore and environment descriptions",
            TemplateCategory::TerrainGeneration => "Procedural terrain and landscape generation",
            TemplateCategory::Custom => "User-defined custom templates",
        }
    }

    /// Returns true if this is a gameplay-related category.
    pub fn is_gameplay(self) -> bool {
        matches!(
            self,
            TemplateCategory::Dialogue
                | TemplateCategory::Behavior
                | TemplateCategory::Combat
                | TemplateCategory::Conversation
        )
    }

    /// Returns true if this is a content generation category.
    pub fn is_content_generation(self) -> bool {
        matches!(
            self,
            TemplateCategory::Narrative
                | TemplateCategory::WorldBuilding
                | TemplateCategory::TerrainGeneration
        )
    }

    /// Returns true if this is a system category.
    pub fn is_system(self) -> bool {
        matches!(self, TemplateCategory::System)
    }

    /// Returns true if this is the custom category.
    pub fn is_custom(self) -> bool {
        matches!(self, TemplateCategory::Custom)
    }

    /// Returns the suggested use case for this category.
    pub fn use_case(self) -> &'static str {
        match self {
            TemplateCategory::Dialogue => "NPC speech, player interactions",
            TemplateCategory::Behavior => "AI decision making, action selection",
            TemplateCategory::Narrative => "Quest text, story beats, objectives",
            TemplateCategory::Combat => "Attack descriptions, combat log",
            TemplateCategory::System => "LLM configuration, response formatting",
            TemplateCategory::Conversation => "Chat, social interactions",
            TemplateCategory::WorldBuilding => "Location descriptions, lore entries",
            TemplateCategory::TerrainGeneration => "Procedural landscapes, biome creation",
            TemplateCategory::Custom => "Specialized use cases",
        }
    }
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
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

impl UsageStats {
    /// Creates new usage statistics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if this template has been used.
    pub fn has_usage(&self) -> bool {
        self.usage_count > 0
    }

    /// Returns true if this template was used recently (within 24 hours).
    pub fn is_recently_used(&self) -> bool {
        if let Some(last) = self.last_used {
            let now = current_timestamp();
            now.saturating_sub(last) < 86400 // 24 hours
        } else {
            false
        }
    }

    /// Returns the usage frequency category.
    pub fn frequency_category(&self) -> &'static str {
        match self.usage_count {
            0 => "Unused",
            1..=10 => "Rare",
            11..=100 => "Occasional",
            101..=1000 => "Frequent",
            _ => "Heavy",
        }
    }

    /// Returns the performance category based on score.
    pub fn performance_category(&self) -> &'static str {
        match (self.performance_score * 100.0) as u32 {
            0..=25 => "Poor",
            26..=50 => "Fair",
            51..=75 => "Good",
            76..=90 => "Excellent",
            _ => "Outstanding",
        }
    }

    /// Returns true if the success rate is acceptable (>= 90%).
    pub fn is_reliable(&self) -> bool {
        self.success_rate >= 0.9
    }

    /// Returns true if the render time is fast (<= 10ms).
    pub fn is_fast(&self) -> bool {
        self.avg_render_time_ms <= 10.0
    }

    /// Returns a summary of the usage statistics.
    pub fn summary(&self) -> String {
        if self.usage_count == 0 {
            return "No usage data".to_string();
        }

        format!(
            "{} uses, {:.1}ms avg, {:.0}% success",
            self.usage_count,
            self.avg_render_time_ms,
            self.success_rate * 100.0
        )
    }

    /// Returns the formatted render time.
    pub fn formatted_render_time(&self) -> String {
        if self.avg_render_time_ms < 1.0 {
            format!("{:.2}ms", self.avg_render_time_ms)
        } else if self.avg_render_time_ms < 1000.0 {
            format!("{:.1}ms", self.avg_render_time_ms)
        } else {
            format!("{:.2}s", self.avg_render_time_ms / 1000.0)
        }
    }
}

impl std::fmt::Display for UsageStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
    }
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

impl RenderMetrics {
    /// Creates new render metrics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if any renders have been performed.
    pub fn has_renders(&self) -> bool {
        self.total_renders > 0
    }

    /// Returns the success rate as a percentage (0.0 to 1.0).
    pub fn success_rate(&self) -> f32 {
        if self.total_renders == 0 {
            return 0.0;
        }
        self.successful_renders as f32 / self.total_renders as f32
    }

    /// Returns the failure rate as a percentage (0.0 to 1.0).
    pub fn failure_rate(&self) -> f32 {
        if self.total_renders == 0 {
            return 0.0;
        }
        self.failed_renders as f32 / self.total_renders as f32
    }

    /// Returns true if the success rate is high (>= 95%).
    pub fn is_healthy(&self) -> bool {
        self.success_rate() >= 0.95
    }

    /// Returns true if the cache is effective (>= 70% hit rate).
    pub fn has_effective_cache(&self) -> bool {
        self.cache_hit_rate >= 0.7
    }

    /// Returns the average render time category.
    pub fn performance_category(&self) -> &'static str {
        match self.avg_render_time_ms as u32 {
            0..=1 => "Instant",
            2..=10 => "Fast",
            11..=50 => "Normal",
            51..=200 => "Slow",
            _ => "Very Slow",
        }
    }

    /// Returns the total render time in human-readable format.
    pub fn formatted_total_time(&self) -> String {
        let ms = self.total_render_time_ms;
        if ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60000 {
            format!("{:.2}s", ms as f64 / 1000.0)
        } else {
            format!("{:.2}m", ms as f64 / 60000.0)
        }
    }

    /// Returns a summary of the metrics.
    pub fn summary(&self) -> String {
        if self.total_renders == 0 {
            return "No renders".to_string();
        }

        format!(
            "{} renders ({:.0}% success), {:.1}ms avg, {:.0}% cache hits",
            self.total_renders,
            self.success_rate() * 100.0,
            self.avg_render_time_ms,
            self.cache_hit_rate * 100.0
        )
    }

    /// Records a successful render.
    pub fn record_success(&mut self, render_time_ms: f32) {
        self.total_renders += 1;
        self.successful_renders += 1;
        self.update_avg_time(render_time_ms);
    }

    /// Records a failed render.
    pub fn record_failure(&mut self) {
        self.total_renders += 1;
        self.failed_renders += 1;
    }

    /// Updates the average render time with a new sample.
    fn update_avg_time(&mut self, render_time_ms: f32) {
        let prev_count = self.successful_renders.saturating_sub(1) as f32;
        let new_count = self.successful_renders as f32;
        if new_count > 0.0 {
            self.avg_render_time_ms =
                (self.avg_render_time_ms * prev_count + render_time_ms) / new_count;
        }
        self.total_render_time_ms += render_time_ms as u64;
    }
}

impl std::fmt::Display for RenderMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
    }
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
        let categories = [
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

    // TemplateFormat tests
    #[test]
    fn test_template_format_all() {
        let all = TemplateFormat::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&TemplateFormat::Handlebars));
        assert!(all.contains(&TemplateFormat::Simple));
        assert!(all.contains(&TemplateFormat::Jinja2));
    }

    #[test]
    fn test_template_format_name() {
        assert_eq!(TemplateFormat::Handlebars.name(), "Handlebars");
        assert_eq!(TemplateFormat::Simple.name(), "Simple");
        assert_eq!(TemplateFormat::Jinja2.name(), "Jinja2");
    }

    #[test]
    fn test_template_format_extension() {
        assert_eq!(TemplateFormat::Handlebars.extension(), ".hbs");
        assert_eq!(TemplateFormat::Simple.extension(), ".txt");
        assert_eq!(TemplateFormat::Jinja2.extension(), ".j2");
    }

    #[test]
    fn test_template_format_icon() {
        assert_eq!(TemplateFormat::Handlebars.icon(), '🔧');
        assert_eq!(TemplateFormat::Simple.icon(), '📝');
        assert_eq!(TemplateFormat::Jinja2.icon(), '🐍');
    }

    #[test]
    fn test_template_format_is_helpers() {
        assert!(TemplateFormat::Handlebars.is_handlebars());
        assert!(!TemplateFormat::Simple.is_handlebars());
        assert!(TemplateFormat::Simple.is_simple());
        assert!(TemplateFormat::Jinja2.is_jinja2());
    }

    #[test]
    fn test_template_format_supports_helpers() {
        assert!(TemplateFormat::Handlebars.supports_helpers());
        assert!(!TemplateFormat::Simple.supports_helpers());
        assert!(TemplateFormat::Jinja2.supports_helpers());
    }

    #[test]
    fn test_template_format_supports_partials() {
        assert!(TemplateFormat::Handlebars.supports_partials());
        assert!(!TemplateFormat::Simple.supports_partials());
        assert!(TemplateFormat::Jinja2.supports_partials());
    }

    #[test]
    fn test_template_format_display() {
        assert_eq!(format!("{}", TemplateFormat::Handlebars), "Handlebars");
        assert_eq!(format!("{}", TemplateFormat::Simple), "Simple");
        assert_eq!(format!("{}", TemplateFormat::Jinja2), "Jinja2");
    }

    // CacheConfig tests
    #[test]
    fn test_cache_config_new() {
        let config = CacheConfig::new();
        assert!(config.enabled);
        assert_eq!(config.max_templates, 1000);
        assert_eq!(config.ttl_seconds, 3600);
    }

    #[test]
    fn test_cache_config_disabled() {
        let config = CacheConfig::disabled();
        assert!(!config.enabled);
        assert_eq!(config.max_templates, 0);
    }

    #[test]
    fn test_cache_config_ttl_display() {
        let config = CacheConfig {
            enabled: true,
            max_templates: 100,
            ttl_seconds: 3600,
        };
        assert_eq!(config.ttl_display(), "1.0h");

        let config2 = CacheConfig {
            enabled: true,
            max_templates: 100,
            ttl_seconds: 120,
        };
        assert_eq!(config2.ttl_display(), "2.0m");

        let config3 = CacheConfig {
            enabled: true,
            max_templates: 100,
            ttl_seconds: 30,
        };
        assert_eq!(config3.ttl_display(), "30s");
    }

    #[test]
    fn test_cache_config_summary() {
        let config = CacheConfig::new();
        assert!(config.summary().contains("Enabled"));

        let disabled = CacheConfig::disabled();
        assert_eq!(disabled.summary(), "Disabled");
    }

    #[test]
    fn test_cache_config_is_valid() {
        let config = CacheConfig::new();
        assert!(config.is_valid());

        let disabled = CacheConfig::disabled();
        assert!(disabled.is_valid());

        let invalid = CacheConfig {
            enabled: true,
            max_templates: 0,
            ttl_seconds: 3600,
        };
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_cache_config_display() {
        let config = CacheConfig::new();
        let display = format!("{}", config);
        assert!(display.contains("Enabled"));
    }

    // ValidationConfig tests
    #[test]
    fn test_validation_config_new() {
        let config = ValidationConfig::new();
        assert!(config.enabled);
        assert!(!config.strict_variables);
    }

    #[test]
    fn test_validation_config_strict() {
        let config = ValidationConfig::strict();
        assert!(config.enabled);
        assert!(config.strict_variables);
        assert!(config.schema_validation);
    }

    #[test]
    fn test_validation_config_permissive() {
        let config = ValidationConfig::permissive();
        assert!(!config.enabled);
    }

    #[test]
    fn test_validation_config_strictness_level() {
        let none = ValidationConfig::permissive();
        assert_eq!(none.strictness_level(), "None");

        let low = ValidationConfig::new();
        assert_eq!(low.strictness_level(), "Low");

        let high = ValidationConfig::strict();
        assert_eq!(high.strictness_level(), "High");
    }

    #[test]
    fn test_validation_config_display() {
        let config = ValidationConfig::new();
        let display = format!("{}", config);
        assert!(display.contains("Validation"));
    }

    // TemplateCategory tests
    #[test]
    fn test_template_category_all() {
        let all = TemplateCategory::all();
        assert_eq!(all.len(), 9);
        assert!(all.contains(&TemplateCategory::Dialogue));
        assert!(all.contains(&TemplateCategory::Custom));
    }

    #[test]
    fn test_template_category_name() {
        assert_eq!(TemplateCategory::Dialogue.name(), "Dialogue");
        assert_eq!(TemplateCategory::WorldBuilding.name(), "World Building");
        assert_eq!(TemplateCategory::TerrainGeneration.name(), "Terrain Generation");
    }

    #[test]
    fn test_template_category_icon() {
        assert_eq!(TemplateCategory::Dialogue.icon(), "💬");
        assert_eq!(TemplateCategory::Combat.icon(), "⚔");
        assert_eq!(TemplateCategory::Custom.icon(), "🔧");
    }

    #[test]
    fn test_template_category_is_gameplay() {
        assert!(TemplateCategory::Dialogue.is_gameplay());
        assert!(TemplateCategory::Combat.is_gameplay());
        assert!(!TemplateCategory::Narrative.is_gameplay());
        assert!(!TemplateCategory::System.is_gameplay());
    }

    #[test]
    fn test_template_category_is_content_generation() {
        assert!(TemplateCategory::Narrative.is_content_generation());
        assert!(TemplateCategory::WorldBuilding.is_content_generation());
        assert!(TemplateCategory::TerrainGeneration.is_content_generation());
        assert!(!TemplateCategory::Dialogue.is_content_generation());
    }

    #[test]
    fn test_template_category_is_system() {
        assert!(TemplateCategory::System.is_system());
        assert!(!TemplateCategory::Dialogue.is_system());
    }

    #[test]
    fn test_template_category_is_custom() {
        assert!(TemplateCategory::Custom.is_custom());
        assert!(!TemplateCategory::Dialogue.is_custom());
    }

    #[test]
    fn test_template_category_display() {
        assert_eq!(format!("{}", TemplateCategory::Dialogue), "Dialogue");
        assert_eq!(format!("{}", TemplateCategory::WorldBuilding), "World Building");
    }

    // TemplateMetadata tests
    #[test]
    fn test_template_metadata_new() {
        let metadata = TemplateMetadata::new("test");
        assert_eq!(metadata.name, "test");
        assert!(metadata.description.is_empty());
    }

    #[test]
    fn test_template_metadata_with_description() {
        let metadata = TemplateMetadata::with_description("test", "A test template");
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.description, "A test template");
    }

    #[test]
    fn test_template_metadata_has_author() {
        let mut metadata = TemplateMetadata::new("test");
        assert!(!metadata.has_author());

        metadata.author = Some("Author".to_string());
        assert!(metadata.has_author());
    }

    #[test]
    fn test_template_metadata_has_tags() {
        let mut metadata = TemplateMetadata::new("test");
        assert!(!metadata.has_tags());

        metadata.tags = vec!["tag1".to_string()];
        assert!(metadata.has_tags());
    }

    #[test]
    fn test_template_metadata_has_required_variables() {
        let mut metadata = TemplateMetadata::new("test");
        assert!(!metadata.has_required_variables());

        metadata.required_variables = vec!["var1".to_string()];
        assert!(metadata.has_required_variables());
    }

    #[test]
    fn test_template_metadata_total_variables() {
        let mut metadata = TemplateMetadata::new("test");
        assert_eq!(metadata.total_variables(), 0);

        metadata.required_variables = vec!["r1".to_string(), "r2".to_string()];
        metadata.optional_variables.insert("o1".to_string(), serde_json::Value::Null);
        assert_eq!(metadata.total_variables(), 3);
    }

    #[test]
    fn test_template_metadata_is_required_variable() {
        let mut metadata = TemplateMetadata::new("test");
        metadata.required_variables = vec!["name".to_string()];

        assert!(metadata.is_required_variable("name"));
        assert!(!metadata.is_required_variable("other"));
    }

    #[test]
    fn test_template_metadata_is_optional_variable() {
        let mut metadata = TemplateMetadata::new("test");
        metadata.optional_variables.insert("color".to_string(), serde_json::json!("red"));

        assert!(metadata.is_optional_variable("color"));
        assert!(!metadata.is_optional_variable("name"));
    }

    #[test]
    fn test_template_metadata_get_default() {
        let mut metadata = TemplateMetadata::new("test");
        metadata.optional_variables.insert("count".to_string(), serde_json::json!(5));

        assert_eq!(metadata.get_default("count"), Some(&serde_json::json!(5)));
        assert_eq!(metadata.get_default("missing"), None);
    }

    #[test]
    fn test_template_metadata_has_tag() {
        let mut metadata = TemplateMetadata::new("test");
        metadata.tags = vec!["combat".to_string(), "NPC".to_string()];

        assert!(metadata.has_tag("combat"));
        assert!(metadata.has_tag("COMBAT")); // Case insensitive
        assert!(metadata.has_tag("npc"));
        assert!(!metadata.has_tag("other"));
    }

    #[test]
    fn test_template_metadata_summary() {
        let metadata = TemplateMetadata::new("test");
        let summary = metadata.summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("Custom")); // Default category
    }

    #[test]
    fn test_template_metadata_display() {
        let metadata = TemplateMetadata::new("greeting");
        let display = format!("{}", metadata);
        assert!(display.contains("greeting"));
    }

    // UsageStats tests
    #[test]
    fn test_usage_stats_new() {
        let stats = UsageStats::new();
        assert_eq!(stats.usage_count, 0);
        assert!(!stats.has_usage());
    }

    #[test]
    fn test_usage_stats_has_usage() {
        let mut stats = UsageStats::new();
        assert!(!stats.has_usage());

        stats.usage_count = 10;
        assert!(stats.has_usage());
    }

    #[test]
    fn test_usage_stats_frequency_category() {
        let mut stats = UsageStats::new();
        assert_eq!(stats.frequency_category(), "Unused");

        stats.usage_count = 5;
        assert_eq!(stats.frequency_category(), "Rare");

        stats.usage_count = 50;
        assert_eq!(stats.frequency_category(), "Occasional");

        stats.usage_count = 500;
        assert_eq!(stats.frequency_category(), "Frequent");

        stats.usage_count = 5000;
        assert_eq!(stats.frequency_category(), "Heavy");
    }

    #[test]
    fn test_usage_stats_performance_category() {
        let mut stats = UsageStats::new();
        assert_eq!(stats.performance_category(), "Poor");

        stats.performance_score = 0.3;
        assert_eq!(stats.performance_category(), "Fair");

        stats.performance_score = 0.6;
        assert_eq!(stats.performance_category(), "Good");

        stats.performance_score = 0.8;
        assert_eq!(stats.performance_category(), "Excellent");

        stats.performance_score = 0.95;
        assert_eq!(stats.performance_category(), "Outstanding");
    }

    #[test]
    fn test_usage_stats_is_reliable() {
        let mut stats = UsageStats::new();
        stats.success_rate = 0.8;
        assert!(!stats.is_reliable());

        stats.success_rate = 0.9;
        assert!(stats.is_reliable());
    }

    #[test]
    fn test_usage_stats_is_fast() {
        let mut stats = UsageStats::new();
        stats.avg_render_time_ms = 5.0;
        assert!(stats.is_fast());

        stats.avg_render_time_ms = 15.0;
        assert!(!stats.is_fast());
    }

    #[test]
    fn test_usage_stats_summary() {
        let stats = UsageStats::new();
        assert_eq!(stats.summary(), "No usage data");

        let mut stats2 = UsageStats::new();
        stats2.usage_count = 100;
        stats2.avg_render_time_ms = 5.5;
        stats2.success_rate = 0.95;
        let summary = stats2.summary();
        assert!(summary.contains("100 uses"));
        assert!(summary.contains("5.5ms"));
        assert!(summary.contains("95%"));
    }

    #[test]
    fn test_usage_stats_formatted_render_time() {
        let mut stats = UsageStats::new();
        stats.avg_render_time_ms = 0.5;
        assert_eq!(stats.formatted_render_time(), "0.50ms");

        stats.avg_render_time_ms = 50.5;
        assert_eq!(stats.formatted_render_time(), "50.5ms");

        stats.avg_render_time_ms = 1500.0;
        assert_eq!(stats.formatted_render_time(), "1.50s");
    }

    #[test]
    fn test_usage_stats_display() {
        let stats = UsageStats::new();
        let display = format!("{}", stats);
        assert_eq!(display, "No usage data");
    }

    // RenderMetrics tests
    #[test]
    fn test_render_metrics_new() {
        let metrics = RenderMetrics::new();
        assert_eq!(metrics.total_renders, 0);
        assert!(!metrics.has_renders());
    }

    #[test]
    fn test_render_metrics_has_renders() {
        let mut metrics = RenderMetrics::new();
        assert!(!metrics.has_renders());

        metrics.total_renders = 1;
        assert!(metrics.has_renders());
    }

    #[test]
    fn test_render_metrics_success_rate() {
        let metrics = RenderMetrics::new();
        assert_eq!(metrics.success_rate(), 0.0);

        let mut metrics2 = RenderMetrics::new();
        metrics2.total_renders = 100;
        metrics2.successful_renders = 95;
        assert!((metrics2.success_rate() - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_render_metrics_failure_rate() {
        let mut metrics = RenderMetrics::new();
        metrics.total_renders = 100;
        metrics.failed_renders = 10;
        assert!((metrics.failure_rate() - 0.10).abs() < 0.001);
    }

    #[test]
    fn test_render_metrics_is_healthy() {
        let mut metrics = RenderMetrics::new();
        metrics.total_renders = 100;
        metrics.successful_renders = 95;
        assert!(metrics.is_healthy());

        metrics.successful_renders = 90;
        assert!(!metrics.is_healthy());
    }

    #[test]
    fn test_render_metrics_has_effective_cache() {
        let mut metrics = RenderMetrics::new();
        metrics.cache_hit_rate = 0.5;
        assert!(!metrics.has_effective_cache());

        metrics.cache_hit_rate = 0.7;
        assert!(metrics.has_effective_cache());
    }

    #[test]
    fn test_render_metrics_performance_category() {
        let mut metrics = RenderMetrics::new();
        assert_eq!(metrics.performance_category(), "Instant");

        metrics.avg_render_time_ms = 5.0;
        assert_eq!(metrics.performance_category(), "Fast");

        metrics.avg_render_time_ms = 30.0;
        assert_eq!(metrics.performance_category(), "Normal");

        metrics.avg_render_time_ms = 100.0;
        assert_eq!(metrics.performance_category(), "Slow");

        metrics.avg_render_time_ms = 500.0;
        assert_eq!(metrics.performance_category(), "Very Slow");
    }

    #[test]
    fn test_render_metrics_formatted_total_time() {
        let mut metrics = RenderMetrics::new();
        metrics.total_render_time_ms = 500;
        assert_eq!(metrics.formatted_total_time(), "500ms");

        metrics.total_render_time_ms = 5000;
        assert_eq!(metrics.formatted_total_time(), "5.00s");

        metrics.total_render_time_ms = 120000;
        assert_eq!(metrics.formatted_total_time(), "2.00m");
    }

    #[test]
    fn test_render_metrics_summary() {
        let metrics = RenderMetrics::new();
        assert_eq!(metrics.summary(), "No renders");

        let mut metrics2 = RenderMetrics::new();
        metrics2.total_renders = 100;
        metrics2.successful_renders = 95;
        metrics2.avg_render_time_ms = 5.0;
        metrics2.cache_hit_rate = 0.8;
        let summary = metrics2.summary();
        assert!(summary.contains("100 renders"));
        assert!(summary.contains("95%"));
    }

    #[test]
    fn test_render_metrics_record_success() {
        let mut metrics = RenderMetrics::new();
        metrics.record_success(10.0);
        metrics.record_success(20.0);

        assert_eq!(metrics.total_renders, 2);
        assert_eq!(metrics.successful_renders, 2);
        assert!((metrics.avg_render_time_ms - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_render_metrics_record_failure() {
        let mut metrics = RenderMetrics::new();
        metrics.record_failure();
        metrics.record_failure();

        assert_eq!(metrics.total_renders, 2);
        assert_eq!(metrics.failed_renders, 2);
        assert_eq!(metrics.successful_renders, 0);
    }

    #[test]
    fn test_render_metrics_display() {
        let metrics = RenderMetrics::new();
        let display = format!("{}", metrics);
        assert_eq!(display, "No renders");
    }
}
