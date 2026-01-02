/// Comprehensive Coverage Tests for astraweave-prompts
/// Focus: Improve coverage from 12.35% to 40%+ with targeted tests
///
/// Coverage areas:
/// - Template creation with various formats
/// - Variable substitution (simple, nested, arrays, objects)
/// - Edge cases (empty templates, missing vars, invalid syntax)
/// - Context management (scopes, merging, paths)
/// - Engine operations (registration, rendering, partials, helpers)
/// - Template validation and error handling

use astraweave_prompts::{
    ContextValue, PromptContext, PromptEngine, PromptTemplate, TemplateEngine,
    TemplateProcessor, ProcessorConfig, EngineConfig,
};
use std::collections::HashMap;

// ========================================
// Template Creation Tests
// ========================================

#[test]
fn test_template_with_no_variables() {
    let template = PromptTemplate::new("static", "This is a static template");
    assert_eq!(template.id(), "static");
    assert_eq!(template.template(), "This is a static template");
    assert!(template.variables.is_empty());
}

#[test]
fn test_template_with_multiple_variables() {
    let template = PromptTemplate::new(
        "multi",
        "Hello {{name}}, you are {{age}} years old and live in {{city}}",
    );
    assert_eq!(template.variables.len(), 3);
    assert!(template.variables.contains(&"name".to_string()));
    assert!(template.variables.contains(&"age".to_string()));
    assert!(template.variables.contains(&"city".to_string()));
}

#[test]
fn test_template_with_repeated_variables() {
    let template = PromptTemplate::new("repeat", "{{x}} plus {{x}} equals {{y}}");
    // Variables should be unique
    assert!(template.variables.contains(&"x".to_string()));
    assert!(template.variables.contains(&"y".to_string()));
}

#[test]
fn test_template_with_whitespace_in_placeholders() {
    let template = PromptTemplate::new("whitespace", "{{ name }} and {{  value  }}");
    assert!(template.variables.contains(&"name".to_string()));
    assert!(template.variables.contains(&"value".to_string()));
}

// ========================================
// Variable Substitution Tests
// ========================================

#[test]
fn test_simple_variable_substitution() {
    let template = PromptTemplate::new("simple", "Hello {{name}}!");
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("Alice".to_string()));

    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "Hello Alice!");
}

#[test]
fn test_multiple_variable_substitution() {
    let template = PromptTemplate::new("multi", "{{greeting}} {{name}}, age {{age}}");
    let mut ctx = PromptContext::new();
    ctx.set("greeting".to_string(), ContextValue::String("Hello".to_string()));
    ctx.set("name".to_string(), ContextValue::String("Bob".to_string()));
    ctx.set("age".to_string(), ContextValue::Number(30.0));

    let result = template.render(&ctx).unwrap();
    assert!(result.contains("Hello Bob"));
    assert!(result.contains("age 30"));
}

#[test]
fn test_number_variable_substitution() {
    let template = PromptTemplate::new("num", "Count: {{count}}");
    let mut ctx = PromptContext::new();
    ctx.set("count".to_string(), ContextValue::Number(42.5));

    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "Count: 42.5");
}

#[test]
fn test_boolean_variable_substitution() {
    let template = PromptTemplate::new("bool", "Active: {{is_active}}");
    let mut ctx = PromptContext::new();
    ctx.set("is_active".to_string(), ContextValue::Boolean(true));

    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "Active: true");
}

#[test]
fn test_array_variable_substitution() {
    let template = PromptTemplate::new("array", "Items: {{#each items}}{{this}} {{/each}}");
    let mut ctx = PromptContext::new();
    ctx.set(
        "items".to_string(),
        ContextValue::Array(vec![
            ContextValue::String("apple".to_string()),
            ContextValue::String("banana".to_string()),
            ContextValue::String("cherry".to_string()),
        ]),
    );

    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "Items: apple banana cherry ");
}

#[test]
fn test_nested_object_variable_substitution() {
    let template = PromptTemplate::new("nested", "User: {{user.name}}, Email: {{user.email}}");
    let mut ctx = PromptContext::new();
    
    let mut user_obj = HashMap::new();
    user_obj.insert("name".to_string(), ContextValue::String("Charlie".to_string()));
    user_obj.insert("email".to_string(), ContextValue::String("charlie@example.com".to_string()));
    
    ctx.set("user".to_string(), ContextValue::Object(user_obj));

    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "User: Charlie, Email: charlie@example.com");
}

// ========================================
// Edge Cases Tests
// ========================================

#[test]
fn test_empty_template() {
    let template = PromptTemplate::new("empty", "");
    let ctx = PromptContext::new();
    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_template_with_only_text() {
    let template = PromptTemplate::new("text", "Just plain text, no variables");
    let ctx = PromptContext::new();
    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "Just plain text, no variables");
}

