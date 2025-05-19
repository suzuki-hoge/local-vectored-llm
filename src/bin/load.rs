use anyhow::Result;
use clap::Parser;
use local_vectored_llm::chroma::ChromaStore;
use local_vectored_llm::document::DocumentProcessor;
use local_vectored_llm::utils::logger;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 処理するドキュメントのディレクトリパス
    #[arg(short, long)]
    input: PathBuf,

    /// チャンクサイズ
    #[arg(short, long, default_value = "1000")]
    chunk_size: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::init_logging()?;

    let cli = Cli::parse();
    let processor = DocumentProcessor::new(cli.chunk_size);
    let chroma = ChromaStore::new().await?;

    let documents = processor.process_directory(&cli.input).await?;
    for document in documents {
        chroma.save(&document).await?;
    }

    println!("ドキュメントの処理が完了しました");
    Ok(())
}
