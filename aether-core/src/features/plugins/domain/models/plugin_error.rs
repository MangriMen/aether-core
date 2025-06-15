use crate::{
    features::{
        plugins::{LoadConfig, LoadConfigType},
        settings::SettingsError,
    },
    shared::IoError,
};

#[derive(thiserror::Error, Debug)]
pub enum PluginError {
    #[error("Error when calling function \"{function_name}\" in plugin with id \"{plugin_id}\": {error}")]
    CallError {
        function_name: String,
        plugin_id: String,
        error: String,
    },

    #[error("Not found loader for: {load_config_type:?}")]
    LoaderNotFoundError { load_config_type: LoadConfigType },

    #[error("Invalid load config: {load_config:?}")]
    InvalidLoadConfigError { load_config: LoadConfig },

    #[error("Plugin with id \"{plugin_id}\" not found")]
    PluginNotFoundError { plugin_id: String },

    #[error("Error while loading plugin")]
    PluginLoadError,

    #[error("Storage failure: {0}")]
    StorageFailure(#[from] IoError),

    #[error("Failed to construct hash")]
    HashConstructError,

    #[error("Error when updating settings: {0}")]
    SettingsError(#[from] SettingsError),

    #[error("Plugin \"{plugin_id}\" tried to access disallowed path \"{path}\"")]
    PluginPathAccessViolationError { plugin_id: String, path: String },

    #[error("Host function error: {0}")]
    HostFunctionError(String),
}
