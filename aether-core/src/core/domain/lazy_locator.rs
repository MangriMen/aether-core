use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::OnceCell;

use crate::{
    features::{
        auth::FsCredentialsStorage,
        events::{InMemoryProgressBarStorage, ProgressServiceImpl, TauriEventEmitter},
        file_watcher::NotifyFileWatcher,
        instance::{
            ContentProviderRegistry, EventEmittingInstanceStorage, FsInstanceStorage,
            FsPackStorage, InstanceEventHandler, InstanceWatcherServiceImpl,
            ModrinthContentProvider,
        },
        java::infra::FsJavaStorage,
        minecraft::{CachedMetadataStorage, FsMetadataStorage, ModrinthMetadataStorage},
        plugins::{
            ExtismPluginLoader, FsPluginSettingsStorage, FsPluginStorage, LoadConfigType,
            PluginLoaderRegistry, PluginRegistry,
        },
        process::InMemoryProcessStorage,
        settings::FsSettingsStorage,
    },
    shared::{ReqwestClient, REQWEST_CLIENT},
};

use super::{ErrorKind, LauncherState};

static LAZY_LOCATOR: OnceCell<Arc<LazyLocator>> = OnceCell::const_new();

const CACHE_TTL: Duration = Duration::from_secs(120);

type ProgressServiceType = ProgressServiceImpl<TauriEventEmitter, InMemoryProgressBarStorage>;

pub struct LazyLocator {
    state: Arc<LauncherState>,
    app_handle: tauri::AppHandle,
    request_client: OnceCell<Arc<ReqwestClient<ProgressServiceType>>>,
    api_client: OnceCell<Arc<ReqwestClient<ProgressServiceType>>>,
    credentials_storage: OnceCell<Arc<FsCredentialsStorage>>,
    settings_storage: OnceCell<Arc<FsSettingsStorage>>,
    process_storage: OnceCell<Arc<InMemoryProcessStorage>>,
    instance_storage:
        OnceCell<Arc<EventEmittingInstanceStorage<TauriEventEmitter, FsInstanceStorage>>>,
    java_storage: OnceCell<Arc<FsJavaStorage>>,
    metadata_storage: OnceCell<
        Arc<
            CachedMetadataStorage<
                FsMetadataStorage,
                ModrinthMetadataStorage<ReqwestClient<ProgressServiceType>>,
            >,
        >,
    >,
    pack_storage: OnceCell<Arc<FsPackStorage>>,
    content_provider_registry: OnceCell<
        Arc<ContentProviderRegistry<ModrinthContentProvider<ReqwestClient<ProgressServiceType>>>>,
    >,
    plugin_settings_storage: OnceCell<Arc<FsPluginSettingsStorage>>,
    plugin_registry: OnceCell<Arc<PluginRegistry>>,
    plugin_loader_registry: OnceCell<Arc<PluginLoaderRegistry<ExtismPluginLoader>>>,
    plugin_storage: OnceCell<Arc<FsPluginStorage>>,
    event_emitter: OnceCell<Arc<TauriEventEmitter>>,
    progress_bar_storage: OnceCell<Arc<InMemoryProgressBarStorage>>,
    progress_service: OnceCell<Arc<ProgressServiceType>>,
    instance_watcher_service: OnceCell<
        Arc<InstanceWatcherServiceImpl<NotifyFileWatcher<InstanceEventHandler<TauriEventEmitter>>>>,
    >,
}

