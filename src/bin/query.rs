//! RAGを使用してChroma DBからLLMに問い合わせるためのバイナリ。

use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};
use tracing::{debug, info};

use local_vectored_llm::{chromadb::ChromaClient, init_logging, ollama::OllamaClient};

// 固定値の定義
const OLLAMA_URL: &str = "http://localhost:11434";
const MODEL: &str = "deepseek-r1:1.5b";
const CHROMA_URL: &str = "http://localhost:18888";
const COLLECTION: &str = "local_files";
const N_RESULTS: usize = 5;

/// コマンドライン引数
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// クエリ文字列（必須）
    #[clap(short, long)]
    query: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // ロギングの初期化
    init_logging();

    // コマンドライン引数の解析
    let args = Args::parse();

    // クライアントの作成
    let ollama_client = OllamaClient::new(OLLAMA_URL, MODEL);
    let chroma_client = ChromaClient::new(CHROMA_URL, COLLECTION);

    // Chroma DBコレクションの初期化
    info!("Chroma DBコレクションを初期化しています: {}", COLLECTION);
    chroma_client.init_collection().await?;

    // クエリ処理
    process_query(&ollama_client, &chroma_client, &args.query, N_RESULTS).await?;

    Ok(())
}

/// クエリを処理する
async fn process_query(
    ollama_client: &OllamaClient,
    chroma_client: &ChromaClient,
    query: &str,
    n_results: usize,
) -> Result<()> {
    // クエリの埋め込みを生成
    info!("クエリの埋め込みを生成しています: {}", query);
    let query_embedding = ollama_client.generate_embedding(query).await?;

    // Chroma DBから類似ドキュメントを検索
    info!("Chroma DBから類似ドキュメントを検索しています");
    let results = chroma_client.query(&query_embedding, n_results).await?;

    if results.is_empty() {
        println!("\n関連するドキュメントが見つかりませんでした。");
        return Ok(());
    }

    // 取得したドキュメントからコンテキストを構築
    let mut context = String::new();
    for (i, result) in results.iter().enumerate() {
        debug!(
            "ドキュメント {}: 距離={}, ソース={}",
            i + 1,
            result.distance,
            result.metadata.get("source").and_then(|v| v.as_str()).unwrap_or("不明")
        );

        context.push_str(&format!("--- ドキュメント {} ---\n", i + 1));
        context.push_str(&result.document);
        context.push_str("\n\n");
    }

    // コンテキスト付きのプロンプトを構築
    let prompt = format!(
        "あなたは役立つアシスタントです。以下の情報を使用してユーザーの質問に答えてください。\n\n\
        コンテキスト情報:\n{}\n\n\
        ユーザーの質問: {}\n\n\
        回答:",
        context, query
    );

    // 回答を生成
    info!("コンテキストを使用して回答を生成しています");
    print!("\n回答を生成中...");
    io::stdout().flush()?;

    let completion = ollama_client.generate_completion(&prompt).await?;

    // "回答を生成中..." メッセージをクリア
    print!("\r                      \r");

    // 回答を表示
    println!("\n{}", completion);

    Ok(())
}
