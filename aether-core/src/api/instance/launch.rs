use std::sync::Arc;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::{
        auth::Credentials,
        minecraft::{
            AssetsService, ClientService, GetVersionManifestUseCase, InstallMinecraftUseCase,
            LaunchMinecraftUseCase, LaunchWithActiveAccountUseCase, LaunchWithCredentialsUseCase,
            LibrariesService, LoaderVersionResolver, MinecraftDownloadService,
        },
        process::{
            GetProcessMetadataByInstanceIdUseCase, MinecraftProcessMetadata, StartProcessUseCase,
        },
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

#[tracing::instrument]
pub async fn run(instance_id: String) -> crate::Result<MinecraftProcessMetadata> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let get_version_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let client_service = ClientService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let assets_service = AssetsService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let libraries_service = LibrariesService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let minecraft_download_service = MinecraftDownloadService::new(
        client_service,
        assets_service,
        libraries_service,
        state.location_info.clone(),
        lazy_locator.get_request_client().await,
        lazy_locator.get_progress_service().await,
    );

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_instance_storage().await,
        loader_version_resolver.clone(),
        get_version_manifest_use_case.clone(),
        state.location_info.clone(),
        minecraft_download_service,
    ));

    let get_process_by_instance_id_use_case = Arc::new(GetProcessMetadataByInstanceIdUseCase::new(
        lazy_locator.get_process_storage().await,
    ));

    let start_process_use_case = Arc::new(StartProcessUseCase::new(
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_process_storage().await,
    ));

    let client_service = ClientService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let assets_service = AssetsService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let libraries_service = LibrariesService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let minecraft_download_service = MinecraftDownloadService::new(
        client_service,
        assets_service,
        libraries_service,
        state.location_info.clone(),
        lazy_locator.get_request_client().await,
        lazy_locator.get_progress_service().await,
    );

    let launch_minecraft_use_case = LaunchMinecraftUseCase::new(
        lazy_locator.get_instance_storage().await,
        loader_version_resolver,
        install_minecraft_use_case,
        get_version_manifest_use_case,
        get_process_by_instance_id_use_case,
        start_process_use_case,
        minecraft_download_service,
    );

    let launch_with_credentials_use_case = LaunchWithCredentialsUseCase::new(
        lazy_locator.get_instance_storage().await,
        lazy_locator.get_settings_storage().await,
        launch_minecraft_use_case,
    );

    LaunchWithActiveAccountUseCase::new(
        lazy_locator.get_credentials_storage().await,
        launch_with_credentials_use_case,
    )
    .execute(instance_id)
    .await
}

#[tracing::instrument]
pub async fn run_credentials(
    instance_id: String,
    credentials: Credentials,
) -> crate::Result<MinecraftProcessMetadata> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let get_version_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let client_service = ClientService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let assets_service = AssetsService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let libraries_service = LibrariesService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let minecraft_download_service = MinecraftDownloadService::new(
        client_service,
        assets_service,
        libraries_service,
        state.location_info.clone(),
        lazy_locator.get_request_client().await,
        lazy_locator.get_progress_service().await,
    );

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_instance_storage().await,
        loader_version_resolver.clone(),
        get_version_manifest_use_case.clone(),
        state.location_info.clone(),
        minecraft_download_service,
    ));

    let get_process_by_instance_id_use_case = Arc::new(GetProcessMetadataByInstanceIdUseCase::new(
        lazy_locator.get_process_storage().await,
    ));

    let start_process_use_case = Arc::new(StartProcessUseCase::new(
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_process_storage().await,
    ));

    let client_service = ClientService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let assets_service = AssetsService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let libraries_service = LibrariesService::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
        state.location_info.clone(),
    );
    let minecraft_download_service = MinecraftDownloadService::new(
        client_service,
        assets_service,
        libraries_service,
        state.location_info.clone(),
        lazy_locator.get_request_client().await,
        lazy_locator.get_progress_service().await,
    );

    let launch_minecraft_use_case = LaunchMinecraftUseCase::new(
        lazy_locator.get_instance_storage().await,
        loader_version_resolver,
        install_minecraft_use_case,
        get_version_manifest_use_case,
        get_process_by_instance_id_use_case,
        start_process_use_case,
        minecraft_download_service,
    );

    LaunchWithCredentialsUseCase::new(
        lazy_locator.get_instance_storage().await,
        lazy_locator.get_settings_storage().await,
        launch_minecraft_use_case,
    )
    .execute((instance_id, credentials))
    .await
}
