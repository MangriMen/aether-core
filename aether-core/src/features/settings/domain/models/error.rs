use crate::shared::IoError;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),
}
