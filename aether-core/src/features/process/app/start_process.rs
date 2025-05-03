use std::sync::Arc;

use async_trait::async_trait;
use tokio::process::Command;
use uuid::Uuid;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::{
        events::{EventEmitter, EventEmitterExt, ProcessEventType},
        process::{MinecraftProcess, MinecraftProcessMetadata, ProcessStorage},
    },
    shared::{domain::AsyncUseCaseWithInputAndError, IOError},
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
}

#[async_trait]
impl<E: EventEmitter, PS: ProcessStorage> AsyncUseCaseWithInputAndError
    for StartProcessUseCase<E, PS>
{
    type Input = (String, Command, Option<String>);
    type Output = MinecraftProcessMetadata;
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let (instance_id, mut command, post_exit_command) = input;

        let minecraft_process = command.spawn().map_err(IOError::from)?;
        let process = MinecraftProcess::from_child(instance_id.clone(), minecraft_process);

        let metadata = process.metadata.clone();

        tokio::spawn(manage_process(
            metadata.uuid,
            instance_id.clone(),
            post_exit_command,
        ));

        self.process_storage.insert(process).await;

        self.event_emitter.emit_process(
            instance_id.clone(),
            metadata.uuid.clone(),
            "Launched Minecraft".to_string(),
            ProcessEventType::Launched,
        )?;

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
