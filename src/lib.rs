//! ローカルファイルをベクトル化し、RAGを通じてDeepseek LLMで利用するためのライブラリ。

use anyhow::Result;
use std::path::Path;
use thiserror::Error;

pub mod chromadb;
pub mod file;
pub mod ollama;

/// アプリケーションで発生する可能性のあるエラー。
#[derive(Error, Debug)]
pub enum AppError {
    #[error("IOエラー: {0}")]
    Io(#[from] std::io::Error),

    #[error("ファイル処理に失敗しました: {0}")]
    FileProcessing(String),

    #[error("Ollama APIエラー: {0}")]
    OllamaApi(String),

    #[error("Chroma DBエラー: {0}")]
    ChromaDb(String),

    #[error("サポートされていないファイルタイプ: {0}")]
    UnsupportedFileType(String),
}

/// ベクトル化できるドキュメントチャンクを表します。
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    /// チャンクの内容。
    pub content: String,

    /// ソースファイルのパス。
    pub source: String,

    /// チャンクに関するメタデータ。
    pub metadata: DocumentMetadata,
}

/// ドキュメントチャンクに関するメタデータ。
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    /// ファイルタイプ。
    pub file_type: String,

    /// ドキュメント内のチャンクインデックス。
    pub chunk_index: usize,

    /// 追加のメタデータ。
    pub additional: serde_json::Value,
}

/// ファイルプロセッサのトレイト。
pub trait FileProcessor {
    /// ファイルを処理してチャンクを返します。
    fn process(&self, path: &Path) -> Result<Vec<DocumentChunk>>;

    /// このプロセッサが指定されたファイルを処理できるかどうかを確認します。
    fn can_handle(&self, path: &Path) -> bool;
}

/// ロギングを初期化します。
pub fn init_logging() {
    tracing_subscriber::fmt::init();
}
