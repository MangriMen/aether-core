use std::path::PathBuf;

use tauri::Emitter;
use uuid::Uuid;

use super::{EventError, EventState, LauncherEvent};

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadingBar {
    // loading_bar_uuid not be used directly by external functions as it may not reflect the current state of the loading bar/hashmap
    pub loading_bar_uuid: Uuid,
    pub message: String,
    pub total: f64,
    pub current: f64,
    #[serde(skip)]
    pub last_sent: f64,
    pub bar_type: LoadingBarType,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct LoadingBarId(pub Uuid);

// When Loading bar id is dropped, we should remove it from the hashmap
impl Drop for LoadingBarId {
    fn drop(&mut self) {
        let loader_uuid = self.0;

        tokio::spawn(async move {
            if let Ok(event_state) = EventState::get() {
                if let Some(app_handle) = &event_state.app {
                    if let Some((_, bar)) = event_state.loading_bars.remove(&loader_uuid) {
                        let loader_uuid = bar.loading_bar_uuid;
                        let event = bar.bar_type.clone();
                        let fraction = bar.current / bar.total;

                        let payload = LoadingPayload {
                            fraction: None,
                            message: "Completed".to_string(),
                            event,
                            loader_uuid,
                        };

                        let res = app_handle
                            .emit(LauncherEvent::Loading.as_str(), payload)
                            .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)));

                        if res.is_err() {
                            log::error!("Exited at {fraction} for loading bar: {:?}", loader_uuid);
                        }
                    }
                } else {
                    event_state.loading_bars.remove(&loader_uuid);
                }
            }
        });
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum LoadingBarType {
    LegacyDataMigration,
    DirectoryMove {
        old: PathBuf,
        new: PathBuf,
    },
    JavaDownload {
        version: u32,
    },
    PackFileDownload {
        instance_path: String,
        pack_name: String,
        icon: Option<String>,
        pack_version: String,
    },
    PackDownload {
        instance_path: String,
        pack_name: String,
        icon: Option<PathBuf>,
        pack_id: Option<String>,
        pack_version: Option<String>,
    },
    MinecraftDownload {
        instance_path: String,
        instance_name: String,
    },
    ProfileUpdate {
        instance_path: String,
        instance_name: String,
    },
    ZipExtract {
        instance_path: String,
        instance_name: String,
    },
    ConfigChange {
        new_path: PathBuf,
    },
    CopyProfile {
        import_location: PathBuf,
        instance_name: String,
    },
    CheckingForUpdates,
    LauncherUpdate {
        version: String,
        current_version: String,
    },
    PluginDownload {
        plugin_name: String,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoadingPayload {
    pub event: LoadingBarType,
    pub loader_uuid: Uuid,
    pub fraction: Option<f64>, // by convention, if optional, it means the loading is done
    pub message: String,
}
