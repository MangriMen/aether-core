use tracing_error::InstrumentError;

#[derive(thiserror::Error, Debug)]
pub enum PluginErrorKind {
    #[error("Function '{0}' not found")]
    FunctionNotFound(String),
    #[error("Error calling plugin function: {0}")]
    CallError(String),
}

#[derive(Debug)]
pub struct PluginError {
    pub raw: std::sync::Arc<PluginErrorKind>,
    pub source: tracing_error::TracedError<std::sync::Arc<PluginErrorKind>>,
}

impl std::error::Error for PluginError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.source)
    }
}

impl<E: Into<PluginErrorKind>> From<E> for PluginError {
    fn from(source: E) -> Self {
        let error = Into::<PluginErrorKind>::into(source);
        let boxed_error = std::sync::Arc::new(error);

        Self {
            raw: boxed_error.clone(),
            source: boxed_error.in_current_span(),
        }
    }
}

impl PluginErrorKind {
    pub fn as_error(self) -> PluginError {
        self.into()
    }
}
