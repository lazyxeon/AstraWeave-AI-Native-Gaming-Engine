use astraweave_prompts::engine::TemplateEngine;
use astraweave_prompts::template::PromptTemplate;
use astraweave_prompts::context::{PromptContext, ContextValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

// --- TemplateEngine Tests ---

#[test]
fn test_render_undefined_template() {
    let engine = TemplateEngine::new();
    let ctx = PromptContext::new();
    let result = engine.render("non_existent", &ctx);
    assert!(result.is_err());
}

#[test]
fn test_register_duplicate_template() {
    let mut engine = TemplateEngine::new();
    let t1 = PromptTemplate::new("test", "Version 1");
    let t2 = PromptTemplate::new("test", "Version 2");
    
    engine.register_template("test", t1).unwrap();
    // Should overwrite or fail depending on implementation. 
    // Usually overwrite is expected behavior for a registry unless strict.
    // Let's assume overwrite for now based on typical map behavior.
    engine.register_template("test", t2).unwrap();
    
    let ctx = PromptContext::new();
    let result = engine.render("test", &ctx).unwrap();
    assert_eq!(result, "Version 2");
}

#[test]
fn test_engine_list_templates() {
    let mut engine = TemplateEngine::new();
    engine.register_template("t1", PromptTemplate::new("t1", "A")).unwrap();
    engine.register_template("t2", PromptTemplate::new("t2", "B")).unwrap();
    
    let templates = engine.list_templates();
    assert!(templates.contains(&"t1".to_string()));
    assert!(templates.contains(&"t2".to_string()));
    assert_eq!(templates.len(), 2);
}

#[test]
fn test_engine_thread_safety() {
    let engine = Arc::new(Mutex::new(TemplateEngine::new()));
    
    {
        let mut e = engine.lock().unwrap();
        e.register_template("thread_test", PromptTemplate::new("thread_test", "Hello {{id}}")).unwrap();
    }
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let engine_clone = engine.clone();
        handles.push(thread::spawn(move || {
            let e = engine_clone.lock().unwrap();
            let mut ctx = PromptContext::new();
            ctx.set("id".to_string(), (i as f64).into());
            e.render("thread_test", &ctx).unwrap()
        }));
    }
    
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.starts_with("Hello"));
    }
}

#[test]
fn test_engine_clear_cache() {
    let mut engine = TemplateEngine::new();
    let template = PromptTemplate::new("test", "Hello {{name}}");
    engine.register_template("test", template).unwrap();

    assert_eq!(engine.list_templates().len(), 1);

    engine.clear_templates();
    assert_eq!(engine.list_templates().len(), 0);
}

// --- PromptTemplate Tests ---

#[test]
fn test_render_with_multiple_variables() {
    let t = PromptTemplate::new("test", "{{greeting}} {{name}}");
    let mut ctx = PromptContext::new();
    ctx.set("greeting".to_string(), "Hi".into());
    ctx.set("name".to_string(), "User".into());
    
    let result = t.render(&ctx).unwrap();
    assert_eq!(result, "Hi User");
}

#[test]
fn test_nested_variable_support() {
    let t = PromptTemplate::new("test", "User: {{user.name}}, Role: {{user.role}}");
    let mut ctx = PromptContext::new();
    
    let mut user_obj = HashMap::new();
    user_obj.insert("name".to_string(), ContextValue::String("Alice".to_string()));
    user_obj.insert("role".to_string(), ContextValue::String("Admin".to_string()));
    
    ctx.set("user".to_string(), ContextValue::Object(user_obj));
    
    let result = t.render(&ctx).unwrap();
    assert_eq!(result, "User: Alice, Role: Admin");
}

