use astraweave_prompts::{TemplateEngine, PromptTemplate, PromptContext};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};

#[test]
fn test_custom_helper() {
    let mut engine = TemplateEngine::new();
    
    // Register a simple helper that upper-cases text
    engine.register_helper("upper", Box::new(|h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
        let param = h.param(0).unwrap();
        let value = param.value().as_str().unwrap();
        out.write(value.to_uppercase().as_str())?;
        Ok(())
    }));

    let template = PromptTemplate::new("test", "Hello {{upper name}}!");
    engine.register_template("test", template).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), "world".into());

    let result = engine.render("test", &ctx).unwrap();
    assert_eq!(result, "Hello WORLD!");
}

#[test]
fn test_partials() {
    let mut engine = TemplateEngine::new();
    
    // Register a partial
    engine.register_partial("header", "Welcome to {{place}}").unwrap();
    
    // Use partial in template
    let template = PromptTemplate::new("page", "{{> header}}! Enjoy your stay.");
    engine.register_template("page", template).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("place".to_string(), "AstraWeave".into());

    let result = engine.render("page", &ctx).unwrap();
    assert_eq!(result, "Welcome to AstraWeave! Enjoy your stay.");
}

#[test]
fn test_built_in_helpers() {
    let mut engine = TemplateEngine::new();
    
    // Test #if helper (standard Handlebars)
    let template = PromptTemplate::new("logic", "{{#if show}}Visible{{else}}Hidden{{/if}}");
    engine.register_template("logic", template).unwrap();

    let mut ctx_true = PromptContext::new();
    ctx_true.set("show".to_string(), true.into());
    assert_eq!(engine.render("logic", &ctx_true).unwrap(), "Visible");

    let mut ctx_false = PromptContext::new();
    ctx_false.set("show".to_string(), false.into());
    assert_eq!(engine.render("logic", &ctx_false).unwrap(), "Hidden");
}

#[test]
fn test_default_helpers() {
    let mut engine = TemplateEngine::new();
    
    // Test json helper
    let template = PromptTemplate::new("json_test", "{{json data}}");
    engine.register_template("json_test", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("data".to_string(), vec!["a", "b"].into());
    
    let output = engine.render("json_test", &ctx).unwrap();
    // JSON output format might vary slightly in whitespace, but should contain the data
    assert!(output.contains("["));
    assert!(output.contains("\"a\""));
    assert!(output.contains("\"b\""));
    assert!(output.contains("]"));
    
    // Test trim helper
    let template = PromptTemplate::new("trim_test", "{{trim text}}");
    engine.register_template("trim_test", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), "  hello world  ".into());
    
    let output = engine.render("trim_test", &ctx).unwrap();
    assert_eq!(output, "hello world");
    
    // Test indent helper
    let template = PromptTemplate::new("indent_test", "{{indent text 4}}");
    engine.register_template("indent_test", template).unwrap();
    
    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), "line1\nline2".into());
    
    let output = engine.render("indent_test", &ctx).unwrap();
    assert_eq!(output, "    line1\n    line2");
}
