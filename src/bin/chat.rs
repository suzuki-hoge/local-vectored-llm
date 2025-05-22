use anyhow::Result;
use clap::Parser;
use local_vectored_llm::chroma::store::ChromaStore;
use local_vectored_llm::ollama::OllamaClient;
use local_vectored_llm::{error, info};
use std::io::{self, Write};

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

    // コレクション一覧を取得
    let collections = chroma.get_collections().await?;

    // コレクション一覧を表示
    println!();
    for (i, collection) in collections.iter().enumerate() {
        println!("{}. {:<30} ( {} documents )", i + 1, collection.name, collection.count);
    }

    // ユーザーにコレクションを選択させる
    print!("\nChoose the collection you want to use ( e.g. 1,3,4 ): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // 選択されたコレクション名を取得
    let selected_collections: Vec<&str> = input
        .trim()
        .split(',')
        .filter_map(|s| {
            s.trim().parse::<usize>().ok().and_then(|n| {
                if n > 0 && n <= collections.len() {
                    Some(collections[n - 1].name.as_str())
                } else {
                    None
                }
            })
        })
        .collect();

    if selected_collections.is_empty() {
        error!("Unexpected collection.");
        return Err(anyhow::anyhow!("exited."));
    }

    info!("Search context... ( from [ {} ] )", selected_collections.join(", "));
    let contexts = chroma.search(&args.question, 5, &selected_collections).await?;
    info!(
        "Found {} contexts: [ {} ]",
        contexts.len(),
        contexts
            .iter()
            .map(|c| format!("{}...", c.chars().take(30).collect::<String>().replace("\n", "")))
            .collect::<Vec<_>>()
            .join(", ")
    );
    info!("Wait response generation...\n");
    let response = ollama.answer(&args.question, &contexts).await?;
    println!("{}", response.trim());

    info!("Complete");

    Ok(())
}
