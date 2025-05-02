use std::sync::Arc;

use async_trait::async_trait;
use tokio::process::Command;
use uuid::Uuid;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::{
        events::{emit_process, ProcessPayloadType},
        process::{MinecraftProcess, MinecraftProcessMetadata, ProcessStorage},
    },
    shared::{domain::AsyncUseCaseWithInputAndError, IOError},
};

use super::{ManageProcessParams, ManageProcessUseCase, TrackProcessUseCase};

pub struct StartProcessUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
}

impl<PS> StartProcessUseCase<PS>
where
    PS: ProcessStorage + Send + Sync,
{
    pub fn new(process_storage: Arc<PS>) -> Self {
        Self { process_storage }
    }
}

#[async_trait]
impl<PS> AsyncUseCaseWithInputAndError for StartProcessUseCase<PS>
where
    PS: ProcessStorage + Send + Sync,
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

        emit_process(
            &instance_id,
            metadata.uuid,
            ProcessPayloadType::Launched,
            "Launched Minecraft",
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
        lazy_locator.get_process_storage().await,
        track_process_use_case,
        state.locations.clone(),
    );

    manage_process_use_case
        .execute(ManageProcessParams {
            process_uuid,
            instance_id,
            post_exit_command,
        })
        .await
}
