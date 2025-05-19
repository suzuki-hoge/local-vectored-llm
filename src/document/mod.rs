use anyhow::Result;
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

    pub async fn process_directory(&self, dir_path: &Path) -> Result<Vec<ProcessedDocument>> {
        let mut documents = Vec::new();
        for entry in walkdir::WalkDir::new(dir_path) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && Self::is_supported_file(path) {
                let processed = self.process_file(path).await?;
                documents.extend(processed);
            }
        }
        Ok(documents)
    }

    async fn process_file(&self, path: &Path) -> Result<Vec<ProcessedDocument>> {
        tracing::info!("Processing file: {:?}", path);

        let content = match path.extension().and_then(|ext| ext.to_str()) {
            Some("txt") => text::extract_text(path)?,
            Some("md") => markdown::extract_text(path)?,
            Some("pdf") => pdf::extract_text(path)?,
            _ => {
                tracing::warn!("Unsupported file type: {:?}", path);
                return Ok(Vec::new());
            }
        };

        // テキスト分割
        let splitter = TextSplitter::new(self.chunk_size, self.chunk_size / 10);
        let chunks = splitter.split(&content);

        // 各チャンクをProcessedDocumentとして返す
        Ok(chunks
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| ProcessedDocument {
                content: chunk,
                source: path.to_string_lossy().to_string(),
                chunk_index: i,
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

pub struct ProcessedDocument {
    pub content: String,
    pub source: String,
    pub chunk_index: usize,
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
