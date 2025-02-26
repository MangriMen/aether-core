use std::path::{Path, PathBuf};

use extism::{Manifest, Plugin, Wasm};
use tokio::sync::Mutex;

use crate::state::LauncherState;

use super::{LauncherPlugin, PluginMetadata, PluginSettings};

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PluginContext {
    pub id: String,
}

#[derive(Debug, Default)]
pub struct PluginState {
    pub dir: PathBuf,
    pub metadata: PluginMetadata,
    plugin: Option<std::sync::Arc<Mutex<LauncherPlugin>>>,
}

impl Drop for PluginState {
    fn drop(&mut self) {
        let plugin_state = std::sync::Arc::new(Mutex::new(std::mem::take(self)));
        tokio::task::spawn_blocking(move || {
            tokio::task::block_in_place(|| {
                let plugin_state = plugin_state.clone();

                let mut plugin_state =
                    tokio::runtime::Handle::current().block_on(plugin_state.lock());
                tokio::runtime::Handle::current()
                    .block_on(plugin_state.unload())
                    .unwrap();
            });
        });
    }
}

impl PluginState {
    pub fn from_dir(dir: &Path) -> crate::Result<Self> {
        let plugin_metadata_path = dir.join("plugin.toml");
        let metadata_string = std::fs::read_to_string(&plugin_metadata_path)
            .map_err(|e| crate::utils::io::IOError::with_path(e, &plugin_metadata_path))?;
        let metadata = toml::from_str::<PluginMetadata>(&metadata_string)?;
        Ok(Self {
            dir: dir.to_path_buf(),
            metadata,
            plugin: None,
        })
    }

    fn get_host_functions(&self) -> Vec<extism::Function> {
        let context = PluginContext {
            id: self.metadata.plugin.id.to_string(),
        };

        let log_debug_fn = extism::Function::new(
            "log_debug",
            [extism::PTR],
            [],
            extism::UserData::new(context.clone()),
            super::host_functions::log_debug,
        );

        let download_file_fn = extism::Function::new(
            "download_file",
            [extism::PTR, extism::PTR],
            [],
            extism::UserData::new(context.clone()),
            super::host_functions::download_file,
        );

        let file_exists_fn = extism::Function::new(
            "file_exists",
            [extism::PTR],
            [extism::ValType::I64],
            extism::UserData::new(context.clone()),
            super::host_functions::file_exists,
        );

        let create_instance_fn = extism::Function::new(
            "create_instance",
            [
                extism::PTR,
                extism::PTR,
                extism::PTR,
                extism::PTR,
                extism::PTR,
                extism::PTR,
            ],
            [],
            extism::UserData::new(context.clone()),
            super::host_functions::create_instance,
        );

        vec![
            log_debug_fn,
            download_file_fn,
            file_exists_fn,
            create_instance_fn,
        ]
    }

    fn get_manifest(
        &self,
        cache_dir: &Path,
        instances_dir: &Path,
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

        let mut manifest = Manifest::new([file])
            .with_allowed_path(cache_dir.to_string_lossy().to_string(), "/cache")
            .with_allowed_path(instances_dir.to_string_lossy().to_string(), "/instances/*");

        if !allowed_hosts.is_empty() {
            manifest = manifest.with_allowed_hosts(allowed_hosts.into_iter());
        }

        if !allowed_paths.is_empty() {
            manifest = manifest.with_allowed_paths(allowed_paths.into_iter());
        }

        manifest
    }

    fn load_wasm_plugin(
        &self,
        cache_dir: &Path,
        instances_dir: &Path,
        settings: &Option<PluginSettings>,
    ) -> crate::Result<LauncherPlugin> {
        let manifest = self.get_manifest(cache_dir, instances_dir, settings);

        let path = &self.dir.join(&self.metadata.wasm.file);
        let plugin: LauncherPlugin = Plugin::new(&manifest, self.get_host_functions(), true)
            .map_err(|e| {
                log::debug!("Failed to load plugin: {:?}", e);
                crate::ErrorKind::PluginLoadError(path.to_string_lossy().to_string()).as_error()
            })?
            .try_into()
            .map_err(|e| {
                log::debug!("Failed to load plugin: {:?}", e);
                crate::ErrorKind::PluginLoadError(path.to_string_lossy().to_string()).as_error()
            })?;

        Ok(plugin)
    }

    pub async fn load(&mut self) -> crate::Result<()> {
        if self.plugin.is_some() {
            return Ok(());
        }

        let state = LauncherState::get().await?;
        let plugin_cache_dir = &state.locations.plugin_cache_dir(&self.metadata.plugin.id);
        let instances_dir = &state.locations.instances_dir();

        tokio::fs::create_dir_all(plugin_cache_dir).await?;
        tokio::fs::create_dir_all(instances_dir).await?;

        let plugin_settings =
            PluginSettings::from_path(&state.locations.plugin_settings(&self.metadata.plugin.id))
                .await?;

        let plugin = self.load_wasm_plugin(plugin_cache_dir, instances_dir, &plugin_settings)?;

        self.plugin = Some(std::sync::Arc::new(Mutex::new(plugin)));

        if let Some(plugin) = &self.plugin {
            let mut plugin = plugin.lock().await;

            if let Err(res) = plugin.on_load(()) {
                log::debug!("Failed to initialize plugin: {}", res);
            }
        }

        self.call().await;

        Ok(())
    }

    pub async fn unload(&mut self) -> crate::Result<()> {
        if let Some(plugin) = &self.plugin {
            let mut plugin = plugin.lock().await;

            if let Err(res) = plugin.on_load(()) {
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

    pub fn get_plugin(&mut self) -> Option<std::sync::Arc<Mutex<LauncherPlugin>>> {
        self.plugin.clone()
    }

    pub async fn call(&self) {
        if let Some(plugin) = &self.plugin {
            let mut plugin = plugin.lock().await;

            if let Err(res) = plugin.call(()) {
                log::debug!("Failed to call plugin: {}", res);
            }
        }
    }
}
