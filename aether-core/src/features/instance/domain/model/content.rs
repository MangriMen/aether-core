use serde::{Deserialize, Serialize};

use crate::features::minecraft::ModLoader;

use super::ContentType;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchParams {
    pub content_type: ContentType,
    pub provider: String,
    pub page: i64,
    pub page_size: i64,
    pub query: Option<String>,
    pub game_versions: Option<Vec<String>>,
    pub loader: ModLoader,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentSearchResult {
    pub page: i64,
    pub page_size: i64,
    pub page_count: i64,
    pub provider: String,
    pub items: Vec<ContentItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentInstallParams {
    pub game_version: String,
    pub loader: Option<String>,
    pub content_type: ContentType,
    pub content_version: Option<String>,
    pub provider: String,
    pub provider_data: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: String,
    pub url: String,
    pub icon_url: String,
    pub versions: Vec<String>,
    pub content_type: ContentType,
    pub provider_data: Option<serde_json::Value>,
}
