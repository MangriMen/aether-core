use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum JavaError {
    #[error("No JRE found for version {version}")]
    NotFound { version: u32 },

    #[error("No JRE found at path: {path:?}")]
    InvalidPath { path: PathBuf },

    #[error("Invalid JRE version: {version}")]
    InvalidVersion { version: String },

    #[error("Failed to get java properties: {reason}")]
    FailedToGetProperties { reason: String },
}
