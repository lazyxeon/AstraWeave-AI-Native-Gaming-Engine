//! Template library management
//!
//! This module provides template library and collection management.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Template library
#[derive(Debug, Clone)]
pub struct TemplateLibrary {
    /// Library name
    #[allow(dead_code)]
    name: String,
    /// Template collections
    #[allow(dead_code)]
    collections: HashMap<String, TemplateCollection>,
    /// Library metadata
    #[allow(dead_code)]
    metadata: LibraryMetadata,
}

/// Backwards-compatible PromptLibrary used by consumers
#[derive(Debug, Clone)]
pub struct PromptLibrary {
    templates: HashMap<String, crate::template::PromptTemplate>,
}

impl PromptLibrary {
    /// Create a new prompt library
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Add a template to the library
    pub fn add_template(&mut self, name: &str, template: crate::template::PromptTemplate) {
        self.templates.insert(name.to_string(), template);
    }

    /// Get a template by name (cloned)
    pub fn get_template(&self, name: &str) -> anyhow::Result<crate::template::PromptTemplate> {
        if let Some(t) = self.templates.get(name) {
            Ok(t.clone())
        } else {
            anyhow::bail!("template not found: {}", name)
        }
    }

    /// Delete a template from the library
    pub fn delete_template(&mut self, name: &str) -> Option<crate::template::PromptTemplate> {
        self.templates.remove(name)
    }

    /// List all templates
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
}

/// Template collection
#[derive(Debug, Clone)]
pub struct TemplateCollection {
    /// Collection name
    name: String,
    /// Templates in this collection
    templates: HashMap<String, String>,
    /// Collection metadata
    #[allow(dead_code)]
    metadata: CollectionMetadata,
}

/// Library metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMetadata {
    /// Library version
    pub version: String,
    /// Library description
    pub description: String,
    /// Library author
    pub author: String,
    /// Creation date
    pub created_at: String,
}

/// Collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    /// Collection version
    pub version: String,
    /// Collection description
    pub description: String,
    /// Collection tags
    pub tags: Vec<String>,
}

impl TemplateLibrary {
    /// Create a new template library
    pub fn new(name: String, metadata: LibraryMetadata) -> Self {
        Self {
            name,
            collections: HashMap::new(),
            metadata,
        }
    }

    /// Add a collection to the library
    pub fn add_collection(&mut self, collection: TemplateCollection) {
        self.collections.insert(collection.name.clone(), collection);
    }

    /// Get a collection by name
    pub fn get_collection(&self, name: &str) -> Option<&TemplateCollection> {
        self.collections.get(name)
    }

    /// List all collections
    pub fn list_collections(&self) -> Vec<&String> {
        self.collections.keys().collect()
    }

    /// Load library from directory
    pub fn load_from_directory(path: PathBuf) -> Result<Self> {
        let mut library = Self::new(
            "default".to_string(),
            LibraryMetadata {
                version: "1.0.0".to_string(),
                description: "Loaded from directory".to_string(),
                author: "AstraWeave".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        );

        let mut default_collection = TemplateCollection::new(
            "default".to_string(),
            CollectionMetadata {
                version: "1.0.0".to_string(),
                description: "Default collection".to_string(),
                tags: vec![],
            },
        );

        if path.exists() && path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "hbs" {
                            let name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let content = std::fs::read_to_string(&path)?;
                            default_collection.add_template(name, content);
                        }
                    }
                }
            }
        }

        library.add_collection(default_collection);
        Ok(library)
    }

    /// Save library to directory
    pub fn save_to_directory(&self, path: PathBuf) -> Result<()> {
        // Stub implementation - would actually write to filesystem
        let _ = path;
        Ok(())
    }
}

impl TemplateCollection {
    /// Create a new template collection
    pub fn new(name: String, metadata: CollectionMetadata) -> Self {
        Self {
            name,
            templates: HashMap::new(),
            metadata,
        }
    }

    /// Add a template to the collection
    pub fn add_template(&mut self, name: String, template: String) {
        self.templates.insert(name, template);
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&String> {
        self.templates.get(name)
    }

    /// List all templates in the collection
    pub fn list_templates(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }

    /// Remove a template from the collection
    pub fn remove_template(&mut self, name: &str) -> Option<String> {
        self.templates.remove(name)
    }
}
