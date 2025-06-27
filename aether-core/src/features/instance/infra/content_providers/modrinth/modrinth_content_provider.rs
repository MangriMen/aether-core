use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use path_slash::PathBufExt;

use crate::{
    features::{
        instance::{
            modrinth::ProjectSearchParams, ContentInstallParams, ContentProvider,
            ContentSearchParams, ContentSearchResult, InstanceError, InstanceFile,
        },
        settings::LocationInfo,
    },
    libs::request_client::RequestClient,
    shared::write_async,
};

use super::{
    get_first_file_from_project_version, modrinth_to_content_response, File, ModrinthApiClient,
    ModrinthProviderData, ModrinthUpdateData, ProjectVersionResponse, MODRINTH_API_URL,
};

pub struct ModrinthContentProvider<RC> {
    api: ModrinthApiClient<RC>,
    location_info: Arc<LocationInfo>,
}

impl<RC: RequestClient> ModrinthContentProvider<RC> {
    pub fn new(
        location_info: Arc<LocationInfo>,
        base_headers: Option<reqwest::header::HeaderMap>,
        request_client: Arc<RC>,
    ) -> Self {
        Self {
            api: ModrinthApiClient::new(MODRINTH_API_URL.to_string(), base_headers, request_client),
            location_info,
        }
    }

    fn parse_provider_data(
        install_params: &ContentInstallParams,
    ) -> Result<ModrinthProviderData, InstanceError> {
        install_params
            .provider_data
            .as_ref()
            .map(|data| serde_json::from_value(data.clone()))
            .transpose()
            .map_err(|_| {
                InstanceError::ContentDownloadError("Failed to parse provider data".to_owned())
            })?
            .ok_or(InstanceError::ContentDownloadError(
                "Provider data not found".to_owned(),
            ))
    }

    async fn resolve_project_version(
        &self,
        install_params: &ContentInstallParams,
        provider_data: &ModrinthProviderData,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        match &install_params.content_version {
            Some(version) => self.api.get_project_version(version).await,
            None => {
                self.api
                    .get_project_version_for_game_version(
                        &provider_data.project_id,
                        &install_params.game_version,
                        &install_params.loader,
                    )
                    .await
            }
        }
    }

    fn get_project_file(
        project_version: &ProjectVersionResponse,
        install_params: &ContentInstallParams,
    ) -> Result<File, InstanceError> {
        get_first_file_from_project_version(project_version).ok_or(
            InstanceError::ContentForGameVersionNotFound {
                game_version: install_params.game_version.to_owned(),
            },
        )
    }

    async fn download_and_save_file(
        &self,
        file_url: &str,
        file_path: &Path,
    ) -> Result<(), InstanceError> {
        let file_bytes = self.api.get_file(file_url).await?;
        Ok(write_async(file_path, &file_bytes).await?)
    }

    fn create_instance_file(
        file: &File,
        version: &ProjectVersionResponse,
        install_params: &ContentInstallParams,
        relative_path: &PathBuf,
        provider_data: &ModrinthProviderData,
    ) -> Result<InstanceFile, InstanceError> {
        let update_data = toml::Value::try_from(&ModrinthUpdateData {
            project_id: provider_data.project_id.clone(),
            version: version.id.clone(),
        })
        .map_err(|_| {
            InstanceError::ContentDownloadError("Failed to parse update data".to_owned())
        })?;

        Ok(InstanceFile {
            name: Some(version.name.clone()),
            hash: file.hashes.sha1.clone(),
            file_name: file.filename.clone(),
            size: file.size as u64,
            content_type: install_params.content_type,
            path: relative_path.to_slash_lossy().to_string(),
            disabled: false,
            update: Some(HashMap::from([(
                install_params.provider.clone(),
                update_data,
            )])),
        })
    }

    fn get_relative_content_path(install_params: &ContentInstallParams, file: &File) -> PathBuf {
        Path::new(install_params.content_type.get_folder()).join(&file.filename)
    }

    fn get_content_path(&self, instance_id: &str, relative_path: &Path) -> PathBuf {
        self.location_info
            .instance_dir(instance_id)
            .join(relative_path)
    }
}

#[async_trait]
impl<RC: RequestClient> ContentProvider for ModrinthContentProvider<RC> {
    async fn search(
        &self,
        search_params: &ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let project_search_params = ProjectSearchParams::try_from(search_params.clone())
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

        let project_search_response = self.api.search(&project_search_params).await?;

        Ok(modrinth_to_content_response(
            search_params,
            &project_search_response,
        ))
    }

    async fn install(
        &self,
        instance_id: &str,
        install_params: &ContentInstallParams,
    ) -> Result<InstanceFile, InstanceError> {
        let provider_data = Self::parse_provider_data(install_params)?;

        let project_version = self
            .resolve_project_version(install_params, &provider_data)
            .await?;

        let file = Self::get_project_file(&project_version, install_params)?;
        let relative_content_path = Self::get_relative_content_path(install_params, &file);
        let content_path = self.get_content_path(instance_id, &relative_content_path);

        self.download_and_save_file(&file.url, &content_path)
            .await?;

        Self::create_instance_file(
            &file,
            &project_version,
            install_params,
            &relative_content_path,
            &provider_data,
        )
    }

    fn get_update_data_id_field(&self) -> String {
        "project_id".to_string()
    }
}
