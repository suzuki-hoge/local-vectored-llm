//! 埋め込み生成のためのOllama APIクライアント。

use anyhow::Result;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::embeddings::request::{GenerateEmbeddingsRequest, EmbeddingsInput};
use ollama_rs::Ollama;
use tracing::debug;

use crate::AppError;

/// Ollama APIのクライアント。
pub struct OllamaClient {
    client: Ollama,
    model: String,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OllamaClient {
    /// 新しいOllamaクライアントを作成します。
    pub fn new() -> Self {
        let client = Ollama::default();
        Self { client, model: "deepseek-r1:1.5b".to_string() }
    }

    /// 指定されたテキストの埋め込みを生成します。
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        debug!("長さ{}のテキストの埋め込みを生成しています", text.len());

        let request = GenerateEmbeddingsRequest::new(
            self.model.clone(),
            EmbeddingsInput::Single(text.to_string()),
        );
        let response = self
            .client
            .generate_embeddings(request)
            .await
            .map_err(|e| AppError::OllamaApi(format!("埋め込み生成に失敗しました: {}", e)))?;

        // Vec<Vec<f32>> から最初のベクトルを返す
        let embeddings_f32: Vec<f32> = response.embeddings.get(0).cloned().unwrap_or_default();

        Ok(embeddings_f32)
    }

    /// 指定されたプロンプトの補完を生成します。
    pub async fn generate_completion(&self, prompt: &str) -> Result<String> {
        debug!("長さ{}のプロンプトの補完を生成しています", prompt.len());

        // リクエストを作成
        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());

        // 生成リクエストを送信
        let response = self
            .client
            .generate(request)
            .await
            .map_err(|e| AppError::OllamaApi(format!("補完生成に失敗しました: {}", e)))?;

        Ok(response.response)
    }
}
