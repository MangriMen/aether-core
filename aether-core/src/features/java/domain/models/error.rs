use std::path::PathBuf;

use crate::{libs::request_client::RequestError, shared::StorageError};

#[derive(thiserror::Error, Debug)]
pub enum JavaError {
    #[error("No JRE found for version {version}")]
    JreNotFound { version: u32 },

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

    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
}
