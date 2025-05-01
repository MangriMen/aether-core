use tauri::Emitter;

use crate::features::events::{EventError, EventState, LauncherEvent, WarningPayload};

pub async fn emit_warning(message: &str) -> crate::Result<()> {
    let event_state = EventState::get()?;
    if let Some(app_handle) = &event_state.app {
        app_handle
            .emit(
                LauncherEvent::Warning.as_str(),
                WarningPayload {
                    message: message.to_string(),
                },
            )
            .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)))?;
    }
    tracing::warn!("{}", message);
    Ok(())
}
