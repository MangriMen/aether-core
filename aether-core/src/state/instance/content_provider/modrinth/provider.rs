use std::{collections::HashMap, path::PathBuf};

use path_slash::PathExt;
use reqwest::Method;

use crate::{
    core::LauncherState,
    state::{ContentRequest, ContentResponse, InstallContentPayload, InstanceFile},
    utils::{fetch::fetch_advanced, io::write_async},
};

use super::{
    get_file_from_project_version, get_project_version, get_project_version_for_game_version,
    modrinth_to_content_response, search_projects, ModrinthProviderData, ModrinthUpdateData,
    DEFAULT_HEADERS,
};

pub fn get_field_to_check_installed() -> String {
    "project_id".to_string()
}

pub async fn search_content(payload: &ContentRequest) -> crate::Result<ContentResponse> {
    let response = search_projects(payload).await?;
    Ok(modrinth_to_content_response(payload, &response))
}

pub async fn install_content(
    id: &str,
    payload: &InstallContentPayload,
) -> crate::Result<InstanceFile> {
    let state = LauncherState::get().await?;

    if let Some(provider_data) = &payload.provider_data {
        let parsed_provider_data =
            serde_json::from_value::<ModrinthProviderData>(provider_data.clone())?;

        let version = if let Some(content_version) = payload.content_version.clone() {
            get_project_version(&content_version, &state.api_semaphore).await?
        } else {
            get_project_version_for_game_version(
                &parsed_provider_data.project_id,
                &payload.game_version,
                &payload.loader,
                &state.api_semaphore,
            )
            .await?
        };

        let file_data = get_file_from_project_version(&version)
            .await
            .ok_or_else(|| {
                crate::ErrorKind::NoValueFor(format!(
                    "Content for version \"{}\" not found",
                    payload.game_version
                ))
                .as_error()
            })?;

        let file = fetch_advanced(
            Method::GET,
            &file_data.url,
            Some(DEFAULT_HEADERS.clone()),
            None,
            None,
            &state.fetch_semaphore,
            None,
        )
        .await?;

        let folder = payload.content_type.get_folder();
        let relative_path = PathBuf::from(folder).join(&file_data.filename);

        let instance_folder = crate::api::instance::get_dir(id).await?;
        let file_path = instance_folder.join(&relative_path);

        write_async(&file_path, &file).await?;

        let update_data = toml::Value::try_from(&ModrinthUpdateData {
            project_id: parsed_provider_data.project_id,
            version: version.id.to_owned(),
        })?;

        let instance_file = InstanceFile {
            name: Some(version.name.to_owned()),
            hash: file_data.hashes.sha1,
            file_name: file_data.filename,
            size: file_data.size as u64,
            content_type: payload.content_type,
            path: relative_path.to_slash_lossy().to_string(),
            disabled: false,
            update: Some(HashMap::from([(payload.provider.to_owned(), update_data)])),
        };

        Ok(instance_file)
    } else {
        Err(crate::ErrorKind::NoValueFor("Not found provider data".to_string()).as_error())
    }
}
