use std::path::{Path, PathBuf};

use crate::chunk_storage::ChunkStorage;

pub struct FileStorage {
    root_path: PathBuf,
    chunk_storage: ChunkStorage,
    chunk_size: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum VyomError {
    #[error(transparent)]
    ChunkError(#[from] crate::chunk_storage::ChunkError),
}

type Result<T, E = VyomError> = std::result::Result<T, E>;

impl FileStorage {
    pub async fn new(root_path: impl AsRef<Path>, chunk_size: usize) -> Result<Self> {
        let chunk_storage = ChunkStorage::new(root_path.as_ref()).await?;
        Ok(FileStorage {
            root_path: root_path.as_ref().to_path_buf(),
            chunk_storage,
            chunk_size,
        })
    }

    pub async fn get_file(&self, file_name: &str) -> Result<Vec<u8>> {
        todo!()
    }

    pub async fn put_file(&self, file_name: &str, data: &[u8]) -> Result<()> {
        todo!()
    }

    pub async fn del_file(&self, file_name: &str) -> Result<()> {
        todo!()
    }
}
