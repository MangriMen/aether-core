use crate::features::{
    instance::ImportConfig,
    plugins::{PluginError, PluginEvent},
};

pub trait PluginInstance: Send + Sync {
    fn function_exists(&self, name: &str) -> bool;

    fn on_load(&mut self) -> Result<(), PluginError>;
    fn on_unload(&mut self) -> Result<(), PluginError>;

    fn supports_handle_events(&self) -> bool;
    fn handle_event(&mut self, event: &PluginEvent) -> Result<(), PluginError>;

    fn supports_get_import_config(&self) -> bool;
    fn get_import_config(&mut self) -> Result<ImportConfig, PluginError>;

    fn supports_import(&self) -> bool;
    fn import(&mut self, url_or_path: &str) -> Result<(), PluginError>;

    fn update(&mut self, instance_id: &str) -> Result<(), PluginError>;
}
