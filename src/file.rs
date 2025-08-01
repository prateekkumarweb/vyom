use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::chunk::ChunkMetadata;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileMetadata {
    name: String,
    size: u64,
    created_at: Timestamp,
    modified_at: Timestamp,
    chunks: Vec<ChunkMetadata>,
}

impl FileMetadata {
    pub fn new(name: String, size: u64, chunks: Vec<ChunkMetadata>) -> Self {
        let now = Timestamp::now();
        FileMetadata {
            name,
            size,
            created_at: now,
            modified_at: now,
            chunks,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn modified_at(&self) -> Timestamp {
        self.modified_at
    }

    pub fn chunks(&self) -> &[ChunkMetadata] {
        &self.chunks
    }
}
