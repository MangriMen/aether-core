use serr::SerializeError;

use crate::shared::IoError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum SettingsError {
    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),
}
