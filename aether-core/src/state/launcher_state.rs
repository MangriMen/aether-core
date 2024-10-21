use std::{path::PathBuf, sync::Arc};

use tokio::sync::{OnceCell, Semaphore};

use crate::utils::fetch::FetchSemaphore;

use super::{LocationInfo, ProcessManager, Settings};

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

    // /// Semaphore to limit concurrent API requests. This is separate from the fetch semaphore
    // /// to keep API functionality while the app is performing intensive tasks.
    // pub api_semaphore: FetchSemaphore,
    /// Process manager
    pub process_manager: ProcessManager,
    // pub(crate) pool: SqlitePool,

    // pub(crate) file_watcher: FileWatcher,
}

impl LauncherState {
    // TODO: Use file for settings
    pub async fn init(settings: &Settings) -> anyhow::Result<()> {
        LAUNCHER_STATE
            .get_or_try_init(|| Self::initialize_state(settings))
            .await?;

        Ok(())
    }

    pub async fn get() -> anyhow::Result<Arc<Self>> {
        if !LAUNCHER_STATE.initialized() {
            while !LAUNCHER_STATE.initialized() {}
        }

        Ok(Arc::clone(
            LAUNCHER_STATE.get().expect("State is not initialized"),
        ))
    }

    #[tracing::instrument]
    async fn initialize_state(settings: &Settings) -> anyhow::Result<Arc<Self>> {
        let locations = LocationInfo {
            settings_dir: PathBuf::from(settings.launcher_dir.clone().unwrap()),
            config_dir: PathBuf::from(settings.metadata_dir.clone().unwrap()),
        };

        let fetch_semaphore = FetchSemaphore(Semaphore::new(settings.max_concurrent_downloads));

        Ok(Arc::new(Self {
            locations,
            process_manager: ProcessManager::new(),
            fetch_semaphore,
        }))
    }
}
