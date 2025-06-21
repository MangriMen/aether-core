use std::path::PathBuf;

use serr::SerializeError;

use crate::{libs::request_client::RequestError, shared::IoError};

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum JavaError {
    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),

    #[error("No java found for version {version}")]
    JavaNotFound { version: u32 },

    #[error("No JRE found at path: {path:?}")]
    InvalidPath { path: PathBuf },

    #[error("Invalid JRE version: {version}")]
    InvalidVersion { version: String },

    #[error("No Java version found to download: version {version}, os {os}, architecture {arch}")]
    JavaDownloadNotFound {
        version: u32,
        os: String,
        arch: String,
    },

    #[error("Failed to get java properties: {reason}")]
    FailedToGetProperties { reason: String },

    #[error("Unpack error: {0}")]
    UnpackError(#[from] zip::result::ZipError),

    #[error("Failed to remove old installation at {path:?}")]
    RemoveOldInstallationError { path: PathBuf },

    #[error("Download error")]
    DownloadError(#[from] RequestError),
}
