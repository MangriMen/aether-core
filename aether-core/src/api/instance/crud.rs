use std::sync::Arc;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::{
        instance::{
            CreateInstanceUseCase, EditInstance, EditInstanceUseCase, GetInstanceUseCase,
            InstallInstanceUseCase, Instance, ListInstancesUseCase, NewInstance,
            RemoveInstanceUseCase,
        },
        java::{
            infra::{AzulJreProvider, FsJavaInstallationService},
            GetJavaUseCase, InstallJavaUseCase, InstallJreUseCase,
        },
        minecraft::{
            AssetsService, ClientService, GetVersionManifestUseCase, InstallMinecraftUseCase,
            LibrariesService, LoaderVersionResolver, MinecraftDownloadService,
        },
    },
};

#[tracing::instrument]
pub async fn create(new_instance: NewInstance) -> crate::Result<String> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let get_loader_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
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

    let get_java_use_case = Arc::new(GetJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
    ));

    let jre_provider = Arc::new(AzulJreProvider::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
    ));

    let install_jre_use_case = Arc::new(InstallJreUseCase::new(jre_provider));

    let install_java_use_case = Arc::new(InstallJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
        install_jre_use_case,
        state.location_info.clone(),
    ));

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_instance_storage().await,
        loader_version_resolver.clone(),
        get_loader_manifest_use_case.clone(),
        state.location_info.clone(),
        minecraft_download_service,
        FsJavaInstallationService,
        get_java_use_case.clone(),
        install_java_use_case.clone(),
    ));

    CreateInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        loader_version_resolver,
        install_minecraft_use_case,
        state.location_info.clone(),
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_instance_watcher_service().await?,
    )
    .execute(new_instance)
    .await
}

#[tracing::instrument]
pub async fn install(instance_id: String, force: bool) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let get_loader_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
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

    let get_java_use_case = Arc::new(GetJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
    ));

    let jre_provider = Arc::new(AzulJreProvider::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
    ));

    let install_jre_use_case = Arc::new(InstallJreUseCase::new(jre_provider));

    let install_java_use_case = Arc::new(InstallJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
        install_jre_use_case,
        state.location_info.clone(),
    ));

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_instance_storage().await,
        loader_version_resolver.clone(),
        get_loader_manifest_use_case.clone(),
        state.location_info.clone(),
        minecraft_download_service,
        FsJavaInstallationService,
        get_java_use_case.clone(),
        install_java_use_case.clone(),
    ));

    InstallInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        install_minecraft_use_case,
    )
    .execute(instance_id, force)
    .await
}

#[tracing::instrument]
pub async fn update(instance_id: String) -> crate::Result<()> {
    if let Ok(instance) = get(instance_id.clone()).await {
        if let Some(pack_info) = instance.pack_info {
            let lazy_locator = LazyLocator::get().await?;
            let plugin_registry = lazy_locator.get_plugin_registry().await;

            if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
                if let Some(plugin) = &plugin.instance {
                    plugin.lock().await.update(&instance_id).map_err(|_| {
                        crate::ErrorKind::InstanceUpdateError(format!(
                            "Failed to import instance from plugin {}",
                            pack_info.pack_type
                        ))
                        .as_error()
                    })?;
                } else {
                    return Err(crate::ErrorKind::InstanceUpdateError(format!(
                        "Can't get plugin \"{}\" to update instance. Check if it is installed and enabled",
                        &pack_info.pack_type
                    ))
                    .as_error());
                }

                return Ok(());
            } else {
                return Err(crate::ErrorKind::InstanceUpdateError(
                    "Unsupported pack type".to_owned(),
                )
                .as_error());
            };
        } else {
            return Err(
                crate::ErrorKind::InstanceUpdateError("There is not pack info".to_owned())
                    .as_error(),
            );
        };
    } else {
        return Err(crate::ErrorKind::UnmanagedProfileError(instance_id.to_string()).as_error());
    }
}

#[tracing::instrument]
pub async fn list() -> crate::Result<Vec<Instance>> {
    let lazy_locator = LazyLocator::get().await?;

    ListInstancesUseCase::new(lazy_locator.get_instance_storage().await)
        .execute()
        .await
}

#[tracing::instrument]
pub async fn get(instance_id: String) -> crate::Result<Instance> {
    let lazy_locator = LazyLocator::get().await?;

    GetInstanceUseCase::new(lazy_locator.get_instance_storage().await)
        .execute(instance_id)
        .await
}

#[tracing::instrument]
pub async fn edit(instance_id: String, edit_instance: EditInstance) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    EditInstanceUseCase::new(lazy_locator.get_instance_storage().await)
        .execute(instance_id, edit_instance)
        .await
}

#[tracing::instrument]
pub async fn remove(instance_id: String) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    RemoveInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        lazy_locator.get_instance_watcher_service().await?,
    )
    .execute(instance_id)
    .await
}
