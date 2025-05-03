use std::sync::Arc;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::{
        instance::{
            CreateInstanceUseCase, EditInstance, EditInstanceUseCase, GetInstanceUseCase,
            InstallInstanceUseCase, Instance, ListInstancesUseCase, NewInstance,
            RemoveInstanceUseCase,
        },
        minecraft::{GetVersionManifestUseCase, InstallMinecraftUseCase, LoaderVersionResolver},
    },
    shared::domain::{AsyncUseCaseWithError, AsyncUseCaseWithInputAndError},
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

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        lazy_locator.get_instance_storage().await,
        loader_version_resolver.clone(),
        get_loader_manifest_use_case.clone(),
        state.location_info.clone(),
    ));

    CreateInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        loader_version_resolver,
        install_minecraft_use_case,
        state.location_info.clone(),
        state.file_watcher.clone(),
    )
    .execute(new_instance)
    .await
}

#[tracing::instrument]
pub async fn install(id: String, force: bool) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let loader_version_resolver = Arc::new(LoaderVersionResolver::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let get_loader_manifest_use_case = Arc::new(GetVersionManifestUseCase::new(
        lazy_locator.get_metadata_storage().await,
    ));

    let install_minecraft_use_case = Arc::new(InstallMinecraftUseCase::new(
        lazy_locator.get_instance_storage().await,
        loader_version_resolver.clone(),
        get_loader_manifest_use_case.clone(),
        state.location_info.clone(),
    ));

    InstallInstanceUseCase::new(
        lazy_locator.get_instance_storage().await,
        install_minecraft_use_case,
    )
    .execute((id, force))
    .await
}

#[tracing::instrument]
pub async fn update(id: String) -> crate::Result<()> {
    if let Ok(instance) = get(id.clone()).await {
        if let Some(pack_info) = instance.pack_info {
            let lazy_locator = LazyLocator::get().await?;
            let plugin_registry = lazy_locator.get_plugin_registry().await;

            if let Ok(plugin) = plugin_registry.get(&pack_info.pack_type) {
                if let Some(plugin) = &plugin.instance {
                    plugin.lock().await.update(&id).map_err(|_| {
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
        return Err(crate::ErrorKind::UnmanagedProfileError(id.to_string()).as_error());
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
pub async fn get(id: String) -> crate::Result<Instance> {
    let lazy_locator = LazyLocator::get().await?;

    GetInstanceUseCase::new(lazy_locator.get_instance_storage().await)
        .execute(id)
        .await
}

#[tracing::instrument]
pub async fn edit(id: String, edit_instance: EditInstance) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    EditInstanceUseCase::new(lazy_locator.get_instance_storage().await)
        .execute((id, edit_instance))
        .await
}

#[tracing::instrument]
pub async fn remove(id: String) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    RemoveInstanceUseCase::new(lazy_locator.get_instance_storage().await)
        .execute(id)
        .await
}
