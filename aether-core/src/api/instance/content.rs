use dashmap::DashMap;
use reqwest::Method;

use crate::{
    event::{emit::emit_instance, InstancePayloadType},
    state::{
        ContentItem, ContentRequest, ContentResponse, ContentType, Instance, InstanceFile,
        LauncherState,
    },
    utils::fetch::fetch_json,
};

use super::get;

pub async fn get_contents(id: &str) -> crate::Result<DashMap<String, InstanceFile>> {
    if let Ok(instance) = get(id).await {
        instance.get_contents().await
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(id.to_string()).as_error())
    }
}

pub async fn remove_content(id: &str, content_path: &str) -> crate::Result<()> {
    Instance::remove_content(id, content_path).await?;
    emit_instance(id, InstancePayloadType::Edited).await?;
    Ok(())
}

pub async fn toggle_disable_content(id: &str, content_path: &str) -> crate::Result<String> {
    let res = Instance::toggle_disable_content(id, content_path).await?;
    emit_instance(id, InstancePayloadType::Edited).await?;
    Ok(res)
}

pub async fn enable_contents<I, D>(id: &str, content_paths: I) -> crate::Result<()>
where
    I: IntoIterator<Item = D>,
    D: AsRef<str>,
{
    Instance::enable_contents(id, content_paths).await?;
    emit_instance(id, InstancePayloadType::Edited).await?;
    Ok(())
}

pub async fn disable_contents<I, D>(id: &str, content_paths: I) -> crate::Result<()>
where
    I: IntoIterator<Item = D>,
    D: AsRef<str>,
{
    Instance::disable_contents(id, content_paths).await?;
    emit_instance(id, InstancePayloadType::Edited).await?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ModrinthContentResponse {
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
struct QueryParams {
    index: &'static str,
    offset: i64,
    limit: i64,
    facets: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
}

async fn get_content_by_modrinth(payload: &ContentRequest) -> crate::Result<ContentResponse> {
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

    let base_url = "https://staging-api.modrinth.com/v2/search";
    let query_params = QueryParams {
        index: "relevance",
        offset: (payload.page - 1) * payload.page_size,
        limit: payload.page_size,
        facets: format!("[{}]", facets.join(",")),
        query: payload.query.clone(),
    };

    let query_string = serde_qs::to_string(&query_params).unwrap();
    let url = format!("{}?{}", base_url, query_string);

    let response = fetch_json::<ModrinthContentResponse>(
        Method::GET,
        &url,
        Some(headers),
        None,
        None,
        &state.api_semaphore,
    )
    .await;

    match response {
        Ok(response) => {
            let page = response.offset / response.limit + 1;

            let items = response
                .hits
                .iter()
                .map(|hit| ContentItem {
                    id: hit.slug.clone(),
                    name: hit.title.clone(),
                    description: Some(hit.description.clone()),
                    content_type: ContentType::from_string(&hit.project_type)
                        .unwrap_or(ContentType::Mod),
                    url: format!("https://modrinth.com/mod/{}", hit.slug),
                    author: hit.author.clone(),
                })
                .collect();

            Ok(ContentResponse {
                page,
                page_size: response.offset,
                page_count: response.total_hits / response.limit,
                provider: payload.provider.to_owned(),
                items,
            })
        }
        Err(e) => {
            println!("{:?}", e);
            Err(e)
        }
    }
}

pub async fn get_content_by_provider(payload: &ContentRequest) -> crate::Result<ContentResponse> {
    match payload.provider.as_str() {
        "modrinth" => get_content_by_modrinth(payload).await,
        _ => {
            let items: Vec<ContentItem> = (0..payload.page_size)
                .map(|i| ContentItem {
                    id: format!("mock-content-item-{}", i),
                    name: format!("Mock content item {}", i),
                    description: Some(format!("This is a mock content item {}", i)),
                    content_type: payload.content_type,
                    url: format!("https://example.com/content-item-{}", i),
                    author: "Mock author".to_string(),
                })
                .collect();

            Ok(ContentResponse {
                page: payload.page,
                page_size: payload.page_size,
                page_count: 1,
                provider: payload.provider.to_owned(),
                items,
            })
        }
    }
}
