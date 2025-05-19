//! ベクトルの保存と取得のためのChroma DBクライアント。

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

use crate::{AppError, DocumentChunk};

/// Chroma DB APIのクライアント。
pub struct ChromaClient {
    client: Client,
    base_url: String,
    collection_name: String,
}

impl ChromaClient {
    /// 新しいChroma DBクライアントを作成します。
    pub fn new(base_url: &str, collection_name: &str) -> Self {
        let client = Client::new();

        Self { client, base_url: base_url.to_string(), collection_name: collection_name.to_string() }
    }

    /// コレクションを初期化します。
    pub async fn init_collection(&self) -> Result<()> {
        let url = format!("{}/api/v1/collections", self.base_url);

        // コレクションが存在するか確認
        let collections_response = self.client.get(&url).send().await.context("コレクションの取得に失敗しました")?;

        if !collections_response.status().is_success() {
            let status = collections_response.status();
            let error_text = collections_response.text().await.unwrap_or_default();
            return Err(AppError::ChromaDb(format!("Chroma DB APIがエラーを返しました {}: {}", status, error_text)).into());
        }

        let collections: CollectionsResponse =
            collections_response.json().await.context("コレクションレスポンスの解析に失敗しました")?;

        // コレクションが存在しない場合は作成
        if !collections.contains(&self.collection_name) {
            debug!("コレクションを作成しています: {}", self.collection_name);

            let create_request =
                CreateCollectionRequest { name: self.collection_name.clone(), metadata: HashMap::new() };

            let create_response =
                self.client.post(&url).json(&create_request).send().await.context("コレクションの作成に失敗しました")?;

            if !create_response.status().is_success() {
                let status = create_response.status();
                let error_text = create_response.text().await.unwrap_or_default();
                return Err(
                    AppError::ChromaDb(format!("コレクションの作成に失敗しました: {} - {}", status, error_text)).into()
                );
            }
        }

        Ok(())
    }

    /// ドキュメントチャンクをコレクションに追加します。
    pub async fn add_documents(&self, chunks: &[DocumentChunk], embeddings: &[Vec<f32>]) -> Result<()> {
        if chunks.is_empty() {
            return Ok(());
        }

        if chunks.len() != embeddings.len() {
            return Err(AppError::ChromaDb(format!(
                "チャンク数 ({}) と埋め込み数 ({}) が一致しません",
                chunks.len(),
                embeddings.len()
            ))
            .into());
        }

        let url = format!("{}/api/v1/collections/{}/add", self.base_url, self.collection_name);

        let ids: Vec<String> = chunks.iter().enumerate().map(|(i, chunk)| format!("{}_{}", chunk.source, i)).collect();

        let metadatas: Vec<HashMap<String, Value>> = chunks
            .iter()
            .map(|chunk| {
                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), Value::String(chunk.source.clone()));
                metadata.insert("file_type".to_string(), Value::String(chunk.metadata.file_type.clone()));
                metadata.insert("chunk_index".to_string(), Value::Number(chunk.metadata.chunk_index.into()));
                metadata.insert("additional".to_string(), chunk.metadata.additional.clone());
                metadata
            })
            .collect();

        let documents: Vec<String> = chunks.iter().map(|chunk| chunk.content.clone()).collect();

        let request = AddDocumentsRequest { ids, embeddings: embeddings.to_vec(), metadatas, documents };

        debug!("{}個のドキュメントをコレクションに追加しています", chunks.len());

        let response =
            self.client.post(&url).json(&request).send().await.context("コレクションへのドキュメント追加に失敗しました")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ChromaDb(format!("ドキュメントの追加に失敗しました: {} - {}", status, error_text)).into());
        }

        Ok(())
    }

    /// 類似ドキュメントをコレクションから検索します。
    pub async fn query(&self, query_embedding: &[f32], n_results: usize) -> Result<Vec<QueryResult>> {
        let url = format!("{}/api/v1/collections/{}/query", self.base_url, self.collection_name);

        let request = QueryRequest {
            query_embeddings: vec![query_embedding.to_vec()],
            n_results,
            include: vec!["documents".to_string(), "metadatas".to_string(), "distances".to_string()],
        };

        debug!("{}件の結果をコレクションから検索しています", n_results);

        let response = self.client.post(&url).json(&request).send().await.context("コレクションの検索に失敗しました")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ChromaDb(format!("ドキュメントの検索に失敗しました: {} - {}", status, error_text)).into());
        }

        let query_response: QueryResponse = response.json().await.context("検索レスポンスの解析に失敗しました")?;

        let mut results = Vec::new();

        if let (Some(documents), Some(metadatas), Some(distances)) =
            (query_response.documents.as_ref().map(|d| d.first()).flatten(),
             query_response.metadatas.as_ref().map(|m| m.first()).flatten(),
             query_response.distances.as_ref().map(|d| d.first()).flatten())
        {
            for i in 0..documents.len() {
                results.push(QueryResult {
                    document: documents[i].clone(),
                    metadata: metadatas[i].clone(),
                    distance: distances[i],
                });
            }
        }

        Ok(results)
    }
}

/// クエリの結果。
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub document: String,
    pub metadata: HashMap<String, Value>,
    pub distance: f32,
}

/// コレクション作成リクエスト。
#[derive(Debug, Serialize)]
struct CreateCollectionRequest {
    name: String,
    metadata: HashMap<String, Value>,
}

/// コレクション一覧レスポンス。
#[derive(Debug, Deserialize)]
struct CollectionsResponse(Vec<CollectionInfo>);

impl CollectionsResponse {
    fn contains(&self, name: &str) -> bool {
        self.0.iter().any(|info| info.name == name)
    }
}

/// コレクション情報。
#[derive(Debug, Deserialize)]
struct CollectionInfo {
    name: String,
}

/// ドキュメント追加リクエスト。
#[derive(Debug, Serialize)]
struct AddDocumentsRequest {
    ids: Vec<String>,
    embeddings: Vec<Vec<f32>>,
    metadatas: Vec<HashMap<String, Value>>,
    documents: Vec<String>,
}

/// クエリリクエスト。
#[derive(Debug, Serialize)]
struct QueryRequest {
    query_embeddings: Vec<Vec<f32>>,
    n_results: usize,
    include: Vec<String>,
}

/// クエリレスポンス。
#[derive(Debug, Deserialize)]
struct QueryResponse {
    ids: Vec<Vec<String>>,
    distances: Option<Vec<Vec<f32>>>,
    metadatas: Option<Vec<Vec<HashMap<String, Value>>>>,
    documents: Option<Vec<Vec<String>>>,
}
