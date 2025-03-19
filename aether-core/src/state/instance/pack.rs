use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstancePack {
    pub index: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct InstancePackIndex {
    pub hash_format: String,
    pub files: Vec<InstancePackFile>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct InstancePackFile {
    pub file: String,
    pub hash: String,
    pub alias: Option<String>,
    pub hash_format: Option<String>,
    pub metafile: Option<bool>,
    pub preserve: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct InstanceMetadataFile {
    pub download: Option<InstanceMetadataFileDownload>,
    pub filename: String,
    pub name: String,
    pub option: Option<InstanceMetadataFileOption>,
    pub side: Option<String>,
    pub update: Option<InstanceMetadataFileUpdate>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct InstanceMetadataFileDownload {
    pub hash_format: String,
    pub hash: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct InstanceMetadataFileOption {
    pub optional: bool,
    pub default: Option<bool>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum InstanceMetadataFileUpdate {
    Custom(serde_json::Value),
}

impl InstancePack {
    pub async fn from_path(path: &Path) -> crate::Result<InstancePack> {
        crate::utils::io::read_toml_async(path.join("pack.toml")).await
    }

    pub async fn write_path(&self, path: &Path) -> crate::Result<()> {
        crate::utils::io::write_toml_async(path.join("pack.toml"), self).await
    }
}

impl InstancePackIndex {
    pub async fn from_path(path: &Path) -> crate::Result<InstancePackIndex> {
        InstancePackIndex::from_file(&path.join("index.toml")).await
    }

    pub async fn from_file(file: &Path) -> crate::Result<InstancePackIndex> {
        crate::utils::io::read_toml_async(file).await
    }

    pub async fn write_path(&self, path: &Path) -> crate::Result<()> {
        self.write_file(&path.join("index.toml")).await
    }

    pub async fn write_file(&self, file: &Path) -> crate::Result<()> {
        crate::utils::io::write_toml_async(file, self).await
    }
}
