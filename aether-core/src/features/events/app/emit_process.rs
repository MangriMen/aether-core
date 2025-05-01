use tauri::Emitter;
use uuid::Uuid;

use crate::features::events::{
    EventError, EventState, LauncherEvent, ProcessPayload, ProcessPayloadType,
};

pub async fn emit_process(
    id: &str,
    uuid: Uuid,
    event: ProcessPayloadType,
    message: &str,
) -> crate::Result<()> {
    let event_state = EventState::get()?;

    if let Some(app_handle) = &event_state.app {
        app_handle
            .emit(
                LauncherEvent::Process.as_str(),
                ProcessPayload {
                    instance_id: id.to_string(),
                    uuid,
                    event,
                    message: message.to_string(),
                },
            )
            .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)))?;
    }

    Ok(())
}
