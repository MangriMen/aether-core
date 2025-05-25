use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    features::{
        plugins::{
            extism_host_functions, plugin_utils::get_default_allowed_paths, LoadConfig,
            PathMapping, Plugin, PluginInstance, PluginLoader, PluginSettings,
        },
        settings::LocationInfo,
    },
    ErrorKind,
};

use super::wasm_cache::{WasmCache, WasmCacheConfig};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PluginContext {
    pub id: String,
}

pub struct ExtismPluginLoader {
    location_info: Arc<LocationInfo>,
}

impl ExtismPluginLoader {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    async fn get_cache_config_path(&self) -> crate::Result<PathBuf> {
        let cache_config = self.location_info.wasm_cache_config();
        if !cache_config.exists() {
            let cache_dir = self.location_info.wasm_cache_dir();

            let config = WasmCacheConfig {
                cache: WasmCache {
                    enabled: true,
                    cleanup_interval: "30m".to_owned(),
                    files_total_size_soft_limit: "1Gi".to_owned(),
                    directory: cache_dir.clone(),
                },
            };

            crate::shared::write_toml_async(&cache_config, config).await?;
            tokio::fs::create_dir_all(&cache_dir).await?;
        }

        Ok(cache_config)
    }

    fn resolve_allowed_paths(
        plugin: &Plugin,
        settings: &Option<PluginSettings>,
        default_allowed_paths: &Option<HashMap<String, PathBuf>>,
    ) -> (Vec<String>, Vec<PathMapping>) {
        let mut allowed_hosts = plugin.manifest.runtime.allowed_hosts.clone();
        let mut allowed_paths = plugin.manifest.runtime.allowed_paths.clone();

        if let Some(settings) = settings {
            allowed_hosts.extend_from_slice(&settings.allowed_hosts);
            allowed_paths.extend_from_slice(&settings.allowed_paths);
        }

        if let Some(default_allowed_paths) = default_allowed_paths {
            for (host_path, plugin_path) in default_allowed_paths {
                allowed_paths.push((host_path.to_string(), plugin_path.to_path_buf()));
            }
        }

        (allowed_hosts, allowed_paths)
    }

    fn construct_wasm_manifest(
        wasm_file: &Path,
        allowed_hosts: &[String],
        allowed_paths: &[PathMapping],
    ) -> extism::Manifest {
        let wasm_file = extism::Wasm::file(wasm_file);
        extism::Manifest::new([wasm_file])
            .with_allowed_hosts(allowed_hosts.iter().cloned())
            .with_allowed_paths(allowed_paths.iter().cloned())
    }

    fn load_extism_plugin(
        &self,
        plugin: &Plugin,
        cache_dir: &Option<PathBuf>,
        default_allowed_paths: &Option<HashMap<String, PathBuf>>,
        settings: &Option<PluginSettings>,
    ) -> crate::Result<extism::Plugin> {
        let wasm_file_path = match &plugin.manifest.load {
            LoadConfig::Extism { file, .. } => file,
            _ => return Err(ErrorKind::PluginLoadError("Not supported".to_string()).as_error()),
        };

        let (allowed_hosts, allowed_paths) =
            Self::resolve_allowed_paths(plugin, settings, default_allowed_paths);

        let absolute_wasm_file_path = self
            .location_info
            .plugin_dir(&plugin.manifest.metadata.id)
            .join(wasm_file_path);

        let manifest =
            Self::construct_wasm_manifest(&absolute_wasm_file_path, &allowed_hosts, &allowed_paths);

        let mut builder = extism::PluginBuilder::new(&manifest)
            .with_functions(Self::get_host_functions(&plugin.manifest.metadata.id))
            .with_wasi(true);

        if let Some(cache_dir) = cache_dir {
            builder = builder.with_cache_config(cache_dir);
        }

        builder.build().map_err(|e| {
            log::debug!("Failed to load plugin: {:?}", e);
            crate::ErrorKind::PluginLoadError(absolute_wasm_file_path.to_string_lossy().to_string())
                .as_error()
        })
    }

    fn get_host_functions(plugin_id: &str) -> Vec<extism::Function> {
        let context = PluginContext {
            id: plugin_id.to_string(),
        };

        let log_fn = extism::Function::new(
            "log",
            [extism::ValType::I64, extism::PTR],
            [],
            extism::UserData::new(context.clone()),
            extism_host_functions::log,
        );

        let get_id_fn = extism::Function::new(
            "get_id",
            [],
            [extism::PTR],
            extism::UserData::new(context.clone()),
            extism_host_functions::get_id,
        );

        let instance_get_dir_fn = extism::Function::new(
            "instance_get_dir",
            [extism::PTR],
            [extism::PTR],
            extism::UserData::new(context.clone()),
            extism_host_functions::instance_get_dir,
        );

        let instance_plugin_get_dir_fn = extism::Function::new(
            "instance_plugin_get_dir",
            [extism::PTR],
            [extism::PTR],
            extism::UserData::new(context.clone()),
            extism_host_functions::instance_plugin_get_dir,
        );

        let instance_create_fn = extism::Function::new(
            "instance_create",
            [
                extism::PTR,
                extism::PTR,
                extism::PTR,
                extism::PTR,
                extism::PTR,
                extism::PTR,
                extism::PTR,
            ],
            [extism::PTR],
            extism::UserData::new(context.clone()),
            extism_host_functions::instance_create,
        );

        let get_or_download_java_fn = extism::Function::new(
            "get_or_download_java",
            [extism::PTR],
            [extism::ValType::I64],
            extism::UserData::new(context.clone()),
            extism_host_functions::get_java,
        );

        let run_command_fn = extism::Function::new(
            "run_command",
            [extism::PTR],
            [extism::PTR],
            extism::UserData::new(context.clone()),
            extism_host_functions::run_command,
        );

        vec![
            log_fn,
            get_id_fn,
            instance_get_dir_fn,
            instance_plugin_get_dir_fn,
            instance_create_fn,
            get_or_download_java_fn,
            run_command_fn,
        ]
    }
}

#[async_trait]
impl PluginLoader for ExtismPluginLoader {
    async fn load(
        &self,
        plugin: &Plugin,
        settings: &Option<PluginSettings>,
    ) -> crate::Result<PluginInstance> {
        let cache_config = self.get_cache_config_path().await?;

        let default_allowed_paths =
            get_default_allowed_paths(&self.location_info, &plugin.manifest.metadata.id);

        for (_, host) in default_allowed_paths.iter() {
            tokio::fs::create_dir_all(host).await?;
        }

        let extism_plugin = self.load_extism_plugin(
            plugin,
            &Some(cache_config),
            &Some(default_allowed_paths),
            settings,
        )?;

        let mut plugin = PluginInstance::from_plugin(extism_plugin, &plugin.manifest.metadata.id)?;

        plugin.on_load()?;

        Ok(plugin)
    }

    async fn unload(&self, instance: Arc<Mutex<PluginInstance>>) -> crate::Result<()> {
        let mut plugin = instance.lock().await;

        if let Err(res) = plugin.on_unload() {
            log::debug!("Failed to unload plugin: {}", res);
        }

        drop(plugin);

        Ok(())
    }
}
