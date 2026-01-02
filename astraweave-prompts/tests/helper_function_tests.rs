/// Tests for helper functions in astraweave-prompts
/// Covers template helpers, utility functions, and edge cases

use astraweave_prompts::{
    ContextValue, PromptContext, PromptTemplate, TemplateEngine,
};

// ========================================
// Built-in Helper Tests
// ========================================

#[test]
fn test_uppercase_helper() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "upper",
        "Original: {{name}}, Upper: {{uppercase name}}"
    );
    engine.register_template("upper", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("alice".to_string()));
    
    let result = engine.render("upper", &ctx).unwrap();
    assert!(result.contains("Upper: ALICE"));
}

#[test]
fn test_lowercase_helper() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "lower",
        "Original: {{name}}, Lower: {{lowercase name}}"
    );
    engine.register_template("lower", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("ALICE".to_string()));
    
    let result = engine.render("lower", &ctx).unwrap();
    assert!(result.contains("Lower: alice"));
}

#[test]
fn test_trim_helper() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "trim_test",
        "Trimmed: [{{trim text}}]"
    );
    engine.register_template("trim_test", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("  hello world  ".to_string()));
    
    let result = engine.render("trim_test", &ctx).unwrap();
    assert!(result.contains("[hello world]"));
}

#[test]
fn test_length_helper_string() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "len",
        "Length: {{length text}}"
    );
    engine.register_template("len", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("hello".to_string()));
    
    let result = engine.render("len", &ctx).unwrap();
    assert!(result.contains("Length: 5"));
}

#[test]
fn test_length_helper_array() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "len_arr",
        "Count: {{length items}}"
    );
    engine.register_template("len_arr", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("items".to_string(), ContextValue::Array(vec![
        ContextValue::String("a".to_string()),
        ContextValue::String("b".to_string()),
        ContextValue::String("c".to_string()),
    ]));
    
    let result = engine.render("len_arr", &ctx).unwrap();
    assert!(result.contains("Count: 3"));
}

#[test]
fn test_json_helper() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "json_test",
        "JSON: {{json data}}"
    );
    engine.register_template("json_test", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("data".to_string(), ContextValue::String("test".to_string()));
    
    let result = engine.render("json_test", &ctx).unwrap();
    assert!(result.contains("\"test\""));
}

#[test]
fn test_indent_helper_default() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "indent_test",
        "{{indent text}}"
    );
    engine.register_template("indent_test", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("line1\nline2".to_string()));
    
    let result = engine.render("indent_test", &ctx).unwrap();
    // Default indent is 2 spaces
    assert!(result.contains("  line1"));
    assert!(result.contains("  line2"));
}

#[test]
fn test_indent_helper_custom_spaces() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "indent_custom",
        "{{indent text 4}}"
    );
    engine.register_template("indent_custom", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("line1\nline2".to_string()));
    
    let result = engine.render("indent_custom", &ctx).unwrap();
    // Should indent with 4 spaces
    assert!(result.contains("    line1"));
    assert!(result.contains("    line2"));
}

// ========================================
// Context Value Conversion Tests
// ========================================

#[test]
fn test_context_value_from_string() {
    let val: ContextValue = "test".to_string().into();
    match val {
        ContextValue::String(s) => assert_eq!(s, "test"),
        _ => panic!("Expected String variant"),
    }
}

#[test]
fn test_context_value_from_str() {
    let val: ContextValue = "test".into();
    match val {
        ContextValue::String(s) => assert_eq!(s, "test"),
        _ => panic!("Expected String variant"),
    }
}

#[test]
fn test_context_value_from_f64() {
    let val: ContextValue = 42.5f64.into();
    match val {
        ContextValue::Number(n) => assert_eq!(n, 42.5),
        _ => panic!("Expected Number variant"),
    }
}

#[test]
fn test_context_value_from_bool() {
    let val_true: ContextValue = true.into();
    let val_false: ContextValue = false.into();
    
    match val_true {
        ContextValue::Boolean(b) => assert!(b),
        _ => panic!("Expected Boolean variant"),
    }
    
    match val_false {
        ContextValue::Boolean(b) => assert!(!b),
        _ => panic!("Expected Boolean variant"),
    }
}

