use astraweave_prompts::template::{PromptTemplate, TemplateProcessor, ProcessorConfig};
use astraweave_prompts::context::PromptContext;
use std::collections::HashMap;

#[test]
fn test_template_creation() {
    let t = PromptTemplate::new("test", "Hello {{name}}!");
    assert_eq!(t.id, "test");
    assert_eq!(t.template, "Hello {{name}}!");
    assert_eq!(t.variables, vec!["name"]);
}

#[test]
fn test_variable_extraction() {
    let t = PromptTemplate::new("test", "{{greeting}} {{name}}, welcome to {{place}}!");
    assert!(t.variables.contains(&"greeting".to_string()));
    assert!(t.variables.contains(&"name".to_string()));
    assert!(t.variables.contains(&"place".to_string()));
    assert_eq!(t.variables.len(), 3);
}

#[test]
fn test_render_simple() {
    let t = PromptTemplate::new("test", "Hello {{name}}!");
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), "World".into());

    let result = t.render(&ctx).unwrap();
    assert_eq!(result, "Hello World!");
}

#[test]
fn test_render_map() {
    let t = PromptTemplate::new("test", "Hello {{name}}!");
    let mut map = HashMap::new();
    map.insert("name".to_string(), "Map".to_string());

    let result = t.render_map(&map).unwrap();
    assert_eq!(result, "Hello Map!");
}

#[test]
fn test_processor_validation() {
    let proc = TemplateProcessor::new(ProcessorConfig::default());
    
    assert!(proc.validate_template("Valid {{template}}").is_ok());
    // Handlebars might be more permissive, but let's check basic syntax
    assert!(proc.validate_template("Invalid {{template").is_err());
}

#[test]
fn test_handlebars_syntax() {
    // Verify Handlebars syntax works
    let t = PromptTemplate::new("test", "Hello {{name}}!");
    assert!(t.variables.contains(&"name".to_string()));
    
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), "World".into());
    
    let result = t.render(&ctx).unwrap();
    assert_eq!(result, "Hello World!");
}
