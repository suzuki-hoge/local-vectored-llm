//! 埋め込み生成のためのOllama APIクライアント。

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

use crate::AppError;

/// Ollama APIのクライアント。
pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    /// 新しいOllamaクライアントを作成します。
    pub fn new(base_url: &str, model: &str) -> Self {
        let client = Client::builder().timeout(Duration::from_secs(60)).build().expect("HTTPクライアントの構築に失敗しました");

        Self { client, base_url: base_url.to_string(), model: model.to_string() }
    }

    /// 指定されたテキストの埋め込みを生成します。
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);

        let request = EmbeddingRequest { model: self.model.clone(), prompt: text.to_string() };

        debug!("長さ{}のテキストの埋め込みを生成しています", text.len());

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .with_context(|| format!("Ollama APIへのリクエスト送信に失敗しました: {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::OllamaApi(format!("Ollama APIがエラーを返しました {}: {}", status, error_text)).into());
        }

        let response: EmbeddingResponse = response.json().await.context("Ollama APIレスポンスの解析に失敗しました")?;

        Ok(response.embedding)
    }

    /// 指定されたプロンプトの補完を生成します。
    pub async fn generate_completion(&self, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);

        let request = CompletionRequest { model: self.model.clone(), prompt: prompt.to_string(), stream: false };

        debug!("長さ{}のプロンプトの補完を生成しています", prompt.len());

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .with_context(|| format!("Ollama APIへのリクエスト送信に失敗しました: {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::OllamaApi(format!("Ollama APIがエラーを返しました {}: {}", status, error_text)).into());
        }

        let response: CompletionResponse = response.json().await.context("Ollama APIレスポンスの解析に失敗しました")?;

        Ok(response.response)
    }
}

/// 埋め込み生成リクエスト。
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

/// 埋め込み生成レスポンス。
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

/// 補完生成リクエスト。
#[derive(Debug, Serialize)]
struct CompletionRequest {
    model: String,
    prompt: String,
    stream: bool,
}

/// 補完生成レスポンス。
#[derive(Debug, Deserialize)]
struct CompletionResponse {
    model: String,
    response: String,
}
