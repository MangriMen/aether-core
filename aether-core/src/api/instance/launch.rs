use crate::{
    api,
    state::{self, Credentials, LauncherState, MinecraftProcessMetadata, Settings},
};

#[tracing::instrument]
pub async fn run(name: &str) -> crate::Result<MinecraftProcessMetadata> {
    let state = LauncherState::get().await?;

    let default_account = Credentials::get_active(&state)
        .await?
        .ok_or_else(|| crate::ErrorKind::NoCredentialsError.as_error())?;

    run_credentials(name, &default_account).await
}

#[tracing::instrument]
pub async fn run_credentials(
    id: &str,
    credentials: &state::Credentials,
) -> crate::Result<MinecraftProcessMetadata> {
    let state = LauncherState::get().await?;
    // TODO: add io semaphore
    let settings = Settings::get(&state).await?;

    let instance = api::instance::get(id).await?;

    let launch_args = api::instance::utils::get_launch_args(&instance, &settings);
    let launch_settings = api::instance::utils::get_launch_settings(&instance, &settings);
    let launch_metadata = api::instance::utils::get_launch_metadata(&instance, &settings);

    api::instance::utils::run_pre_launch_command(&instance, &settings).await?;

    crate::launcher::launch_minecraft(
        &instance,
        &launch_args,
        &launch_settings,
        &launch_metadata,
        credentials,
    )
    .await
}
