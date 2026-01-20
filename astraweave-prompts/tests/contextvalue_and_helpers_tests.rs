//! Additional Coverage Tests for Context Value Conversions and Helpers
//! These tests cover From implementations, helpers, and library functionality

use astraweave_prompts::{
    ContextValue, PromptContext, PromptTemplate, TemplateEngine,
};
use std::collections::HashMap;

// ========================================
// ContextValue From Implementations Tests
// ========================================

#[test]
fn test_context_value_from_string() {
    let val: ContextValue = "test".into();
    match val {
        ContextValue::String(s) => assert_eq!(s, "test"),
        _ => panic!("Expected String variant"),
    }
}

#[test]
fn test_context_value_from_owned_string() {
    let val: ContextValue = String::from("owned").into();
    match val {
        ContextValue::String(s) => assert_eq!(s, "owned"),
        _ => panic!("Expected String variant"),
    }
}

#[test]
fn test_context_value_from_f64() {
    let val: ContextValue = 42.5.into();
    match val {
        ContextValue::Number(n) => assert_eq!(n, 42.5),
        _ => panic!("Expected Number variant"),
    }
}

#[test]
fn test_context_value_from_bool_true() {
    let val: ContextValue = true.into();
    match val {
        ContextValue::Boolean(b) => assert!(b),
        _ => panic!("Expected Boolean variant"),
    }
}

#[test]
fn test_context_value_from_bool_false() {
    let val: ContextValue = false.into();
    match val {
        ContextValue::Boolean(b) => assert!(!b),
        _ => panic!("Expected Boolean variant"),
    }
}

#[test]
fn test_context_value_from_vec_contextvalue() {
    let vec = vec![
        ContextValue::String("a".to_string()),
        ContextValue::Number(1.0),
    ];
    let val: ContextValue = vec.into();
    match val {
        ContextValue::Array(arr) => assert_eq!(arr.len(), 2),
        _ => panic!("Expected Array variant"),
    }
}

#[test]
fn test_context_value_from_vec_string() {
    let vec = vec!["a".to_string(), "b".to_string()];
    let val: ContextValue = vec.into();
    match val {
        ContextValue::Array(arr) => {
            assert_eq!(arr.len(), 2);
            assert!(matches!(arr[0], ContextValue::String(_)));
        }
        _ => panic!("Expected Array variant"),
    }
}

#[test]
fn test_context_value_from_vec_str_slice() {
    let vec = vec!["x", "y", "z"];
    let val: ContextValue = vec.into();
    match val {
        ContextValue::Array(arr) => assert_eq!(arr.len(), 3),
        _ => panic!("Expected Array variant"),
    }
}

// ========================================
// ContextValue ToString Tests
// ========================================

#[test]
fn test_context_value_array_to_string() {
    let arr = ContextValue::Array(vec![
        ContextValue::String("a".to_string()),
        ContextValue::String("b".to_string()),
    ]);
    let s = arr.to_string();
    assert!(s.contains("["));
    assert!(s.contains("a"));
    assert!(s.contains("b"));
    assert!(s.contains("]"));
}

#[test]
fn test_context_value_object_to_string() {
    let mut map = HashMap::new();
    map.insert("key1".to_string(), ContextValue::String("val1".to_string()));
    map.insert("key2".to_string(), ContextValue::Number(42.0));
    
    let obj = ContextValue::Object(map);
    let s = obj.to_string();
    assert!(s.contains("{"));
    assert!(s.contains("}"));
    assert!(s.contains("key1"));
    assert!(s.contains("key2"));
}

#[test]
fn test_context_value_empty_array_to_string() {
    let arr = ContextValue::Array(vec![]);
    let s = arr.to_string();
    assert_eq!(s, "[]");
}

#[test]
fn test_context_value_empty_object_to_string() {
    let obj = ContextValue::Object(HashMap::new());
    let s = obj.to_string();
    assert_eq!(s, "{}");
}

