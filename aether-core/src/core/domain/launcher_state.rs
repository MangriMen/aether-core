use std::{collections::HashSet, path::PathBuf, sync::Arc};

use tokio::sync::{OnceCell, Semaphore};

use crate::{
    core::domain::LazyLocator,
    features::{
        instance::InstanceWatcherService,
        settings::{
            FsSettingsStorage, Hooks, LocationInfo, MemorySettings, Settings, SettingsStorage,
            WindowSize,
        },
    },
    shared::domain::FetchSemaphore,
};

// Global state
// RwLock on state only has concurrent reads, except for config dir change which takes control of the State
static LAUNCHER_STATE: OnceCell<Arc<LauncherState>> = OnceCell::const_new();

#[derive(Debug)]
pub struct LauncherState {
    // Information about files location
    pub location_info: Arc<LocationInfo>,

    /// Semaphore used to limit concurrent network requests and avoid errors
    pub fetch_semaphore: Arc<FetchSemaphore>,

    // /// Semaphore used to limit concurrent I/O and avoid errors
    // pub io_semaphore: IoSemaphore,

    // ///
    /// Semaphore to limit concurrent API requests. This is separate from the fetch semaphore
    /// to keep API functionality while the app is performing intensive tasks.
    pub api_semaphore: Arc<FetchSemaphore>,
}

impl LauncherState {
    pub async fn init(
        launcher_dir: PathBuf,
        metadata_dir: PathBuf,
        app_handle: tauri::AppHandle,
    ) -> crate::Result<()> {
        LAUNCHER_STATE
            .get_or_try_init(|| Self::initialize(launcher_dir, metadata_dir, app_handle))
            .await?;

        Ok(())
    }

    #[tracing::instrument]
    pub async fn get() -> crate::Result<Arc<Self>> {
        if !LAUNCHER_STATE.initialized() {
            tracing::error!(
                "Attempted to get state before it is initialized - this should never happen!\n{:?}",
                std::backtrace::Backtrace::capture()
            );

            while !LAUNCHER_STATE.initialized() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }

        Ok(Arc::clone(
            LAUNCHER_STATE.get().expect("State is not initialized!"),
        ))
    }

    pub async fn initialized() -> bool {
        LAUNCHER_STATE.initialized()
    }

    #[tracing::instrument]
    async fn initialize(
        launcher_dir: PathBuf,
        metadata_dir: PathBuf,
        app_handle: tauri::AppHandle,
    ) -> crate::Result<Arc<Self>> {
        log::info!("Initializing state");

        let settings_storage = FsSettingsStorage::new(&launcher_dir.clone());

        let settings = if let Ok(settings) = settings_storage.get().await {
            settings
        } else {
            let settings = Settings {
                launcher_dir,
                metadata_dir,
                max_concurrent_downloads: 10,

                memory: MemorySettings { maximum: 2048 },
                game_resolution: WindowSize(960, 540),
                custom_env_vars: vec![],
                extra_launch_args: vec![],
                hooks: Hooks::default(),

                enabled_plugins: HashSet::default(),
            };

            settings_storage.upsert(&settings).await?;

            settings
        };

        log::info!("Initialize locations");
        let location_info = Arc::new(LocationInfo {
            settings_dir: settings.launcher_dir.clone(),
            config_dir: settings.metadata_dir.clone(),
        });

        log::info!("Initialize fetch semaphore");
        let fetch_semaphore = Arc::new(FetchSemaphore(Semaphore::new(
            settings.max_concurrent_downloads,
        )));

        log::info!("Initialize api semaphore");
        let api_semaphore = Arc::new(FetchSemaphore(Semaphore::new(
            settings.max_concurrent_downloads,
        )));

        log::info!("Initialize file watcher");
        // let file_watcher = Arc::new(fs_watcher::init_watcher().await?);
        // fs_watcher::watch_instances(&file_watcher, &location_info).await;

        log::info!("State initialized");

        log::info!("Initializing service locator");

        let state = Arc::new(Self {
            location_info,
            fetch_semaphore,
            api_semaphore,
        });

        LazyLocator::init(state.clone(), app_handle).await?;

        let lazy_locator = LazyLocator::get().await?;
        lazy_locator
            .get_instance_watcher_service()
            .await?
            .watch_instances()
            .await?;

        Ok(state)
    }
}