impl LazyLocator {
    pub async fn init(
        state: Arc<LauncherState>,
        app_handle: tauri::AppHandle,
    ) -> crate::Result<()> {
        LAZY_LOCATOR
            .get_or_init(|| async {
                Arc::new(Self {
                    state,
                    app_handle,
                    request_client: OnceCell::new(),
                    api_client: OnceCell::new(),
                    credentials_storage: OnceCell::new(),
                    settings_storage: OnceCell::new(),
                    process_storage: OnceCell::new(),
                    instance_storage: OnceCell::new(),
                    java_storage: OnceCell::new(),
                    metadata_storage: OnceCell::new(),
                    pack_storage: OnceCell::new(),
                    content_provider_registry: OnceCell::new(),
                    plugin_settings_storage: OnceCell::new(),
                    plugin_registry: OnceCell::new(),
                    plugin_loader_registry: OnceCell::new(),
                    plugin_storage: OnceCell::new(),
                    event_emitter: OnceCell::new(),
                    progress_bar_storage: OnceCell::new(),
                    progress_service: OnceCell::new(),
                    instance_watcher_service: OnceCell::new(),
                })
            })
            .await;

        Ok(())
    }

    pub async fn get() -> crate::Result<Arc<Self>> {
        if !LAZY_LOCATOR.initialized() {
            tracing::error!(
                "Attempted to get LazyLocator before it is initialized - this should never happen!\n{:?}",
                std::backtrace::Backtrace::capture()
            );

            Self::wait_for_initialization().await?;
        }

        LAZY_LOCATOR.get().map(Arc::clone).ok_or_else(|| {
            ErrorKind::LauncherError("LazyLocator is not initialized!".to_string()).as_error()
        })
    }

