//! Template loading and management
//!
//! This module provides functionality for loading templates from files
//! and managing template collections.

use crate::{PromptTemplate, TemplateMetadata};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Loader for prompt templates
pub struct PromptLoader {
    /// Supported file extensions
    extensions: Vec<String>,
}

impl Default for PromptLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptLoader {
    /// Create a new prompt loader
    pub fn new() -> Self {
        Self {
            extensions: vec!["hbs".to_string(), "prompt".to_string()],
        }
    }

    /// Set supported extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Load templates from a directory recursively
    pub fn load_from_dir<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<PromptTemplate>> {
        let mut templates = Vec::new();
        let dir_path = dir.as_ref();

        if !dir_path.exists() {
            return Ok(templates);
        }

        for entry in WalkDir::new(dir_path).follow_links(true) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if self.extensions.iter().any(|e| e == ext.to_string_lossy().as_ref()) {
                        match self.load_file(entry.path()) {
                            Ok(template) => templates.push(template),
                            Err(e) => {
                                eprintln!("Failed to load template from {:?}: {}", entry.path(), e);
                            }
                        }
                    }
                }
            }
        }
        Ok(templates)
    }

    /// Load a single template file
    pub fn load_file<P: AsRef<Path>>(&self, path: P) -> Result<PromptTemplate> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read template file: {:?}", path.as_ref()))?;
        
        let (metadata, template_content) = self.parse_frontmatter(&content)?;
        
        // Use filename as ID if not specified in metadata
        let id = if let Some(ref meta) = metadata {
            meta.name.clone()
        } else {
            path.as_ref()
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        };
        
        let mut template = PromptTemplate::new(id, template_content);
        template.metadata = metadata;
        
        Ok(template)
    }

    /// Parse frontmatter from template content
    /// Supports TOML frontmatter delimited by +++
    fn parse_frontmatter(&self, content: &str) -> Result<(Option<TemplateMetadata>, String)> {
        if content.starts_with("+++") {
            if let Some(end) = content[3..].find("+++") {
                let frontmatter = &content[3..end+3];
                let body = &content[end+6..];
                
                let metadata: TemplateMetadata = toml::from_str(frontmatter)
                    .context("Failed to parse TOML frontmatter")?;
                    
                return Ok((Some(metadata), body.trim().to_string()));
            }
        }
        
        Ok((None, content.to_string()))
    }
}
