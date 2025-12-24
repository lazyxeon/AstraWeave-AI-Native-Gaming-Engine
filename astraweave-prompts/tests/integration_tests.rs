/// Integration tests for astraweave-prompts crate
/// Tests complete workflows and public API usage patterns

use astraweave_prompts::{
    ContextValue, PromptContext, PromptLibrary, PromptTemplate, TemplateEngine,
    TemplateMetadata, TemplateCategory, UsageStats, PromptsConfig,
    PromptValidator, PromptFormatter, PromptAnalyzer, ValidationRules, FormattingOptions,
    OptimizationEngine, OptimizationConfig,
};
use astraweave_prompts::optimization::{TemplateCache, CacheConfig};
use std::collections::HashMap;

// ========================================
// End-to-End Workflow Tests
// ========================================

#[test]
fn test_complete_template_workflow() {
    // Create engine
    let mut engine = TemplateEngine::new();
    
    // Create template with metadata
    let mut meta = TemplateMetadata::default();
    meta.name = "greeting".to_string();
    meta.description = "A greeting template".to_string();
    meta.category = TemplateCategory::Conversation;
    meta.required_variables = vec!["name".to_string()];
    
    let template = PromptTemplate::new_with_metadata(
        "greeting",
        "Hello {{name}}! Welcome to {{place}}.",
        meta
    );
    
    // Register template
    engine.register_template("greeting", template).unwrap();
    
    // Create context
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("Alice".to_string()));
    ctx.set("place".to_string(), ContextValue::String("Wonderland".to_string()));
    
    // Render
    let result = engine.render("greeting", &ctx).unwrap();
    assert_eq!(result, "Hello Alice! Welcome to Wonderland.");
}

#[test]
fn test_template_with_partial_workflow() {
    let mut engine = TemplateEngine::new();
    
    // Register a partial for reusable header
    engine.register_partial("header", "=== {{title}} ===").unwrap();
    
    // Register template that uses partial
    let template = PromptTemplate::new(
        "document",
        "{{> header}}\n\nContent: {{content}}\n\n{{> header}}"
    );
    engine.register_template("document", template).unwrap();
    
    // Create context
    let mut ctx = PromptContext::new();
    ctx.set("title".to_string(), ContextValue::String("Important Document".to_string()));
    ctx.set("content".to_string(), ContextValue::String("This is the main content.".to_string()));
    
    // Render
    let result = engine.render("document", &ctx).unwrap();
    assert!(result.contains("=== Important Document ==="));
    assert!(result.contains("This is the main content."));
}

#[test]
fn test_nested_object_workflow() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "user_profile",
        "Name: {{user.name}}\nEmail: {{user.email}}\nRole: {{user.role}}"
    );
    engine.register_template("user_profile", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set_path("user.name", ContextValue::String("Bob".to_string()));
    ctx.set_path("user.email", ContextValue::String("bob@example.com".to_string()));
    ctx.set_path("user.role", ContextValue::String("Developer".to_string()));
    
    let result = engine.render("user_profile", &ctx).unwrap();
    assert!(result.contains("Name: Bob"));
    assert!(result.contains("Email: bob@example.com"));
    assert!(result.contains("Role: Developer"));
}

#[test]
fn test_array_iteration_workflow() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "list",
        "Items:\n{{#each items}}- {{this}}\n{{/each}}"
    );
    engine.register_template("list", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("items".to_string(), ContextValue::Array(vec![
        ContextValue::String("First".to_string()),
        ContextValue::String("Second".to_string()),
        ContextValue::String("Third".to_string()),
    ]));
    
    let result = engine.render("list", &ctx).unwrap();
    assert!(result.contains("- First"));
    assert!(result.contains("- Second"));
    assert!(result.contains("- Third"));
}

