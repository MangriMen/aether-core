use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    features::{
        events::{emit_process, ProcessPayloadType},
        instance::{FsInstanceStorage, EventEmittingInstanceStorage},
        process::ProcessManager,
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

pub struct ManageProcessUseCase<PM: ProcessManager> {
    process_manager: Arc<PM>,
    track_process_use_case: Arc<TrackProcessUseCase<PM, EventEmittingInstanceStorage<FsInstanceStorage>>>,
    location_info: Arc<LocationInfo>,
}

impl<PM: ProcessManager> ManageProcessUseCase<PM> {
    pub fn new(
        process_manager: Arc<PM>,
        track_process_use_case: Arc<
            TrackProcessUseCase<PM, EventEmittingInstanceStorage<FsInstanceStorage>>,
        >,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            process_manager,
            track_process_use_case,
            location_info,
        }
    }
}

#[async_trait]
impl<PM> AsyncUseCaseWithInputAndError for ManageProcessUseCase<PM>
where
    PM: ProcessManager + Send + Sync,
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

        self.process_manager.remove(process_uuid);
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
