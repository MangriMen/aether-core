use std::{process::ExitStatus, sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    features::{instance::InstanceManager, process::ProcessManager},
    shared::domain::AsyncUseCaseWithInput,
};

const PROCESS_CHECK_INTERVAL: Duration = Duration::from_millis(50);

pub struct TrackProcessParams {
    pub process_uuid: Uuid,
    pub instance_id: String,
}

pub struct TrackProcessUseCase<PM: ProcessManager, IM: InstanceManager> {
    process_manager: Arc<PM>,
    instance_manager: Arc<IM>,
}

impl<PM: ProcessManager, IM: InstanceManager> TrackProcessUseCase<PM, IM> {
    pub fn new(process_manager: Arc<PM>, instance_manager: Arc<IM>) -> Self {
        Self {
            process_manager,
            instance_manager,
        }
    }

    async fn update_playtime(&self, last_updated: &mut DateTime<Utc>, id: &str, force: bool) {
        let elapsed_seconds = Utc::now()
            .signed_duration_since(*last_updated)
            .num_seconds();

        if elapsed_seconds >= 60 || force {
            let result = async {
                self.instance_manager
                    .upsert_with(id, |instance| {
                        instance.time_played += elapsed_seconds as u64;
                        Ok(())
                    })
                    .await
            }
            .await;

            if let Err(e) = result {
                tracing::warn!("Failed to update playtime for profile {}: {}", id, e);
            }

            *last_updated = Utc::now();
        }
    }
}

#[async_trait]
impl<PM, IM> AsyncUseCaseWithInput for TrackProcessUseCase<PM, IM>
where
    PM: ProcessManager + Send + Sync,
    IM: InstanceManager + Send + Sync,
{
    type Input = TrackProcessParams;
    type Output = ExitStatus;

    async fn execute(&self, params: Self::Input) -> Self::Output {
        let TrackProcessParams {
            process_uuid,
            instance_id,
        } = params;

        let mut last_updated_playtime = Utc::now();

        loop {
            match self.process_manager.try_wait(process_uuid) {
                Ok(Some(Some(exit_status))) => {
                    // Process exited successfully
                    self.update_playtime(&mut last_updated_playtime, &instance_id, true)
                        .await;
                    return exit_status;
                }
                Ok(Some(None)) => {} // Still running
                Ok(None) | Err(_) => {
                    self.update_playtime(&mut last_updated_playtime, &instance_id, true)
                        .await;
                    return ExitStatus::default();
                }
            }

            tokio::time::sleep(PROCESS_CHECK_INTERVAL).await;
            self.update_playtime(&mut last_updated_playtime, &instance_id, false)
                .await;
        }
    }
}
