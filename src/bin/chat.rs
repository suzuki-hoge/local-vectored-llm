use anyhow::Result;
use clap::Parser;
use local_vectored_llm::chroma::store::ChromaStore;
use local_vectored_llm::info;
use local_vectored_llm::ollama::OllamaClient;
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
    println!("利用可能なコレクション:");
    for (i, collection) in collections.iter().enumerate() {
        println!("{}. {} ({}件)", i + 1, collection.name, collection.count);
    }

    // ユーザーにコレクションを選択させる
    print!("コレクション番号を選択してください (1-{}): ", collections.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: usize = input.trim().parse()?;

    if selection < 1 || selection > collections.len() {
        return Err(anyhow::anyhow!("無効なコレクション番号です"));
    }

    let selected_collection = &collections[selection - 1];

    info!("Search context ...");
    let contexts = chroma.search(&args.question, 5, &selected_collection.name).await?;
    info!(
        "Found {} contexts: [ {} ]",
        contexts.len(),
        contexts.iter().map(|c| format!("{} ...", c.chars().take(30).collect::<String>().replace("\n", ""))).collect::<Vec<_>>().join(", ")
    );
    info!("Wait response generation ...");
    let response = ollama.answer(&args.question, &contexts).await?;
    println!("{}", response.trim());

    Ok(())
}
