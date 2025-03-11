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
    pub icon_url: String,
    pub versions: Vec<String>,
    pub latest_version: String,
    pub provider_data: Option<serde_json::Value>,
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
