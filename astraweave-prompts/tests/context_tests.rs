use astraweave_prompts::context::{ContextValue, PromptContext};
use std::collections::HashMap;

#[test]
fn test_context_basic_ops() {
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), "World".into());
    ctx.set("count".to_string(), 42.0.into());
    ctx.set("active".to_string(), true.into());

    assert_eq!(
        ctx.get("name").unwrap().to_string(),
        "World"
    );
    assert_eq!(
        ctx.get("count").unwrap().to_string(),
        "42"
    );
    assert_eq!(
        ctx.get("active").unwrap().to_string(),
        "true"
    );
    assert!(ctx.get("missing").is_none());
}

#[test]
fn test_context_scopes() {
    let mut ctx = PromptContext::new();
    ctx.set("global".to_string(), "global_val".into());
    ctx.set("shadowed".to_string(), "global_shadow".into());

    ctx.push_scope();
    ctx.set("local".to_string(), "local_val".into());
    ctx.set("shadowed".to_string(), "local_shadow".into());

    // Check local scope visibility
    assert_eq!(ctx.get("local").unwrap().to_string(), "local_val");
    assert_eq!(ctx.get("global").unwrap().to_string(), "global_val");
    assert_eq!(ctx.get("shadowed").unwrap().to_string(), "local_shadow");

    // Pop scope
    let popped = ctx.pop_scope().unwrap();
    assert_eq!(popped.get("local").unwrap().to_string(), "local_val");

    // Check global scope restoration
    assert!(ctx.get("local").is_none());
    assert_eq!(ctx.get("global").unwrap().to_string(), "global_val");
    assert_eq!(ctx.get("shadowed").unwrap().to_string(), "global_shadow");
}

#[test]
fn test_context_complex_types() {
    let mut ctx = PromptContext::new();
    
    // Array
    let arr = vec![
        ContextValue::from("item1"),
        ContextValue::from("item2"),
    ];
    ctx.set("list".to_string(), ContextValue::Array(arr));

    // Object
    let mut obj = HashMap::new();
    obj.insert("key".to_string(), ContextValue::from("value"));
    ctx.set("map".to_string(), ContextValue::Object(obj));

    assert_eq!(ctx.get("list").unwrap().to_string(), "[item1, item2]");
    assert_eq!(ctx.get("map").unwrap().to_string(), "{key: value}");
}

#[test]
fn test_to_string_map() {
    let mut ctx = PromptContext::new();
    ctx.set("a".to_string(), "1".into());
    
    ctx.push_scope();
    ctx.set("b".to_string(), "2".into());

    let map = ctx.to_string_map();
    assert_eq!(map.get("a").unwrap(), "1");
    assert_eq!(map.get("b").unwrap(), "2");
}
