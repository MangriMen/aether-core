use std::{collections::HashMap, path::PathBuf};

use dashmap::DashMap;

use crate::{
    core::{
        domain::{LazyLocator, ServiceLocator},
        LauncherState,
    },
    features::instance::{
        ChangeContentState, ChangeContentStateUseCase, ContentInstallParams, ContentSearchParams,
        ContentSearchResult, ContentStateAction, ContentType, ImportContent, ImportContentUseCase,
        InstanceFile, ListContentUseCase, RemoveContent, RemoveContentUseCase,
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub async fn get_contents(instance_id: String) -> crate::Result<DashMap<String, InstanceFile>> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    ListContentUseCase::new(
        lazy_locator.get_pack_storage().await,
        state.locations.clone(),
    )
    .execute(instance_id)
    .await
}

pub async fn remove_content(instance_id: String, content_path: String) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    RemoveContentUseCase::new(
        lazy_locator.get_pack_storage().await,
        state.locations.clone(),
    )
    .execute(RemoveContent::single(instance_id, content_path))
    .await
}

pub async fn remove_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    RemoveContentUseCase::new(
        lazy_locator.get_pack_storage().await,
        state.locations.clone(),
    )
    .execute(RemoveContent::multiple(instance_id, content_paths))
    .await
}

pub async fn enable_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let state = LauncherState::get().await?;

    ChangeContentStateUseCase::new(state.locations.clone())
        .execute(ChangeContentState::multiple(
            instance_id,
            content_paths,
            ContentStateAction::Enable,
        ))
        .await
}

pub async fn disable_contents(
    instance_id: String,
    content_paths: Vec<String>,
) -> crate::Result<()> {
    let state = LauncherState::get().await?;

    ChangeContentStateUseCase::new(state.locations.clone())
        .execute(ChangeContentState::multiple(
            instance_id,
            content_paths,
            ContentStateAction::Disable,
        ))
        .await
}

pub async fn import_contents(
    instance_id: String,
    content_type: ContentType,
    source_paths: Vec<PathBuf>,
) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    ImportContentUseCase::new(
        lazy_locator.get_pack_storage().await,
        state.locations.clone(),
    )
    .execute(ImportContent::multiple(
        instance_id,
        content_type,
        source_paths,
    ))
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
