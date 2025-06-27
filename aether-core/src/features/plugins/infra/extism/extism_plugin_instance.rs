use crate::features::{
    instance::ImportConfig,
    plugins::{PluginError, PluginEvent, PluginInstance},
};

#[derive(Debug)]
pub struct ExtismPluginInstance {
    inner: extism::Plugin,
    id: String,
}

impl ExtismPluginInstance {
    pub fn new(plugin: extism::Plugin, id: String) -> Self {
        Self { inner: plugin, id }
    }

    fn call<'a, 'b, T: extism::ToBytes<'a>, U: extism::FromBytes<'b>>(
        &'b mut self,
        name: &str,
        args: T,
    ) -> Result<U, PluginError> {
        self.inner
            .call::<T, U>(name, args)
            .map_err(|e| PluginError::CallError {
                function_name: name.to_owned(),
                plugin_id: self.id.to_owned(),
                error: e.to_string(),
            })
    }
}

impl PluginInstance for ExtismPluginInstance {
    fn function_exists(&self, name: &str) -> bool {
        self.inner.function_exists(name)
    }

    fn on_load(&mut self) -> Result<(), PluginError> {
        self.call::<(), ()>("on_load", ())
    }
    fn on_unload(&mut self) -> Result<(), PluginError> {
        self.call::<(), ()>("on_unload", ())
    }

    fn supports_get_import_config(&self) -> bool {
        self.function_exists("get_import_config")
    }

    fn get_import_config(&mut self) -> Result<ImportConfig, PluginError> {
        self.call::<(), ImportConfig>("get_import_config", ())
    }

    fn supports_import(&self) -> bool {
        self.function_exists("import")
    }

    fn import(&mut self, url_or_path: &str) -> Result<(), PluginError> {
        self.call::<String, ()>("import", url_or_path.to_string())
    }

    fn update(&mut self, instance_id: &str) -> Result<(), PluginError> {
        self.call::<String, ()>("update", instance_id.to_string())
    }

    fn supports_handle_events(&self) -> bool {
        self.function_exists("handle_event")
    }

    fn handle_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
        self.call::<PluginEvent, ()>("handle_event", event.clone())
    }
}