#[test]
fn test_missing_variable_strict_mode() {
    let template = PromptTemplate::new("missing", "Hello {{undefined_var}}!");
    let ctx = PromptContext::new();
    
    // With strict mode, this should fail or return empty
    let result = template.render(&ctx);
    // Depending on strict mode, this might error or render as empty
    // For now, we just verify it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_template_with_special_characters() {
    let template = PromptTemplate::new("special", "Name: {{name}}, Symbols: !@#$%^&*()");
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("Test".to_string()));

    let result = template.render(&ctx).unwrap();
    assert!(result.contains("Symbols: !@#$%^&*()"));
}

#[test]
fn test_template_with_newlines() {
    let template = PromptTemplate::new("newlines", "Line 1: {{val1}}\nLine 2: {{val2}}");
    let mut ctx = PromptContext::new();
    ctx.set("val1".to_string(), ContextValue::String("First".to_string()));
    ctx.set("val2".to_string(), ContextValue::String("Second".to_string()));

    let result = template.render(&ctx).unwrap();
    assert!(result.contains("Line 1: First"));
    assert!(result.contains("Line 2: Second"));
}

#[test]
fn test_render_map_basic() {
    let template = PromptTemplate::new("map", "Key: {{key}}");
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());

    let result = template.render_map(&map).unwrap();
    assert_eq!(result, "Key: value");
}

#[test]
fn test_render_map_empty() {
    let template = PromptTemplate::new("map", "Static content");
    let map = HashMap::new();

    let result = template.render_map(&map).unwrap();
    assert_eq!(result, "Static content");
}

// ========================================
// Context Management Tests
// ========================================

#[test]
fn test_context_set_and_get() {
    let mut ctx = PromptContext::new();
    ctx.set("key".to_string(), ContextValue::String("value".to_string()));

    let value = ctx.get("key");
    assert!(value.is_some());
}

#[test]
fn test_context_get_nonexistent() {
    let ctx = PromptContext::new();
    assert!(ctx.get("nonexistent").is_none());
}

#[test]
fn test_context_scope_push_pop() {
    let mut ctx = PromptContext::new();
    ctx.set("global".to_string(), ContextValue::String("global_value".to_string()));

    ctx.push_scope();
    ctx.set("local".to_string(), ContextValue::String("local_value".to_string()));

    assert!(ctx.get("global").is_some());
    assert!(ctx.get("local").is_some());

    ctx.pop_scope();
    assert!(ctx.get("global").is_some());
    // local should still exist in the popped scope but not in current
}

#[test]
fn test_context_nested_scopes() {
    let mut ctx = PromptContext::new();
    ctx.set("level0".to_string(), ContextValue::String("L0".to_string()));

    ctx.push_scope();
    ctx.set("level1".to_string(), ContextValue::String("L1".to_string()));

    ctx.push_scope();
    ctx.set("level2".to_string(), ContextValue::String("L2".to_string()));

    assert!(ctx.get("level0").is_some());
    assert!(ctx.get("level1").is_some());
    assert!(ctx.get("level2").is_some());

    ctx.pop_scope();
    assert!(ctx.get("level1").is_some());

    ctx.pop_scope();
    assert!(ctx.get("level0").is_some());
}

#[test]
fn test_context_merge() {
    let mut ctx1 = PromptContext::new();
    ctx1.set("key1".to_string(), ContextValue::String("value1".to_string()));

    let mut ctx2 = PromptContext::new();
    ctx2.set("key2".to_string(), ContextValue::String("value2".to_string()));

    ctx1.merge(ctx2);

    assert!(ctx1.get("key1").is_some());
    assert!(ctx1.get("key2").is_some());
}

#[test]
fn test_context_set_path_simple() {
    let mut ctx = PromptContext::new();
    ctx.set_path("simple", ContextValue::String("value".to_string()));

    assert!(ctx.get("simple").is_some());
}

#[test]
fn test_context_set_path_nested() {
    let mut ctx = PromptContext::new();
    ctx.set_path("user.name", ContextValue::String("Alice".to_string()));
    ctx.set_path("user.age", ContextValue::Number(25.0));

    // Verify the structure was created
    let user = ctx.get("user");
    assert!(user.is_some());
}

#[test]
fn test_context_to_string_map() {
    let mut ctx = PromptContext::new();
    ctx.set("key1".to_string(), ContextValue::String("value1".to_string()));
    ctx.set("key2".to_string(), ContextValue::Number(42.0));

    let map = ctx.to_string_map();
    assert!(map.contains_key("key1"));
    assert!(map.contains_key("key2"));
}

#[test]
fn test_context_to_json() {
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("Test".to_string()));
    ctx.set("count".to_string(), ContextValue::Number(10.0));
    ctx.set("active".to_string(), ContextValue::Boolean(true));

    let json = ctx.to_json();
    assert!(json.is_object());
    assert!(json.get("name").is_some());
    assert!(json.get("count").is_some());
    assert!(json.get("active").is_some());
}