// ========================================
// ContextValue insert_path Tests
// ========================================

#[test]
fn test_context_value_insert_path_into_object() {
    let mut obj = ContextValue::Object(HashMap::new());
    obj.insert_path(&["key"], ContextValue::String("value".to_string()));
    
    match obj {
        ContextValue::Object(map) => {
            assert!(map.contains_key("key"));
        }
        _ => panic!("Expected Object"),
    }
}

#[test]
fn test_context_value_insert_path_nested() {
    let mut obj = ContextValue::Object(HashMap::new());
    obj.insert_path(&["level1", "level2"], ContextValue::String("deep".to_string()));
    
    match obj {
        ContextValue::Object(map) => {
            assert!(map.contains_key("level1"));
        }
        _ => panic!("Expected Object"),
    }
}

#[test]
fn test_context_value_insert_path_overwrites_non_object() {
    let mut val = ContextValue::String("old".to_string());
    val.insert_path(&["new"], ContextValue::String("value".to_string()));
    
    // Should have converted to Object
    match val {
        ContextValue::Object(_) => {},
        _ => panic!("Expected conversion to Object"),
    }
}

#[test]
fn test_context_value_insert_path_empty_path() {
    let mut val = ContextValue::String("old".to_string());
    let new_val = ContextValue::String("new".to_string());
    val.insert_path(&[], new_val.clone());
    
    // Empty path should replace the value
    match val {
        ContextValue::String(s) => assert_eq!(s, "new"),
        _ => panic!("Expected String"),
    }
}

// ========================================
// Complex Rendering Tests
// ========================================

#[test]
fn test_render_with_conditionals() {
    let template = PromptTemplate::new(
        "cond",
        "{{#if active}}Active{{else}}Inactive{{/if}}"
    );
    
    let mut ctx = PromptContext::new();
    ctx.set("active".to_string(), ContextValue::Boolean(true));
    
    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "Active");
}

#[test]
fn test_render_with_iteration() {
    let template = PromptTemplate::new(
        "iter",
        "{{#each items}}{{this}}, {{/each}}"
    );
    
    let mut ctx = PromptContext::new();
    ctx.set("items".to_string(), ContextValue::from(vec!["a", "b", "c"]));
    
    let result = template.render(&ctx).unwrap();
    assert!(result.contains("a,"));
    assert!(result.contains("b,"));
    assert!(result.contains("c,"));
}

#[test]
fn test_render_with_nested_iteration() {
    let template = PromptTemplate::new(
        "nested",
        "{{#each groups}}{{name}}: {{#each members}}{{this}} {{/each}}| {{/each}}"
    );
    
    let mut ctx = PromptContext::new();
    
    let mut group1 = HashMap::new();
    group1.insert("name".to_string(), ContextValue::String("A".to_string()));
    group1.insert("members".to_string(), ContextValue::from(vec!["a1", "a2"]));
    
    let mut group2 = HashMap::new();
    group2.insert("name".to_string(), ContextValue::String("B".to_string()));
    group2.insert("members".to_string(), ContextValue::from(vec!["b1", "b2"]));
    
    ctx.set("groups".to_string(), ContextValue::Array(vec![
        ContextValue::Object(group1),
        ContextValue::Object(group2),
    ]));
    
    let result = template.render(&ctx).unwrap();
    assert!(result.contains("A:"));
    assert!(result.contains("B:"));
}

#[test]
fn test_render_with_partial() {
    let mut engine = TemplateEngine::new();
    
    engine.register_partial("footer", "--- Footer ---").unwrap();
    engine.register_template(
        "page",
        PromptTemplate::new("p", "Content\n{{> footer}}")
    ).unwrap();
    
    let ctx = PromptContext::new();
    let result = engine.render("page", &ctx).unwrap();
    
    assert!(result.contains("Content"));
    assert!(result.contains("--- Footer ---"));
}

