use std::path::PathBuf;

use crate::{features::minecraft::LoaderVersionPreference, shared::IoError};

#[derive(Debug, thiserror::Error)]
pub enum MinecraftError {
    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),

    #[error("Minecraft version {version} not found")]
    VersionNotFoundError { version: String },

    // TODO: maybe specify message in which case it throws
    #[error("Minecraft version not found for loader version {loader_version_preference:?}")]
    MinecraftVersionForLoaderNotFoundError {
        loader_version_preference: LoaderVersionPreference,
    },

    #[error("Loader version {loader_version_preference:?} not found")]
    LoaderVersionNotFoundError {
        loader_version_preference: LoaderVersionPreference,
    },

    #[error(
        "Loader version not specified. If loader is not vanilla, loader version must be specified"
    )]
    LoaderVersionNotSpecified,

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

    #[error("No java found at path: {path:?}")]
    JavaNotFound { path: PathBuf },

    #[error("Java version {version} not found")]
    JavaVersionNotFound { version: u32 },

    #[error("Java auto installation failed: {0}")]
    JavaInstallationFailed(String),
}
