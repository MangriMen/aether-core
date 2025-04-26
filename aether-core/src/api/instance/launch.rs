use crate::{
    core::LauncherState,
    features::{
        auth::{Credentials, FsCredentialsStorage},
        minecraft::{self},
        process::MinecraftProcessMetadata,
        settings::FsSettingsStorage,
    },
};

use super::get_manager;

#[tracing::instrument]
pub async fn run(instance_id: &str) -> crate::Result<MinecraftProcessMetadata> {
    let state = LauncherState::get().await?;

    let settings_storage = FsSettingsStorage::new(&state.locations.settings_dir);
    let credentials_storage = FsCredentialsStorage::new(&state.locations.settings_dir);
    let instance_manager = get_manager(&state);
    let metadata_storage = crate::api::metadata::get_storage().await?;

    minecraft::run(
        &settings_storage,
        &credentials_storage,
        &instance_manager,
        &metadata_storage,
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
    let instance_manager = get_manager(&state);
    let metadata_storage = crate::api::metadata::get_storage().await?;

    minecraft::run_credentials(
        &settings_storage,
        &instance_manager,
        &metadata_storage,
        id,
        credentials,
    )
    .await
}