#[test]
fn test_context_value_from_vec_string() {
    let vec = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let val: ContextValue = vec.into();
    
    match val {
        ContextValue::Array(arr) => assert_eq!(arr.len(), 3),
        _ => panic!("Expected Array variant"),
    }
}

#[test]
fn test_context_value_from_vec_str() {
    let vec = vec!["a", "b", "c"];
    let val: ContextValue = vec.into();
    
    match val {
        ContextValue::Array(arr) => assert_eq!(arr.len(), 3),
        _ => panic!("Expected Array variant"),
    }
}

// ========================================
// Template Edge Cases
// ========================================

#[test]
fn test_template_with_consecutive_variables() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "consecutive",
        "{{first}}{{second}}{{third}}"
    );
    engine.register_template("consecutive", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("first".to_string(), ContextValue::String("A".to_string()));
    ctx.set("second".to_string(), ContextValue::String("B".to_string()));
    ctx.set("third".to_string(), ContextValue::String("C".to_string()));
    
    let result = engine.render("consecutive", &ctx).unwrap();
    assert_eq!(result, "ABC");
}

#[test]
fn test_template_with_only_whitespace() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new("whitespace", "   \n\t\n   ");
    engine.register_template("whitespace", template).unwrap();
    
    let ctx = PromptContext::new();
    let result = engine.render("whitespace", &ctx).unwrap();
    assert!(result.chars().all(|c| c.is_whitespace()));
}

#[test]
fn test_template_with_numeric_string_variables() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new(
        "numbers",
        "First: {{var1}}, Second: {{var2}}, Third: {{var3}}"
    );
    engine.register_template("numbers", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("var1".to_string(), ContextValue::String("123".to_string()));
    ctx.set("var2".to_string(), ContextValue::Number(456.0));
    ctx.set("var3".to_string(), ContextValue::String("789".to_string()));
    
    let result = engine.render("numbers", &ctx).unwrap();
    assert!(result.contains("First: 123"));
    assert!(result.contains("Second: 456"));
    assert!(result.contains("Third: 789"));
}

#[test]
fn test_template_with_empty_variable_value() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new("empty_val", "Value: [{{value}}]");
    engine.register_template("empty_val", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("value".to_string(), ContextValue::String("".to_string()));
    
    let result = engine.render("empty_val", &ctx).unwrap();
    assert_eq!(result, "Value: []");
}

#[test]
fn test_template_with_zero_number() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new("zero", "Count: {{count}}");
    engine.register_template("zero", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("count".to_string(), ContextValue::Number(0.0));
    
    let result = engine.render("zero", &ctx).unwrap();
    assert!(result.contains("Count: 0"));
}

