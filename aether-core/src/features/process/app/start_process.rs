use std::sync::Arc;

use tokio::process::Command;
use uuid::Uuid;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::{
        events::{EventEmitter, EventEmitterExt, ProcessEventType},
        process::{MinecraftProcess, MinecraftProcessMetadata, ProcessStorage},
    },
    shared::IoError,
};

use super::{ManageProcessParams, ManageProcessUseCase, TrackProcessUseCase};

pub struct StartProcessUseCase<E: EventEmitter, PS: ProcessStorage> {
    event_emitter: Arc<E>,
    process_storage: Arc<PS>,
}

impl<E: EventEmitter, PS: ProcessStorage> StartProcessUseCase<E, PS> {
    pub fn new(event_emitter: Arc<E>, process_storage: Arc<PS>) -> Self {
        Self {
            event_emitter,
            process_storage,
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
        mut command: Command,
        post_exit_command: Option<String>,
    ) -> crate::Result<MinecraftProcessMetadata> {
        let minecraft_process = command.spawn().map_err(IoError::from)?;
        let process = MinecraftProcess::from_child(instance_id.clone(), minecraft_process);

        let metadata = process.metadata.clone();

        tokio::spawn(manage_process(
            metadata.uuid,
            instance_id.clone(),
            post_exit_command,
        ));

        self.process_storage.insert(process).await;

        self.event_emitter
            .emit_process(
                instance_id.clone(),
                metadata.uuid,
                "Launched Minecraft".to_string(),
                ProcessEventType::Launched,
            )
            .await?;

        Ok(metadata)
    }
}

async fn manage_process(
    process_uuid: Uuid,
    instance_id: String,
    post_exit_command: Option<String>,
) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let track_process_use_case = Arc::new(TrackProcessUseCase::new(
        lazy_locator.get_process_storage().await,
        lazy_locator.get_instance_storage().await,
    ));

    let manage_process_use_case = ManageProcessUseCase::new(
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_process_storage().await,
        track_process_use_case,
        state.location_info.clone(),
    );

    manage_process_use_case
        .execute(ManageProcessParams {
            process_uuid,
            instance_id,
            post_exit_command,
        })
        .await
}
