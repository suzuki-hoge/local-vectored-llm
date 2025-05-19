use ollama_rs::error::OllamaError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ChromaDB error: {0}")]
    ChromaDB(#[from] anyhow::Error),

    #[error("Ollama error: {0}")]
    Ollama(#[from] OllamaError),

    #[error("Document processing error: {0}")]
    Document(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),
}
