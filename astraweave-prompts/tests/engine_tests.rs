use astraweave_prompts::engine::{TemplateEngine, EngineConfig, PromptEngine};
use astraweave_prompts::template::PromptTemplate;
use astraweave_prompts::context::PromptContext;

#[test]
fn test_engine_registration() {
    let mut engine = TemplateEngine::new();
    let t = PromptTemplate::new("greet", "Hello {{name}}");
    
    assert!(engine.register_template("greet", t).is_ok());
}

#[test]
fn test_engine_render() {
    let mut engine = TemplateEngine::new();
    let t = PromptTemplate::new("greet", "Hello {{name}}");
    engine.register_template("greet", t).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), "Engine".into());

    let result = engine.render("greet", &ctx).unwrap();
    assert_eq!(result, "Hello Engine");
}

#[test]
fn test_engine_missing_template() {
    let engine = TemplateEngine::new();
    let ctx = PromptContext::new();
    
    assert!(engine.render("missing", &ctx).is_err());
}

#[test]
fn test_prompt_engine_limits() {
    let config = EngineConfig {
        max_template_size: 10,
        enable_caching: true,
    };
    let mut engine = PromptEngine::new(config);
    
    // "Hello World" is 11 bytes, should fail
    let result = engine.register_template("large".to_string(), "Hello World".to_string());
    assert!(result.is_err());
}

#[test]
fn test_load_templates_from_dir() {
    use std::io::Write;
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let p1 = dir.path().join("t1.hbs");
    let mut f1 = std::fs::File::create(&p1).unwrap();
    writeln!(f1, "Template 1").unwrap();

    let mut engine = TemplateEngine::new();
    engine.load_templates_from_dir(dir.path()).unwrap();

    let ctx = PromptContext::new();
    let result = engine.render("t1", &ctx).unwrap();
    assert_eq!(result.trim(), "Template 1");
}
