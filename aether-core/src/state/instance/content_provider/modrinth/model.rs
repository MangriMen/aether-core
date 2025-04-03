pub const MODRINTH_API_URL: &str = "https://api.modrinth.com/v2";

lazy_static::lazy_static! {
    pub static ref DEFAULT_HEADERS: reqwest::header::HeaderMap = {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "User-Agent",
            "MangriMen/aether".to_string().parse().unwrap(),
        );
        headers
    };
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ModrinthProviderData {
    pub project_id: String,
}

#[derive(serde::Serialize)]
pub struct SearchProjectsParams {
    pub index: &'static str,
    pub offset: i64,
    pub limit: i64,
    pub facets: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SearchProjectsResponse {
    pub hits: Vec<Hit>,
    pub offset: i64,
    pub limit: i64,
    pub total_hits: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Hit {
    pub project_id: String,
    pub project_type: String,
    pub slug: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub display_categories: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: i64,
    pub follows: i64,
    pub icon_url: String,
    pub date_created: String,
    pub date_modified: String,
    pub latest_version: String,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
    pub color: Option<i64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectVersionResponse {
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub id: String,
    pub project_id: String,
    pub author_id: String,
    pub featured: bool,
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub changelog_url: Option<serde_json::Value>,
    pub date_published: String,
    pub downloads: i64,
    pub version_type: String,
    pub status: String,
    pub requested_status: Option<serde_json::Value>,
    pub files: Vec<File>,
    pub dependencies: Vec<Option<serde_json::Value>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct File {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Hashes {
    pub sha1: String,
    pub sha512: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ModrinthProjectResponse {
    pub client_side: String,
    pub server_side: String,
    pub game_versions: Vec<String>,
    pub id: String,
    pub slug: String,
    pub project_type: String,
    pub team: String,
    pub organization: Option<serde_json::Value>,
    pub title: String,
    pub description: String,
    pub body: String,
    pub body_url: Option<serde_json::Value>,
    pub published: String,
    pub updated: String,
    pub approved: String,
    pub queued: Option<serde_json::Value>,
    pub status: String,
    pub requested_status: Option<serde_json::Value>,
    pub moderator_message: Option<serde_json::Value>,
    pub license: License,
    pub downloads: i64,
    pub followers: i64,
    pub categories: Vec<String>,
    pub additional_categories: Vec<Option<serde_json::Value>>,
    pub loaders: Vec<String>,
    pub versions: Vec<String>,
    pub icon_url: String,
    pub issues_url: String,
    pub source_url: String,
    pub wiki_url: String,
    pub discord_url: String,
    pub donation_urls: Vec<Option<serde_json::Value>>,
    pub gallery: Vec<Option<serde_json::Value>>,
    pub color: i64,
    pub thread_id: String,
    pub monetization_status: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<serde_json::Value>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ListProjectVersionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loaders: Option<Vec<String>>,
    pub game_versions: Vec<String>,
}

pub type ListProjectsVersionsResponse = Vec<ProjectVersionResponse>;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ModrinthUpdateData {
    pub project_id: String,
    pub version: String,
}
