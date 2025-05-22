use crate::chroma::document::{ChunkMetadata, CollectionName, Document, FileMetadata, Metadata, SearchMetadata};
use crate::{info, warn};
use anyhow::{anyhow, Result};
use chrono::DateTime;
use std::path::Path;

pub mod markdown;
pub mod pdf;
pub mod text;

pub struct DocumentProcessor {
    chunk_size: usize,
}

pub type Processed = (Vec<Document>, CollectionName);

impl DocumentProcessor {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    pub async fn process_directory(&self, root_path: &Path) -> Result<Vec<Processed>> {
        let mut result = Vec::new();
        for entry in walkdir::WalkDir::new(root_path) {
            let entry = entry?;
            let full_path = entry.path();

            if full_path.is_file() && Self::is_supported_file(full_path) {
                result.push(self.process_file(root_path, full_path).await?);
                info!(
                    "Converted: {}",
                    full_path.to_string_lossy().to_string().replace(&format!("{}/", &root_path.to_string_lossy()), "")
                );
            }
        }
        Ok(result)
    }

    async fn process_file(&self, root_path: &Path, full_path: &Path) -> Result<Processed> {
        let content = match full_path.extension().and_then(|ext| ext.to_str()) {
            Some("txt") => text::extract_text(full_path)?,
            Some("md") => markdown::extract_text(full_path)?,
            Some("pdf") => pdf::extract_text(full_path)?,
            _ => {
                warn!("Unsupported file type: {}", full_path.display());
                return Err(anyhow!("unsupported file"));
            }
        };

        let metadata = std::fs::metadata(full_path)?;

        let path = full_path.to_string_lossy().to_string().replace(&format!("{}/", &root_path.to_string_lossy()), "");
        let created_at = DateTime::from(metadata.created()?);
        let updated_at = DateTime::from(metadata.modified()?);

        // テキスト分割
        let splitter = TextSplitter::new(self.chunk_size, self.chunk_size / 10);
        let chunks = splitter.split(&content);

        let collection_name = Self::fix_collection_name(&path);

        Ok((
            chunks
                .into_iter()
                .enumerate()
                .map(|(index, chunk)| Document {
                    id: format!("{}-{}", path, index),
                    content: chunk,
                    metadata: Metadata {
                        file: FileMetadata { path: path.clone(), created_at, updated_at },
                        chunk: ChunkMetadata { index },
                        search: SearchMetadata {},
                    },
                })
                .collect(),
            collection_name.to_string(),
        ))
    }

    fn fix_collection_name(path: &str) -> String {
        let slash = path.chars().filter(|c| c == &'/').count();
        if slash == 0 {
            "root".to_string()
        } else if slash == 1 {
            path.split("/").take(1).collect::<Vec<_>>().join("-")
        } else {
            path.split("/").take(2).collect::<Vec<_>>().join("-")
        }
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

    #[tokio::test]
    async fn text_depth1() {
        let testdata = Path::new("./testdata").canonicalize().unwrap();
        process_and_assert(
            testdata.join("root1"),
            testdata.join("root1/sample.txt"),
            50,
            "sample.txt",
            "root",
            "これはテスト用のサンプルテキストです",
        )
        .await;
    }

    #[tokio::test]
    async fn pdf_depth2() {
        let testdata = Path::new("./testdata").canonicalize().unwrap();
        process_and_assert(
            testdata.join("root2"),
            testdata.join("root2/pj1/sample.pdf"),
            50,
            "pj1/sample.pdf",
            "pj1",
            "これはテスト用のサンプルテキストです",
        )
        .await;
    }

    #[tokio::test]
    async fn text_depth3() {
        let testdata = Path::new("./testdata").canonicalize().unwrap();
        process_and_assert(
            testdata.join("root3"),
            testdata.join("root3/pj1/dir1/sample.txt"),
            50,
            "pj1/dir1/sample.txt",
            "pj1-dir1",
            "これはテスト用のサンプルテキストです",
        )
        .await;
    }

    #[tokio::test]
    async fn text_depth4() {
        let testdata = Path::new("./testdata").canonicalize().unwrap();
        process_and_assert(
            testdata.join("root4"),
            testdata.join("root4/pj1/dir1/dir2/sample.txt"),
            50,
            "pj1/dir1/dir2/sample.txt",
            "pj1-dir1",
            "これはテスト用のサンプルテキストです",
        )
        .await;
    }

    async fn process_and_assert<P: AsRef<Path>>(
        root_dir: P,
        target_path: P,
        size: usize,
        exp_path: &str,
        exp_collection_name: &str,
        exp_first_text: &str,
    ) {
        let processor = DocumentProcessor::new(size);

        let (documents, collection_name) =
            processor.process_file(root_dir.as_ref(), target_path.as_ref()).await.unwrap();

        // 結果の検証
        assert!(!documents.is_empty(), "Empty result");

        assert_eq!(collection_name, exp_collection_name, "Unexpected collection name");

        // 最初のチャンクの検証
        let first_chunk = &documents[0];
        assert_eq!(first_chunk.metadata.file.path, exp_path, "Unexpected file path");
        assert_eq!(first_chunk.metadata.chunk.index, 0, "Unexpected chunk index");
        assert!(first_chunk.content.starts_with(exp_first_text), "Unexpected text");

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
