use std::path::PathBuf;

use crate::shared::IoError;

#[derive(Debug, thiserror::Error)]
pub enum MinecraftError {
    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),

    #[error("Minecraft version {version} not found")]
    VersionNotFoundError { version: String },

    #[error("Path not found: {path:?}")]
    PathNotFoundError { path: PathBuf, entity_type: String },

    #[error("Error while parsing libraries: {0}")]
    ParseError(#[from] daedalus::Error),

    #[error("Failed to execute pre-launch command")]
    PreLaunchCommandError { code: i32 },

    #[error("Failed to execute post-launch command")]
    PostLaunchCommandError { code: i32 },

    #[error("Modloader processor error: {0}")]
    ModLoaderProcessorError(String),
}
