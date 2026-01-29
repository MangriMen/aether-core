use serde::{de::Error, Deserialize, Serialize};

use crate::features::{
    instance::{ContentItem, ContentSearchParams, ContentType},
    minecraft::ModLoader,
};

use super::get_facet;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthProviderData {
    pub project_id: String,
}

#[derive(Serialize)]
pub struct ProjectSearchParams {
    pub index: &'static str,
    pub offset: i64,
    pub limit: i64,
    pub facets: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectSearchResponse {
    pub hits: Vec<Hit>,
    pub offset: i64,
    pub limit: i64,
    pub total_hits: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct File {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hashes {
    pub sha1: String,
    pub sha512: String,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct ModrinthProjectResponse {
//     pub client_side: String,
//     pub server_side: String,
//     pub game_versions: Vec<String>,
//     pub id: String,
//     pub slug: String,
//     pub project_type: String,
//     pub team: String,
//     pub organization: Option<serde_json::Value>,
//     pub title: String,
//     pub description: String,
//     pub body: String,
//     pub body_url: Option<serde_json::Value>,
//     pub published: String,
//     pub updated: String,
//     pub approved: String,
//     pub queued: Option<serde_json::Value>,
//     pub status: String,
//     pub requested_status: Option<serde_json::Value>,
//     pub moderator_message: Option<serde_json::Value>,
//     pub license: License,
//     pub downloads: i64,
//     pub followers: i64,
//     pub categories: Vec<String>,
//     pub additional_categories: Vec<Option<serde_json::Value>>,
//     pub loaders: Vec<String>,
//     pub versions: Vec<String>,
//     pub icon_url: String,
//     pub issues_url: String,
//     pub source_url: String,
//     pub wiki_url: String,
//     pub discord_url: String,
//     pub donation_urls: Vec<Option<serde_json::Value>>,
//     pub gallery: Vec<Option<serde_json::Value>>,
//     pub color: i64,
//     pub thread_id: String,
//     pub monetization_status: String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct License {
//     pub id: String,
//     pub name: String,
//     pub url: Option<serde_json::Value>,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct ListProjectVersionsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loaders: Option<Vec<String>>,
    pub game_versions: Vec<String>,
}

pub type ListProjectsVersionsResponse = Vec<ProjectVersionResponse>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModrinthUpdateData {
    pub project_id: String,
    pub version: String,
}

impl TryFrom<ContentSearchParams> for ProjectSearchParams {
    type Error = serde_json::Error;

    fn try_from(value: ContentSearchParams) -> Result<Self, Self::Error> {
        let categories = match value.content_type {
            ContentType::Mod => match value.loader {
                ModLoader::Vanilla => None,
                loader => Some(get_facet("categories", &[loader.as_str().to_owned()])),
            },
            _ => None,
        };

        let versions = value
            .game_versions
            .as_ref()
            .map(|game_versions| get_facet("versions", game_versions));

        let project_types = get_facet("project_type", &[value.content_type.get_name().to_owned()]);

        let mut facets: Vec<Vec<String>> = Vec::new();

        if let Some(categories) = categories {
            facets.push(categories);
        }

        if let Some(versions) = versions {
            facets.push(versions);
        }

        facets.push(project_types);

        let facets_string = serde_json::to_string(&facets)?;

        Ok(Self {
            index: "relevance",
            offset: (value.page - 1) * value.page_size,
            limit: value.page_size,
            facets: facets_string,
            query: value.query.clone(),
        })
    }
}

impl TryFrom<Hit> for ContentItem {
    type Error = serde_json::Error;

    fn try_from(value: Hit) -> Result<Self, Self::Error> {
        let content_type = ContentType::from_string(&value.project_type).ok_or_else(|| {
            serde_json::Error::unknown_field(
                "project_type",
                &["mod", "datapack", "resourcepack", "shader"],
            )
        })?;
        let url = format!("https://modrinth.com/mod/{}", value.slug);

        let provider_data = Some(serde_json::to_value(&ModrinthProviderData {
            project_id: value.project_id.to_owned(),
        })?);

        Ok(Self {
            id: value.slug.clone(),
            name: value.title.clone(),
            description: Some(value.description.clone()),
            content_type,
            url,
            author: value.author.clone(),
            icon_url: value.icon_url.clone(),
            versions: value.versions.clone(),
            provider_data,
        })
    }
}
