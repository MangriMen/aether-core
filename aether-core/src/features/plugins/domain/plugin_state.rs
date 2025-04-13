use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use extism::{Manifest, PluginBuilder, Wasm};
use tokio::sync::Mutex;

use crate::{
    core::LauncherState,
    features::plugins::{
        extism_host_functions, FsPluginSettingsStorage, LauncherPlugin, PluginSettings,
        PluginSettingsStorage, WasmCache, WasmCacheConfig,
    },
    shared::sha1_async,
};

use super::PluginMetadata;

pub fn get_default_allowed_paths(
    state: &LauncherState,
    plugin_id: &str,
) -> HashMap<String, PathBuf> {
    HashMap::from([
        (
            "/cache".to_owned(),
            state.locations.plugin_cache_dir(plugin_id),
        ),
        ("/instances".to_owned(), state.locations.instances_dir()),
    ])
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginContext {
    pub id: String,
}

#[derive(Debug, Default, Clone)]
pub struct PluginState {
    pub dir: PathBuf,
    pub metadata: PluginMetadata,
    pub plugin_hash: String,
    plugin: Option<std::sync::Arc<Mutex<LauncherPlugin>>>,
}

impl PluginState {
    pub async fn from_dir(dir: &Path) -> crate::Result<Self> {
        let plugin_metadata_path = dir.join("plugin.toml");
        let metadata_string = std::fs::read_to_string(&plugin_metadata_path)
            .map_err(|e| crate::shared::IOError::with_path(e, &plugin_metadata_path))?;
        let metadata = toml::from_str::<PluginMetadata>(&metadata_string)?;

        let plugin_file = crate::shared::read_async(dir.join(&metadata.wasm.file)).await?;
        let plugin_hash = sha1_async(plugin_file).await?;

        Ok(Self {
            dir: dir.to_path_buf(),
            metadata,
            plugin_hash,
            plugin: None,
        })
    }

    fn get_host_functions(&self) -> Vec<extism::Function> {
        let context = PluginContext {
            id: self.metadata.plugin.id.to_string(),
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

    fn get_manifest(
        &self,
        default_allowed_paths: &Option<HashMap<String, PathBuf>>,
        settings: &Option<PluginSettings>,
    ) -> Manifest {
        let file = Wasm::file(self.dir.join(&self.metadata.wasm.file));

        let mut allowed_hosts = self.metadata.wasm.allowed_hosts.clone().unwrap_or_default();
        let mut allowed_paths = self.metadata.wasm.allowed_paths.clone().unwrap_or_default();

        if let Some(settings) = settings {
            if let Some(hosts) = &settings.allowed_hosts {
                allowed_hosts.extend_from_slice(hosts);
            }
            if let Some(paths) = &settings.allowed_paths {
                allowed_paths.extend_from_slice(paths);
            }
        }

        let mut manifest = Manifest::new([file]);

        if let Some(default_allowed_paths) = default_allowed_paths {
            for (path, host) in default_allowed_paths {
                manifest = manifest.with_allowed_path(host.to_string_lossy().to_string(), path);
            }
        }

        if !allowed_hosts.is_empty() {
            manifest = manifest.with_allowed_hosts(allowed_hosts.into_iter());
        }

        if !allowed_paths.is_empty() {
            manifest = manifest.with_allowed_paths(allowed_paths.into_iter());
        }

        log::debug!("Manifest: {:?}", manifest);

        manifest
    }

    fn load_wasm_plugin(
        &self,
        cache_dir: &Option<PathBuf>,
        default_allowed_paths: &Option<HashMap<String, PathBuf>>,
        settings: &Option<PluginSettings>,
    ) -> crate::Result<LauncherPlugin> {
        let manifest = self.get_manifest(default_allowed_paths, settings);

        let path = &self.dir.join(&self.metadata.wasm.file);
        let mut builder = PluginBuilder::new(&manifest)
            .with_functions(self.get_host_functions())
            .with_wasi(true);

        if let Some(cache_dir) = cache_dir {
            builder = builder.with_cache_config(cache_dir);
        }

        let inner_plugin = builder.build().map_err(|e| {
            log::debug!("Failed to load plugin: {:?}", e);
            crate::ErrorKind::PluginLoadError(path.to_string_lossy().to_string()).as_error()
        })?;

        let plugin = LauncherPlugin::from_plugin(inner_plugin, &self.metadata.plugin.id)?;

        Ok(plugin)
    }

    pub async fn load(&mut self) -> crate::Result<()> {
        if self.plugin.is_some() {
            return Ok(());
        }

        let state = LauncherState::get().await?;

        let plugin_storage = FsPluginSettingsStorage;
        let plugin_settings = plugin_storage.get(&state, &self.metadata.plugin.id).await?;

        let default_allowed_paths = get_default_allowed_paths(&state, &self.metadata.plugin.id);

        for (_, host) in default_allowed_paths.iter() {
            tokio::fs::create_dir_all(host).await?;
        }

        let cache_config = state.locations.wasm_cache_config();
        if !cache_config.exists() {
            let cache_dir = state.locations.wasm_cache_dir();

            let cache = WasmCache {
                enabled: true,
                cleanup_interval: "30m".to_owned(),
                files_total_size_soft_limit: "1Gi".to_owned(),
                directory: cache_dir.clone(),
            };
            let config = WasmCacheConfig { cache };

            crate::shared::write_toml_async(&cache_config, config).await?;
            tokio::fs::create_dir_all(&cache_dir).await?;
        }

        let plugin = self.load_wasm_plugin(
            &Some(cache_config),
            &Some(default_allowed_paths),
            &plugin_settings,
        )?;

        self.plugin = Some(std::sync::Arc::new(Mutex::new(plugin)));

        if let Some(plugin) = &self.plugin {
            let mut plugin = plugin.lock().await;

            if let Err(res) = plugin.on_load() {
                log::debug!("Failed to initialize plugin: {}", res);
            }
        }

        Ok(())
    }

    pub async fn unload(&mut self) -> crate::Result<()> {
        if let Some(plugin) = &self.plugin {
            let mut plugin = plugin.lock().await;

            if let Err(res) = plugin.on_unload() {
                log::debug!("Failed to unload plugin: {}", res);
            }

            drop(plugin);
        }

        self.plugin = None;

        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.plugin.is_some()
    }

    pub fn get_plugin(&self) -> Option<std::sync::Arc<Mutex<LauncherPlugin>>> {
        self.plugin.clone()
    }
}