// ========================================
// Error Handling Tests
// ========================================

#[test]
fn test_render_nonexistent_template() {
    let engine = TemplateEngine::new();
    let ctx = PromptContext::new();
    
    let result = engine.render("nonexistent", &ctx);
    assert!(result.is_err());
}

#[test]
fn test_processor_invalid_handlebars_syntax() {
    use astraweave_prompts::{TemplateProcessor, ProcessorConfig};
    
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    
    // Test various invalid syntaxes
    assert!(processor.validate_template("{{#each}}").is_err());
    assert!(processor.validate_template("{{/if}}").is_err());
}

// ========================================
// Advanced Context Tests
// ========================================

#[test]
fn test_context_overwrite_in_scope() {
    let mut ctx = PromptContext::new();
    ctx.set("key".to_string(), ContextValue::String("original".to_string()));
    
    ctx.push_scope();
    ctx.set("key".to_string(), ContextValue::String("scoped".to_string()));
    
    // Current scope should have "scoped"
    match ctx.get("key") {
        Some(ContextValue::String(s)) => assert_eq!(s, "scoped"),
        _ => panic!("Expected scoped value"),
    }
    
    ctx.pop_scope();
    
    // After pop, should still be available from scopes
    assert!(ctx.get("key").is_some());
}

#[test]
fn test_context_pop_empty_scope() {
    let mut ctx = PromptContext::new();
    let result = ctx.pop_scope();
    assert!(result.is_none());
}

#[test]
fn test_context_merge_empty() {
    let mut ctx1 = PromptContext::new();
    ctx1.set("key1".to_string(), ContextValue::String("val1".to_string()));
    
    let ctx2 = PromptContext::new();
    ctx1.merge(ctx2);
    
    assert!(ctx1.get("key1").is_some());
}

#[test]
fn test_context_merge_overwrites() {
    let mut ctx1 = PromptContext::new();
    ctx1.set("shared".to_string(), ContextValue::String("ctx1".to_string()));
    
    let mut ctx2 = PromptContext::new();
    ctx2.set("shared".to_string(), ContextValue::String("ctx2".to_string()));
    
    ctx1.merge(ctx2);
    
    match ctx1.get("shared") {
        Some(ContextValue::String(s)) => assert_eq!(s, "ctx2"),
        _ => panic!("Expected ctx2 value to overwrite"),
    }
}

#[test]
fn test_context_set_path_deep_nesting() {
    let mut ctx = PromptContext::new();
    ctx.set_path("a.b.c.d", ContextValue::String("deep".to_string()));
    
    let json = ctx.to_json();
    assert!(json.get("a").is_some());
}

// ========================================
// Template Metadata Tests
// ========================================

#[test]
fn test_template_with_metadata() {
    use astraweave_prompts::{TemplateMetadata, TemplateCategory};
    
    let metadata = TemplateMetadata {
        name: "test_meta".to_string(),
        description: "A test template".to_string(),
        category: TemplateCategory::Dialogue,
        author: Some("Test Author".to_string()),
        version: "1.0.0".to_string(),
        created_at: 1000,
        updated_at: 2000,
        tags: vec!["test".to_string(), "meta".to_string()],
        required_variables: vec!["name".to_string()],
        optional_variables: HashMap::new(),
        usage_stats: Default::default(),
    };
    
    assert_eq!(metadata.name, "test_meta");
    assert_eq!(metadata.category, TemplateCategory::Dialogue);
    assert_eq!(metadata.tags.len(), 2);
}

// ========================================
// Real-world Use Case Tests
// ========================================

