use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
};

use crate::chunk::{ChunkHash, ChunkMetadata};

pub struct ChunkStorage {
    chunks_dir: PathBuf,
    temp_dir: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("chunk not found: {0}")]
    ChunkNotFound(ChunkHash),
    #[error("corrupt chunk: {0}")]
    CorruptChunk(ChunkHash),
}

type Result<T, E = ChunkError> = std::result::Result<T, E>;

impl ChunkStorage {
    #[tracing::instrument(skip(root_dir))]
    pub async fn new(root_dir: impl AsRef<Path>) -> Result<Self> {
        let root_path = root_dir.as_ref();
        let chunks_dir = root_path.join("chunks");
        let temp_dir = root_path.join("temp");

        fs::create_dir_all(&chunks_dir).await?;
        fs::create_dir_all(&temp_dir).await?;

        Ok(Self {
            chunks_dir,
            temp_dir,
        })
    }

    fn get_chunk_path(&self, chunk_hash: &ChunkHash) -> PathBuf {
        let prefix = chunk_hash.get_prefix();
        let prefix_dir = self.chunks_dir.join(prefix);
        prefix_dir.join(chunk_hash.as_str())
    }

    #[tracing::instrument(skip(self, data))]
    pub async fn store_chunk(&self, data: &[u8]) -> Result<ChunkMetadata> {
        let hash = Sha256::digest(data);
        let chunk_hash = ChunkHash::new(format!("{hash:x}"));

        let chunk_path = self.get_chunk_path(&chunk_hash);
        if chunk_path.exists() {
            let metadata = fs::metadata(&chunk_path).await?;
            return Ok(ChunkMetadata::new(chunk_hash, metadata.len()));
        }

        // Chunk path is concatenation of chunk_dir and hash, so parent cannot be none
        #[allow(clippy::unwrap_used)]
        let prefix_dir = chunk_path.parent().unwrap();
        fs::create_dir_all(prefix_dir).await?;

        let temp_path = self.temp_dir.join(format!("chunk_{}", chunk_hash.as_str()));
        let mut file = fs::File::create(&temp_path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;

        fs::rename(&temp_path, &chunk_path).await?;

        Ok(ChunkMetadata::new(chunk_hash, data.len() as u64))
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_chunk(&self, hash: &ChunkHash) -> Result<Vec<u8>> {
        let chunk_path = self.get_chunk_path(hash);
        if !chunk_path.exists() {
            return Err(ChunkError::ChunkNotFound(hash.clone()));
        }

        let data = fs::read(&chunk_path).await?;
        let computed_hash = Sha256::digest(&data);
        let computed_hash = format!("{computed_hash:x}");

        if computed_hash != hash.as_str() {
            return Err(ChunkError::CorruptChunk(hash.clone()));
        }

        Ok(data)
    }

    pub fn chunk_exists(&self, hash: &ChunkHash) -> bool {
        let chunk_path = self.get_chunk_path(hash);
        chunk_path.exists()
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_chunk(&self, hash: &ChunkHash) -> Result<()> {
        let chunk_path = self.get_chunk_path(hash);

        if chunk_path.exists() {
            fs::remove_file(chunk_path).await?;
        }

        Ok(())
    }
}

pub struct ChunkManager {
    pub(crate) storage: ChunkStorage,
    pub(crate) chunk_size: usize,
}

impl ChunkManager {
    #[tracing::instrument(skip(root_path))]
    pub async fn new(root_path: impl AsRef<Path>, chunk_size: usize) -> Result<Self> {
        let storage = ChunkStorage::new(root_path).await?;
        Ok(Self {
            storage,
            chunk_size,
        })
    }

    #[tracing::instrument(skip(self, reader))]
    pub async fn chunk_file<R: AsyncRead + Unpin>(
        &self,
        mut reader: R,
    ) -> Result<Vec<ChunkMetadata>> {
        let mut chunks = Vec::new();
        let mut buffer = vec![0u8; self.chunk_size];

        loop {
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            let chunk_data = &buffer[..bytes_read];
            let chunk_metadata = self.storage.store_chunk(chunk_data).await?;
            chunks.push(chunk_metadata);
        }

        Ok(chunks)
    }
}
