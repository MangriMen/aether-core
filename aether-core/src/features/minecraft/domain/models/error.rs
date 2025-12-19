use std::path::PathBuf;

use serr::SerializeError;

use crate::{features::minecraft::LoaderVersionPreference, shared::IoError};

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum MinecraftDomainError {
    #[error("Minecraft version \"{version}\" not found")]
    VersionNotFound { version: String },

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

    #[error("Stable or latest loader could not be resolved")]
    DefaultLoaderVersionNotFound,

    #[error("Modloader processor failed: {reason}")]
    ModLoaderProcessorFailed { reason: String },

    #[error("Path not found: {path:?}")]
    PathNotFound { path: PathBuf, entity_type: String },

    #[error("No java found at path: {path:?}")]
    JavaNotFound { path: PathBuf },

    #[error("Java version {version} not found")]
    JavaVersionNotFound { version: u32 },

    #[error("Java auto installation failed: {0}")]
    JavaInstallationFailed(String),

    #[error("Failed to execute pre-launch command")]
    PreLaunchCommandFailed { code: i32 },

    #[error("Failed to execute post-launch command")]
    PostLaunchCommandFailed { code: i32 },

    // External
    #[error("Error while parsing libraries: {0}")]
    LibraryParse(#[from] daedalus::Error),

    // Infrastructure
    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),
}
