use tauri::Emitter;

use crate::features::events::{
    EventError, EventState, InstancePayload, InstancePayloadType, LauncherEvent,
};

pub async fn emit_instance(instance_id: &str, event: InstancePayloadType) -> crate::Result<()> {
    let event_state = EventState::get()?;

    if let Some(app_handle) = &event_state.app {
        app_handle
            .emit(
                LauncherEvent::Instance.as_str(),
                InstancePayload {
                    instance_path_id: instance_id.to_string(),
                    event,
                },
            )
            .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)))?;
    }
    Ok(())
}
