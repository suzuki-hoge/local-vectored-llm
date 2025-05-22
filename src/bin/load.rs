use anyhow::Result;
use clap::Parser;
use local_vectored_llm::chroma::store::ChromaStore;
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

    let processed = processor.process_directory(&args.input).await?;

    let mut success_count = 0;
    let mut error_sources = vec![];

    for (index, (documents, collection_name)) in processed.iter().enumerate() {
        for document in documents {
            match chroma.save(document, collection_name).await {
                Ok(_) => {
                    info!("[ {} / {} ] Saved: {}", index + 1, processed.len(), &document.id,);
                    success_count += 1;
                }
                Err(e) => {
                    warn!("[ {} / {} ] Failed: {}", index + 1, processed.len(), e,);
                    error_sources.push(&document.metadata.file.path);
                }
            }
        }
    }

    info!("Processed: success = {}, failure = {}", success_count, error_sources.len());

    if !error_sources.is_empty() {
        error_sources.into_iter().for_each(|s| warn!("Failed: {}", s))
    }

    Ok(())
}