    async fn wait_for_initialization() -> crate::Result<()> {
        const INIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
        const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

        let start = std::time::Instant::now();

        while !LAZY_LOCATOR.initialized() {
            if start.elapsed() > INIT_TIMEOUT {
                return Err(ErrorKind::LauncherError(
                    "LazyLocator initialization timeout".to_string(),
                )
                .as_error());
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }

        Ok(())
    }

    pub async fn get_request_client(&self) -> Arc<ReqwestClient<ProgressServiceType>> {
        self.request_client
            .get_or_init(|| async {
                Arc::new(ReqwestClient::new(
                    self.get_progress_service().await,
                    (*REQWEST_CLIENT).clone(),
                    self.state.fetch_semaphore.clone(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_api_client(&self) -> Arc<ReqwestClient<ProgressServiceType>> {
        self.api_client
            .get_or_init(|| async {
                Arc::new(ReqwestClient::new(
                    self.get_progress_service().await,
                    (*REQWEST_CLIENT).clone(),
                    self.state.api_semaphore.clone(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_credentials_storage(&self) -> Arc<FsCredentialsStorage> {
        self.credentials_storage
            .get_or_init(|| async {
                Arc::new(FsCredentialsStorage::new(
                    &self.state.location_info.settings_dir,
                ))
            })
            .await
            .clone()
    }

    pub async fn get_settings_storage(&self) -> Arc<FsSettingsStorage> {
        self.settings_storage
            .get_or_init(|| async {
                Arc::new(FsSettingsStorage::new(
                    &self.state.location_info.settings_dir,
                ))
            })
            .await
            .clone()
    }

    pub async fn get_process_storage(&self) -> Arc<InMemoryProcessStorage> {
        self.process_storage
            .get_or_init(|| async { Arc::new(InMemoryProcessStorage::default()) })
            .await
            .clone()
    }

    pub async fn get_instance_storage(
        &self,
    ) -> Arc<EventEmittingInstanceStorage<TauriEventEmitter, FsInstanceStorage>> {
        self.instance_storage
            .get_or_init(|| async {
                Arc::new(EventEmittingInstanceStorage::new(
                    self.get_event_emitter().await,
                    FsInstanceStorage::new(self.state.location_info.clone()),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_java_storage(&self) -> Arc<FsJavaStorage> {
        self.java_storage
            .get_or_init(|| async {
                Arc::new(FsJavaStorage::new(&self.state.location_info.java_dir()))
            })
            .await
            .clone()
    }

    pub async fn get_metadata_storage(
        &self,
    ) -> Arc<
        CachedMetadataStorage<
            FsMetadataStorage,
            ModrinthMetadataStorage<ReqwestClient<ProgressServiceType>>,
        >,
    > {
        self.metadata_storage
            .get_or_init(|| async {
                Arc::new(CachedMetadataStorage::new(
                    FsMetadataStorage::new(&self.state.location_info.cache_dir(), Some(CACHE_TTL)),
                    ModrinthMetadataStorage::new(self.get_request_client().await),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_pack_storage(&self) -> Arc<FsPackStorage> {
        self.pack_storage
            .get_or_init(|| async {
                Arc::new(FsPackStorage::new(self.state.location_info.clone()))
            })
            .await
            .clone()
    }

    pub async fn get_content_provider_registry(
        &self,
    ) -> Arc<ContentProviderRegistry<ModrinthContentProvider<ReqwestClient<ProgressServiceType>>>>
    {
        self.content_provider_registry
            .get_or_init(|| async {
                let providers = HashMap::from([(
                    "modrinth".to_string(),
                    ModrinthContentProvider::new(
                        self.state.location_info.clone(),
                        None,
                        self.get_request_client().await,
                    ),
                )]);

                Arc::new(ContentProviderRegistry::new(providers))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_settings_storage(&self) -> Arc<FsPluginSettingsStorage> {
        self.plugin_settings_storage
            .get_or_init(|| async {
                Arc::new(FsPluginSettingsStorage::new(
                    self.state.location_info.clone(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_registry(&self) -> Arc<PluginRegistry> {
        self.plugin_registry
            .get_or_init(|| async { Arc::new(PluginRegistry::default()) })
            .await
            .clone()
    }

    pub async fn get_plugin_loader_registry(
        &self,
    ) -> Arc<PluginLoaderRegistry<ExtismPluginLoader>> {
        self.plugin_loader_registry
            .get_or_init(|| async {
                let loaders = HashMap::from([(
                    LoadConfigType::Extism,
                    ExtismPluginLoader::new(self.state.location_info.clone()),
                )]);

                Arc::new(PluginLoaderRegistry::new(loaders))
            })
            .await
            .clone()
    }

    pub async fn get_plugin_storage(&self) -> Arc<FsPluginStorage> {
        self.plugin_storage
            .get_or_init(|| async {
                Arc::new(FsPluginStorage::new(self.state.location_info.clone()))
            })
            .await
            .clone()
    }

    pub async fn get_event_emitter(&self) -> Arc<TauriEventEmitter> {
        self.event_emitter
            .get_or_init(|| async { Arc::new(TauriEventEmitter::new(self.app_handle.clone())) })
            .await
            .clone()
    }

    pub async fn get_progress_bar_storage(&self) -> Arc<InMemoryProgressBarStorage> {
        self.progress_bar_storage
            .get_or_init(|| async { Arc::new(InMemoryProgressBarStorage::default()) })
            .await
            .clone()
    }

    pub async fn get_progress_service(&self) -> Arc<ProgressServiceType> {
        self.progress_service
            .get_or_init(|| async {
                Arc::new(ProgressServiceImpl::new(
                    self.get_event_emitter().await,
                    self.get_progress_bar_storage().await,
                ))
            })
            .await
            .clone()
    }

    pub async fn get_instance_watcher_service(
        &self,
    ) -> crate::Result<
        Arc<InstanceWatcherServiceImpl<NotifyFileWatcher<InstanceEventHandler<TauriEventEmitter>>>>,
    > {
        self.instance_watcher_service
            .get_or_try_init(|| async {
                let watcher = NotifyFileWatcher::new(Arc::new(InstanceEventHandler::new(
                    self.get_event_emitter().await,
                )))?;

                let service = InstanceWatcherServiceImpl::new(
                    Arc::new(watcher),
                    self.state.location_info.clone(),
                );

                Ok(Arc::new(service))
            })
            .await
            .cloned()
    }
}
