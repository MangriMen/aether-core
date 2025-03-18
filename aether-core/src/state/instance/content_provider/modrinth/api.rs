use reqwest::Method;

use crate::{
    state::{ContentRequest, ContentType, LauncherState, ModLoader},
    utils::fetch::{fetch_json, FetchSemaphore},
};

use super::{
    File, ListProjectVersionsParams, ListProjectsVersionsResponse, ProjectVersionResponse,
    SearchProjectsParams, SearchProjectsResponse, DEFAULT_HEADERS, MODRINTH_API_URL,
};

pub fn get_facet(facet: &str, values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| format!("{}:{}", facet, value))
        .collect::<Vec<String>>()
}

pub async fn search_projects(payload: &ContentRequest) -> crate::Result<SearchProjectsResponse> {
    let state = LauncherState::get().await?;

    let categories = match payload.content_type {
        ContentType::Mod => match payload.loader {
            ModLoader::Vanilla => None,
            loader => Some(get_facet("categories", &[loader.as_meta_str().to_owned()])),
        },
        _ => None,
    };

    let versions = payload
        .game_versions
        .as_ref()
        .map(|game_versions| get_facet("versions", game_versions));

    let project_types = get_facet(
        "project_type",
        &[payload.content_type.get_name().to_owned()],
    );

    let mut facets: Vec<Vec<String>> = Vec::new();

    if let Some(categories) = categories {
        facets.push(categories);
    }

    if let Some(versions) = versions {
        facets.push(versions);
    }

    facets.push(project_types);

    let facets_string = serde_json::to_string(&facets)?;

    let query_params = SearchProjectsParams {
        index: "relevance",
        offset: (payload.page - 1) * payload.page_size,
        limit: payload.page_size,
        facets: facets_string,
        query: payload.query.clone(),
    };

    let query_string = serde_qs::to_string(&query_params).unwrap();
    let url = format!("{}/search?{}", MODRINTH_API_URL, query_string);

    fetch_json::<SearchProjectsResponse>(
        Method::GET,
        &url,
        Some(DEFAULT_HEADERS.clone()),
        None,
        None,
        &state.api_semaphore,
    )
    .await
}

pub async fn get_file_for_game_version(
    project_id: &str,
    game_version: &str,
    loader: &Option<String>,
    api_semaphore: &FetchSemaphore,
) -> crate::Result<File> {
    let params = ListProjectVersionsParams {
        loaders: loader.as_ref().map(|l| vec![l.clone()]),
        game_versions: vec![game_version.to_string()],
    };

    let query_string = serde_qs::to_string(&params).unwrap();
    let url = format!(
        "{}/project/{}/version?{}",
        MODRINTH_API_URL, project_id, query_string
    );

    let response = fetch_json::<ListProjectsVersionsResponse>(
        Method::GET,
        &url,
        Some(DEFAULT_HEADERS.clone()),
        None,
        None,
        api_semaphore,
    )
    .await?;

    let version = response
        .iter()
        .find(|v| v.game_versions.contains(&game_version.to_string()));

    version
        .and_then(|v| {
            v.files
                .iter()
                .find(|file| file.primary)
                .cloned()
                .or_else(|| v.files.first().cloned())
        })
        .ok_or_else(|| {
            crate::ErrorKind::NoValueFor(format!(
                "Content for version \"{}\" not found",
                game_version
            ))
            .as_error()
        })
}

pub async fn get_file_for_project_version(
    project_version: &str,
    api_semaphore: &FetchSemaphore,
) -> crate::Result<File> {
    let url = format!("{}/version/{}", MODRINTH_API_URL, project_version);

    let response = fetch_json::<ProjectVersionResponse>(
        Method::GET,
        &url,
        Some(DEFAULT_HEADERS.clone()),
        None,
        None,
        api_semaphore,
    )
    .await?;

    response.files.first().cloned().ok_or_else(|| {
        crate::ErrorKind::NoValueFor(format!("Content version \"{}\" not found", project_version))
            .as_error()
    })
}
