use std::sync::Arc;

use crate::{
    features::{
        auth::CredentialsStorage,
        events::{EventEmitter, ProgressService},
        instance::{InstanceError, InstanceStorage, LaunchInstanceUseCase},
        java::{JavaInstallationService, JavaStorage},
        minecraft::{MinecraftDownloader, ReadMetadataStorage},
        process::{MinecraftProcessMetadata, ProcessStorage},
        settings::GlobalInstanceSettingsStorage,
    },
    libs::request_client::RequestClient,
};

pub struct LaunchInstanceWithActiveAccountUseCase<
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
    PS: ProcessStorage,
    CS: CredentialsStorage,
    GISS: GlobalInstanceSettingsStorage,
    E: EventEmitter,
    MD: MinecraftDownloader,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    RC: RequestClient,
> {
    credentials_storage: Arc<CS>,
    launch_instance_use_case: LaunchInstanceUseCase<IS, MS, PS, GISS, E, MD, PGS, JIS, JS, RC>,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        PS: ProcessStorage + 'static,
        CS: CredentialsStorage,
        GISS: GlobalInstanceSettingsStorage,
        E: EventEmitter + 'static,
        MD: MinecraftDownloader,
        PGS: ProgressService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        RC: RequestClient,
    > LaunchInstanceWithActiveAccountUseCase<IS, MS, PS, CS, GISS, E, MD, PGS, JIS, JS, RC>
{
    pub fn new(
        credentials_storage: Arc<CS>,
        launch_with_credentials_use_case: LaunchInstanceUseCase<
            IS,
            MS,
            PS,
            GISS,
            E,
            MD,
            PGS,
            JIS,
            JS,
            RC,
        >,
    ) -> Self {
        Self {
            credentials_storage,
            launch_instance_use_case: launch_with_credentials_use_case,
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
    ) -> Result<MinecraftProcessMetadata, InstanceError> {
        let default_account = self.credentials_storage.get_active().await?;

        self.launch_instance_use_case
            .execute(instance_id, default_account)
            .await
    }
}
