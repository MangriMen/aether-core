use uuid::Uuid;

use crate::shared::IoError;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Credentials with id: {id} not found")]
    CredentialsNotFound { id: Uuid },

    #[error("Active credentials not found")]
    NoActiveCredentials,

    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),
}
