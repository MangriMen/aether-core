use std::sync::Arc;

use tokio::sync::OnceCell;

use crate::{
    features::{auth::FsCredentialsStorage, settings::FsSettingsStorage},
    shared::infra::{ReqwestClient, REQWEST_CLIENT},
};

use super::{ErrorKind, LauncherState};

static LAZY_LOCATOR: OnceCell<Arc<LazyLocator>> = OnceCell::const_new();

pub struct LazyLocator {
    state: Arc<LauncherState>,
    request_client: OnceCell<Arc<ReqwestClient>>,
    api_client: OnceCell<Arc<ReqwestClient>>,
    auth_storage: OnceCell<Arc<FsCredentialsStorage>>,
    settings_storage: OnceCell<Arc<FsSettingsStorage>>,
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
}
