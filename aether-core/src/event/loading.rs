use std::path::PathBuf;

use tauri::Emitter;
use uuid::Uuid;

use super::{EventError, EventState};

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
pub struct LoadingBarId(Uuid);

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
                            fraction: Some(fraction),
                            message: "Completed".to_string(),
                            event,
                            loader_uuid,
                        };

                        app_handle
                            .emit("loading", payload)
                            .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)));
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
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoadingPayload {
    pub event: LoadingBarType,
    pub loader_uuid: Uuid,
    pub fraction: Option<f64>, // by convention, if optional, it means the loading is done
    pub message: String,
}

pub async fn init_loading_unsafe(
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<LoadingBarId> {
    let event_state = EventState::get()?;
    let key = LoadingBarId(Uuid::new_v4());

    event_state.loading_bars.insert(
        key.0,
        LoadingBar {
            loading_bar_uuid: key.0,
            message: title.to_string(),
            total,
            current: 0.0,
            last_sent: 0.0,
            bar_type,
        },
    );
    // attempt an initial loading_emit event to the frontend
    emit_loading(&key, 0.0, None).await?;

    Ok(key)
}

pub async fn init_loading(
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<LoadingBarId> {
    let key = init_loading_unsafe(bar_type, total, title).await?;
    Ok(key)
}

pub async fn init_or_edit_loading(
    id: Option<LoadingBarId>,
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<LoadingBarId> {
    if let Some(id) = id {
        edit_loading(&id, bar_type, total, title).await?;

        Ok(id)
    } else {
        init_loading(bar_type, total, title).await
    }
}

// Edits a loading bar's type
// This also resets the bar's current progress to 0
pub async fn edit_loading(
    id: &LoadingBarId,
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<()> {
    let event_state = EventState::get()?;

    if let Some(mut bar) = event_state.loading_bars.get_mut(&id.0) {
        bar.bar_type = bar_type;
        bar.total = total;
        bar.message = title.to_string();
        bar.current = 0.0;
        bar.last_sent = 0.0;
    };

    emit_loading(id, 0.0, None).await?;

    Ok(())
}

pub async fn emit_loading(
    key: &LoadingBarId,
    increment_frac: f64,
    message: Option<&str>,
) -> crate::Result<()> {
    let event_state = EventState::get()?;

    let mut loading_bar = match event_state.loading_bars.get_mut(&key.0) {
        Some(f) => f,
        None => {
            return Err(EventError::NoLoadingBar(key.0).into());
        }
    };

    // Tick up loading bar
    loading_bar.current += increment_frac;
    let display_frac = loading_bar.current / loading_bar.total;
    let opt_display_frac = if display_frac >= 1.0 {
        None // by convention, when its done, we submit None
             // any further updates will be ignored (also sending None)
    } else {
        Some(display_frac)
    };

    if f64::abs(display_frac - loading_bar.last_sent) > 0.005 {
        if let Some(app_handle) = &event_state.app {
            app_handle
                .emit(
                    "loading",
                    LoadingPayload {
                        fraction: opt_display_frac,
                        message: message.unwrap_or(&loading_bar.message).to_string(),
                        event: loading_bar.bar_type.clone(),
                        loader_uuid: loading_bar.loading_bar_uuid,
                    },
                )
                .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)))?;
        }

        loading_bar.last_sent = display_frac;
    }

    Ok(())
}
