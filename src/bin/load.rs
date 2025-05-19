use anyhow::Result;
use clap::Parser;
use local_vectored_llm::document::DocumentProcessor;
use local_vectored_llm::utils;
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
    utils::init_logging()?;

    let cli = Cli::parse();
    let processor = DocumentProcessor::new(cli.chunk_size).await?;

    processor.process_directory(&cli.input).await?;

    println!("ドキュメントの処理が完了しました");
    Ok(())
}
