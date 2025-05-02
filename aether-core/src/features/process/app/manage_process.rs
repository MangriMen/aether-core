use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    features::{
        events::{emit_process, ProcessPayloadType},
        instance::{EventEmittingInstanceStorage, FsInstanceStorage},
        process::ProcessStorage,
        settings::LocationInfo,
    },
    shared::{
        domain::{AsyncUseCaseWithInput, AsyncUseCaseWithInputAndError, SerializableCommand},
        IOError,
    },
};

use super::{TrackProcessParams, TrackProcessUseCase};

pub struct ManageProcessParams {
    pub process_uuid: Uuid,
    pub instance_id: String,
    pub post_exit_command: Option<String>,
}

pub struct ManageProcessUseCase<PS: ProcessStorage> {
    process_storage: Arc<PS>,
    track_process_use_case:
        Arc<TrackProcessUseCase<PS, EventEmittingInstanceStorage<FsInstanceStorage>>>,
    location_info: Arc<LocationInfo>,
}

impl<PS: ProcessStorage> ManageProcessUseCase<PS> {
    pub fn new(
        process_storage: Arc<PS>,
        track_process_use_case: Arc<
            TrackProcessUseCase<PS, EventEmittingInstanceStorage<FsInstanceStorage>>,
        >,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            process_storage,
            track_process_use_case,
            location_info,
        }
    }
}

#[async_trait]
impl<PS> AsyncUseCaseWithInputAndError for ManageProcessUseCase<PS>
where
    PS: ProcessStorage + Send + Sync,
{
    type Input = ManageProcessParams;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, params: Self::Input) -> Result<Self::Output, Self::Error> {
        let ManageProcessParams {
            process_uuid,
            instance_id,
            post_exit_command,
        } = params;

        let mc_exit_status = self
            .track_process_use_case
            .execute(TrackProcessParams {
                process_uuid,
                instance_id: instance_id.clone(),
            })
            .await;

        self.process_storage.remove(process_uuid).await;

        emit_process(
            &instance_id,
            process_uuid,
            ProcessPayloadType::Finished,
            "Exited process",
        )
        .await?;

        if mc_exit_status.success() {
            if let Some(command) = post_exit_command {
                let instance_dir = self.location_info.instance_dir(&instance_id);
                if let Ok(cmd) = SerializableCommand::from_string(&command, Some(&instance_dir)) {
                    cmd.to_tokio_command()
                        .spawn()
                        .map_err(|e| IOError::with_path(e, instance_dir))?;
                }
            }
        }

        Ok(())
    }
}
