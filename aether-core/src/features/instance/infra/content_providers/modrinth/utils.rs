use crate::features::instance::{ContentSearchParams, ContentSearchResult};

use super::{File, ProjectSearchResponse, ProjectVersionResponse};

pub fn modrinth_to_content_response(
    request: &ContentSearchParams,
    response: &ProjectSearchResponse,
) -> ContentSearchResult {
    let items = response
        .hits
        .iter()
        .filter_map(|hit| hit.clone().try_into().ok())
        .collect();

    let page = response.offset / response.limit + 1;
    let page_count = (response.total_hits as f64 / response.limit as f64).ceil() as i64;

    ContentSearchResult {
        page,
        page_size: response.offset,
        page_count,
        provider: request.provider.to_owned(),
        items,
    }
}

pub fn get_facet(facet: &str, values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| format!("{}:{}", facet, value))
        .collect::<Vec<String>>()
}

pub fn get_first_file_from_project_version(version: &ProjectVersionResponse) -> Option<File> {
    version
        .files
        .iter()
        .find(|file| file.primary)
        .cloned()
        .or_else(|| version.files.first().cloned())
}