#[test]
fn test_conditional_rendering() {
    let t = PromptTemplate::new("test", "{{#if show}}Visible{{else}}Hidden{{/if}}");
    
    let mut ctx_true = PromptContext::new();
    ctx_true.set("show".to_string(), true.into());
    assert_eq!(t.render(&ctx_true).unwrap(), "Visible");
    
    let mut ctx_false = PromptContext::new();
    ctx_false.set("show".to_string(), false.into());
    assert_eq!(t.render(&ctx_false).unwrap(), "Hidden");
}

#[test]
fn test_loop_rendering() {
    let t = PromptTemplate::new("test", "{{#each items}}- {{this}}\n{{/each}}");
    let mut ctx = PromptContext::new();
    let items = vec![
        ContextValue::String("A".to_string()),
        ContextValue::String("B".to_string()),
        ContextValue::String("C".to_string()),
    ];
    ctx.set("items".to_string(), ContextValue::Array(items));
    
    let result = t.render(&ctx).unwrap();
    assert!(result.contains("- A"));
    assert!(result.contains("- B"));
    assert!(result.contains("- C"));
}

#[test]
fn test_escape_braces() {
    // Handlebars uses \{{ to escape
    let t = PromptTemplate::new("test", "\\{{literal}}");
    let ctx = PromptContext::new();
    let result = t.render(&ctx).unwrap();
    assert_eq!(result, "{{literal}}");
}

#[test]
fn test_whitespace_handling() {
    // Handlebars ~ removes whitespace
    let t = PromptTemplate::new("test", "  {{~value~}}  ");
    let mut ctx = PromptContext::new();
    ctx.set("value".to_string(), "content".into());
    
    let result = t.render(&ctx).unwrap();
    assert_eq!(result, "content");
}

#[test]
fn test_comment_stripping() {
    let t = PromptTemplate::new("test", "Visible {{! Comment }}");
    let ctx = PromptContext::new();
    let result = t.render(&ctx).unwrap();
    assert_eq!(result.trim(), "Visible");
}

// --- TemplateContext Tests ---

#[test]
fn test_set_variable_types() {
    let mut ctx = PromptContext::new();
    
    ctx.set("string".to_string(), "text".into());
    ctx.set("number".to_string(), 42.0.into()); // f64
    ctx.set("float".to_string(), 1.234.into());
    ctx.set("bool".to_string(), true.into());
    
    let array = vec![
        ContextValue::Number(1.0),
        ContextValue::Number(2.0),
        ContextValue::Number(3.0),
    ];
    ctx.set("array".to_string(), ContextValue::Array(array));
    
    let t = PromptTemplate::new("test", "{{string}} {{number}} {{float}} {{bool}} {{array.[0]}}");
    let result = t.render(&ctx).unwrap();
    
    assert!(result.contains("text"));
    assert!(result.contains("42"));
    assert!(result.contains("1.234"), "Expected '1.234' in result: {result}");
    assert!(result.contains("true"));
    assert!(result.contains("1"));
}

#[test]
fn test_context_merge() {
    let mut ctx1 = PromptContext::new();
    ctx1.set("var1".to_string(), "value1".into());

    let mut ctx2 = PromptContext::new();
    ctx2.set("var2".to_string(), "value2".into());

    ctx1.merge(ctx2);

    let json = ctx1.to_json();
    let obj = json.as_object().unwrap();
    
    assert_eq!(obj.get("var1").unwrap().as_str().unwrap(), "value1");
    assert_eq!(obj.get("var2").unwrap().as_str().unwrap(), "value2");
}

#[test]
fn test_template_clone() {
    let template = PromptTemplate::new("test", "Hello {{name}}");
    let cloned = template.clone();
    
    assert_eq!(template.id, cloned.id);
    assert_eq!(template.template, cloned.template);
}

#[test]
fn test_template_debug_output() {
    let template = PromptTemplate::new("test", "Hello {{name}}");
    let debug_str = format!("{:?}", template);
    
    assert!(debug_str.contains("PromptTemplate"));
    assert!(debug_str.contains("test"));
    assert!(debug_str.contains("Hello {{name}}"));
}