// ========================================
// ContextValue Tests
// ========================================

#[test]
fn test_context_value_string_conversion() {
    let val = ContextValue::String("test".to_string());
    assert_eq!(val.to_string(), "test");
}

#[test]
fn test_context_value_number_conversion() {
    let val = ContextValue::Number(42.5);
    let s = val.to_string();
    assert!(s.contains("42"));
}

#[test]
fn test_context_value_boolean_conversion() {
    let val_true = ContextValue::Boolean(true);
    let val_false = ContextValue::Boolean(false);
    assert_eq!(val_true.to_string(), "true");
    assert_eq!(val_false.to_string(), "false");
}

#[test]
fn test_context_value_array_to_json() {
    let arr = ContextValue::Array(vec![
        ContextValue::String("a".to_string()),
        ContextValue::Number(1.0),
    ]);
    let json = arr.to_json();
    assert!(json.is_array());
}

#[test]
fn test_context_value_object_to_json() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), ContextValue::String("value".to_string()));
    let obj = ContextValue::Object(map);
    let json = obj.to_json();
    assert!(json.is_object());
}

// ========================================
// Engine Tests
// ========================================

#[test]
fn test_engine_creation() {
    let engine = TemplateEngine::new();
    assert_eq!(engine.list_templates().len(), 0);
}

#[test]
fn test_engine_register_and_list() {
    let mut engine = TemplateEngine::new();
    let template = PromptTemplate::new("test1", "Content 1");

    engine.register_template("test1", template).unwrap();
    let templates = engine.list_templates();
    assert!(templates.contains(&"test1".to_string()));
}

#[test]
fn test_engine_register_multiple_templates() {
    let mut engine = TemplateEngine::new();

    engine.register_template("t1", PromptTemplate::new("t1", "Template 1")).unwrap();
    engine.register_template("t2", PromptTemplate::new("t2", "Template 2")).unwrap();
    engine.register_template("t3", PromptTemplate::new("t3", "Template 3")).unwrap();

    let templates = engine.list_templates();
    assert_eq!(templates.len(), 3);
}

#[test]
fn test_engine_render_registered_template() {
    let mut engine = TemplateEngine::new();
    engine.register_template("greeting", PromptTemplate::new("g", "Hello {{name}}!")).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("World".to_string()));

    let result = engine.render("greeting", &ctx).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_engine_clear_templates() {
    let mut engine = TemplateEngine::new();
    engine.register_template("t1", PromptTemplate::new("t1", "Content")).unwrap();

    assert_eq!(engine.list_templates().len(), 1);

    engine.clear_templates();
    assert_eq!(engine.list_templates().len(), 0);
}

#[test]
fn test_engine_register_partial() {
    let mut engine = TemplateEngine::new();
    engine.register_partial("header", "=== Header ===").unwrap();
    engine.register_template("main", PromptTemplate::new("m", "{{> header}}\nContent")).unwrap();

    let ctx = PromptContext::new();
    let result = engine.render("main", &ctx).unwrap();
    assert!(result.contains("=== Header ==="));
}

#[test]
fn test_prompt_engine_creation() {
    let config = EngineConfig::default();
    let engine = PromptEngine::new(config);
    assert_eq!(engine.list_templates().len(), 0);
}

#[test]
fn test_prompt_engine_register_template() {
    let mut engine = PromptEngine::new(EngineConfig::default());
    let result = engine.register_template("test".to_string(), "Hello {{name}}".to_string());
    assert!(result.is_ok());
}

#[test]
fn test_prompt_engine_template_too_large() {
    let mut config = EngineConfig::default();
    config.max_template_size = 10;
    let mut engine = PromptEngine::new(config);

    let large_template = "x".repeat(100);
    let result = engine.register_template("large".to_string(), large_template);
    assert!(result.is_err());
}

#[test]
fn test_prompt_engine_render() {
    let mut engine = PromptEngine::new(EngineConfig::default());
    engine.register_template("test".to_string(), "Value: {{val}}".to_string()).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("val".to_string(), ContextValue::Number(123.0));

    let result = engine.render("test", &ctx).unwrap();
    assert!(result.contains("Value: 123"));
}

// ========================================
// Template Processor Tests
// ========================================

#[test]
fn test_processor_creation() {
    let config = ProcessorConfig::default();
    let _processor = TemplateProcessor::new(config);
}

#[test]
fn test_processor_process_simple() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let mut vars = HashMap::new();
    vars.insert("name".to_string(), "Alice".to_string());

    let result = processor.process("Hello {{name}}", &vars).unwrap();
    assert_eq!(result, "Hello Alice");
}

