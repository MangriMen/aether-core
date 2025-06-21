use serr::SerializeError;

use crate::shared::IoError;

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum ProcessError {
    #[error("Error when killing process with id: {id}")]
    KillError { id: String },

    #[error("Error while waiting process with id: {id}")]
    WaitError { id: String },

    #[error("Error while spawning process: {0}")]
    SpawnError(#[from] IoError),
}
