use uuid::Uuid;

use super::{
    progress_bar_error::ProgressBarStorageError, EventEmitter, EventError, InstanceEvent,
    InstanceEventType, LauncherEvent, ProcessEvent, ProcessEventType, ProgressBar, ProgressBarId,
    ProgressBarStorage, WarningEvent,
};

pub trait EventEmitterExt: EventEmitter {
    fn emit_instance(
        &self,
        instance_id: String,
        event: InstanceEventType,
    ) -> Result<(), EventError>;

    fn emit_process(
        &self,
        instance_id: String,
        process_id: Uuid,
        message: String,
        event: ProcessEventType,
    ) -> Result<(), EventError>;

    fn emit_warning(&self, message: String) -> Result<(), EventError>;
}

impl<E> EventEmitterExt for E
where
    E: EventEmitter + Send + Sync,
{
    fn emit_instance(
        &self,
        instance_id: String,
        event: InstanceEventType,
    ) -> Result<(), EventError> {
        self.emit(
            LauncherEvent::Instance.as_str(),
            InstanceEvent { instance_id, event },
        )
    }

    fn emit_process(
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
    }

    fn emit_warning(&self, message: String) -> Result<(), EventError> {
        self.emit(LauncherEvent::Warning.as_str(), WarningEvent { message })
    }
}

pub trait ProgressBarStorageExt: ProgressBarStorage {
    fn upsert_with<F>(
        &self,
        progress_bar_id: &ProgressBarId,
        update_fn: F,
    ) -> Result<(), ProgressBarStorageError>
    where
        F: FnOnce(&mut ProgressBar) -> Result<(), ProgressBarStorageError> + Send;
}

impl<PS> ProgressBarStorageExt for PS
where
    PS: ProgressBarStorage,
{
    fn upsert_with<F>(
        &self,
        progress_bar_id: &ProgressBarId,
        update_fn: F,
    ) -> Result<(), ProgressBarStorageError>
    where
        F: FnOnce(&mut ProgressBar) -> Result<(), ProgressBarStorageError> + Send,
    {
        let mut progress_bar = self.get(progress_bar_id.0)?.clone();
        update_fn(&mut progress_bar)?;
        self.upsert(progress_bar_id.0, progress_bar)?;
        Ok(())
    }
}
