use crate::embedding::EmbeddingStore;
use crate::utils::is_supported_file;
use anyhow::Result;
use ollama_rs::generation::embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest};
use ollama_rs::Ollama;
use pdf::file::FileOptions;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct DocumentProcessor {
    chunk_size: usize,
    embedding_store: EmbeddingStore,
    ollama: Ollama,
}

impl DocumentProcessor {
    pub async fn new(chunk_size: usize) -> Result<Self> {
        Ok(Self {
            chunk_size,
            embedding_store: EmbeddingStore::new().await?,
            ollama: Ollama::new("http://localhost:11434", 11434),
        })
    }

    pub async fn process_directory(&self, dir_path: &Path) -> Result<()> {
        for entry in WalkDir::new(dir_path) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_supported_file(path) {
                self.process_file(path).await?;
            }
        }
        Ok(())
    }

    async fn process_file(&self, path: &Path) -> Result<()> {
        tracing::info!("Processing file: {:?}", path);

        let content = match path.extension().and_then(|ext| ext.to_str()) {
            Some("txt") | Some("md") => fs::read_to_string(path)?,
            Some("pdf") => self.extract_text_from_pdf(path)?,
            _ => {
                tracing::warn!("Unsupported file type: {:?}", path);
                return Ok(());
            }
        };

        // テキスト分割
        let splitter = TextSplitter::new(self.chunk_size, self.chunk_size / 10);
        let chunks = splitter.split(&content);

        // 各チャンクに対して埋め込みを生成して保存
        for (i, chunk) in chunks.iter().enumerate() {
            let embedding = self.generate_embedding(chunk).await?;
            self.embedding_store.store_embeddings(chunk, embedding, path.to_string_lossy().to_string(), i).await?;
        }

        Ok(())
    }

    fn extract_text_from_pdf(&self, path: &Path) -> Result<String> {
        let pdf = FileOptions::cached().open(path)?;
        let mut text = String::new();

        for page in pdf.pages().flatten() {
            if let Some(content) = &page.contents {
                text.push_str(&format!("{:?}", content));
                text.push('\n');
            }
        }

        Ok(text)
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let req =
            GenerateEmbeddingsRequest::new("mxbai-embed-large".to_string(), EmbeddingsInput::Single(text.to_string()));
        let result = self.ollama.generate_embeddings(req).await?;
        // embeddingsはVec<Vec<f32>>型なので、Singleの場合は最初の要素を返す
        Ok(result.embeddings.into_iter().next().unwrap_or_default())
    }
}

// 独自のTextSplitter実装
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