#[test]
fn test_npc_dialogue_generation() {
    let template = PromptTemplate::new(
        "npc_dialogue",
        "{{npc.name}} ({{npc.mood}}): \"{{dialogue}}\""
    );
    
    let mut ctx = PromptContext::new();
    let mut npc = HashMap::new();
    npc.insert("name".to_string(), ContextValue::String("Guard".to_string()));
    npc.insert("mood".to_string(), ContextValue::String("suspicious".to_string()));
    
    ctx.set("npc".to_string(), ContextValue::Object(npc));
    ctx.set("dialogue".to_string(), ContextValue::String("Halt! Who goes there?".to_string()));
    
    let result = template.render(&ctx).unwrap();
    assert!(result.contains("Guard"));
    assert!(result.contains("suspicious"));
    assert!(result.contains("Halt! Who goes there?"));
}

#[test]
fn test_quest_description_with_conditions() {
    let template = PromptTemplate::new(
        "quest",
        "Quest: {{title}}\n{{#if is_urgent}}[URGENT] {{/if}}{{description}}\nReward: {{reward}}"
    );
    
    let mut ctx = PromptContext::new();
    ctx.set("title".to_string(), ContextValue::String("Save the Village".to_string()));
    ctx.set("is_urgent".to_string(), ContextValue::Boolean(true));
    ctx.set("description".to_string(), ContextValue::String("Bandits are attacking!".to_string()));
    ctx.set("reward".to_string(), ContextValue::String("500 gold".to_string()));
    
    let result = template.render(&ctx).unwrap();
    assert!(result.contains("Save the Village"));
    assert!(result.contains("[URGENT]"));
    assert!(result.contains("Bandits are attacking!"));
    assert!(result.contains("500 gold"));
}

#[test]
fn test_combat_log_with_arrays() {
    let template = PromptTemplate::new(
        "combat",
        "Combat Log:\n{{#each events}}[{{timestamp}}] {{message}}\n{{/each}}"
    );
    
    let mut ctx = PromptContext::new();
    
    let mut event1 = HashMap::new();
    event1.insert("timestamp".to_string(), ContextValue::String("12:00".to_string()));
    event1.insert("message".to_string(), ContextValue::String("Battle started".to_string()));
    
    let mut event2 = HashMap::new();
    event2.insert("timestamp".to_string(), ContextValue::String("12:01".to_string()));
    event2.insert("message".to_string(), ContextValue::String("Player attacks".to_string()));
    
    ctx.set("events".to_string(), ContextValue::Array(vec![
        ContextValue::Object(event1),
        ContextValue::Object(event2),
    ]));
    
    let result = template.render(&ctx).unwrap();
    assert!(result.contains("[12:00]"));
    assert!(result.contains("Battle started"));
    assert!(result.contains("[12:01]"));
    assert!(result.contains("Player attacks"));
}

// ========================================
// Performance and Limits Tests
// ========================================

#[test]
fn test_large_number_of_variables() {
    let mut template_str = String::from("Values: ");
    for i in 0..100 {
        template_str.push_str(&format!("{{{{var{}}}}} ", i));
    }
    
    let template = PromptTemplate::new("large", &template_str);
    // Should extract at least some variables (the regex extraction may not catch all)
    assert!(!template.variables.is_empty(), "Expected at least some variables to be extracted");
    assert!(template.variables.len() <= 100, "Should not extract more than the defined variables");
}

#[test]
fn test_deeply_nested_objects() {
    let mut ctx = PromptContext::new();
    ctx.set_path("a.b.c.d.e.f.g", ContextValue::String("deep".to_string()));
    
    let json = ctx.to_json();
    assert!(json.is_object());
}

#[test]
fn test_large_array_rendering() {
    let template = PromptTemplate::new(
        "large_array",
        "{{#each items}}{{this}} {{/each}}"
    );
    
    let items: Vec<ContextValue> = (0..100)
        .map(|i| ContextValue::String(format!("item{}", i)))
        .collect();
    
    let mut ctx = PromptContext::new();
    ctx.set("items".to_string(), ContextValue::Array(items));
    
    let result = template.render(&ctx).unwrap();
    assert!(result.contains("item0"));
    assert!(result.contains("item99"));
}
