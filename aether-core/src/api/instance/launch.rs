use crate::{
    core::LauncherState,
    features::{
        auth::{Credentials, FsCredentialsStorage},
        instance::FsInstanceStorage,
        minecraft::{self},
        process::MinecraftProcessMetadata,
        settings::FsSettingsStorage,
    },
};

#[tracing::instrument]
pub async fn run(instance_id: &str) -> crate::Result<MinecraftProcessMetadata> {
    let state = LauncherState::get().await?;

    let settings_storage = FsSettingsStorage::new(&state.locations.settings_dir);
    let credentials_storage = FsCredentialsStorage::new(&state.locations.settings_dir);
    let instance_storage = FsInstanceStorage::new(state.locations.clone());

    minecraft::run(
        &settings_storage,
        &credentials_storage,
        &instance_storage,
        instance_id,
    )
    .await
}

#[tracing::instrument]
pub async fn run_credentials(
    id: &str,
    credentials: &Credentials,
) -> crate::Result<MinecraftProcessMetadata> {
    let state = LauncherState::get().await?;

    let settings_storage = FsSettingsStorage::new(&state.locations.settings_dir);
    let instance_storage = FsInstanceStorage::new(state.locations.clone());

    minecraft::run_credentials(&settings_storage, &instance_storage, id, credentials).await
}
