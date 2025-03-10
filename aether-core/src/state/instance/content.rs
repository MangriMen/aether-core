use super::ContentType;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: String,
    pub content_type: ContentType,
    pub url: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentRequest {
    pub content_type: ContentType,
    pub provider: String,
    pub page: i64,
    pub page_size: i64,
    pub query: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentResponse {
    pub page: i64,
    pub page_size: i64,
    pub page_count: i64,
    pub provider: String,
    pub items: Vec<ContentItem>,
}
