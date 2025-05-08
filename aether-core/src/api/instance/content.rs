use std::{collections::HashMap, path::PathBuf};

use dashmap::DashMap;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::instance::{
        ChangeContentState, ChangeContentStateUseCase, ContentInstallParams, ContentSearchParams,
        ContentSearchResult, ContentStateAction, ContentType, GetProviderMetadataUseCase,
        ImportContent, ImportContentUseCase, InstallContentUseCase, InstanceFile,
        ListContentUseCase, ListProvidersUseCase, RemoveContent, RemoveContentUseCase,
        SearchContentUseCase,
    },
    shared::domain::{AsyncUseCaseWithError, AsyncUseCaseWithInputAndError},
};

pub async fn get_contents(instance_id: String) -> crate::Result<DashMap<String, InstanceFile>> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    ListContentUseCase::new(
        lazy_locator.get_pack_storage().await,
        state.location_info.clone(),
    )
    .execute(instance_id)
    .await
}

pub async fn remove_content(instance_id: String, content_path: String) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    RemoveContentUseCase::new(
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_pack_storage().await,
        state.location_info.clone(),
    )
    .execute(RemoveContent::single(instance_id, content_path))
    .await
}

pub async fn remove_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    RemoveContentUseCase::new(
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_pack_storage().await,
        state.location_info.clone(),
    )
    .execute(RemoveContent::multiple(instance_id, content_paths))
    .await
}

pub async fn enable_contents(instance_id: String, content_paths: Vec<String>) -> crate::Result<()> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    ChangeContentStateUseCase::new(
        lazy_locator.get_event_emitter().await,
        state.location_info.clone(),
    )
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
    let lazy_locator = LazyLocator::get().await?;

    ChangeContentStateUseCase::new(
        lazy_locator.get_event_emitter().await,
        state.location_info.clone(),
    )
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
        lazy_locator.get_event_emitter().await,
        lazy_locator.get_pack_storage().await,
        state.location_info.clone(),
    )
    .execute(ImportContent::multiple(
        instance_id,
        content_type,
        source_paths,
    ))
    .await
}

pub async fn get_content_providers() -> crate::Result<HashMap<String, String>> {
    let lazy_locator = LazyLocator::get().await?;

    ListProvidersUseCase::new(lazy_locator.get_content_provider_registry().await)
        .execute()
        .await
}

pub async fn search_content(
    search_params: ContentSearchParams,
) -> crate::Result<ContentSearchResult> {
    let lazy_locator = LazyLocator::get().await?;

    SearchContentUseCase::new(lazy_locator.get_content_provider_registry().await)
        .execute(search_params)
        .await
}

pub async fn get_metadata_field_to_check_installed(provider_id: String) -> crate::Result<String> {
    let lazy_locator = LazyLocator::get().await?;

    GetProviderMetadataUseCase::new(lazy_locator.get_content_provider_registry().await)
        .execute(provider_id)
        .await
}

pub async fn install_content(
    instance_id: String,
    install_params: ContentInstallParams,
) -> crate::Result<()> {
    let lazy_locator = LazyLocator::get().await?;

    InstallContentUseCase::new(
        lazy_locator.get_pack_storage().await,
        lazy_locator.get_content_provider_registry().await,
    )
    .execute((instance_id, install_params))
    .await
}
