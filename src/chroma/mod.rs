use anyhow::Result;
use chromadb::client::ChromaClient;
use chromadb::client::ChromaClientOptions;
use chromadb::collection::{CollectionEntries, QueryOptions};
use ollama_rs::generation::embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest};
use ollama_rs::Ollama;
use serde_json::json;

pub struct ChromaStore {
    client: ChromaClient,
    collection_name: String,
    ollama: Ollama,
}

impl ChromaStore {
    pub async fn new() -> Result<Self> {
        let options = ChromaClientOptions { url: Some("http://localhost:18888".to_string()), ..Default::default() };
        let client = ChromaClient::new(options).await?;
        Ok(Self { client, collection_name: "default".to_string(), ollama: Ollama::new("http://localhost", 11434) })
    }

    pub async fn save(&self, document: &crate::document::ProcessedDocument) -> Result<()> {
        let embedding = self.generate_embedding(&document.content).await?;

        let collection = self.client.get_or_create_collection(&self.collection_name, None).await?;

        let metadata = json!({
            "source": &document.source,
            "chunk_index": document.chunk_index,
        });

        let id = format!("{}-{}", document.source, document.chunk_index);
        let entries = CollectionEntries {
            ids: vec![&id],
            metadatas: Some(vec![metadata.as_object().unwrap().clone()]),
            documents: Some(vec![&document.content]),
            embeddings: Some(vec![embedding]),
        };

        collection.add(entries, None).await?;
        Ok(())
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let query_embedding = self.generate_embedding(query).await?;
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

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let req =
            GenerateEmbeddingsRequest::new("deepseek-r1:1.5b".to_string(), EmbeddingsInput::Single(text.to_string()));
        let result = self.ollama.generate_embeddings(req).await?;
        Ok(result.embeddings.into_iter().next().unwrap_or_default())
    }
}
