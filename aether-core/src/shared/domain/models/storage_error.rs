use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("Value not found at path: {path}")]
    NotFound { path: PathBuf },

    #[error("Failed to read from storage: {0}")]
    ReadError(String),

    #[error("Failed to write to storage: {0}")]
    WriteError(String),

    #[error("Cache expired: {path}")]
    CacheExpired { path: PathBuf },
}
