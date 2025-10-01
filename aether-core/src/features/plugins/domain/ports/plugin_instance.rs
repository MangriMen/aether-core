use crate::features::{instance::ImportConfig, plugins::PluginError};

pub trait PluginInstance: Send + Sync {
    fn get_id(&self) -> String;
    fn supports(&self, name: &str) -> bool;

    fn on_load(&mut self) -> Result<(), PluginError>;
    fn on_unload(&mut self) -> Result<(), PluginError>;

    fn call_bytes<'b>(&'b mut self, name: &str, args: &[u8]) -> Result<&'b [u8], PluginError>;
}

pub trait PluginInstanceExt: PluginInstance {
    fn call<'a, 'b, T: extism::ToBytes<'a>, U: extism::FromBytes<'b>>(
        &'b mut self,
        name: &str,
        args: T,
    ) -> Result<U, PluginError> {
        let id = self.get_id();
        let map_err = |e: extism::Error| -> PluginError {
            PluginError::CallError {
                function_name: name.to_owned(),
                plugin_id: id.to_owned(),
                error: e.to_string(),
            }
        };

        use extism::{FromBytes as F, ToBytes as T};

        let args_b = T::to_bytes(&args).map_err(map_err)?;
        let result_b = self.call_bytes(name, args_b.as_ref())?;
        let result = F::from_bytes(result_b).map_err(map_err)?;
        Ok(result)
    }
}

impl<T: PluginInstance + ?Sized> PluginInstanceExt for T {}

#[derive(Debug, Clone, Copy)]
pub enum PluginFunction {
    GetImportConfig,
    Import,
    Update,
}

impl PluginFunction {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluginFunction::GetImportConfig => "get_import_config",
            PluginFunction::Import => "import",
            PluginFunction::Update => "update",
        }
    }
}

pub trait DefaultPluginInstanceFunctionsExt: PluginInstance {
    fn supports_get_import_config(&mut self) -> bool {
        self.supports(PluginFunction::GetImportConfig.as_str())
    }
    fn get_import_config(&mut self) -> Result<ImportConfig, PluginError> {
        self.call(PluginFunction::GetImportConfig.as_str(), ())
    }

    fn supports_import(&mut self) -> bool {
        self.supports(PluginFunction::Import.as_str())
    }
    fn import(&mut self, path_or_url: &str) -> Result<bool, PluginError> {
        self.call(PluginFunction::Import.as_str(), path_or_url)
    }

    fn supports_update(&mut self) -> bool {
        self.supports(PluginFunction::Update.as_str())
    }
    fn update(&mut self, instance_id: &str) -> Result<(), PluginError> {
        self.call(PluginFunction::Update.as_str(), instance_id)
    }
}

impl<T: PluginInstance + ?Sized> DefaultPluginInstanceFunctionsExt for T {}
