use std::path::{Path, PathBuf};

use crate::{chunk_storage::ChunkManager, file::FileMetadata};

pub struct FileStorage {
    root_path: PathBuf,
    chunk_manager: ChunkManager,
    file_db: rocksdb::DB,
}

#[derive(Debug, thiserror::Error)]
pub enum VyomError {
    #[error(transparent)]
    ChunkError(#[from] crate::chunk_storage::ChunkError),
    #[error(transparent)]
    RocksdbError(#[from] rocksdb::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

type Result<T, E = VyomError> = std::result::Result<T, E>;

impl FileStorage {
    pub async fn new(root_path: impl AsRef<Path>, chunk_size: usize) -> Result<Self> {
        let chunk_manager = ChunkManager::new(root_path.as_ref(), chunk_size).await?;
        let file_db_path = root_path.as_ref().join("file_db");
        let file_db = rocksdb::DB::open_default(file_db_path)?;

        Ok(FileStorage {
            root_path: root_path.as_ref().to_path_buf(),
            chunk_manager,
            file_db,
        })
    }

    pub async fn get_file(&self, file_name: &str) -> Result<Option<Vec<u8>>> {
        let Some(file_metadata_bytes) = self.file_db.get(file_name)? else {
            return Ok(None);
        };
        let file_metadata: FileMetadata = serde_json::from_slice(&file_metadata_bytes)?;
        let mut data = Vec::with_capacity(file_metadata.size() as usize);

        for chunk in file_metadata.chunks() {
            let chunk_data = self.chunk_manager.storage.get_chunk(&chunk.hash()).await?;
            data.extend_from_slice(&chunk_data);
        }

        Ok(Some(data))
    }

    pub async fn put_file(&self, file_name: &str, data: &[u8]) -> Result<()> {
        let chunks = self.chunk_manager.chunk_file(data).await?;
        let file_metadata = FileMetadata::new(file_name.to_string(), data.len() as u64, chunks);
        let file_metadata = serde_json::to_vec(&file_metadata)?;
        self.file_db.put(file_name, &file_metadata)?;
        Ok(())
    }

    pub async fn del_file(&self, file_name: &str) -> Result<()> {
        self.file_db.delete(file_name)?;
        Ok(())
    }
}
