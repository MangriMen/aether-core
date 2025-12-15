use crate::{
    features::java::{JavaError, JavaUnpackError},
    shared::IoError,
};

impl From<zip::result::ZipError> for JavaUnpackError {
    fn from(err: zip::result::ZipError) -> Self {
        match err {
            zip::result::ZipError::Io(io) => Self::Io(IoError::IoError(io)),
            zip::result::ZipError::InvalidArchive(msg) => Self::InvalidArchive(msg.into_owned()),
            zip::result::ZipError::UnsupportedArchive(s) => Self::UnsupportedArchive(s.to_string()),
            zip::result::ZipError::FileNotFound => Self::FileNotFound,
            zip::result::ZipError::InvalidPassword => Self::InvalidPassword,
            _ => Self::Other(err.to_string()),
        }
    }
}

impl From<zip::result::ZipError> for JavaError {
    fn from(err: zip::result::ZipError) -> Self {
        JavaError::UnpackFailed(JavaUnpackError::from(err))
    }
}
