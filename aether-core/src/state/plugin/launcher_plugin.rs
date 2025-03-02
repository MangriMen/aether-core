use crate::state::ImportConfig;

#[derive(Debug)]
pub struct LauncherPlugin {
    pub inner: extism::Plugin,
    pub public_id: String,
}

impl LauncherPlugin {
    fn get_error(&self, e: extism::Error) -> crate::Error {
        crate::ErrorKind::PluginCallError(self.public_id.to_string(), e).as_error()
    }

    pub fn from_plugin(plugin: extism::Plugin, id: &str) -> crate::Result<Self> {
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

    pub(super) fn on_load(&mut self) -> crate::Result<()> {
        self.inner
            .call::<(), ()>("on_load", ())
            .map_err(|e| self.get_error(e))
    }
    pub(super) fn on_unload(&mut self) -> crate::Result<()> {
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
}
