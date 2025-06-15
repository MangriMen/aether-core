use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::ReadDir;

#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("{source}, path: {path}")]
    IOPathError {
        #[source]
        source: std::io::Error,
        path: String,
    },
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl IoError {
    pub fn from(source: std::io::Error) -> Self {
        Self::IOError(source)
    }

    pub fn with_path(source: std::io::Error, path: impl AsRef<Path>) -> Self {
        Self::IOPathError {
            source,
            path: path.as_ref().to_string_lossy().to_string(),
        }
    }
}

// Bytes

pub async fn read_async(path: impl AsRef<Path>) -> Result<Vec<u8>, IoError> {
    let path_ref = path.as_ref();
    tokio::fs::read(path_ref)
        .await
        .map_err(|err| IoError::with_path(err, path_ref))
}

pub async fn write_async(path: impl AsRef<Path>, data: impl AsRef<[u8]>) -> Result<(), IoError> {
    let path_ref = path.as_ref();

    if let Some(parent) = path_ref.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| IoError::with_path(err, parent))?;
    }

    tokio::fs::write(path_ref, data)
        .await
        .map_err(|err| IoError::with_path(err, path_ref))
}

// JSON

pub async fn read_json_async<T>(path: impl AsRef<Path>) -> Result<T, IoError>
where
    T: DeserializeOwned,
{
    let bytes = read_async(&path).await?;
    serde_json::from_slice(&bytes).map_err(|e| IoError::DeserializationError(e.to_string()))
}

pub async fn write_json_async<T>(path: impl AsRef<Path>, data: T) -> Result<(), IoError>
where
    T: Serialize,
{
    let bytes =
        serde_json::to_vec(&data).map_err(|e| IoError::SerializationError(e.to_string()))?;
    write_async(path, bytes).await
}

pub async fn ensure_read_json_async<T>(path: impl AsRef<Path>) -> Result<T, IoError>
where
    T: Serialize + DeserializeOwned + Default,
{
    let path_ref = path.as_ref();

    if !path_ref.exists() {
        let default = T::default();
        write_json_async(path, &default).await?;
        return Ok(default);
    }

    read_json_async(path).await
}

// TOML

pub async fn read_toml_async<T>(path: impl AsRef<Path>) -> Result<T, IoError>
where
    T: DeserializeOwned,
{
    let bytes = read_async(&path).await?;
    let toml_str =
        std::str::from_utf8(&bytes).map_err(|e| IoError::DeserializationError(e.to_string()))?;

    toml::from_str(toml_str).map_err(|e| IoError::DeserializationError(e.to_string()))
}

pub async fn write_toml_async<T>(path: impl AsRef<Path>, data: T) -> Result<(), IoError>
where
    T: Serialize,
{
    let toml_str =
        toml::to_string(&data).map_err(|e| IoError::SerializationError(e.to_string()))?;
    write_async(path, toml_str).await
}

// dunce canonicalize

pub fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf, IoError> {
    let path_ref = path.as_ref();
    dunce::canonicalize(path_ref).map_err(|e| IoError::with_path(e, path_ref))
}

pub async fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), IoError> {
    let from_ref = from.as_ref();
    tokio::fs::rename(from_ref, to.as_ref())
        .await
        .map_err(|e| IoError::with_path(e, from_ref))
}

pub async fn remove_file(path: impl AsRef<Path>) -> Result<(), IoError> {
    let path_ref = path.as_ref();
    tokio::fs::remove_file(path_ref)
        .await
        .map_err(|e| IoError::with_path(e, path_ref))
}

pub async fn create_dir_all(path: impl AsRef<Path>) -> Result<(), IoError> {
    let path_ref = path.as_ref();
    tokio::fs::create_dir_all(path_ref)
        .await
        .map_err(|e| IoError::with_path(e, path_ref))
}

pub async fn read_dir(path: impl AsRef<Path>) -> Result<ReadDir, IoError> {
    let path_ref = path.as_ref();
    tokio::fs::read_dir(path_ref)
        .await
        .map_err(|e| IoError::with_path(e, path_ref))
}
