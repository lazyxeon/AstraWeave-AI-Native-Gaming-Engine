use astraweave_prompts::{PromptLoader, TemplateCategory};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_load_simple_file() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("simple.hbs");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "Hello {{{{name}}}}!").unwrap();

    let loader = PromptLoader::new();
    let template = loader.load_file(&file_path).unwrap();

    assert_eq!(template.id, "simple");
    assert_eq!(template.template.trim(), "Hello {{name}}!");
    assert!(template.metadata.is_none());
}

#[test]
fn test_load_with_frontmatter() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("meta.hbs");
    let mut file = File::create(&file_path).unwrap();
    
    let content = r#"+++
name = "greeting"
version = "1.0.0"
description = "A friendly greeting"
category = "Dialogue"
tags = ["friendly", "intro"]
required_variables = ["name"]
+++
Hello {{name}}! How are you?"#;

    // No need to escape braces here as we are using write! with a raw string literal content directly
    // But wait, writeln!(file, "{}", content) uses formatting.
    // So braces in content need to be escaped if passed to format string.
    // Or just use file.write_all(content.as_bytes())
    
    file.write_all(content.as_bytes()).unwrap();

    let loader = PromptLoader::new();
    let template = loader.load_file(&file_path).unwrap();

    assert_eq!(template.id, "greeting");
    assert_eq!(template.template.trim(), "Hello {{name}}! How are you?");
    
    let meta = template.metadata.unwrap();
    assert_eq!(meta.name, "greeting");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(meta.category, TemplateCategory::Dialogue);
    assert_eq!(meta.tags, vec!["friendly", "intro"]);
}

#[test]
fn test_load_from_dir() {
    let dir = TempDir::new().unwrap();
    
    // File 1
    let p1 = dir.path().join("t1.hbs");
    let mut f1 = File::create(&p1).unwrap();
    writeln!(f1, "Template 1").unwrap();
    
    // File 2 in subdir
    let sub = dir.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();
    let p2 = sub.join("t2.prompt");
    let mut f2 = File::create(&p2).unwrap();
    writeln!(f2, "Template 2").unwrap();
    
    // Ignored file
    let p3 = dir.path().join("ignored.txt");
    let mut f3 = File::create(&p3).unwrap();
    writeln!(f3, "Ignored").unwrap();

    let loader = PromptLoader::new();
    let templates = loader.load_from_dir(dir.path()).unwrap();

    assert_eq!(templates.len(), 2);
    let ids: Vec<String> = templates.iter().map(|t| t.id.clone()).collect();
    assert!(ids.contains(&"t1".to_string()));
    assert!(ids.contains(&"t2".to_string()));
}
