use core::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkHash(String);

impl ChunkHash {
    pub const fn new(hash: String) -> Self {
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn get_prefix(&self) -> &str {
        &self.0[..2]
    }
}

impl fmt::Display for ChunkHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChunkMetadata {
    hash: ChunkHash,
    size: u64,
}

impl ChunkMetadata {
    pub const fn new(hash: ChunkHash, size: u64) -> Self {
        Self { hash, size }
    }

    pub const fn hash(&self) -> &ChunkHash {
        &self.hash
    }

    pub const fn size(&self) -> u64 {
        self.size
    }
}
