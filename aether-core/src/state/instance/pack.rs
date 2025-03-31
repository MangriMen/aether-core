use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::state::CONTENT_METADATA_FILE_NAME;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ContentMetadata {
    pub files: Vec<ContentMetadataEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ContentMetadataEntry {
    pub file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ContentMetadataFile {
    pub file_name: String,
    pub name: Option<String>,
    pub hash: String,
    pub download: Option<ContentMetadataFileDownload>,
    pub option: Option<ContentMetadataFileOption>,
    pub side: Option<String>,
    pub update_provider: Option<String>,
    pub update: Option<HashMap<String, toml::Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ContentMetadataFileDownload {
    pub hash: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ContentMetadataFileOption {
    pub optional: bool,
    pub default: Option<bool>,
    pub description: Option<String>,
}

impl ContentMetadata {
    pub async fn from_path(path: &Path) -> crate::Result<ContentMetadata> {
        ContentMetadata::from_file(&path.join(CONTENT_METADATA_FILE_NAME)).await
    }

    pub async fn from_file(file: &Path) -> crate::Result<ContentMetadata> {
        crate::utils::io::read_toml_async(file).await
    }

    pub async fn write_path(&self, path: &Path) -> crate::Result<()> {
        self.write_file(&path.join(CONTENT_METADATA_FILE_NAME))
            .await
    }

    pub async fn write_file(&self, file: &Path) -> crate::Result<()> {
        crate::utils::io::write_toml_async(file, self).await
    }
}
