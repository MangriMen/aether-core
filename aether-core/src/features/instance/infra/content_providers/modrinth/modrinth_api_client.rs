use std::sync::Arc;

use bytes::Bytes;

use crate::{
    features::instance::InstanceError,
    libs::request_client::{Request, RequestClient, RequestClientExt},
};

use super::{
    ListProjectVersionsParams, ListProjectsVersionsResponse, ProjectSearchParams,
    ProjectSearchResponse, ProjectVersionResponse,
};

pub struct ModrinthApiClient<RC> {
    base_url: String,
    base_headers: Option<reqwest::header::HeaderMap>,
    request_client: Arc<RC>,
}

impl<RC: RequestClient> ModrinthApiClient<RC> {
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
    ) -> Result<ProjectSearchResponse, InstanceError> {
        let query_string = serde_qs::to_string(&search_params).unwrap();
        let url = format!("{}/search?{query_string}", self.base_url);

        let mut request = Request::get(&url);
        if let Some(base_headers) = self.base_headers.clone() {
            request = request.with_headers(base_headers);
        }

        self.request_client
            .fetch_json_with_progress(request, None)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))
    }

    pub async fn get_project_version(
        &self,
        project_version: &str,
    ) -> Result<ProjectVersionResponse, InstanceError> {
        let url = format!("{}/version/{project_version}", self.base_url);

        let mut request = Request::get(&url);
        if let Some(base_headers) = self.base_headers.clone() {
            request = request.with_headers(base_headers);
        }

        self.request_client
            .fetch_json_with_progress(request, None)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))
    }

    fn find_best_version(
        &self,
        response: &ListProjectsVersionsResponse,
        game_version: &str,
        loader: &Option<String>,
    ) -> Option<ProjectVersionResponse> {
        response
            .iter()
            .filter(|v| {
                let is_right_game_version = v.game_versions.contains(&game_version.to_string());
                let is_right_loader = loader
                    .clone()
                    .map(|loader| v.loaders.contains(&loader))
                    .unwrap_or(true);

                is_right_game_version && is_right_loader
            })
            .max_by(|a, b| {
                // Compare by version type (stability)
                let stability_priority = |version_type: &str| match version_type {
                    "release" => 3,
                    "beta" => 2,
                    "alpha" => 1,
                    _ => 0,
                };

                let a_stability = stability_priority(&a.version_type);
                let b_stability = stability_priority(&b.version_type);

                // Get newest by date if version type equals
                a_stability
                    .cmp(&b_stability)
                    .then(a.date_published.cmp(&b.date_published))
            })
            .cloned()
    }

    pub async fn get_project_version_for_game_version(
        &self,
        project_id: &str,
        game_version: &str,
        loader: &Option<String>,
    ) -> Result<ProjectVersionResponse, InstanceError> {
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
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))?;

        if let Some(version) = self.find_best_version(&response, game_version, loader) {
            Ok(version.clone())
        } else {
            Err(InstanceError::ContentForGameVersionNotFound {
                game_version: game_version.to_owned(),
            })
        }
    }

    pub async fn get_file(&self, url: &str) -> Result<Bytes, InstanceError> {
        let mut request = Request::get(url);
        if let Some(base_headers) = self.base_headers.clone() {
            request = request.with_headers(base_headers);
        }

        self.request_client
            .fetch_bytes_with_progress(request, None)
            .await
            .map_err(|err| InstanceError::ContentDownloadError(err.to_string()))
    }
}
