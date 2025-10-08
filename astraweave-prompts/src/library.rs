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
    name: String,
    /// Template collections
    collections: HashMap<String, TemplateCollection>,
    /// Library metadata
    metadata: LibraryMetadata,
}

/// Template collection
#[derive(Debug, Clone)]
pub struct TemplateCollection {
    /// Collection name
    name: String,
    /// Templates in this collection
    templates: HashMap<String, String>,
    /// Collection metadata
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
        // Stub implementation - would actually read from filesystem
        let metadata = LibraryMetadata {
            version: "1.0.0".to_string(),
            description: "Loaded from directory".to_string(),
            author: "AstraWeave".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        Ok(Self::new("default".to_string(), metadata))
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