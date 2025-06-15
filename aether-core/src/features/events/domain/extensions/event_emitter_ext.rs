use async_trait::async_trait;
use log::debug;
use uuid::Uuid;

use crate::features::events::{
    EventEmitter, EventError, InstanceEvent, InstanceEventType, LauncherEvent, ProcessEvent,
    ProcessEventType, WarningEvent,
};

#[async_trait]
pub trait EventEmitterExt: EventEmitter {
    async fn emit_instance(
        &self,
        instance_id: String,
        event: InstanceEventType,
    ) -> Result<(), EventError> {
        self.emit(
            LauncherEvent::Instance.as_str(),
            InstanceEvent { instance_id, event },
        )
        .await
    }

    async fn emit_process(
        &self,
        instance_id: String,
        process_id: Uuid,
        message: String,
        event: ProcessEventType,
    ) -> Result<(), EventError> {
        self.emit(
            LauncherEvent::Process.as_str(),
            ProcessEvent {
                instance_id,
                process_id,
                event,
                message,
            },
        )
        .await
    }

    async fn emit_warning(&self, message: String) -> Result<(), EventError> {
        self.emit(LauncherEvent::Warning.as_str(), WarningEvent { message })
            .await
    }

    async fn emit_process_safe(
        &self,
        instance_id: String,
        process_id: Uuid,
        message: String,
        event: ProcessEventType,
    ) {
        if let Err(e) = self
            .emit_process(instance_id, process_id, message, event)
            .await
        {
            debug!("Failed to emit process: {e}")
        }
    }
}

#[async_trait]
impl<E: EventEmitter> EventEmitterExt for E {}
