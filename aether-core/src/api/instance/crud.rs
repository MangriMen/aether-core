use std::path::PathBuf;

use crate::{
    core::LauncherState,
    features::instance::{EditInstance, FsInstanceStorage, Instance, InstanceService, NewInstance},
};

fn get_service(state: &LauncherState) -> InstanceService<FsInstanceStorage> {
    let instance_storage = FsInstanceStorage::new(state.locations.clone());

    InstanceService::new(
        instance_storage,
        state.locations.clone(),
        state.file_watcher.clone(),
    )
}

#[tracing::instrument]
pub async fn create(new_instance: NewInstance) -> crate::Result<String> {
    let state = LauncherState::get().await?;
    let metadata_storage = crate::api::metadata::get_storage().await?;
    let instance_service = get_service(&state);

    instance_service
        .create(&metadata_storage, &new_instance)
        .await
}

#[tracing::instrument]
pub async fn install(id: &str, force: bool) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let instance_service = get_service(&state);

    instance_service.install(id, force).await
}

#[tracing::instrument]
pub async fn update(id: &str) -> crate::Result<()> {
    if let Ok(instance) = get(id).await {
        if let Some(pack_info) = instance.pack_info {
            let state = LauncherState::get().await?;
            let plugin_manager = state.plugin_manager.read().await;

            if let Ok(plugin) = plugin_manager.get_plugin(&pack_info.pack_type) {
                if let Some(plugin) = plugin.get_plugin() {
                    plugin.lock().await.update(id).map_err(|_| {
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
    let state = LauncherState::get().await?;
    let instance_service = get_service(&state);

    instance_service.list().await
}

#[tracing::instrument]
pub async fn get(id: &str) -> crate::Result<Instance> {
    let state = LauncherState::get().await?;
    let instance_service = get_service(&state);

    instance_service.get(id).await
}

#[tracing::instrument]
pub async fn edit(id: &str, edit_instance: &EditInstance) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let instance_service = get_service(&state);

    instance_service.edit(id, edit_instance).await
}

#[tracing::instrument]
pub async fn remove(id: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let instance_storage = FsInstanceStorage::new(state.locations.clone());

    let instance_service = InstanceService::new(
        instance_storage,
        state.locations.clone(),
        state.file_watcher.clone(),
    );

    instance_service.remove(id).await
}

#[tracing::instrument]
pub async fn get_dir(id: &str) -> crate::Result<PathBuf> {
    let state = LauncherState::get().await?;
    Ok(state.locations.instance_dir(id))
}
