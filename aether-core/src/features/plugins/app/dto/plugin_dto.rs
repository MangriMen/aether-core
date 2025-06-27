use crate::features::plugins::{Plugin, PluginManifest};
use dashmap::mapref::{multiple::RefMulti as DashMapRefMulti, one::Ref as DashMapRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDto {
    pub manifest: PluginManifest,
    pub enabled: bool,
}

impl From<Plugin> for PluginDto {
    fn from(value: Plugin) -> Self {
        Self {
            manifest: value.manifest.clone(),
            enabled: value.is_loaded(),
        }
    }
}

impl From<DashMapRef<'_, String, Plugin>> for PluginDto {
    fn from(value: DashMapRef<'_, String, Plugin>) -> Self {
        Self {
            manifest: value.manifest.clone(),
            enabled: value.is_loaded(),
        }
    }
}

impl From<DashMapRefMulti<'_, String, Plugin>> for PluginDto {
    fn from(value: DashMapRefMulti<'_, String, Plugin>) -> Self {
        Self {
            manifest: value.manifest.clone(),
            enabled: value.is_loaded(),
        }
    }
}
