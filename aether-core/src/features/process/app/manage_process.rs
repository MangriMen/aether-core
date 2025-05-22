use std::sync::Arc;

use uuid::Uuid;

use crate::{
    features::{
        events::{EventEmitter, EventEmitterExt, ProcessEventType},
        instance::{EventEmittingInstanceStorage, FsInstanceStorage},
        process::ProcessStorage,
        settings::LocationInfo,
    },
    shared::{IOError, SerializableCommand},
};

use super::{TrackProcessParams, TrackProcessUseCase};

pub struct ManageProcessParams {
    pub process_uuid: Uuid,
    pub instance_id: String,
    pub post_exit_command: Option<String>,
}

pub struct ManageProcessUseCase<E: EventEmitter, PS: ProcessStorage> {
    event_emitter: Arc<E>,
    process_storage: Arc<PS>,
    track_process_use_case:
        Arc<TrackProcessUseCase<PS, EventEmittingInstanceStorage<E, FsInstanceStorage>>>,
    location_info: Arc<LocationInfo>,
}

impl<E: EventEmitter, PS: ProcessStorage> ManageProcessUseCase<E, PS> {
    pub fn new(
        event_emitter: Arc<E>,
        process_storage: Arc<PS>,
        track_process_use_case: Arc<
            TrackProcessUseCase<PS, EventEmittingInstanceStorage<E, FsInstanceStorage>>,
        >,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            event_emitter,
            process_storage,
            track_process_use_case,
            location_info,
        }
    }
    pub async fn execute(&self, params: ManageProcessParams) -> crate::Result<()> {
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

        self.event_emitter
            .emit_process(
                instance_id.clone(),
                process_uuid,
                "Exited process".to_string(),
                ProcessEventType::Finished,
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
