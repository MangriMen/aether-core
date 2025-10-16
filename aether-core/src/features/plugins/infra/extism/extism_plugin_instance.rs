use crate::features::plugins::{PluginError, PluginInstance, PluginInstanceExt};

#[derive(Debug)]
pub struct ExtismPluginInstance {
    id: String,
    inner: extism::Plugin,
}

impl ExtismPluginInstance {
    pub fn new(plugin: extism::Plugin, id: String) -> Self {
        Self { inner: plugin, id }
    }
}

impl PluginInstance for ExtismPluginInstance {
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn supports(&self, name: &str) -> bool {
        self.inner.function_exists(name)
    }

    fn on_load(&mut self) -> Result<(), PluginError> {
        if !self.supports("on_load") {
            return Ok(());
        }

        self.call("on_load", ())
    }
    fn on_unload(&mut self) -> Result<(), PluginError> {
        if !self.supports("on_unload") {
            return Ok(());
        }

        self.call("on_load", ())
    }

    fn call_bytes<'b>(&'b mut self, name: &str, args: &[u8]) -> Result<&'b [u8], PluginError> {
        self.inner
            .call(name, args)
            .map_err(|e| PluginError::CallError {
                function_name: name.to_owned(),
                plugin_id: self.id.to_owned(),
                error: e.to_string(),
            })
    }

    // fn supports_handle_events(&self) -> bool {
    //     self.supports("handle_event")
    // }

    // fn handle_event(&mut self, event: &PluginEvent) -> Result<(), PluginError> {
    //     self.call::<PluginEvent, ()>("handle_event", event.clone())
    // }
}
