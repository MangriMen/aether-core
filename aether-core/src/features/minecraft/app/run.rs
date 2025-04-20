use crate::features::{
    auth::{storage::CredentialsStorage, Credentials},
    instance::{self, InstanceStorage},
    process::MinecraftProcessMetadata,
    settings::SettingsStorage,
};

use super::launch_minecraft;

pub async fn run<SS, CS, IS>(
    settings_storage: &SS,
    credentials_storage: &CS,
    instance_storage: &IS,
    instance_id: &str,
) -> crate::Result<MinecraftProcessMetadata>
where
    SS: SettingsStorage + ?Sized,
    CS: CredentialsStorage + ?Sized,
    IS: InstanceStorage + ?Sized,
{
    let default_account = credentials_storage
        .get_active()
        .await?
        .ok_or_else(|| crate::ErrorKind::NoCredentialsError.as_error())?;

    run_credentials(
        settings_storage,
        instance_storage,
        instance_id,
        &default_account,
    )
    .await
}

pub async fn run_credentials<SS, IS>(
    settings_storage: &SS,
    instance_storage: &IS,
    instance_id: &str,
    credentials: &Credentials,
) -> crate::Result<MinecraftProcessMetadata>
where
    SS: SettingsStorage + ?Sized,
    IS: InstanceStorage + ?Sized,
{
    let settings = settings_storage.get().await?;
    let instance = instance_storage.get(instance_id).await?;

    let launch_settings = instance::get_merged_settings(&instance, &settings);

    launch_minecraft(&instance, &launch_settings, credentials).await
}
