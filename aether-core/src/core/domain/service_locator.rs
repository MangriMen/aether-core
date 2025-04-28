use std::{collections::HashMap, sync::Arc};

use tokio::sync::{OnceCell, RwLock};

use crate::features::{
    instance::{ContentProviderService, FsPackStorage, ModrinthContentProvider},
    plugins::{
        ExtismPluginLoader, FsPluginSettingsStorage, FsPluginStorage, LoadConfigType,
        PluginService, PluginSettingsManagerImpl,
    },
    settings::FsSettingsStorage,
};

use super::LauncherState;

static SERVICE_LOCATOR: OnceCell<Arc<ServiceLocator>> = OnceCell::const_new();

pub type PluginServiceType = PluginService<
    FsSettingsStorage,
    FsPluginStorage,
    PluginSettingsManagerImpl<FsPluginSettingsStorage>,
    ExtismPluginLoader,
>;

pub struct ServiceLocator {
    pub plugin_service: RwLock<PluginServiceType>,
    pub plugin_settings_manager: Arc<PluginSettingsManagerImpl<FsPluginSettingsStorage>>,
    pub content_provider_service:
        Arc<ContentProviderService<FsPackStorage, ModrinthContentProvider>>,
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
        let plugin_storage = FsPluginStorage::new(state.locations.clone());
        let plugin_settings_manager = Arc::new(PluginSettingsManagerImpl::new(
            FsPluginSettingsStorage::new(state.locations.clone()),
        ));
        let settings_storage = FsSettingsStorage::new(&state.locations.settings_dir);
        let loaders = HashMap::from([(
            LoadConfigType::Extism,
            ExtismPluginLoader::new(state.locations.clone()),
        )]);

        let plugin_service = RwLock::new(PluginService::new(
            settings_storage,
            plugin_storage,
            plugin_settings_manager.clone(),
            loaders,
        ));

        let content_providers = HashMap::from([(
            "modrinth".to_string(),
            ModrinthContentProvider::new(
                state.api_semaphore.clone(),
                state.locations.clone(),
                None,
            ),
        )]);

        let pack_storage = FsPackStorage::new(state.locations.clone());
        let content_provider_service =
            Arc::new(ContentProviderService::new(pack_storage, content_providers));

        Ok(Arc::new(Self {
            plugin_service,
            plugin_settings_manager,
            content_provider_service,
        }))
    }
}
