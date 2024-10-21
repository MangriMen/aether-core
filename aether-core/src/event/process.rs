use tauri::Emitter;
use uuid::Uuid;

use super::{EventError, EventState};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProcessPayload {
    pub profile_path_id: String,
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
    profile_path: &str,
    uuid: Uuid,
    event: ProcessPayloadType,
    message: &str,
) -> anyhow::Result<()> {
    let event_state = EventState::get()?;

    if let Some(app_handle) = &event_state.app {
        app_handle
            .emit(
                "process",
                ProcessPayload {
                    profile_path_id: profile_path.to_string(),
                    uuid,
                    event,
                    message: message.to_string(),
                },
            )
            .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)))?;
    }

    Ok(())
}
