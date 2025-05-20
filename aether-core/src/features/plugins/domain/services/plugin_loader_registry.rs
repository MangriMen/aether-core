use std::collections::HashMap;

use crate::{
    features::plugins::{LoadConfigType, PluginLoader},
    ErrorKind,
};

#[derive(Default)]
pub struct PluginLoaderRegistry<PL> {
    loaders: HashMap<LoadConfigType, PL>,
}

impl<PL> PluginLoaderRegistry<PL>
where
    PL: PluginLoader + Send + Sync,
{
    pub fn new(loaders: HashMap<LoadConfigType, PL>) -> Self {
        Self { loaders }
    }

    pub fn get(&self, load_config_type: &LoadConfigType) -> crate::Result<&PL> {
        self.loaders.get(load_config_type).ok_or_else(|| {
            ErrorKind::PluginLoadError(format!("Not found loader for {:?}", &load_config_type))
                .as_error()
        })
    }

    // pub fn list(&self) -> HashMap<LoadConfigType, String> {
    //     self.loaders.keys().collect()
    // }

    pub fn register(&mut self, load_config_type: LoadConfigType, provider: PL) {
        self.loaders.insert(load_config_type, provider);
    }

    pub fn unregister(&mut self, load_config_type: &LoadConfigType) {
        self.loaders.remove(load_config_type);
    }
}
