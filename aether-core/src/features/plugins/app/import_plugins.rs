use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    features::{
        events::EventEmitter,
        plugins::{PluginError, PluginLoader, PluginManifest, PluginStorage, SyncPluginsUseCase},
        settings::{LocationInfo, SettingsStorage},
    },
    shared::{read_async, IoError},
};

pub struct ImportPluginsUseCase<
    PS: PluginStorage,
    SS: SettingsStorage,
    PL: PluginLoader,
    E: EventEmitter,
> {
    location_info: Arc<LocationInfo>,
    sync_plugins_use_case: Arc<SyncPluginsUseCase<PS, SS, PL, E>>,
}

impl<PS: PluginStorage, SS: SettingsStorage, PL: PluginLoader, E: EventEmitter>
    ImportPluginsUseCase<PS, SS, PL, E>
{
    pub fn new(
        location_info: Arc<LocationInfo>,
        sync_plugins_use_case: Arc<SyncPluginsUseCase<PS, SS, PL, E>>,
    ) -> Self {
        Self {
            location_info,
            sync_plugins_use_case,
        }
    }

    pub async fn execute(&self, paths: Vec<PathBuf>) -> Result<(), PluginError> {
        for path in paths {
            self.unpack_plugin(&path).await?
        }

        self.sync_plugins_use_case.execute().await?;
        Ok(())
    }

    async fn unpack_plugin(&self, file_path: &Path) -> Result<(), PluginError> {
        let get_error = || PluginError::ImportError {
            path: file_path.to_path_buf(),
        };

        let file = read_async(file_path).await?;
        let mut archive = zip::ZipArchive::new(Cursor::new(file)).map_err(|_| get_error())?;

        let manifest: PluginManifest = {
            let mut manifest_file = archive.by_name("manifest.toml").map_err(|_| get_error())?;

            let mut content = String::new();
            let read_count = manifest_file
                .read_to_string(&mut content)
                .map_err(|_| get_error())?;

            if read_count == 0 {
                return Err(get_error());
            };

            toml::from_str(&content).map_err(|e| IoError::DeserializationError(e.to_string()))
        }?;

        let plugin_id = manifest.metadata.id;

        let plugin_dir = self.location_info.plugin_dir(&plugin_id);

        archive.extract(plugin_dir).map_err(|_| get_error())?;

        Ok(())
    }
}
