//! ローカルファイルをベクトル化してChroma DBに保存するためのバイナリ。

use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use std::path::PathBuf;
use tracing::{info, warn};

use local_vectored_llm::{chromadb::ChromaClient, file::FileProcessorRegistry, init_logging, ollama::OllamaClient};

/// コマンドライン引数
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// ベクトル化するファイルを含むディレクトリ
    #[clap(name = "DIR")]
    dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // ロギングの初期化
    init_logging();

    // コマンドライン引数の解析
    let args = Args::parse();

    info!("ディレクトリ内のファイルをベクトル化しています: {}", args.dir.display());

    // クライアントの作成
    let ollama_client = OllamaClient::new();
    let chroma_client = ChromaClient::new().await?;

    // Chroma DBコレクションの初期化
    info!("Chroma DBコレクションを初期化しています");
    chroma_client.init_collection().await?;

    // ファイルプロセッサレジストリの作成
    let registry = FileProcessorRegistry::new();

    // 処理可能なファイルを検索
    info!("処理可能なファイルを検索しています...");
    let files = registry.find_files(&args.dir)?;
    info!("{}個の処理可能なファイルが見つかりました", files.len());

    // ファイルをバッチで処理
    let mut processed_count = 0;
    let mut chunk_count = 0;

    for file_batch in files.chunks(10) {
        let mut all_chunks = Vec::new();

        // バッチ内の各ファイルを処理
        for file in file_batch {
            info!("ファイルを処理しています: {}", file.display());
            match registry.process_file(file) {
                Ok(chunks) => {
                    info!("  - {}個のチャンクを抽出しました", chunks.len());
                    all_chunks.extend(chunks);
                }
                Err(e) => {
                    warn!("  - ファイルの処理に失敗しました: {}", e);
                }
            }
        }

        // バッチ内のすべてのチャンクの埋め込みを生成
        info!("{}個のチャンクの埋め込みを生成しています", all_chunks.len());
        let mut embeddings = Vec::with_capacity(all_chunks.len());

        // 埋め込みを並行処理で生成
        let mut embedding_futures = futures::stream::FuturesUnordered::new();

        for chunk in &all_chunks {
            embedding_futures.push(ollama_client.generate_embedding(&chunk.content));
        }

        while let Some(result) = embedding_futures.next().await {
            match result {
                Ok(embedding) => {
                    embeddings.push(embedding);
                }
                Err(e) => {
                    warn!("埋め込みの生成に失敗しました: {}", e);
                    // チャンクとの整合性を保つためのダミー埋め込みを追加
                    embeddings.push(vec![0.0; 1536]); // 1536次元の埋め込みを想定
                }
            }
        }

        // チャンクと埋め込みをChroma DBに保存
        info!("{}個のチャンクをChroma DBに保存しています", all_chunks.len());
        chroma_client.add_documents(&all_chunks, &embeddings).await?;

        processed_count += file_batch.len();
        chunk_count += all_chunks.len();
        info!("進捗状況: {}/{} ファイル, {} チャンク", processed_count, files.len(), chunk_count);
    }

    info!("ベクトル化完了！ {}個のファイル、{}個のチャンクを処理しました", files.len(), chunk_count);
    Ok(())
}
