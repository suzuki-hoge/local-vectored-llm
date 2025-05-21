use anyhow::Result;
use clap::Parser;
use local_vectored_llm::chroma::ChromaStore;
use local_vectored_llm::document::DocumentProcessor;
use local_vectored_llm::{info, warn};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arg {
    /// 処理するドキュメントのディレクトリパス
    #[arg(short, long)]
    input: PathBuf,

    /// チャンクサイズ
    #[arg(short, long, default_value = "1000")]
    chunk_size: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arg::parse();
    let processor = DocumentProcessor::new(args.chunk_size);
    let chroma = ChromaStore::new().await?;

    let documents = processor.process_directory(&args.input).await?;

    let mut success_count = 0;
    let mut error_sources = vec![];

    for (index, document) in documents.iter().enumerate() {
        match chroma.save(document).await {
            Ok(_) => {
                info!("Saved: {} - {} [ {} / {} ]", &document.source, &document.chunk_index + 1, index + 1, documents.len());
                success_count += 1;
            }
            Err(e) => {
                warn!("Failed: {} [ e = {} ] [ {} / {} ]", &document.source, e, index + 1, documents.len());
                error_sources.push(&document.source);
            }
        }
    }

    info!("Processed: success = {}, failure = {}", success_count, error_sources.len());

    if !error_sources.is_empty() {
        error_sources.into_iter().for_each(|s| warn!("Failed: {}", s))
    }

    Ok(())
}
