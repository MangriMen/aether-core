use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        auth::CredentialsStorage,
        events::{EventEmitter, ProgressService},
        instance::InstanceStorage,
        minecraft::{MinecraftDownloader, ReadMetadataStorage},
        process::{MinecraftProcessMetadata, ProcessStorage},
        settings::SettingsStorage,
    },
    shared::domain::AsyncUseCaseWithInputAndError,
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
> {
    credentials_storage: Arc<CS>,
    launch_with_credentials_use_case: LaunchWithCredentialsUseCase<IS, MS, PS, SS, E, MD, PGS>,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        PS: ProcessStorage,
        CS: CredentialsStorage,
        SS: SettingsStorage,
        E: EventEmitter,
        MD: MinecraftDownloader,
        PGS: ProgressService,
    > LaunchWithActiveAccountUseCase<IS, MS, PS, CS, SS, E, MD, PGS>
{
    pub fn new(
        credentials_storage: Arc<CS>,
        launch_with_credentials_use_case: LaunchWithCredentialsUseCase<IS, MS, PS, SS, E, MD, PGS>,
    ) -> Self {
        Self {
            credentials_storage,
            launch_with_credentials_use_case,
        }
    }
}

#[async_trait]
impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        PS: ProcessStorage,
        CS: CredentialsStorage,
        SS: SettingsStorage,
        E: EventEmitter,
        MD: MinecraftDownloader,
        PGS: ProgressService,
    > AsyncUseCaseWithInputAndError
    for LaunchWithActiveAccountUseCase<IS, MS, PS, CS, SS, E, MD, PGS>
{
    type Input = String;
    type Output = MinecraftProcessMetadata;
    type Error = crate::Error;

    async fn execute(&self, instance_id: Self::Input) -> Result<Self::Output, Self::Error> {
        let default_account = self
            .credentials_storage
            .get_active()
            .await?
            .ok_or_else(|| crate::ErrorKind::NoCredentialsError.as_error())?;

        self.launch_with_credentials_use_case
            .execute((instance_id, default_account))
            .await
    }
}
