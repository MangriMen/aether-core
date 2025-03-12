use reqwest::Method;
use serde_json::json;

use crate::{
    state::{ContentItem, ContentRequest, ContentResponse, ContentType, LauncherState},
    utils::{
        fetch::{fetch_advanced, fetch_json},
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
            latest_version: hit.latest_version.clone(),
            provider_data: Some(json!({
                "project_id": hit.project_id
            })),
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
pub struct ModrinthContentVersionResponse {
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

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct File {
    hashes: Hashes,
    url: String,
    filename: String,
    primary: bool,
    size: i64,
    file_type: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Hashes {
    sha1: String,
    sha512: String,
}

pub async fn install_content(id: &str, payload: &ContentItem) -> crate::Result<()> {
    let state = LauncherState::get().await?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "User-Agent",
        "MangriMen/aether".to_string().parse().unwrap(),
    );

    let url = format!(
        "https://api.modrinth.com/v2/version/{}",
        payload.latest_version
    );

    let response = fetch_json::<ModrinthContentVersionResponse>(
        Method::GET,
        &url,
        Some(headers.clone()),
        None,
        None,
        &state.api_semaphore,
    )
    .await?;

    let file_data = &response.files[0];

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
}
