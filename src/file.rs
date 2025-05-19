//! ファイル処理ユーティリティ。

use anyhow::{Context, Result};
use pdf_extract::extract_text;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{AppError, DocumentChunk, DocumentMetadata, FileProcessor};

/// ファイルプロセッサのレジストリ。
pub struct FileProcessorRegistry {
    processors: Vec<Box<dyn FileProcessor>>,
}

impl Default for FileProcessorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FileProcessorRegistry {
    /// デフォルトのプロセッサを持つ新しいレジストリを作成します。
    pub fn new() -> Self {
        let mut registry = Self { processors: Vec::new() };

        // デフォルトプロセッサを登録
        registry.register(Box::new(TextFileProcessor));
        registry.register(Box::new(PdfFileProcessor));

        registry
    }

    /// 新しいファイルプロセッサを登録します。
    pub fn register(&mut self, processor: Box<dyn FileProcessor>) {
        self.processors.push(processor);
    }

    /// 適切なプロセッサを使用してファイルを処理します。
    pub fn process_file(&self, path: &Path) -> Result<Vec<DocumentChunk>> {
        for processor in &self.processors {
            if processor.can_handle(path) {
                return processor.process(path);
            }
        }

        Err(AppError::UnsupportedFileType(
            path.extension().and_then(|ext| ext.to_str()).unwrap_or("unknown").to_string(),
        )
        .into())
    }

    /// ディレクトリ内の処理可能なすべてのファイルを検索します。
    pub fn find_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_file() && self.processors.iter().any(|p| p.can_handle(path)) {
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }
}

/// テキストファイル用プロセッサ。
pub struct TextFileProcessor;

impl FileProcessor for TextFileProcessor {
    fn process(&self, path: &Path) -> Result<Vec<DocumentChunk>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("テキストファイルの読み込みに失敗しました: {}", path.display()))?;

        // 段落による単純なチャンク分割
        let chunks = chunk_text(&content, 1000);

        Ok(chunks
            .into_iter()
            .enumerate()
            .map(|(i, content)| DocumentChunk {
                content,
                source: path.display().to_string(),
                metadata: DocumentMetadata {
                    file_type: "text".to_string(),
                    chunk_index: i,
                    additional: serde_json::json!({}),
                },
            })
            .collect())
    }

    fn can_handle(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.eq_ignore_ascii_case("txt")).unwrap_or(false)
    }
}

/// PDFファイル用プロセッサ。
pub struct PdfFileProcessor;

impl FileProcessor for PdfFileProcessor {
    fn process(&self, path: &Path) -> Result<Vec<DocumentChunk>> {
        // pdf-extractを使用してPDFからテキストを抽出
        let text = extract_text(path)
            .map_err(|e| AppError::FileProcessing(format!("PDFからのテキスト抽出に失敗しました: {}", e)))?;

        // 段落による単純なチャンク分割
        let chunks = chunk_text(&text, 1000);

        Ok(chunks
            .into_iter()
            .enumerate()
            .map(|(i, content)| DocumentChunk {
                content,
                source: path.display().to_string(),
                metadata: DocumentMetadata {
                    file_type: "pdf".to_string(),
                    chunk_index: i,
                    additional: serde_json::json!({}),
                },
            })
            .collect())
    }

    fn can_handle(&self, path: &Path) -> bool {
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.eq_ignore_ascii_case("pdf")).unwrap_or(false)
    }
}

/// 指定されたサイズに近い大きさのチャンクにテキストを分割します。
fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();

    for paragraph in text.split("\n\n") {
        if current_chunk.len() + paragraph.len() > chunk_size && !current_chunk.is_empty() {
            chunks.push(current_chunk);
            current_chunk = String::new();
        }

        if !current_chunk.is_empty() {
            current_chunk.push_str("\n\n");
        }

        current_chunk.push_str(paragraph);
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}
