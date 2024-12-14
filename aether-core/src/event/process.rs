use tauri::Emitter;
use uuid::Uuid;

use super::{EventError, EventState};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProcessPayload {
    pub instance_name_id: String,
    pub uuid: Uuid,
    pub event: ProcessPayloadType,
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ProcessPayloadType {
    Launched,
    Finished,
}

// emit_process(uuid, pid, event, message)
#[allow(unused_variables)]
pub async fn emit_process(
    name_id: &str,
    uuid: Uuid,
    event: ProcessPayloadType,
    message: &str,
) -> crate::Result<()> {
    let event_state = EventState::get()?;

    if let Some(app_handle) = &event_state.app {
        app_handle
            .emit(
                "process",
                ProcessPayload {
                    instance_name_id: name_id.to_string(),
                    uuid,
                    event,
                    message: message.to_string(),
                },
            )
            .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)))?;
    }

    Ok(())
}
