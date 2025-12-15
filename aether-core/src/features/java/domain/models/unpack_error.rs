use serde::{Deserialize, Serialize};

use crate::shared::IoError;

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum JavaUnpackError {
    #[error(transparent)]
    Io(#[from] IoError),

    #[error("Invalid archive: {0}")]
    InvalidArchive(String),

    #[error("Unsupported archive: {0}")]
    UnsupportedArchive(String),

    #[error("File not found")]
    FileNotFound,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("{0}")]
    Other(String),
}
