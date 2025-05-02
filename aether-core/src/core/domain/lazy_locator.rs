use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::OnceCell;

use crate::{
    features::{
        auth::FsCredentialsStorage,
        instance::{
            EventEmittingInstanceStorage, FsInstanceStorage, FsPackStorage,
            ModrinthContentProvider, ProviderRegistry,
        },
        java::infra::FsJavaStorage,
        minecraft::{CachedMetadataStorage, FsMetadataStorage, ModrinthMetadataStorage},
        process::InMemoryProcessManager,
        settings::FsSettingsStorage,
    },
    shared::infra::{ReqwestClient, REQWEST_CLIENT},
};

use super::{ErrorKind, LauncherState};

static LAZY_LOCATOR: OnceCell<Arc<LazyLocator>> = OnceCell::const_new();

const CACHE_TTL: Duration = Duration::from_secs(120);

pub struct LazyLocator {
    state: Arc<LauncherState>,
    request_client: OnceCell<Arc<ReqwestClient>>,
    api_client: OnceCell<Arc<ReqwestClient>>,
    auth_storage: OnceCell<Arc<FsCredentialsStorage>>,
    settings_storage: OnceCell<Arc<FsSettingsStorage>>,
    process_manager: OnceCell<Arc<InMemoryProcessManager>>,
    instance_storage: OnceCell<Arc<EventEmittingInstanceStorage<FsInstanceStorage>>>,
    java_storage: OnceCell<Arc<FsJavaStorage>>,
    metadata_storage: OnceCell<
        Arc<CachedMetadataStorage<FsMetadataStorage, ModrinthMetadataStorage<ReqwestClient>>>,
    >,
    pack_storage: OnceCell<Arc<FsPackStorage>>,
    provider_registry: OnceCell<Arc<ProviderRegistry<ModrinthContentProvider<ReqwestClient>>>>,
}

impl LazyLocator {
    pub async fn init(state: Arc<LauncherState>) -> crate::Result<()> {
        LAZY_LOCATOR
            .get_or_init(|| async {
                Arc::new(Self {
                    state,
                    request_client: OnceCell::new(),
                    api_client: OnceCell::new(),
                    auth_storage: OnceCell::new(),
                    settings_storage: OnceCell::new(),
                    process_manager: OnceCell::new(),
                    instance_storage: OnceCell::new(),
                    java_storage: OnceCell::new(),
                    metadata_storage: OnceCell::new(),
                    pack_storage: OnceCell::new(),
                    provider_registry: OnceCell::new(),
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

    pub async fn get_request_client(&self) -> Arc<ReqwestClient> {
        self.request_client
            .get_or_init(|| async {
                Arc::new(ReqwestClient::new(
                    (*REQWEST_CLIENT).clone(),
                    self.state.fetch_semaphore.clone(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_api_client(&self) -> Arc<ReqwestClient> {
        self.api_client
            .get_or_init(|| async {
                Arc::new(ReqwestClient::new(
                    (*REQWEST_CLIENT).clone(),
                    self.state.api_semaphore.clone(),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_auth_storage(&self) -> Arc<FsCredentialsStorage> {
        self.auth_storage
            .get_or_init(|| async {
                Arc::new(FsCredentialsStorage::new(
                    &self.state.locations.settings_dir,
                ))
            })
            .await
            .clone()
    }

    pub async fn get_settings_storage(&self) -> Arc<FsSettingsStorage> {
        self.settings_storage
            .get_or_init(|| async {
                Arc::new(FsSettingsStorage::new(&self.state.locations.settings_dir))
            })
            .await
            .clone()
    }

    pub async fn get_process_manager(&self) -> Arc<InMemoryProcessManager> {
        self.process_manager
            .get_or_init(|| async { Arc::new(InMemoryProcessManager::default()) })
            .await
            .clone()
    }

    pub async fn get_instance_storage(
        &self,
    ) -> Arc<EventEmittingInstanceStorage<FsInstanceStorage>> {
        self.instance_storage
            .get_or_init(|| async {
                Arc::new(EventEmittingInstanceStorage::new(FsInstanceStorage::new(
                    self.state.locations.clone(),
                )))
            })
            .await
            .clone()
    }

    pub async fn get_java_storage(&self) -> Arc<FsJavaStorage> {
        self.java_storage
            .get_or_init(|| async {
                Arc::new(FsJavaStorage::new(&self.state.locations.java_dir()))
            })
            .await
            .clone()
    }

    pub async fn get_metadata_storage(
        &self,
    ) -> Arc<CachedMetadataStorage<FsMetadataStorage, ModrinthMetadataStorage<ReqwestClient>>> {
        self.metadata_storage
            .get_or_init(|| async {
                Arc::new(CachedMetadataStorage::new(
                    FsMetadataStorage::new(&self.state.locations.cache_dir(), Some(CACHE_TTL)),
                    ModrinthMetadataStorage::new(self.get_request_client().await),
                ))
            })
            .await
            .clone()
    }

    pub async fn get_pack_storage(&self) -> Arc<FsPackStorage> {
        self.pack_storage
            .get_or_init(|| async { Arc::new(FsPackStorage::new(self.state.locations.clone())) })
            .await
            .clone()
    }

    pub async fn get_provider_registry(
        &self,
    ) -> Arc<ProviderRegistry<ModrinthContentProvider<ReqwestClient>>> {
        self.provider_registry
            .get_or_init(|| async {
                let providers = HashMap::from([(
                    "modrinth".to_string(),
                    ModrinthContentProvider::new(
                        self.state.locations.clone(),
                        None,
                        self.get_request_client().await,
                    ),
                )]);

                Arc::new(ProviderRegistry::new(providers))
            })
            .await
            .clone()
    }
}
