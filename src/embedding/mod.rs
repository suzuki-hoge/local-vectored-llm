use anyhow::Result;
use chromadb::client::ChromaClient;
use chromadb::client::ChromaClientOptions;
use chromadb::collection::{CollectionEntries, QueryOptions};
use serde_json::json;

pub struct EmbeddingStore {
    client: ChromaClient,
    collection_name: String,
}

impl EmbeddingStore {
    pub async fn new() -> Result<Self> {
        let options = ChromaClientOptions { url: Some("http://localhost:8000".to_string()), ..Default::default() };
        let client = ChromaClient::new(options).await?;
        Ok(Self { client, collection_name: "default".to_string() })
    }

    pub async fn store_embeddings(
        &self,
        text: &str,
        embedding: Vec<f32>,
        source: String,
        chunk_index: usize,
    ) -> Result<()> {
        let collection = self.client.get_or_create_collection(&self.collection_name, None).await?;

        let metadata = json!({
            "source": source,
            "chunk_index": chunk_index,
        });

        let entries = CollectionEntries {
            documents: Some(vec![text]),
            embeddings: Some(vec![embedding]),
            metadatas: Some(vec![metadata.as_object().unwrap().clone()]),
            ids: Vec::new(),
        };

        collection.add(entries, None).await?;
        Ok(())
    }

    pub async fn search(&self, query_embedding: Vec<f32>, limit: usize) -> Result<Vec<String>> {
        let collection = self.client.get_collection(&self.collection_name).await?;
        let options = QueryOptions {
            query_embeddings: Some(vec![query_embedding]),
            n_results: Some(limit),
            ..Default::default()
        };
        let results = collection.query(options, None).await?;
        let docs = results.documents.unwrap_or_default();
        Ok(docs.into_iter().flatten().collect())
    }
}