#[test]
fn test_conditional_rendering_workflow() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "conditional",
        "{{#if show_message}}Message: {{message}}{{/if}}"
    );
    engine.register_template("conditional", template).unwrap();
    
    // Test with condition true
    let mut ctx = PromptContext::new();
    ctx.set("show_message".to_string(), ContextValue::Boolean(true));
    ctx.set("message".to_string(), ContextValue::String("Hello!".to_string()));
    
    let result = engine.render("conditional", &ctx).unwrap();
    assert!(result.contains("Message: Hello!"));
    
    // Test with condition false
    let mut ctx2 = PromptContext::new();
    ctx2.set("show_message".to_string(), ContextValue::Boolean(false));
    ctx2.set("message".to_string(), ContextValue::String("Hello!".to_string()));
    
    let result2 = engine.render("conditional", &ctx2).unwrap();
    assert!(!result2.contains("Message: Hello!"));
}

// ========================================
// Library Management Tests
// ========================================

#[test]
fn test_library_basic_operations() {
    let mut lib = PromptLibrary::new();
    
    let template1 = PromptTemplate::new("t1", "Template 1");
    let template2 = PromptTemplate::new("t2", "Template 2");
    
    lib.add_template("t1", template1);
    lib.add_template("t2", template2);
    
    assert_eq!(lib.list_templates().len(), 2);
    
    let retrieved = lib.get_template("t1").unwrap();
    assert_eq!(retrieved.id(), "t1");
    
    lib.delete_template("t1");
    assert_eq!(lib.list_templates().len(), 1);
}

#[test]
fn test_library_template_replacement() {
    let mut lib = PromptLibrary::new();
    
    lib.add_template("test", PromptTemplate::new("test", "Original"));
    lib.add_template("test", PromptTemplate::new("test", "Updated"));
    
    let template = lib.get_template("test").unwrap();
    assert!(template.template().contains("Updated"));
}

// ========================================
// Validation Tests
// ========================================

#[test]
fn test_validator_length_checks() {
    let rules = ValidationRules {
        min_length: 10,
        max_length: 100,
        required_variables: vec![],
        forbidden_patterns: vec![],
    };
    
    // Too short
    assert!(PromptValidator::validate("short", &rules).is_err());
    
    // Just right
    assert!(PromptValidator::validate("This is a valid prompt length", &rules).is_ok());
    
    // Too long
    let long_prompt = "a".repeat(150);
    assert!(PromptValidator::validate(&long_prompt, &rules).is_err());
}

#[test]
fn test_validator_required_variables() {
    let rules = ValidationRules {
        min_length: 5,
        max_length: 1000,
        required_variables: vec!["name".to_string(), "age".to_string()],
        forbidden_patterns: vec![],
    };
    
    // Missing variables
    assert!(PromptValidator::validate("Hello world", &rules).is_err());
    
    // Has all required variables
    let valid_prompt = "Hello {{name}}, you are {{age}} years old";
    assert!(PromptValidator::validate(valid_prompt, &rules).is_ok());
}

#[test]
fn test_validator_forbidden_patterns() {
    let rules = ValidationRules {
        min_length: 5,
        max_length: 1000,
        required_variables: vec![],
        forbidden_patterns: vec!["badword".to_string(), "forbidden".to_string()],
    };
    
    // Contains forbidden pattern
    assert!(PromptValidator::validate("This contains badword text", &rules).is_err());
    
    // Clean prompt
    assert!(PromptValidator::validate("This is a clean prompt", &rules).is_ok());
}

#[test]
fn test_validator_well_formed_braces() {
    assert!(PromptValidator::is_well_formed("Hello {{name}}!"));
    assert!(PromptValidator::is_well_formed("{{a}} {{b}} {{c}}"));
    assert!(!PromptValidator::is_well_formed("Hello {{name}"));
    assert!(!PromptValidator::is_well_formed("Hello name}}"));
    assert!(!PromptValidator::is_well_formed("{{{{}}"));
}

// ========================================
// Formatter Tests
// ========================================

#[test]
fn test_formatter_minify() {
    let prompt = "Line 1\n  \n  Line 2  \n\n\nLine 3  ";
    let minified = PromptFormatter::minify(prompt);
    
    assert!(!minified.contains('\n'));
    assert_eq!(minified, "Line 1 Line 2 Line 3");
}

#[test]
fn test_formatter_format_with_indentation() {
    let options = FormattingOptions {
        indent_size: 4,
        use_tabs: false,
        max_line_length: 80,
    };
    
    let prompt = "Line 1\n  Indented line\nLine 3";
    let formatted = PromptFormatter::format(prompt, &options);
    
    assert!(formatted.contains("Line 1"));
    assert!(formatted.contains("Line 3"));
}

