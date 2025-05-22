use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

pub type CollectionName = String;

#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub file: FileMetadata,
    pub chunk: ChunkMetadata,
    pub search: SearchMetadata,
}

impl Metadata {
    pub fn to_map(&self) -> Map<String, Value> {
        json!({
            "file_path": self.file.path,
            "file_created_at": self.file.created_at.timestamp(),
            "file_updated_at": self.file.updated_at.timestamp(),
            "chunk_index": self.chunk.index
        })
        .as_object()
        .unwrap()
        .clone()
    }

    pub fn from_map(map: Map<String, Value>) -> Self {
        Self {
            file: FileMetadata {
                path: map.get("file_path").unwrap().as_str().unwrap().to_string(),
                created_at: DateTime::from_timestamp(map.get("file_created_at").unwrap().as_i64().unwrap(), 0).unwrap(),
                updated_at: DateTime::from_timestamp(map.get("file_updated_at").unwrap().as_i64().unwrap(), 0).unwrap(),
            },
            chunk: ChunkMetadata { index: map.get("chunk_index").unwrap().as_u64().unwrap() as usize },
            search: SearchMetadata {},
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMetadata {
    // 今後の拡張性のため
}
