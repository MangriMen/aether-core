use async_trait::async_trait;
use serde::Serialize;
use tauri::Emitter;

use crate::features::events::{EventEmitter, EventError};

pub struct TauriEventEmitter {
    app_handle: tauri::AppHandle,
}

impl TauriEventEmitter {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl EventEmitter for TauriEventEmitter {
    async fn emit<P: Serialize + Clone + Send>(
        &self,
        event: &str,
        payload: P,
    ) -> Result<(), EventError> {
        self.app_handle
            .emit(event, payload)
            .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)))
    }
}
