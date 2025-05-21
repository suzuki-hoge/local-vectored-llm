use crate::chroma::document::{ChunkMetadata, Document, FileMetadata, Metadata, SearchMetadata};
use crate::{info, warn};
use anyhow::Result;
use chrono::DateTime;
use std::path::Path;

pub mod markdown;
pub mod pdf;
pub mod text;

pub struct DocumentProcessor {
    chunk_size: usize,
}

impl DocumentProcessor {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    pub async fn process_directory(&self, dir_path: &Path) -> Result<Vec<Document>> {
        let mut result = Vec::new();
        for entry in walkdir::WalkDir::new(dir_path) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && Self::is_supported_file(path) {
                let documents = self.process_file(path).await?;
                result.extend(documents);
                info!("Converted: {}", path.display());
            }
        }
        Ok(result)
    }

    async fn process_file(&self, path: &Path) -> Result<Vec<Document>> {
        let content = match path.extension().and_then(|ext| ext.to_str()) {
            Some("txt") => text::extract_text(path)?,
            Some("md") => markdown::extract_text(path)?,
            Some("pdf") => pdf::extract_text(path)?,
            _ => {
                warn!("Unsupported file type: {}", path.display());
                return Ok(Vec::new());
            }
        };

        let metadata = std::fs::metadata(path)?;

        let created_at = DateTime::from(metadata.created()?);
        let updated_at = DateTime::from(metadata.modified()?);

        // テキスト分割
        let splitter = TextSplitter::new(self.chunk_size, self.chunk_size / 10);
        let chunks = splitter.split(&content);

        Ok(chunks
            .into_iter()
            .enumerate()
            .map(|(index, chunk)| Document {
                id: format!("{}-{}", path.to_string_lossy(), index),
                content: chunk,
                metadata: Metadata {
                    file: FileMetadata { path: path.to_string_lossy().to_string(), created_at, updated_at },
                    chunk: ChunkMetadata { index },
                    search: SearchMetadata {},
                },
            })
            .collect())
    }

    fn is_supported_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return matches!(ext_str.to_lowercase().as_str(), "txt" | "pdf" | "md");
            }
        }
        false
    }
}

struct TextSplitter {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl TextSplitter {
    fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self { chunk_size, chunk_overlap }
    }

    fn split(&self, text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();
        let mut start = 0;
        while start < chars.len() {
            let end = usize::min(start + self.chunk_size, chars.len());
            let chunk: String = chars[start..end].iter().collect();
            chunks.push(chunk);
            if end == chars.len() {
                break;
            }
            start += self.chunk_size - self.chunk_overlap;
        }
        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn text() {
        process_and_assert("sample.txt", 50, "これはテスト用のサンプルテキストです").await;
    }

    #[tokio::test]
    async fn pdf() {
        process_and_assert("sample.pdf", 50, "これはテスト用のサンプルテキストです").await;
    }

    async fn process_and_assert(target: &str, size: usize, first_exp_text: &str) {
        let processor = DocumentProcessor::new(size);
        let test_file = PathBuf::from(format!("testdata/{}", target));

        let documents = processor.process_file(&test_file).await.unwrap();

        // 結果の検証
        assert!(!documents.is_empty(), "Empty result");

        // 最初のチャンクの検証
        let first_chunk = &documents[0];
        assert_eq!(first_chunk.metadata.file.path, test_file.to_string_lossy().to_string());
        assert_eq!(first_chunk.metadata.chunk.index, 0);
        assert!(first_chunk.content.starts_with(first_exp_text));

        // チャンクサイズの検証
        for document in &documents {
            assert!(document.content.chars().count() <= size, "Unexpected chunk size");
        }

        // 全チャンク間の重複（文字単位）を確認
        for i in 0..documents.len() - 1 {
            let chunk1 = &documents[i].content;
            let chunk2 = &documents[i + 1].content;

            // チャンク1の末尾5文字とチャンク2の先頭5文字を取得
            let tail: String = chunk1.chars().rev().take(5).collect::<Vec<_>>().into_iter().rev().collect();
            let head: String = chunk2.chars().take(5).collect();

            // 末尾と先頭が一致することを確認
            assert_eq!(tail, head, "No overlap between chunk{} and chunk{}", i, i + 1);
        }
    }
}
