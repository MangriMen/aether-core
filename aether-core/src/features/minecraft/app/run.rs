use std::sync::Arc;

use crate::features::{
    auth::{storage::CredentialsStorage, Credentials},
    instance::{self, InstanceStorage},
    minecraft::ReadMetadataStorage,
    process::MinecraftProcessMetadata,
    settings::SettingsStorage,
};

use super::launch_minecraft;

pub async fn run<SS, CS, IS, MS>(
    settings_storage: &SS,
    credentials_storage: &CS,
    instance_storage: Arc<IS>,
    metadata_storage: Arc<MS>,
    instance_id: &str,
) -> crate::Result<MinecraftProcessMetadata>
where
    SS: SettingsStorage + ?Sized,
    CS: CredentialsStorage + ?Sized,
    IS: InstanceStorage + Send + Sync,
    MS: ReadMetadataStorage,
{
    let default_account = credentials_storage
        .get_active()
        .await?
        .ok_or_else(|| crate::ErrorKind::NoCredentialsError.as_error())?;

    run_credentials(
        settings_storage,
        instance_storage,
        metadata_storage,
        instance_id,
        &default_account,
    )
    .await
}

pub async fn run_credentials<SS, IS, MS>(
    settings_storage: &SS,
    instance_storage: Arc<IS>,
    metadata_storage: Arc<MS>,
    instance_id: &str,
    credentials: &Credentials,
) -> crate::Result<MinecraftProcessMetadata>
where
    SS: SettingsStorage + ?Sized,
    IS: InstanceStorage + Send + Sync,
    MS: ReadMetadataStorage,
{
    let settings = settings_storage.get().await?;
    let instance = instance_storage.get(instance_id).await?;

    let launch_settings = instance::resolve_launch_settings(&instance, &settings);

    launch_minecraft(
        instance_storage,
        metadata_storage,
        &instance,
        &launch_settings,
        credentials,
    )
    .await
}