#[test]
fn test_formatter_pretty_print() {
    let prompt = "Hello {{name}}, your score is {{score}}!";
    let pretty = PromptFormatter::pretty_print(prompt);
    
    assert!(pretty.contains("**{name}**"));
    assert!(pretty.contains("**{score}**"));
}

// ========================================
// Analyzer Tests
// ========================================

#[test]
fn test_analyzer_estimate_tokens() {
    let text = "This is a test sentence with ten words in it here.";
    let tokens = PromptAnalyzer::estimate_tokens(text);
    
    assert!(tokens > 0);
    assert!(tokens <= text.split_whitespace().count());
}

#[test]
fn test_analyzer_extract_variables() {
    let prompt = "Hello {{name}}! Your {{item}} is ready at {{location}}.";
    let vars = PromptAnalyzer::extract_variables(prompt);
    
    assert_eq!(vars.len(), 3);
    assert!(vars.contains(&"name".to_string()));
    assert!(vars.contains(&"item".to_string()));
    assert!(vars.contains(&"location".to_string()));
}

#[test]
fn test_analyzer_calculate_complexity() {
    let simple = "Hello world";
    let complex = "a ".repeat(100) + &"{{var1}} {{var2}} {{var3}}".repeat(10);
    
    let simple_score = PromptAnalyzer::calculate_complexity(simple);
    let complex_score = PromptAnalyzer::calculate_complexity(&complex);
    
    assert!(simple_score < complex_score);
}

#[test]
fn test_analyzer_calculate_readability() {
    let easy = "Short. Sentences. Are. Easy. To. Read.";
    let hard = "This is a very long sentence with many words that continues on and on and makes it harder to read and understand the content being presented here.";
    
    let easy_score = PromptAnalyzer::calculate_readability(easy);
    let hard_score = PromptAnalyzer::calculate_readability(hard);
    
    // Higher scores = better readability
    assert!(easy_score > hard_score);
}

#[test]
fn test_analyzer_generate_stats() {
    let prompt = "Hello {{name}}!\nWelcome to our system.";
    let stats = PromptAnalyzer::generate_stats(prompt);
    
    assert!(stats.contains_key("character_count"));
    assert!(stats.contains_key("word_count"));
    assert!(stats.contains_key("line_count"));
    assert!(stats.contains_key("variable_count"));
    assert!(stats.contains_key("estimated_tokens"));
    
    assert_eq!(stats.get("variable_count").unwrap(), "1");
    assert_eq!(stats.get("line_count").unwrap(), "2");
}

#[test]
fn test_analyzer_full_analysis() {
    let prompt = "Hello {{name}}, you have {{count}} messages.";
    let analysis = PromptAnalyzer::analyze(prompt);
    
    assert!(analysis.estimated_tokens > 0);
    assert!(analysis.complexity_score <= 100);
    assert_eq!(analysis.variables.len(), 2);
    assert!(analysis.readability_score <= 100);
}

// ========================================
// Optimization Tests
// ========================================

#[test]
fn test_optimization_engine_basic() {
    let config = OptimizationConfig::default();
    let mut engine = OptimizationEngine::new(config);
    
    let prompt = "Hello world";
    let optimized = engine.optimize_prompt(prompt).unwrap();
    
    assert_eq!(optimized, prompt);
    assert_eq!(engine.get_metrics().templates_processed, 1);
}

#[test]
fn test_optimization_compression() {
    let mut config = OptimizationConfig::default();
    config.max_prompt_length = 50;
    config.enable_compression = true;
    
    let mut engine = OptimizationEngine::new(config);
    
    let prompt = "  Line 1  \n\n  Line 2  \n  Line 3  \n  This is a longer line  ";
    let optimized = engine.optimize_prompt(prompt).unwrap();
    
    // Compression should remove newlines and extra spaces
    assert!(!optimized.contains('\n'));
    assert!(optimized.len() < prompt.len());
}

