use log::error;
use serde::Serialize;
use tauri::Emitter;
use uuid::Uuid;

use crate::features::events::EventState;

use super::{LauncherEvent, ProgressEvent, ProgressEventType};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressBar {
    // id not be used directly by external functions as it may not reflect the current state
    pub id: Uuid,
    pub message: String,
    pub total: f64,
    pub current: f64,
    #[serde(skip)]
    pub last_sent: f64,
    pub progress_type: ProgressEventType,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressBarId(pub Uuid);

// When Loading bar id is dropped, we should remove it from the state
impl Drop for ProgressBarId {
    fn drop(&mut self) {
        let progress_bar_id = self.0;
        tokio::spawn(async move {
            match EventState::get() {
                Ok(event_state) => {
                    if let Some(app_handle) = &event_state.app {
                        let removed_progress_bar =
                            event_state.progress_bars.remove(&progress_bar_id);

                        if let Some((_, progress_bar)) = removed_progress_bar {
                            let completion_event = ProgressEvent {
                                fraction: None,
                                message: "Completed".to_string(),
                                event: progress_bar.progress_type,
                                progress_bar_id: progress_bar.id,
                            };

                            if let Err(e) =
                                app_handle.emit(LauncherEvent::Loading.as_str(), completion_event)
                            {
                                error!(
                                    "Exited at {:.2}% for progress bar: {}: {:?}",
                                    (progress_bar.current / progress_bar.total) * 100.0,
                                    progress_bar.id,
                                    e
                                );
                            }
                        }
                    }
                }
                Err(e) => error!("Failed to get EventState: {:?}", e),
            }
        });
    }
}
