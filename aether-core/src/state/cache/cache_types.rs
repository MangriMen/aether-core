use crate::state::ContentType;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CachedFile {
    pub hash: String,
    pub project_id: String,
    pub version_id: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CachedFileUpdate {
    pub hash: String,
    pub game_version: String,
    pub loaders: Vec<String>,
    pub update_version_id: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CachedFileHash {
    pub path: String,
    pub size: u64,
    pub hash: String,
    pub content_type: Option<ContentType>,
}
