use std::{collections::HashMap, path::Path};

use dashmap::DashMap;

use crate::{
    core::{domain::ServiceLocator, LauncherState},
    features::instance::{
        ContentInstallParams, ContentSearchParams, ContentSearchResult, ContentService,
        ContentType, FsPackStorage, InstanceFile,
    },
};

fn get_pack_storage(state: &LauncherState) -> FsPackStorage {
    FsPackStorage::new(state.locations.clone())
}

fn get_service(state: &LauncherState) -> ContentService<FsPackStorage> {
    ContentService::new(get_pack_storage(state), state.locations.clone())
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
    let service_locator = ServiceLocator::get().await?;

    service_locator
        .content_provider_service
        .list_providers()
        .await
}

pub async fn get_content_by_provider(
    search_params: &ContentSearchParams,
) -> crate::Result<ContentSearchResult> {
    let service_locator = ServiceLocator::get().await?;

    service_locator
        .content_provider_service
        .search(search_params)
        .await
}

pub async fn get_metadata_field_to_check_installed(provider_id: &str) -> crate::Result<String> {
    let service_locator = ServiceLocator::get().await?;

    service_locator
        .content_provider_service
        .get_update_data_id_field(provider_id)
}

pub async fn install_content(
    instance_id: &str,
    install_params: &ContentInstallParams,
) -> crate::Result<()> {
    let service_locator = ServiceLocator::get().await?;

    service_locator
        .content_provider_service
        .install(instance_id, install_params)
        .await
}
