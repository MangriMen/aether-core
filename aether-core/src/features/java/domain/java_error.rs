use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum JavaError {
    #[error("No JRE found for required version: {version}")]
    NoJREFound { version: u32 },

    #[error("No JRE found at path: {path:?}")]
    NoJREFoundAtPath { path: PathBuf },

    #[error("Invalid JRE version string: {version}")]
    InvalidJREVersion { version: String },
}
