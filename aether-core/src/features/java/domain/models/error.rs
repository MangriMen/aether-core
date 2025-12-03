use std::path::PathBuf;

use serr::SerializeError;

use crate::{libs::request_client::RequestError, shared::IoError};

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum JavaError {
    // Location errors
    #[error("No JRE found for version {version}")]
    NotFound { version: u32 },

    #[error("No JRE found at path: {path:?}")]
    InvalidPath { path: PathBuf },

    #[error("No JRE version found to download: version {version}, os {os}, architecture {arch}")]
    VersionNotAvailable {
        version: u32,
        os: String,
        arch: String,
    },

    #[error("Invalid JRE version: {version}")]
    InvalidVersion { version: String },

    #[error("Failed to get java properties: {reason}")]
    FailedToGetProperties { reason: String },

    #[error("Failed to remove old installation at {path:?}")]
    RemoveOldInstallationError { path: PathBuf },

    // Infrastructure errors
    #[error("Download failed: {0}")]
    DownloadFailed(#[from] RequestError),

    #[error("Unpack failed: {0}")]
    UnpackFailed(#[from] super::JavaUnpackError),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] IoError),
}
