use std::{path::PathBuf, sync::Arc};

use tokio::sync::{OnceCell, RwLock, Semaphore};

use crate::{
    features::settings::Settings,
    state::{fs_watcher, FsWatcher, LocationInfo, PluginManager, ProcessManager},
    utils::fetch::FetchSemaphore,
};

// Global state
// RwLock on state only has concurrent reads, except for config dir change which takes control of the State
static LAUNCHER_STATE: OnceCell<Arc<LauncherState>> = OnceCell::const_new();

#[derive(Debug)]
pub struct LauncherState {
    // Information about files location
    pub locations: LocationInfo,
    /// Semaphore used to limit concurrent network requests and avoid errors
    pub fetch_semaphore: FetchSemaphore,

    // /// Semaphore used to limit concurrent I/O and avoid errors
    // pub io_semaphore: IoSemaphore,
    /// Semaphore to limit concurrent API requests. This is separate from the fetch semaphore
    /// to keep API functionality while the app is performing intensive tasks.
    pub api_semaphore: FetchSemaphore,
    /// Process manager
    pub process_manager: ProcessManager,
    pub plugin_manager: RwLock<PluginManager>,
    pub(crate) file_watcher: FsWatcher,
    // pub(crate) pool: SqlitePool,
}

impl LauncherState {
    pub async fn init(settings: &Settings) -> crate::Result<()> {
        LAUNCHER_STATE
            .get_or_try_init(|| Self::initialize(settings))
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
    async fn initialize(settings: &Settings) -> crate::Result<Arc<Self>> {
        log::info!("Initializing state");

        log::info!("Initialize locations");
        let locations = LocationInfo {
            settings_dir: PathBuf::from(settings.launcher_dir.clone().unwrap()),
            config_dir: PathBuf::from(settings.metadata_dir.clone().unwrap()),
        };

        log::info!("Initialize fetch semaphore");
        let fetch_semaphore = FetchSemaphore(Semaphore::new(settings.max_concurrent_downloads));

        log::info!("Initialize api semaphore");
        let api_semaphore = FetchSemaphore(Semaphore::new(settings.max_concurrent_downloads));

        log::info!("Initialize process manager");
        let process_manager = ProcessManager::default();

        log::info!("Initialize file watcher");
        let file_watcher = fs_watcher::init_watcher().await?;
        fs_watcher::watch_instances(&file_watcher, &locations).await;

        log::info!("Initialize plugin manager");
        let plugin_manager = RwLock::new(PluginManager::default());

        log::info!("State initialized");

        Ok(Arc::new(Self {
            locations,
            fetch_semaphore,
            api_semaphore,
            process_manager,
            plugin_manager,
            file_watcher,
        }))
    }
}
