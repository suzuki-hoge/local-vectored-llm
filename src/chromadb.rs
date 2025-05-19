//! ベクトルの保存と取得のためのChroma DBクライアント。

use anyhow::Result;
use chromadb::client::ChromaAuthMethod;
use chromadb::client::{ChromaClient as ChromaClientImpl, ChromaClientOptions};
use chromadb::collection::{CollectionEntries, QueryOptions};
use serde_json::{Map, Value};
use tracing::debug;

use crate::{AppError, DocumentChunk};

/// Chroma DB APIのクライアント。
pub struct ChromaClient {
    client: ChromaClientImpl,
    collection_name: String,
}

impl ChromaClient {
    /// 新しいChroma DBクライアントを作成します。
    pub async fn new() -> Result<Self> {
        let options = ChromaClientOptions {
            url: Some("http://localhost:18888".to_string()),
            auth: ChromaAuthMethod::None,
            database: "default_database".to_string(),
        };
        let client = ChromaClientImpl::new(options).await?;
        Ok(Self { client, collection_name: "local_files".to_string() })
    }

    /// コレクションを初期化します。
    pub async fn init_collection(&self) -> Result<()> {
        debug!("コレクションを初期化しています: {}", self.collection_name);
        self.client
            .create_collection(&self.collection_name, None, true)
            .await
            .map_err(|e| AppError::ChromaDb(format!("コレクションの初期化に失敗しました: {}", e)))?;
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

        let collection = self
            .client
            .get_collection(&self.collection_name)
            .await
            .map_err(|e| AppError::ChromaDb(format!("コレクションの取得に失敗しました: {}", e)))?;

        let id_strings: Vec<String> =
            chunks.iter().enumerate().map(|(i, chunk)| format!("{}_{}", chunk.source, i)).collect();
        let ids: Vec<&str> = id_strings.iter().map(|s| s.as_str()).collect();
        let metadatas: Vec<Map<String, Value>> = chunks
            .iter()
            .map(|chunk| {
                let mut metadata = Map::new();
                metadata.insert("source".to_string(), Value::String(chunk.source.clone()));
                metadata.insert("file_type".to_string(), Value::String(chunk.metadata.file_type.clone()));
                metadata.insert("chunk_index".to_string(), Value::Number(chunk.metadata.chunk_index.into()));
                if let Some(additional_str) = chunk.metadata.additional.as_str() {
                    metadata.insert("additional".to_string(), Value::String(additional_str.to_string()));
                } else {
                    metadata.insert("additional".to_string(), Value::String(chunk.metadata.additional.to_string()));
                }
                metadata
            })
            .collect();
        let doc_strings: Vec<String> = chunks.iter().map(|chunk| chunk.content.clone()).collect();
        let documents: Vec<&str> = doc_strings.iter().map(|s| s.as_str()).collect();

        debug!("{}個のドキュメントをコレクションに追加しています", chunks.len());

        let entries = CollectionEntries {
            ids,
            embeddings: Some(embeddings.to_vec()),
            metadatas: Some(metadatas),
            documents: Some(documents),
        };
        collection
            .upsert(entries, None)
            .await
            .map_err(|e| AppError::ChromaDb(format!("ドキュメントの追加に失敗しました: {}", e)))?;

        Ok(())
    }

    /// 類似ドキュメントをコレクションから検索します。
    pub async fn query(&self, query_embedding: &[f32], n_results: usize) -> Result<Vec<QueryResult>> {
        let collection = self
            .client
            .get_collection(&self.collection_name)
            .await
            .map_err(|e| AppError::ChromaDb(format!("コレクションの取得に失敗しました: {}", e)))?;

        debug!("{}件の結果をコレクションから検索しています", n_results);

        let query = QueryOptions {
            query_embeddings: Some(vec![query_embedding.to_vec()]),
            query_texts: None,
            n_results: Some(n_results),
            where_metadata: None,
            where_document: None,
            include: Some(vec!["documents", "metadatas", "distances"]),
        };
        let results = collection
            .query(query, None)
            .await
            .map_err(|e| AppError::ChromaDb(format!("ドキュメントの検索に失敗しました: {}", e)))?;

        let documents = results.documents.as_ref().and_then(|v| v.first());
        let metadatas = results.metadatas.as_ref().and_then(|v| v.first());
        let distances = results.distances.as_ref().and_then(|v| v.first());

        let mut query_results = Vec::new();
        if let (Some(documents), Some(metadatas), Some(distances)) = (documents, metadatas, distances) {
            for i in 0..documents.len() {
                query_results.push(QueryResult {
                    document: documents[i].clone(),
                    metadata: metadatas[i].clone().unwrap_or_default(),
                    distance: distances[i],
                });
            }
        }

        Ok(query_results)
    }
}

/// クエリの結果。
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub document: String,
    pub metadata: Map<String, Value>,
    pub distance: f32,
}
