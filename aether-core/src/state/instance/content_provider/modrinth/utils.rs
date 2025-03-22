use crate::state::{ContentItem, ContentRequest, ContentResponse, ContentType};

use super::{Hit, ModrinthProviderData, SearchProjectsResponse};

pub fn hit_to_content_item(hit: &Hit) -> crate::Result<ContentItem> {
    let content_type = ContentType::from_string(&hit.project_type).unwrap_or(ContentType::Mod);
    let url = format!("https://modrinth.com/mod/{}", hit.slug);

    let provider_data = Some(serde_json::to_value(&ModrinthProviderData {
        project_id: hit.project_id.to_owned(),
    })?);

    Ok(ContentItem {
        id: hit.slug.clone(),
        name: hit.title.clone(),
        description: Some(hit.description.clone()),
        content_type,
        url,
        author: hit.author.clone(),
        icon_url: hit.icon_url.clone(),
        versions: hit.versions.clone(),
        provider_data,
    })
}

pub fn modrinth_to_content_response(
    request: &ContentRequest,
    response: &SearchProjectsResponse,
) -> ContentResponse {
    let items = response
        .hits
        .iter()
        .filter_map(|hit| hit_to_content_item(hit).ok())
        .collect();

    let page = response.offset / response.limit + 1;
    let page_count = (response.total_hits as f64 / response.limit as f64).ceil() as i64;

    ContentResponse {
        page,
        page_size: response.offset,
        page_count,
        provider: request.provider.to_owned(),
        items,
    }
}
