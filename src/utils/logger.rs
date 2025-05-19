use anyhow::Result;
use tracing_subscriber::FmtSubscriber;

pub fn init_logging() -> Result<()> {
    let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
