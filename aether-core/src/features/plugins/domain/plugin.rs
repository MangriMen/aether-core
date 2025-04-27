use std::{collections::HashMap, path::PathBuf, sync::Arc};

use tokio::sync::Mutex;

use crate::features::{plugins::PluginInstance, settings::LocationInfo};

use super::PluginManifest;

#[derive(Debug, Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub hash: String,
    pub instance: Option<Arc<Mutex<PluginInstance>>>,
}

impl Plugin {
    pub fn is_loaded(&self) -> bool {
        self.instance.is_some()
    }
}

pub fn get_default_allowed_paths(
    location_info: &LocationInfo,
    plugin_id: &str,
) -> HashMap<String, PathBuf> {
    HashMap::from([
        (
            "/cache".to_owned(),
            location_info.plugin_cache_dir(plugin_id),
        ),
        ("/instances".to_owned(), location_info.instances_dir()),
    ])
}
