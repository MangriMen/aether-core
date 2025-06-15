use std::sync::Arc;

use crate::{
    features::{
        auth::CredentialsStorage,
        events::{EventEmitter, ProgressService},
        instance::InstanceStorage,
        java::{JavaInstallationService, JavaStorage},
        minecraft::{MinecraftDownloader, ReadMetadataStorage},
        process::{MinecraftProcessMetadata, ProcessStorage},
        settings::SettingsStorage,
    },
    libs::request_client::RequestClient,
};

use super::LaunchWithCredentialsUseCase;

pub struct LaunchWithActiveAccountUseCase<
    IS: InstanceStorage,
    MS: ReadMetadataStorage,
    PS: ProcessStorage,
    CS: CredentialsStorage,
    SS: SettingsStorage,
    E: EventEmitter,
    MD: MinecraftDownloader,
    PGS: ProgressService,
    JIS: JavaInstallationService,
    JS: JavaStorage,
    RC: RequestClient,
> {
    credentials_storage: Arc<CS>,
    launch_with_credentials_use_case:
        LaunchWithCredentialsUseCase<IS, MS, PS, SS, E, MD, PGS, JIS, JS, RC>,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        PS: ProcessStorage + 'static,
        CS: CredentialsStorage,
        SS: SettingsStorage,
        E: EventEmitter + 'static,
        MD: MinecraftDownloader,
        PGS: ProgressService,
        JIS: JavaInstallationService,
        JS: JavaStorage,
        RC: RequestClient,
    > LaunchWithActiveAccountUseCase<IS, MS, PS, CS, SS, E, MD, PGS, JIS, JS, RC>
{
    pub fn new(
        credentials_storage: Arc<CS>,
        launch_with_credentials_use_case: LaunchWithCredentialsUseCase<
            IS,
            MS,
            PS,
            SS,
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
            launch_with_credentials_use_case,
        }
    }

    pub async fn execute(&self, instance_id: String) -> crate::Result<MinecraftProcessMetadata> {
        let default_account = self.credentials_storage.get_active().await?;

        self.launch_with_credentials_use_case
            .execute(instance_id, default_account)
            .await
    }
}
