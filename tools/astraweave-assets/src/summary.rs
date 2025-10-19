use crate::config::LockEntry;
use prettytable::{format, Cell, Row, Table};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Summary of fetch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchSummary {
    pub total_assets: usize,
    pub downloaded: usize,
    pub cached: usize,
    pub failed: usize,
    pub assets: Vec<AssetSummary>,
}

/// Individual asset summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSummary {
    pub handle: String,
    pub id: String,
    pub kind: String,
    pub status: String, // "downloaded", "cached", "failed"
    pub resolved_res: String,
    pub paths: HashMap<String, PathBuf>,
    pub error: Option<String>,
}

impl FetchSummary {
    /// Create new summary
    pub fn new() -> Self {
        Self {
            total_assets: 0,
            downloaded: 0,
            cached: 0,
            failed: 0,
            assets: Vec::new(),
        }
    }

    /// Add successful download
    pub fn add_downloaded(&mut self, entry: &LockEntry) {
        self.total_assets += 1;
        self.downloaded += 1;

        self.assets.push(AssetSummary {
            handle: entry.handle.clone(),
            id: entry.id.clone(),
            kind: entry.kind.clone(),
            status: "downloaded".to_string(),
            resolved_res: entry.resolved_res.clone(),
            paths: entry.paths.clone(),
            error: None,
        });
    }

    /// Add cached asset
    pub fn add_cached(&mut self, handle: String, id: String, kind: String, res: String) {
        self.total_assets += 1;
        self.cached += 1;

        self.assets.push(AssetSummary {
            handle,
            id,
            kind,
            status: "cached".to_string(),
            resolved_res: res,
            paths: HashMap::new(),
            error: None,
        });
    }

    /// Add failed asset
    pub fn add_failed(&mut self, handle: String, id: String, kind: String, error: String) {
        self.total_assets += 1;
        self.failed += 1;

        self.assets.push(AssetSummary {
            handle,
            id,
            kind,
            status: "failed".to_string(),
            resolved_res: String::new(),
            paths: HashMap::new(),
            error: Some(error),
        });
    }

    /// Print as table
    pub fn print_table(&self) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);

        // Header
        table.add_row(Row::new(vec![
            Cell::new("Handle"),
            Cell::new("Asset ID"),
            Cell::new("Type"),
            Cell::new("Status"),
            Cell::new("Resolution"),
            Cell::new("Files"),
        ]));

        // Rows
        for asset in &self.assets {
            let status_icon = match asset.status.as_str() {
                "downloaded" => "✅",
                "cached" => "💾",
                "failed" => "❌",
                _ => "❓",
            };

            let status_text = format!("{} {}", status_icon, asset.status);
            let file_count = format!("{} maps", asset.paths.len());

            table.add_row(Row::new(vec![
                Cell::new(&asset.handle),
                Cell::new(&asset.id),
                Cell::new(&asset.kind),
                Cell::new(&status_text),
                Cell::new(&asset.resolved_res),
                Cell::new(&file_count),
            ]));
        }

        // Print table
        table.printstd();

        // Summary
        println!("\n📊 Summary:");
        println!("  Total assets: {}", self.total_assets);
        println!("  ✅ Downloaded: {}", self.downloaded);
        println!("  💾 Cached: {}", self.cached);
        println!("  ❌ Failed: {}", self.failed);
    }

    /// Export as JSON
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

impl Default for FetchSummary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_json() {
        let mut summary = FetchSummary::new();

        summary.add_cached(
            "test".to_string(),
            "test_asset".to_string(),
            "texture".to_string(),
            "2k".to_string(),
        );

        let json = summary.to_json().unwrap();
        assert!(json.contains("\"total_assets\": 1"));
        assert!(json.contains("\"cached\": 1"));
    }
}
