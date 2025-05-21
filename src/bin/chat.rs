use anyhow::Result;
use clap::Parser;
use local_vectored_llm::chroma::store::ChromaStore;
use local_vectored_llm::info;
use local_vectored_llm::ollama::OllamaClient;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arg {
    /// 質問
    #[arg(short, long)]
    question: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arg::parse();
    let chroma = ChromaStore::new().await?;
    let ollama = OllamaClient::new();

    info!("Search context");
    let contexts = chroma.search(&args.question, 5).await?;
    info!("Found {} contexts: {:?}", contexts.len(), contexts);
    let response = ollama.answer(&args.question, &contexts).await?;
    println!("{}", response.trim());

    Ok(())
}