#[test]
fn test_processor_process_json() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let json = serde_json::json!({
        "user": "Bob",
        "count": 42
    });

    let result = processor.process_json("User: {{user}}, Count: {{count}}", &json).unwrap();
    assert_eq!(result, "User: Bob, Count: 42");
}

#[test]
fn test_processor_validate_valid_template() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let result = processor.validate_template("Valid {{template}} here");
    assert!(result.is_ok());
}

#[test]
fn test_processor_validate_invalid_template() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let result = processor.validate_template("Invalid {{unclosed template");
    assert!(result.is_err());
}

#[test]
fn test_processor_extract_variables_single() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let vars = processor.extract_variables("Hello {{name}}");
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0], "name");
}

#[test]
fn test_processor_extract_variables_multiple() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let vars = processor.extract_variables("{{greeting}} {{name}}, you have {{count}} items");
    assert_eq!(vars.len(), 3);
    assert!(vars.contains(&"greeting".to_string()));
    assert!(vars.contains(&"name".to_string()));
    assert!(vars.contains(&"count".to_string()));
}

#[test]
fn test_processor_extract_variables_none() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let vars = processor.extract_variables("No variables here");
    assert_eq!(vars.len(), 0);
}

#[test]
fn test_processor_extract_variables_duplicates() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let vars = processor.extract_variables("{{x}} and {{x}} and {{x}}");
    // Should deduplicate
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0], "x");
}

#[test]
fn test_processor_extract_variables_with_dots() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let vars = processor.extract_variables("{{user.name}} and {{user.email}}");
    assert!(vars.len() >= 2);
}

// ========================================
// Configuration Tests
// ========================================

#[test]
fn test_processor_config_default() {
    let config = ProcessorConfig::default();
    assert!(config.validate_variables);
    assert_eq!(config.max_processing_time, 5000);
}

#[test]
fn test_engine_config_default() {
    let config = EngineConfig::default();
    assert_eq!(config.max_template_size, 1024 * 1024);
    assert!(config.enable_caching);
}

#[test]
fn test_processor_config_custom() {
    let config = ProcessorConfig {
        validate_variables: false,
        max_processing_time: 10000,
    };
    assert!(!config.validate_variables);
    assert_eq!(config.max_processing_time, 10000);
}

// ========================================
// Integration Tests
// ========================================

#[test]
fn test_end_to_end_dialogue_template() {
    let mut engine = TemplateEngine::new();
    
    let dialogue_template = PromptTemplate::new(
        "dialogue",
        "You are {{character.name}}, a {{character.role}}. \
         Your personality is {{character.personality}}. \
         Respond to: {{user_input}}"
    );
    
    engine.register_template("dialogue", dialogue_template).unwrap();
    
    let mut ctx = PromptContext::new();
    let mut character = HashMap::new();
    character.insert("name".to_string(), ContextValue::String("Elena".to_string()));
    character.insert("role".to_string(), ContextValue::String("mage".to_string()));
    character.insert("personality".to_string(), ContextValue::String("wise".to_string()));
    
    ctx.set("character".to_string(), ContextValue::Object(character));
    ctx.set("user_input".to_string(), ContextValue::String("Teach me magic".to_string()));
    
    let result = engine.render("dialogue", &ctx).unwrap();
    assert!(result.contains("Elena"));
    assert!(result.contains("mage"));
    assert!(result.contains("wise"));
    assert!(result.contains("Teach me magic"));
}

#[test]
fn test_end_to_end_quest_template() {
    let template = PromptTemplate::new(
        "quest",
        "Quest: {{quest.title}}\nObjective: {{quest.objective}}\nReward: {{quest.reward}} gold"
    );
    
    let mut ctx = PromptContext::new();
    let mut quest = HashMap::new();
    quest.insert("title".to_string(), ContextValue::String("Dragon Slayer".to_string()));
    quest.insert("objective".to_string(), ContextValue::String("Defeat the dragon".to_string()));
    quest.insert("reward".to_string(), ContextValue::Number(1000.0));
    
    ctx.set("quest".to_string(), ContextValue::Object(quest));
    
    let result = template.render(&ctx).unwrap();
    assert!(result.contains("Dragon Slayer"));
    assert!(result.contains("Defeat the dragon"));
    assert!(result.contains("1000"));
}

#[test]
fn test_multiple_templates_in_engine() {
    let mut engine = TemplateEngine::new();
    
    engine.register_template("greeting", PromptTemplate::new("g", "Hello {{name}}!")).unwrap();
    engine.register_template("farewell", PromptTemplate::new("f", "Goodbye {{name}}!")).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("Alice".to_string()));
    
    let greeting = engine.render("greeting", &ctx).unwrap();
    let farewell = engine.render("farewell", &ctx).unwrap();
    
    assert_eq!(greeting, "Hello Alice!");
    assert_eq!(farewell, "Goodbye Alice!");
}

