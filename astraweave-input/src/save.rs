use anyhow::Result;
use std::fs;
use crate::bindings::BindingSet;

pub fn save_bindings(path: &str, b: &BindingSet) -> Result<()> {
    let txt = serde_json::to_string_pretty(b)?;
    fs::create_dir_all(std::path::Path::new(path).parent().unwrap_or(std::path::Path::new(".")))?;
    fs::write(path, txt)?;
    Ok(())
}

pub fn load_bindings(path: &str) -> Option<BindingSet> {
    let txt = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&txt).ok()
}
