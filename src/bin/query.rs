//! RAGを使用してChroma DBからLLMに問い合わせるためのバイナリ。

use anyhow::Result;
use clap::Parser;
use tracing::info;

use local_vectored_llm::{chromadb::ChromaClient, ollama::OllamaClient};

/// ベクトル検索を実行します。
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 検索クエリ
    #[arg(short, long)]
    query: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // ロギングを初期化
    tracing_subscriber::fmt::init();

    // コマンドライン引数を解析
    let args = Args::parse();

    // クライアントを初期化
    let ollama = OllamaClient::new();
    let chroma = ChromaClient::new();

    // コレクションを初期化
    chroma.init_collection().await?;

    // クエリの埋め込みを生成
    let query_embedding = ollama.generate_embedding(&args.query).await?;

    // 類似ドキュメントを検索
    let results = chroma.query(&query_embedding, 5).await?;

    // 結果を表示
    for (i, result) in results.iter().enumerate() {
        info!("結果 {}:", i + 1);
        info!("ドキュメント: {}", result.document);
        info!("メタデータ: {:?}", result.metadata);
        info!("距離: {}", result.distance);
        info!("---");
    }

    Ok(())
}
