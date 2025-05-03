use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        auth::CredentialsStorage,
        events::{EventEmitter, ProgressBarStorage},
        instance::InstanceStorage,
        minecraft::ReadMetadataStorage,
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
    PBS: ProgressBarStorage,
> {
    credentials_storage: Arc<CS>,
    launch_with_credentials_use_case: LaunchWithCredentialsUseCase<IS, MS, PS, SS, E, PBS>,
}

impl<
        IS: InstanceStorage,
        MS: ReadMetadataStorage,
        PS: ProcessStorage,
        CS: CredentialsStorage,
        SS: SettingsStorage,
        E: EventEmitter,
        PBS: ProgressBarStorage,
    > LaunchWithActiveAccountUseCase<IS, MS, PS, CS, SS, E, PBS>
{
    pub fn new(
        credentials_storage: Arc<CS>,
        launch_with_credentials_use_case: LaunchWithCredentialsUseCase<IS, MS, PS, SS, E, PBS>,
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
        PBS: ProgressBarStorage,
    > AsyncUseCaseWithInputAndError for LaunchWithActiveAccountUseCase<IS, MS, PS, CS, SS, E, PBS>
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
