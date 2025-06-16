use tracing_error::InstrumentError;

use crate::{
    features::{auth, events, file_watcher, java, minecraft, plugins, process, settings},
    libs::request_client,
};

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error(transparent)]
    AuthError(#[from] auth::AuthError),

    #[error(transparent)]
    EventError(#[from] events::EventError),

    #[error(transparent)]
    FileWatcherError(#[from] file_watcher::FileWatcherError),

    #[error(transparent)]
    JavaError(#[from] java::JavaError),

    #[error(transparent)]
    MinecraftError(#[from] minecraft::MinecraftError),

    #[error(transparent)]
    PluginError(#[from] plugins::PluginError),

    #[error(transparent)]
    ProcessError(#[from] process::ProcessError),

    #[error(transparent)]
    SettingsError(#[from] settings::SettingsError),

    #[error("Request error: {0}")]
    RequestError(#[from] request_client::RequestError),

    /// Other errors

    #[error("Serialization error (JSON): {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("Unable to read {0} from any source")]
    NoValueFor(String),

    #[error("Metadata error: {0}")]
    MetadataError(#[from] daedalus::Error),

    #[error("I/O error: {0}")]
    IOError(#[from] crate::shared::IoError),

    #[error("I/O (std) error: {0}")]
    StdIOError(#[from] std::io::Error),

    #[error("Error launching Minecraft: {0}")]
    LauncherError(String),

    #[error("Error fetching URL: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("Join handle error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Recv error: {0}")]
    RecvError(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("Error acquiring semaphore: {0}")]
    AcquireError(#[from] tokio::sync::AcquireError),

    #[error("Instance {0} is not managed by the app!")]
    UnmanagedProfileError(String),

    #[error("Instance import error: {0}")]
    InstanceImportError(String),

    #[error("Instance update error: {0}")]
    InstanceUpdateError(String),

    #[error("Error parsing date: {0}")]
    ChronoParseError(#[from] chrono::ParseError),

    #[error("Error stripping prefix: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),

    #[error("Error: {0}")]
    OtherError(String),

    #[error("Serialization error (TOML): {0}")]
    TomlSerializationError(#[from] toml::ser::Error),

    #[error("Deserialization error (TOML): {0}")]
    TomlDeserializationError(#[from] toml::de::Error),

    #[error("Content provider not found: {provider}")]
    ContentProviderNotFound { provider: String },

    #[error("Content import duplicate error: {content_path}")]
    ContentImportDuplicateError { content_path: String },

    #[error("Storage error: {0}")]
    StorageError(#[from] crate::shared::StorageError),
}

#[derive(Debug)]
pub struct Error {
    pub raw: std::sync::Arc<ErrorKind>,
    pub source: tracing_error::TracedError<std::sync::Arc<ErrorKind>>,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.source)
    }
}

impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(source: E) -> Self {
        let error = Into::<ErrorKind>::into(source);
        let boxed_error = std::sync::Arc::new(error);

        Self {
            raw: boxed_error.clone(),
            source: boxed_error.in_current_span(),
        }
    }
}

impl ErrorKind {
    pub fn as_error(self) -> Error {
        self.into()
    }
}

pub type Result<T> = core::result::Result<T, Error>;
