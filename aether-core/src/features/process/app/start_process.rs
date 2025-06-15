use std::sync::Arc;

use tokio::process::Command;

use crate::{
    features::{
        events::{EventEmitter, EventEmitterExt, ProcessEventType},
        process::{MinecraftProcess, MinecraftProcessMetadata, ProcessError, ProcessStorage},
    },
    shared::IoError,
};

use super::{ManageProcessParams, ManageProcessUseCase};

pub struct StartProcessUseCase<E: EventEmitter, PS: ProcessStorage> {
    event_emitter: Arc<E>,
    process_storage: Arc<PS>,
    manage_process_use_case: Arc<ManageProcessUseCase<E, PS>>,
}

impl<E: EventEmitter + 'static, PS: ProcessStorage + 'static> StartProcessUseCase<E, PS> {
    pub fn new(
        event_emitter: Arc<E>,
        process_storage: Arc<PS>,
        manage_process_use_case: Arc<ManageProcessUseCase<E, PS>>,
    ) -> Self {
        Self {
            event_emitter,
            process_storage,
            manage_process_use_case,
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
        mut command: Command,
        post_exit_command: Option<String>,
    ) -> Result<MinecraftProcessMetadata, ProcessError> {
        let minecraft_process = command.spawn().map_err(IoError::from)?;
        let process = MinecraftProcess::from_child(instance_id.clone(), minecraft_process);

        let metadata = process.metadata.clone();

        let manage_process_use_case = self.manage_process_use_case.clone();
        let instance_id_clone = instance_id.clone();

        tokio::spawn(async move {
            let _ = manage_process_use_case
                .execute(ManageProcessParams {
                    process_uuid: metadata.uuid,
                    instance_id: instance_id_clone,
                    post_exit_command,
                })
                .await;
        });

        self.process_storage.insert(process).await;

        self.event_emitter
            .emit_process_safe(
                instance_id.clone(),
                metadata.uuid,
                "Launched Minecraft".to_string(),
                ProcessEventType::Launched,
            )
            .await;

        Ok(metadata)
    }
}