#[test]
fn test_optimization_metrics_tracking() {
    let config = OptimizationConfig::default();
    let mut engine = OptimizationEngine::new(config);
    
    for i in 0..10 {
        engine.optimize_prompt(&format!("Prompt {}", i)).unwrap();
    }
    
    let metrics = engine.get_metrics();
    assert_eq!(metrics.templates_processed, 10);
    assert!(metrics.avg_processing_time_ms >= 0.0);
}

#[test]
fn test_optimization_reset_metrics() {
    let config = OptimizationConfig::default();
    let mut engine = OptimizationEngine::new(config);
    
    engine.optimize_prompt("test").unwrap();
    assert_eq!(engine.get_metrics().templates_processed, 1);
    
    engine.reset_metrics();
    assert_eq!(engine.get_metrics().templates_processed, 0);
}

// ========================================
// Cache Tests
// ========================================

#[test]
fn test_cache_basic_operations() {
    let config = CacheConfig::default();
    let mut cache = TemplateCache::new(config);
    
    cache.put("key1".to_string(), "template1".to_string());
    
    let result = cache.get("key1");
    assert_eq!(result, Some("template1".to_string()));
    
    let stats = cache.stats();
    assert_eq!(stats.size, 1);
}

#[test]
fn test_cache_miss() {
    let config = CacheConfig::default();
    let mut cache = TemplateCache::new(config);
    
    let result = cache.get("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_cache_eviction_on_size_limit() {
    let config = CacheConfig {
        max_size: 3,
        ttl_seconds: 3600,
    };
    
    let mut cache = TemplateCache::new(config);
    
    cache.put("key1".to_string(), "template1".to_string());
    cache.put("key2".to_string(), "template2".to_string());
    cache.put("key3".to_string(), "template3".to_string());
    
    // Access key2 and key3 to increase their access count
    cache.get("key2");
    cache.get("key3");
    
    // Adding key4 should evict key1 (LRU)
    cache.put("key4".to_string(), "template4".to_string());
    
    assert!(cache.get("key1").is_none());
    assert!(cache.get("key2").is_some());
}

#[test]
fn test_cache_clear() {
    let config = CacheConfig::default();
    let mut cache = TemplateCache::new(config);
    
    cache.put("key1".to_string(), "template1".to_string());
    cache.put("key2".to_string(), "template2".to_string());
    
    assert_eq!(cache.stats().size, 2);
    
    cache.clear();
    assert_eq!(cache.stats().size, 0);
}

// ========================================
// Configuration Tests
// ========================================

#[test]
fn test_prompts_config_defaults() {
    let config = PromptsConfig::default();
    
    assert_eq!(config.templates_dir, "templates");
    assert!(config.hot_reload);
    assert_eq!(config.max_template_size, 1024 * 1024);
    assert!(config.cache_config.enabled);
}

#[test]
fn test_validation_rules_defaults() {
    let rules = ValidationRules::default();
    
    assert_eq!(rules.max_length, 8000);
    assert_eq!(rules.min_length, 10);
    assert!(rules.required_variables.is_empty());
    assert!(rules.forbidden_patterns.is_empty());
}

#[test]
fn test_formatting_options_defaults() {
    let options = FormattingOptions::default();
    
    assert_eq!(options.indent_size, 2);
    assert!(!options.use_tabs);
    assert_eq!(options.max_line_length, 80);
}

// ========================================
// Template Metadata Tests
// ========================================

#[test]
fn test_template_metadata_creation() {
    let meta = TemplateMetadata {
        name: "test_template".to_string(),
        description: "A test template".to_string(),
        category: TemplateCategory::Dialogue,
        author: Some("Test Author".to_string()),
        version: "1.0.0".to_string(),
        created_at: 0,
        updated_at: 0,
        tags: vec!["test".to_string(), "example".to_string()],
        required_variables: vec!["name".to_string()],
        optional_variables: HashMap::new(),
        usage_stats: UsageStats::default(),
    };
    
    assert_eq!(meta.name, "test_template");
    assert_eq!(meta.category, TemplateCategory::Dialogue);
    assert_eq!(meta.tags.len(), 2);
}

#[test]
fn test_template_with_metadata() {
    let mut meta = TemplateMetadata::default();
    meta.name = "greeting".to_string();
    meta.description = "Greeting template".to_string();
    meta.category = TemplateCategory::Conversation;
    
    let template = PromptTemplate::new_with_metadata(
        "greeting",
        "Hello {{name}}!",
        meta.clone()
    );
    
    assert_eq!(template.id(), "greeting");
    assert_eq!(template.metadata().name, "greeting");
    assert_eq!(template.metadata().category, TemplateCategory::Conversation);
}

#[test]
fn test_template_category_variants() {
    let categories = vec![
        TemplateCategory::Dialogue,
        TemplateCategory::Behavior,
        TemplateCategory::Narrative,
        TemplateCategory::Combat,
        TemplateCategory::System,
        TemplateCategory::Conversation,
        TemplateCategory::WorldBuilding,
        TemplateCategory::TerrainGeneration,
        TemplateCategory::Custom,
    ];
    
    assert_eq!(categories.len(), 9);
}

// ========================================
// Error Handling Tests
// ========================================

#[test]
fn test_template_not_found_error() {
    let engine = TemplateEngine::new();
    let ctx = PromptContext::new();
    
    let result = engine.render("nonexistent", &ctx);
    assert!(result.is_err());
}

#[test]
fn test_library_template_not_found() {
    let lib = PromptLibrary::new();
    
    let result = lib.get_template("nonexistent");
    assert!(result.is_err());
}

// ========================================
// Unicode and Special Characters Tests
// ========================================

#[test]
fn test_unicode_template() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "unicode",
        "Hello {{name}}! 你好 {{chinese_name}}! Привет {{russian_name}}!"
    );
    engine.register_template("unicode", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("Alice".to_string()));
    ctx.set("chinese_name".to_string(), ContextValue::String("爱丽丝".to_string()));
    ctx.set("russian_name".to_string(), ContextValue::String("Алиса".to_string()));
    
    let result = engine.render("unicode", &ctx).unwrap();
    assert!(result.contains("Alice"));
    assert!(result.contains("爱丽丝"));
    assert!(result.contains("Алиса"));
}

