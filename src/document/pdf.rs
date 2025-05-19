use anyhow::Result;
use pdf::file::FileOptions;
use std::path::Path;

pub fn extract_text(path: &Path) -> Result<String> {
    let pdf = FileOptions::cached().open(path)?;
    let mut text = String::new();

    for page in pdf.pages().flatten() {
        if let Some(content) = &page.contents {
            text.push_str(&format!("{:?}", content));
            text.push('\n');
        }
    }

    Ok(text)
}
