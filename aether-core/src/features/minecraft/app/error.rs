use serr::SerializeError;

use crate::{
    features::{java::JavaDomainError, minecraft::MinecraftDomainError},
    shared::IoError,
};

#[derive(Debug, thiserror::Error, SerializeError)]
pub enum MinecraftApplicationError {
    #[error(transparent)]
    Domain(#[from] MinecraftDomainError),

    #[error(transparent)]
    JavaError(#[from] JavaDomainError),

    #[error("Failed to parse libraries: {0}")]
    LibraryParse(#[from] daedalus::Error),

    #[error("Storage operation failed: {0}")]
    Storage(#[from] IoError),
}
