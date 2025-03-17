use reqwest::Method;

use crate::{
    state::{
        ContentItem, ContentRequest, ContentResponse, ContentType, InstallContentPayload,
        LauncherState,
    },
    utils::{
        fetch::{fetch_advanced, fetch_json, FetchSemaphore},
        io::write_async,
    },
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ModrinthContentSearchResponse {
    hits: Vec<Hit>,
    offset: i64,
    limit: i64,
    total_hits: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Hit {
    project_id: String,
    project_type: String,
    slug: String,
    author: String,
    title: String,
    description: String,
    categories: Vec<String>,
    display_categories: Vec<String>,
    versions: Vec<String>,
    downloads: i64,
    follows: i64,
    icon_url: String,
    date_created: String,
    date_modified: String,
    latest_version: String,
    license: String,
    client_side: String,
    server_side: String,
    gallery: Vec<String>,
    featured_gallery: Option<String>,
    color: Option<i64>,
}

#[derive(serde::Serialize)]
struct ModrinthQueryParams {
    index: &'static str,
    offset: i64,
    limit: i64,
    facets: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
}

async fn get_raw_content(payload: &ContentRequest) -> crate::Result<ModrinthContentSearchResponse> {
    let state = LauncherState::get().await?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "User-Agent",
        "MangriMen/aether".to_string().parse().unwrap(),
    );

    let mut facets: Vec<String> = Vec::new();

    facets.push(format!(
        "[\"project_type:{}\"]",
        payload.content_type.get_name()
    ));

    // let base_url = "https://staging-api.modrinth.com/v2/search";
    let base_url = "https://api.modrinth.com/v2/search";
    let query_params = ModrinthQueryParams {
        index: "relevance",
        offset: (payload.page - 1) * payload.page_size,
        limit: payload.page_size,
        facets: format!("[{}]", facets.join(",")),
        query: payload.query.clone(),
    };

    let query_string = serde_qs::to_string(&query_params).unwrap();
    let url = format!("{}?{}", base_url, query_string);

    fetch_json::<ModrinthContentSearchResponse>(
        Method::GET,
        &url,
        Some(headers),
        None,
        None,
        &state.api_semaphore,
    )
    .await
}

fn modrinth_to_content_response(
    request: &ContentRequest,
    response: &ModrinthContentSearchResponse,
) -> ContentResponse {
    let page = response.offset / response.limit + 1;

    let items = response
        .hits
        .iter()
        .map(|hit| ContentItem {
            id: hit.slug.clone(),
            name: hit.title.clone(),
            description: Some(hit.description.clone()),
            content_type: ContentType::from_string(&hit.project_type).unwrap_or(ContentType::Mod),
            url: format!("https://modrinth.com/mod/{}", hit.slug),
            author: hit.author.clone(),
            icon_url: hit.icon_url.clone(),
            versions: hit.versions.clone(),
            provider_data: Some(
                serde_json::to_value(&ModrinthProviderData {
                    project_id: hit.project_id.to_owned(),
                    latest_version: hit.latest_version.to_owned(),
                })
                .unwrap(),
            ),
        })
        .collect();

    ContentResponse {
        page,
        page_size: response.offset,
        page_count: response.total_hits / response.limit,
        provider: request.provider.to_owned(),
        items,
    }
}

pub async fn get_content(payload: &ContentRequest) -> crate::Result<ContentResponse> {
    let response = get_raw_content(payload).await?;

    Ok(modrinth_to_content_response(payload, &response))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ModrinthProjectVersionResponse {
    game_versions: Vec<String>,
    loaders: Vec<String>,
    id: String,
    project_id: String,
    author_id: String,
    featured: bool,
    name: String,
    version_number: String,
    changelog: String,
    changelog_url: Option<serde_json::Value>,
    date_published: String,
    downloads: i64,
    version_type: String,
    status: String,
    requested_status: Option<serde_json::Value>,
    files: Vec<File>,
    dependencies: Vec<Option<serde_json::Value>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct File {
    hashes: Hashes,
    url: String,
    filename: String,
    primary: bool,
    size: i64,
    file_type: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Hashes {
    sha1: String,
    sha512: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct ModrinthProviderData {
    project_id: String,
    latest_version: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ModrinthProjectResponse {
    client_side: String,
    server_side: String,
    game_versions: Vec<String>,
    id: String,
    slug: String,
    project_type: String,
    team: String,
    organization: Option<serde_json::Value>,
    title: String,
    description: String,
    body: String,
    body_url: Option<serde_json::Value>,
    published: String,
    updated: String,
    approved: String,
    queued: Option<serde_json::Value>,
    status: String,
    requested_status: Option<serde_json::Value>,
    moderator_message: Option<serde_json::Value>,
    license: License,
    downloads: i64,
    followers: i64,
    categories: Vec<String>,
    additional_categories: Vec<Option<serde_json::Value>>,
    loaders: Vec<String>,
    versions: Vec<String>,
    icon_url: String,
    issues_url: String,
    source_url: String,
    wiki_url: String,
    discord_url: String,
    donation_urls: Vec<Option<serde_json::Value>>,
    gallery: Vec<Option<serde_json::Value>>,
    color: i64,
    thread_id: String,
    monetization_status: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct License {
    id: String,
    name: String,
    url: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ModrinthListProjectsVersionsParams {
    loaders: Option<Vec<String>>,
    game_versions: Vec<String>,
}

pub type ModrinthListProjectsVersionsResponse = Vec<ModrinthProjectVersionResponse>;

pub async fn get_file_data_for_game_version(
    project_id: &str,
    game_version: &str,
    loader: &Option<String>,
    headers: reqwest::header::HeaderMap,
    api_semaphore: &FetchSemaphore,
) -> crate::Result<File> {
    let mut params = ModrinthListProjectsVersionsParams {
        loaders: None,
        game_versions: vec![game_version.to_string()],
    };

    if let Some(loader) = loader {
        params.loaders = Some(vec![loader.to_string()]);
    }

    let query_string = serde_qs::to_string(&params).unwrap();
    let url = format!(
        "https://api.modrinth.com/v2/project/{}/version?{}",
        project_id, query_string
    );

    let response = fetch_json::<ModrinthListProjectsVersionsResponse>(
        Method::GET,
        &url,
        Some(headers),
        None,
        None,
        api_semaphore,
    )
    .await?;

    let version_index = response.iter().find(|project_versions| {
        project_versions
            .game_versions
            .iter()
            .any(|version| version == game_version)
    });

    if let Some(version_index) = version_index {
        let file = version_index.files.iter().find(|file| file.primary);

        if let Some(file) = file {
            Ok(file.clone())
        } else {
            Ok({
                version_index
                    .files
                    .first()
                    .ok_or(crate::ErrorKind::NoValueFor(format!(
                        "Content for version \"{}\" not found",
                        game_version
                    )))?
                    .clone()
            })
        }
    } else {
        Err(crate::ErrorKind::NoValueFor(format!(
            "Content for version \"{}\" not found",
            game_version
        ))
        .as_error())
    }
}

pub async fn get_file_data_for_content_version(
    content_version: &str,
    headers: reqwest::header::HeaderMap,
    api_semaphore: &FetchSemaphore,
) -> crate::Result<File> {
    let url = format!("https://api.modrinth.com/v2/version/{}", content_version);

    let response = fetch_json::<ModrinthProjectVersionResponse>(
        Method::GET,
        &url,
        Some(headers.clone()),
        None,
        None,
        api_semaphore,
    )
    .await?;

    Ok(response.files[0].clone())
}

pub async fn install_content(id: &str, payload: &InstallContentPayload) -> crate::Result<()> {
    let state = LauncherState::get().await?;

    if let Some(provider_data) = &payload.provider_data {
        let provider_data = serde_json::from_value::<ModrinthProviderData>(provider_data.clone())?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "User-Agent",
            "MangriMen/aether".to_string().parse().unwrap(),
        );

        let file_data = if let Some(content_version) = payload.content_version.clone() {
            get_file_data_for_content_version(
                &content_version,
                headers.clone(),
                &state.api_semaphore,
            )
            .await?
        } else {
            get_file_data_for_game_version(
                &provider_data.project_id,
                &payload.game_version,
                &payload.loader,
                headers.clone(),
                &state.api_semaphore,
            )
            .await?
        };

        let file = fetch_advanced(
            Method::GET,
            &file_data.url,
            Some(headers),
            None,
            None,
            &state.fetch_semaphore,
            None,
        )
        .await?;

        let instance_folder = crate::api::instance::get_dir(id).await?;

        write_async(
            instance_folder
                .join(payload.content_type.get_folder())
                .join(&file_data.filename),
            file,
        )
        .await?;

        Ok(())
    } else {
        Err(crate::ErrorKind::NoValueFor("Not found provider data".to_string()).as_error())
    }
}
