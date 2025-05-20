use std::sync::Arc;

use bytes::Bytes;

use crate::shared::{Request, RequestClient, RequestClientExt};

use super::{
    ListProjectVersionsParams, ListProjectsVersionsResponse, ProjectSearchParams,
    ProjectSearchResponse, ProjectVersionResponse,
};

pub struct ModrinthApiClient<RC> {
    base_url: String,
    base_headers: Option<reqwest::header::HeaderMap>,
    request_client: Arc<RC>,
}

impl<RC> ModrinthApiClient<RC>
where
    RC: RequestClient + Send + Sync,
{
    pub fn new(
        base_url: String,
        base_headers: Option<reqwest::header::HeaderMap>,
        request_client: Arc<RC>,
    ) -> Self {
        Self {
            base_url,
            base_headers,
            request_client,
        }
    }

    pub async fn search(
        &self,
        search_params: &ProjectSearchParams,
    ) -> crate::Result<ProjectSearchResponse> {
        let query_string = serde_qs::to_string(&search_params).unwrap();
        let url = format!("{}/search?{query_string}", self.base_url);

        let mut request = Request::get(&url);
        if let Some(base_headers) = self.base_headers.clone() {
            request = request.with_headers(base_headers);
        }

        self.request_client
            .fetch_json_with_progress(request, None)
            .await
    }

    pub async fn get_project_version(
        &self,
        project_version: &str,
    ) -> crate::Result<ProjectVersionResponse> {
        let url = format!("{}/version/{project_version}", self.base_url);

        let mut request = Request::get(&url);
        if let Some(base_headers) = self.base_headers.clone() {
            request = request.with_headers(base_headers);
        }

        self.request_client
            .fetch_json_with_progress(request, None)
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

        let mut request = Request::get(&url);
        if let Some(base_headers) = self.base_headers.clone() {
            request = request.with_headers(base_headers);
        }

        let response: ListProjectsVersionsResponse = self
            .request_client
            .fetch_json_with_progress(request, None)
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
        let mut request = Request::get(url);
        if let Some(base_headers) = self.base_headers.clone() {
            request = request.with_headers(base_headers);
        }

        self.request_client
            .fetch_bytes_with_progress(request, None)
            .await
    }
}
