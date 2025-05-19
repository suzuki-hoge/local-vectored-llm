use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn extract_text(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}
