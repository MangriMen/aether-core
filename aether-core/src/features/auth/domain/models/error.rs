use uuid::Uuid;

use crate::shared::StorageError;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error(transparent)]
    StorageError(#[from] StorageError),

    #[error("Credentials with id: {id} not found")]
    CredentialsNotFound { id: Uuid },
}