#[test]
fn test_special_characters_in_template() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "special",
        "Symbols: !@#$%^&*()_+-=[]{}|;':\",./<>? Value: {{value}}"
    );
    engine.register_template("special", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("value".to_string(), ContextValue::String("test".to_string()));
    
    let result = engine.render("special", &ctx).unwrap();
    assert!(result.contains("!@#$%^&*()"));
}

#[test]
fn test_emoji_in_template() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "emoji",
        "Hello {{name}}! 👋 Have a great day! 🌟"
    );
    engine.register_template("emoji", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("World".to_string()));
    
    let result = engine.render("emoji", &ctx).unwrap();
    assert!(result.contains("👋"));
    assert!(result.contains("🌟"));
}

// ========================================
// Performance and Stress Tests
// ========================================

#[test]
fn test_large_number_of_templates() {
    let mut engine = TemplateEngine::new();
    
    for i in 0..100 {
        let template = PromptTemplate::new(
            format!("template_{}", i),
            format!("Template {} content: {{{{value}}}}", i)
        );
        engine.register_template(&format!("template_{}", i), template).unwrap();
    }
    
    assert_eq!(engine.list_templates().len(), 100);
}

#[test]
fn test_large_context_object() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new("large", "Data: {{data.field1}} {{data.field50}}");
    engine.register_template("large", template).unwrap();
    
    let mut ctx = PromptContext::new();
    
    // Create a large nested object
    for i in 0..100 {
        ctx.set_path(
            &format!("data.field{}", i),
            ContextValue::String(format!("value{}", i))
        );
    }
    
    let result = engine.render("large", &ctx).unwrap();
    assert!(result.contains("value1"));
    assert!(result.contains("value50"));
}

#[test]
fn test_deeply_nested_context() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "nested",
        "Value: {{level1.level2.level3.level4.value}}"
    );
    engine.register_template("nested", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set_path(
        "level1.level2.level3.level4.value",
        ContextValue::String("deep_value".to_string())
    );
    
    let result = engine.render("nested", &ctx).unwrap();
    assert_eq!(result, "Value: deep_value");
}
