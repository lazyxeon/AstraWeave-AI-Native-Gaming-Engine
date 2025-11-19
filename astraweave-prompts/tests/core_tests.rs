/// Prompts Core Tests - Sprint 2
/// High-value tests for TemplateEngine, PromptTemplate, TemplateContext

use astraweave_prompts::{
    PromptContext, PromptTemplate, TemplateEngine, TemplateProcessor, ProcessorConfig,
};
use std::collections::HashMap;

#[test]
fn test_context_new() {
    let ctx = PromptContext::new();
    assert!(ctx.get("nonexistent").is_none());
}

#[test]
fn test_context_set_get() {
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), "Test".into());
    assert!(ctx.get("name").is_some());
}

#[test]
fn test_template_new() {
    let template = PromptTemplate::new("test", "Hello {{name}}!");
    assert_eq!(template.id(), "test");
}

#[test]
fn test_template_render() {
    let template = PromptTemplate::new("test", "Hello {{name}}!");
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), "World".into());
    
    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_engine_register() {
    let mut engine = TemplateEngine::new();
    let template = PromptTemplate::new("test", "Content");
    assert!(engine.register_template("test", template).is_ok());
}

#[test]
fn test_engine_render() {
    let mut engine = TemplateEngine::new();
    engine.register_template("test", PromptTemplate::new("1", "Hello {{x}}!")).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("x".to_string(), "There".into());
    
    let result = engine.render("test", &ctx).unwrap();
    assert_eq!(result, "Hello There!");
}

#[test]
fn test_processor_validate() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    assert!(processor.validate_template("{{var}}").is_ok());
    assert!(processor.validate_template("{{unclosed").is_err());
}

#[test]
fn test_processor_extract_vars() {
    let processor = TemplateProcessor::new(ProcessorConfig::default());
    let vars = processor.extract_variables("{{a}} and {{b}}");
    assert!(vars.len() >= 2);
}

#[test]
fn test_context_scopes() {
    let mut ctx = PromptContext::new();
    ctx.set("global".to_string(), "val".into());
    ctx.push_scope();
    assert!(ctx.get("global").is_some());
    ctx.pop_scope();
}

#[test]
fn test_template_map_render() {
    let template = PromptTemplate::new("test", "{{key}}");
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    let result = template.render_map(&map).unwrap();
    assert_eq!(result, "value");
}