#[test]
fn test_template_with_negative_number() {
    let mut engine = TemplateEngine::new();
    
    let template = PromptTemplate::new("negative", "Temperature: {{temp}}");
    engine.register_template("negative", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("temp".to_string(), ContextValue::Number(-5.5));
    
    let result = engine.render("negative", &ctx).unwrap();
    assert!(result.contains("-5.5"));
}

#[test]
fn test_template_with_very_long_variable_name() {
    let mut engine = TemplateEngine::new();
    
    let long_var = "this_is_a_very_long_variable_name_with_many_characters_for_testing";
    let template = PromptTemplate::new(
        "long_var",
        format!("Value: {{{{{}}}}}", long_var)
    );
    engine.register_template("long_var", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set(long_var.to_string(), ContextValue::String("test_value".to_string()));
    
    let result = engine.render("long_var", &ctx).unwrap();
    assert!(result.contains("test_value"));
}

// ========================================
// Context Path Operations
// ========================================

#[test]
fn test_context_set_path_overwrite() {
    let mut ctx = PromptContext::new();
    
    ctx.set_path("user.name", ContextValue::String("Alice".to_string()));
    ctx.set_path("user.name", ContextValue::String("Bob".to_string()));
    
    // The second set should overwrite the first
    let json = ctx.to_json();
    assert_eq!(json["user"]["name"], "Bob");
}

#[test]
fn test_context_set_path_deep_nesting() {
    let mut ctx = PromptContext::new();
    
    ctx.set_path(
        "level1.level2.level3.level4.level5.value",
        ContextValue::String("deep".to_string())
    );
    
    let json = ctx.to_json();
    assert_eq!(json["level1"]["level2"]["level3"]["level4"]["level5"]["value"], "deep");
}

#[test]
fn test_context_set_path_multiple_siblings() {
    let mut ctx = PromptContext::new();
    
    ctx.set_path("user.first_name", ContextValue::String("Alice".to_string()));
    ctx.set_path("user.last_name", ContextValue::String("Smith".to_string()));
    ctx.set_path("user.age", ContextValue::Number(30.0));
    
    let json = ctx.to_json();
    assert_eq!(json["user"]["first_name"], "Alice");
    assert_eq!(json["user"]["last_name"], "Smith");
    assert_eq!(json["user"]["age"], 30.0);
}

// ========================================
// Array Operations
// ========================================

#[test]
fn test_context_empty_array() {
    let mut ctx = PromptContext::new();
    ctx.set("items".to_string(), ContextValue::Array(vec![]));
    
    let json = ctx.to_json();
    assert!(json["items"].is_array());
    assert_eq!(json["items"].as_array().unwrap().len(), 0);
}

#[test]
fn test_context_mixed_type_array() {
    let mut ctx = PromptContext::new();
    
    let arr = ContextValue::Array(vec![
        ContextValue::String("text".to_string()),
        ContextValue::Number(42.0),
        ContextValue::Boolean(true),
    ]);
    
    ctx.set("mixed".to_string(), arr);
    
    let json = ctx.to_json();
    let array = json["mixed"].as_array().unwrap();
    assert_eq!(array.len(), 3);
    assert_eq!(array[0], "text");
    assert_eq!(array[1], 42.0);
    assert_eq!(array[2], true);
}

#[test]
fn test_context_nested_arrays() {
    let mut ctx = PromptContext::new();
    
    let inner = ContextValue::Array(vec![
        ContextValue::String("a".to_string()),
        ContextValue::String("b".to_string()),
    ]);
    
    let outer = ContextValue::Array(vec![
        inner.clone(),
        inner.clone(),
    ]);
    
    ctx.set("nested".to_string(), outer);
    
    let json = ctx.to_json();
    assert!(json["nested"].is_array());
    assert!(json["nested"][0].is_array());
}

// ========================================
// Object Operations
// ========================================

#[test]
fn test_context_empty_object() {
    let mut ctx = PromptContext::new();
    
    use std::collections::HashMap;
    let empty_obj = ContextValue::Object(HashMap::new());
    ctx.set("obj".to_string(), empty_obj);
    
    let json = ctx.to_json();
    assert!(json["obj"].is_object());
    assert_eq!(json["obj"].as_object().unwrap().len(), 0);
}

#[test]
fn test_context_nested_objects() {
    let mut ctx = PromptContext::new();
    
    use std::collections::HashMap;
    
    let mut inner = HashMap::new();
    inner.insert("key".to_string(), ContextValue::String("value".to_string()));
    
    let mut outer = HashMap::new();
    outer.insert("inner".to_string(), ContextValue::Object(inner));
    
    ctx.set("outer".to_string(), ContextValue::Object(outer));
    
    let json = ctx.to_json();
    assert_eq!(json["outer"]["inner"]["key"], "value");
}

// ========================================
// Scope Management Edge Cases
// ========================================

#[test]
fn test_scope_variable_shadowing() {
    let mut ctx = PromptContext::new();
    
    ctx.set("x".to_string(), ContextValue::String("global".to_string()));
    
    ctx.push_scope();
    ctx.set("x".to_string(), ContextValue::String("local".to_string()));
    
    // Should get local value
    let val = ctx.get("x").unwrap();
    assert_eq!(val.to_string(), "local");
    
    ctx.pop_scope();
    
    // Should get global value again
    let val = ctx.get("x").unwrap();
    assert_eq!(val.to_string(), "global");
}

#[test]
fn test_multiple_scope_pops() {
    let mut ctx = PromptContext::new();
    
    ctx.set("level0".to_string(), ContextValue::String("L0".to_string()));
    
    ctx.push_scope();
    ctx.set("level1".to_string(), ContextValue::String("L1".to_string()));
    
    ctx.push_scope();
    ctx.set("level2".to_string(), ContextValue::String("L2".to_string()));
    
    // Pop twice
    ctx.pop_scope();
    ctx.pop_scope();
    
    // Should have level0
    assert!(ctx.get("level0").is_some());
}

#[test]
fn test_pop_scope_returns_correct_variables() {
    let mut ctx = PromptContext::new();
    
    ctx.push_scope();
    ctx.set("local1".to_string(), ContextValue::String("value1".to_string()));
    ctx.set("local2".to_string(), ContextValue::String("value2".to_string()));
    
    let popped = ctx.pop_scope();
    assert!(popped.is_some());
    
    let vars = popped.unwrap();
    assert!(vars.contains_key("local1"));
    assert!(vars.contains_key("local2"));
}

// ========================================
// Context Merge Tests
// ========================================

#[test]
fn test_merge_overwrites_existing_keys() {
    let mut ctx1 = PromptContext::new();
    ctx1.set("key".to_string(), ContextValue::String("value1".to_string()));
    
    let mut ctx2 = PromptContext::new();
    ctx2.set("key".to_string(), ContextValue::String("value2".to_string()));
    
    ctx1.merge(ctx2);
    
    // ctx2's value should overwrite ctx1's
    let val = ctx1.get("key").unwrap();
    assert_eq!(val.to_string(), "value2");
}

#[test]
fn test_merge_combines_different_keys() {
    let mut ctx1 = PromptContext::new();
    ctx1.set("key1".to_string(), ContextValue::String("value1".to_string()));
    
    let mut ctx2 = PromptContext::new();
    ctx2.set("key2".to_string(), ContextValue::String("value2".to_string()));
    
    ctx1.merge(ctx2);
    
    assert!(ctx1.get("key1").is_some());
    assert!(ctx1.get("key2").is_some());
}

#[test]
fn test_merge_with_empty_context() {
    let mut ctx1 = PromptContext::new();
    ctx1.set("key".to_string(), ContextValue::String("value".to_string()));
    
    let ctx2 = PromptContext::new();
    
    ctx1.merge(ctx2);
    
    assert!(ctx1.get("key").is_some());
}

// ========================================
// Template Metadata Tests
// ========================================

#[test]
fn test_template_with_metadata_accessor() {
    use astraweave_prompts::TemplateMetadata;
    
    let mut meta = TemplateMetadata::default();
    meta.name = "test".to_string();
    meta.description = "Test description".to_string();
    
    let template = PromptTemplate::new_with_metadata(
        "test",
        "Hello {{name}}",
        meta
    );
    
    let retrieved_meta = template.metadata();
    assert_eq!(retrieved_meta.name, "test");
    assert_eq!(retrieved_meta.description, "Test description");
}

#[test]
fn test_template_with_metadata_builder() {
    use astraweave_prompts::{TemplateMetadata, TemplateCategory};
    
    let mut meta = TemplateMetadata::default();
    meta.name = "builder_test".to_string();
    meta.category = TemplateCategory::Dialogue;
    
    let template = PromptTemplate::new("test", "Content")
        .with_metadata(meta.clone());
    
    assert_eq!(template.metadata().category, TemplateCategory::Dialogue);
}

// ========================================
// Serialization Tests
// ========================================

#[test]
fn test_template_serialization_roundtrip() {
    let template = PromptTemplate::new("test", "Hello {{name}}");
    
    let json = serde_json::to_string(&template).unwrap();
    let deserialized: PromptTemplate = serde_json::from_str(&json).unwrap();
    
    assert_eq!(template.id(), deserialized.id());
    assert_eq!(template.template(), deserialized.template());
}

#[test]
fn test_context_value_serialization_all_types() {
    use std::collections::HashMap;
    
    let values = vec![
        ContextValue::String("test".to_string()),
        ContextValue::Number(42.0),
        ContextValue::Boolean(true),
        ContextValue::Array(vec![ContextValue::String("item".to_string())]),
        ContextValue::Object({
            let mut map = HashMap::new();
            map.insert("key".to_string(), ContextValue::String("value".to_string()));
            map
        }),
    ];
    
    for val in values {
        let json = serde_json::to_string(&val).unwrap();
        let deserialized: ContextValue = serde_json::from_str(&json).unwrap();
        // Just verify it deserializes without error
        drop(deserialized);
    }
}
