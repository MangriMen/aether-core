use std::{collections::HashMap, sync::Arc};

use tokio::sync::{OnceCell, RwLock};

use crate::features::{
    plugins::{
        ExtismPluginLoader, FsPluginSettingsStorage, FsPluginStorage, LoadConfigType, PluginService,
    },
    settings::FsSettingsStorage,
};

use super::LauncherState;

static SERVICE_LOCATOR: OnceCell<Arc<ServiceLocator>> = OnceCell::const_new();

pub type PluginServiceType =
    PluginService<FsSettingsStorage, FsPluginStorage, FsPluginSettingsStorage, ExtismPluginLoader>;

pub struct ServiceLocator {
    pub plugin_service: RwLock<PluginServiceType>,
}

impl ServiceLocator {
    pub async fn init(state: &LauncherState) -> crate::Result<()> {
        SERVICE_LOCATOR
            .get_or_try_init(|| Self::initialize(state))
            .await?;

        Ok(())
    }

    #[tracing::instrument]
    pub async fn get() -> crate::Result<Arc<Self>> {
        if !SERVICE_LOCATOR.initialized() {
            tracing::error!(
                "Attempted to get service locator before it is initialized - this should never happen!\n{:?}",
                std::backtrace::Backtrace::capture()
            );

            while !SERVICE_LOCATOR.initialized() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }

        Ok(Arc::clone(
            SERVICE_LOCATOR.get().expect("State is not initialized!"),
        ))
    }

    pub async fn initialized() -> bool {
        SERVICE_LOCATOR.initialized()
    }

    pub async fn initialize(state: &LauncherState) -> crate::Result<Arc<Self>> {
        let plugin_storage = FsPluginStorage::new(state.location_info.clone());
        let plugin_settings_storage =
            Arc::new(FsPluginSettingsStorage::new(state.location_info.clone()));

        let settings_storage = FsSettingsStorage::new(&state.location_info.settings_dir);
        let loaders = HashMap::from([(
            LoadConfigType::Extism,
            ExtismPluginLoader::new(state.location_info.clone()),
        )]);

        let plugin_service = RwLock::new(PluginService::new(
            settings_storage,
            plugin_storage,
            plugin_settings_storage.clone(),
            loaders,
        ));

        Ok(Arc::new(Self { plugin_service }))
    }
}
