use crate::chroma::document::{CollectionName, Document, Metadata};
use anyhow::Result;
use chromadb::client::ChromaClient;
use chromadb::client::ChromaClientOptions;
use chromadb::collection::{CollectionEntries, QueryOptions};
use ollama_rs::generation::embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest};
use ollama_rs::Ollama;

pub struct ChromaStore {
    client: ChromaClient,
    ollama: Ollama,
}

#[derive(Debug)]
pub struct CollectionInfo {
    pub name: String,
    pub count: usize,
}

impl ChromaStore {
    pub async fn new() -> Result<Self> {
        let options = ChromaClientOptions { url: Some("http://localhost:18888".to_string()), ..Default::default() };
        let client = ChromaClient::new(options).await?;
        Ok(Self { client, ollama: Ollama::new("http://localhost", 11434) })
    }

    pub async fn get_collections(&self) -> Result<Vec<CollectionInfo>> {
        let collections = self.client.list_collections().await?;
        let mut result = Vec::new();

        for collection in collections {
            let count = collection.count().await?;
            result.push(CollectionInfo { name: collection.name().to_string(), count });
        }

        result.sort_by_key(|c| c.name.to_string());
        Ok(result)
    }

    pub async fn get_collection_documents(&self, collection_name: &str) -> Result<Vec<Document>> {
        let collection = self.client.get_collection(collection_name).await?;
        let count = collection.count().await?;
        let results = collection.peek(count).await?;

        let mut result = Vec::new();
        let ids = results.ids;
        let documents = results.documents.unwrap_or_default();
        let metadatas = results.metadatas.unwrap_or_default();

        for (i, id) in ids.into_iter().enumerate() {
            let content = documents.get(i).and_then(|c| c.clone()).unwrap_or_default();
            let metadata_map = metadatas.get(i).and_then(|m| m.clone()).unwrap();
            result.push(Document { id, content, metadata: Metadata::from_map(metadata_map) });
        }

        Ok(result)
    }

    pub async fn save(&self, document: &Document, collection_name: &CollectionName) -> Result<()> {
        let embedding = self.generate_embedding(&document.content).await?;

        let collection = self.client.get_or_create_collection(collection_name, None).await?;

        let entries = CollectionEntries {
            ids: vec![&document.id],
            metadatas: Some(vec![document.metadata.to_map()]),
            documents: Some(vec![&document.content]),
            embeddings: Some(vec![embedding]),
        };

        collection.add(entries, None).await?;
        Ok(())
    }

    pub async fn search(&self, query: &str, limit: usize, collection_names: &[&str]) -> Result<Vec<String>> {
        let query_embedding = self.generate_embedding(query).await?;
        let mut all_results = Vec::new();

        for collection_name in collection_names {
            let collection = self.client.get_collection(collection_name).await?;
            let options = QueryOptions {
                query_embeddings: Some(vec![query_embedding.clone()]),
                n_results: Some(limit),
                ..Default::default()
            };
            let results = collection.query(options, None).await?;
            if let Some(docs) = results.documents {
                all_results.extend(docs.into_iter().flatten());
            }
        }

        // 結果を重複排除して返す
        all_results.sort();
        all_results.dedup();

        // 指定されたlimitを超えないように調整
        if all_results.len() > limit {
            all_results.truncate(limit);
        }

        Ok(all_results)
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let req = GenerateEmbeddingsRequest::new(
            "7shi/ezo-gemma-2-jpn:2b-instruct-q8_0".to_string(),
            EmbeddingsInput::Single(text.to_string()),
        );
        let result = self.ollama.generate_embeddings(req).await?;
        Ok(result.embeddings.into_iter().next().unwrap_or_default())
    }
}
