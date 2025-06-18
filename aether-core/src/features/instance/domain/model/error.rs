use std::path::PathBuf;

use uuid::Uuid;

use crate::{
    features::{
        auth::AuthError, minecraft::MinecraftError, process::ProcessError, settings::SettingsError,
    },
    shared::IoError,
};

#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),

    #[error("Content provider \"\" not found")]
    ContentProviderNotFound { provider_id: String },

    #[error("Instance \"{instance_id}\" still in installing state")]
    InstanceStillInstalling { instance_id: String },

    #[error("Instance \"{instance_id}\" already running with pid \"{process_id}\"")]
    InstanceAlreadyRunning {
        instance_id: String,
        process_id: Uuid,
    },

    #[error("Prelaunch command error with code: {code}")]
    PrelaunchCommandError { code: i32 },

    #[error("Instance validation error: \"{field}\" is invalid because \"{reason}\"")]
    ValidationError { field: String, reason: String },

    #[error("Failed to construct hash")]
    HashConstructError,

    #[error("Failed to import instance: {0}")]
    InstanceImportError(String),

    #[error("Failed to update instance: {0}")]
    InstanceUpdateError(String),

    #[error("Unmanaged instance")]
    UnmanagedInstance { instance_id: String },

    // Content
    #[error("Found duplicate content at {content_path}")]
    ContentDuplication { content_path: String },

    #[error("Can't get content filename at path: {path}")]
    ContentFilename { path: PathBuf },

    #[error("Error when downloading content")]
    ContentDownloadError(String),

    #[error("Not found content for minecraft version \"{game_version}\"")]
    ContentForGameVersionNotFound { game_version: String },

    // Features errors
    #[error("Settings load error")]
    SettingsLoadError(#[from] SettingsError),

    #[error("Failed to get launch command")]
    MinecraftError(#[from] MinecraftError),

    #[error("Failed to launch instance")]
    ProcessError(#[from] ProcessError),

    #[error("Credentials load error")]
    CredentialsError(#[from] AuthError),
}
