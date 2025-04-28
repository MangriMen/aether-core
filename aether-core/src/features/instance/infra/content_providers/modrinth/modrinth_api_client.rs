use std::sync::Arc;

use bytes::Bytes;
use reqwest::Method;

use crate::shared::{fetch_advanced, fetch_json, FetchSemaphore};

use super::{
    ListProjectVersionsParams, ListProjectsVersionsResponse, ProjectSearchParams,
    ProjectSearchResponse, ProjectVersionResponse,
};

pub struct ModrinthApiClient {
    base_url: String,
    base_headers: Option<reqwest::header::HeaderMap>,
    api_semaphore: Arc<FetchSemaphore>,
}

impl ModrinthApiClient {
    pub fn new(
        base_url: String,
        api_semaphore: Arc<FetchSemaphore>,
        base_headers: Option<reqwest::header::HeaderMap>,
    ) -> Self {
        Self {
            base_url,
            base_headers,
            api_semaphore,
        }
    }

    pub async fn search(
        &self,
        search_params: &ProjectSearchParams,
    ) -> crate::Result<ProjectSearchResponse> {
        let query_string = serde_qs::to_string(&search_params).unwrap();
        let url = format!("{}/search?{query_string}", self.base_url);

        fetch_json(
            Method::GET,
            &url,
            self.base_headers.clone(),
            None,
            None,
            &self.api_semaphore,
        )
        .await
    }

    pub async fn get_project_version(
        &self,
        project_version: &str,
    ) -> crate::Result<ProjectVersionResponse> {
        let url = format!("{}/version/{project_version}", self.base_url);

        fetch_json(
            Method::GET,
            &url,
            self.base_headers.clone(),
            None,
            None,
            &self.api_semaphore,
        )
        .await
    }

    pub async fn get_project_version_for_game_version(
        &self,
        project_id: &str,
        game_version: &str,
        loader: &Option<String>,
    ) -> crate::Result<ProjectVersionResponse> {
        let params = ListProjectVersionsParams {
            loaders: loader.as_ref().map(|l| vec![l.clone()]),
            game_versions: vec![game_version.to_string()],
        };

        let query_string = serde_qs::to_string(&params).unwrap();
        let url = format!(
            "{}/project/{project_id}/version?{query_string}",
            self.base_url
        );

        let response = fetch_json::<ListProjectsVersionsResponse>(
            Method::GET,
            &url,
            self.base_headers.clone(),
            None,
            None,
            &self.api_semaphore,
        )
        .await?;

        let version = response
            .iter()
            .find(|v| v.game_versions.contains(&game_version.to_string()));

        if let Some(version) = version {
            Ok(version.clone())
        } else {
            Err(crate::ErrorKind::NoValueFor(format!(
                "Content for version \"{}\" not found",
                game_version
            ))
            .as_error())
        }
    }

    pub async fn get_file(&self, url: &str) -> crate::Result<Bytes> {
        fetch_advanced(
            Method::GET,
            url,
            self.base_headers.clone(),
            None,
            None,
            &self.api_semaphore,
            None,
        )
        .await
    }
}
