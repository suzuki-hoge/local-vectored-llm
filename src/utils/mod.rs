use anyhow::Result;
use std::path::Path;
use tracing_subscriber::FmtSubscriber;

pub mod error;

pub fn init_logging() -> Result<()> {
    let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

pub fn is_supported_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return matches!(ext_str.to_lowercase().as_str(), "txt" | "pdf" | "md");
        }
    }
    false
}
