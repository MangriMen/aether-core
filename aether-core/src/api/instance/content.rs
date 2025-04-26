use std::{collections::HashMap, path::Path};

use dashmap::DashMap;

use crate::{
    core::LauncherState,
    features::instance::{
        content_provider, ContentMetadataFile, ContentMetadataStorage, ContentRequest,
        ContentResponse, ContentService, ContentType, FsContentMetadataStorage,
        InstallContentPayload, InstanceFile,
    },
};

fn get_content_metadata_storage(state: &LauncherState) -> FsContentMetadataStorage {
    FsContentMetadataStorage::new(state.locations.clone())
}

fn get_service(state: &LauncherState) -> ContentService<FsContentMetadataStorage> {
    ContentService::new(get_content_metadata_storage(state), state.locations.clone())
}

pub async fn get_contents(instance_id: &str) -> crate::Result<DashMap<String, InstanceFile>> {
    let state = LauncherState::get().await?;
    let content_service = get_service(&state);

    content_service.list(instance_id).await
}

pub async fn remove_content(instance_id: &str, content_path: &str) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let content_service = get_service(&state);

    content_service.remove(instance_id, content_path).await
}

pub async fn remove_contents(instance_id: &str, content_paths: &[String]) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let content_service = get_service(&state);

    content_service
        .remove_many(instance_id, content_paths)
        .await
}

pub async fn toggle_disable_content(
    instance_id: &str,
    content_path: &str,
) -> crate::Result<String> {
    let state = LauncherState::get().await?;
    let content_service = get_service(&state);

    content_service
        .toggle_disable_content(instance_id, content_path)
        .await
}

pub async fn enable_contents(instance_id: &str, content_paths: &[String]) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let content_service = get_service(&state);

    content_service
        .enable_many(instance_id, content_paths)
        .await
}

pub async fn disable_contents(instance_id: &str, content_paths: &[String]) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let content_service = get_service(&state);

    content_service
        .disable_many(instance_id, content_paths)
        .await
}

pub async fn import_contents(
    instance_id: &str,
    content_type: ContentType,
    content_paths: Vec<&Path>,
) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let content_service = get_service(&state);

    content_service
        .import_many(instance_id, content_type, &content_paths)
        .await
}

pub async fn get_content_providers() -> crate::Result<HashMap<String, String>> {
    Ok(HashMap::from([
        // ("Curseforge".to_string(), "curseforge".to_string()),
        ("Modrinth".to_string(), "modrinth".to_string()),
    ]))
}

pub async fn get_content_by_provider(payload: &ContentRequest) -> crate::Result<ContentResponse> {
    match payload.provider.as_str() {
        "modrinth" => content_provider::modrinth::search_content(payload).await,
        _ => Err(crate::ErrorKind::ContentProviderNotFound {
            provider: payload.provider.to_string(),
        }
        .as_error()),
    }
}

pub async fn get_metadata_field_to_check_installed(provider: &str) -> crate::Result<String> {
    match provider {
        "modrinth" => Ok(content_provider::modrinth::get_field_to_check_installed()),
        _ => Err(crate::ErrorKind::ContentProviderNotFound {
            provider: provider.to_string(),
        }
        .as_error()),
    }
}

pub async fn install_content(
    instance_id: &str,
    payload: &InstallContentPayload,
) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let content_metadata_storage = get_content_metadata_storage(&state);

    let instance_dir = state.locations.instance_dir(instance_id);

    let instance_file = match payload.provider.as_str() {
        "modrinth" => content_provider::modrinth::install_content(&instance_dir, payload).await,
        _ => Err(crate::ErrorKind::ContentProviderNotFound {
            provider: payload.provider.to_string(),
        }
        .as_error()),
    }?;

    content_metadata_storage
        .update_content_metadata_file(
            instance_id,
            &instance_file.path,
            &ContentMetadataFile {
                name: instance_file.name.clone(),
                file_name: instance_file.file_name.clone(),
                hash: instance_file.hash,
                download: None,
                option: None,
                side: None,
                update_provider: Some(payload.provider.to_owned()),
                update: instance_file.update,
            },
        )
        .await?;

    Ok(())
}
