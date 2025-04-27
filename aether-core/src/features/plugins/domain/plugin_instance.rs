use crate::features::{instance::ImportConfig, plugins::PluginError};

use super::{PluginApi, PluginEvent};

#[derive(Debug)]
pub struct PluginInstance<P: PluginApi = extism::Plugin> {
    pub inner: P,
    pub public_id: String,
}

impl<P: PluginApi> PluginInstance<P> {
    fn get_error(&self, e: PluginError) -> crate::Error {
        crate::ErrorKind::PluginCallError(self.public_id.to_string(), e).as_error()
    }

    pub fn from_plugin(plugin: P, id: &str) -> crate::Result<Self> {
        if !plugin.function_exists("on_load") || !plugin.function_exists("on_unload") {
            return Err(crate::ErrorKind::PluginLoadError(format!(
                "Plugin {} is missing required functions",
                id
            ))
            .as_error());
        }

        Ok(Self {
            inner: plugin,
            public_id: id.to_string(),
        })
    }

    pub fn on_load(&mut self) -> crate::Result<()> {
        self.inner
            .call::<(), ()>("on_load", ())
            .map_err(|e| self.get_error(e))
    }
    pub fn on_unload(&mut self) -> crate::Result<()> {
        self.inner
            .call::<(), ()>("on_unload", ())
            .map_err(|e| self.get_error(e))
    }

    pub fn supports_get_import_config(&self) -> bool {
        self.inner.function_exists("get_import_config")
    }

    pub fn get_import_config(&mut self) -> crate::Result<ImportConfig> {
        self.inner
            .call::<(), ImportConfig>("get_import_config", ())
            .map_err(|e| self.get_error(e))
    }

    pub fn supports_import(&self) -> bool {
        self.inner.function_exists("import")
    }

    pub fn import(&mut self, url_or_path: &str) -> crate::Result<()> {
        self.inner
            .call::<String, ()>("import", url_or_path.to_string())
            .map_err(|e| self.get_error(e))
    }

    pub fn update(&mut self, instance_id: &str) -> crate::Result<()> {
        self.inner
            .call::<String, ()>("update", instance_id.to_string())
            .map_err(|e| self.get_error(e))
    }

    pub fn supports_handle_events(&self) -> bool {
        self.inner.function_exists("handle_event")
    }

    pub fn handle_event(&mut self, event: &PluginEvent) -> crate::Result<()> {
        self.inner
            .call::<PluginEvent, ()>("handle_event", event.clone())
            .map_err(|e| self.get_error(e))
    }
}
