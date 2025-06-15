use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::ManifestError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginManifest {
    pub metadata: PluginMetadata,
    pub runtime: RuntimeConfig,
    pub load: LoadConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: semver::Version,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    #[serde(default)]
    pub allowed_paths: Vec<PathMapping>,
}

pub type PathMapping = (String, PathBuf); // (path on disk, plugin path)

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum LoadConfigType {
    Extism,
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoadConfig {
    Extism {
        file: PathBuf,
        #[serde(default)]
        memory_limit: Option<usize>,
    },
    Native {
        lib_path: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiConfig {
    pub version: semver::VersionReq,
    #[serde(default)]
    pub features: Vec<String>,
}

impl PluginManifest {
    pub fn validate(
        &self,
        api_version: semver::Version,
        base_dir: &Path,
    ) -> Result<(), ManifestError> {
        self.runtime.validate()?;
        self.load.validate(base_dir)?;
        self.api.validate(api_version)?;

        Ok(())
    }
}

impl RuntimeConfig {
    pub fn validate(&self) -> Result<(), ManifestError> {
        for mapping in &self.allowed_paths {
            if Path::new(&mapping.0).is_absolute() {
                return Err(ManifestError::InvalidPathMapping);
            }
        }

        Ok(())
    }
}

impl LoadConfig {
    pub fn validate(&self, base_dir: &Path) -> Result<(), ManifestError> {
        match &self {
            Self::Extism { file, .. } => {
                let full_path = base_dir.join(file);
                if !full_path.exists() {
                    return Err(ManifestError::InvalidFilePath {
                        path: file.to_path_buf(),
                    });
                }
            }
            Self::Native { lib_path } => {
                let full_path = base_dir.join(lib_path);
                if !full_path.exists() {
                    return Err(ManifestError::InvalidFilePath {
                        path: lib_path.to_path_buf(),
                    });
                }
            }
        }

        Ok(())
    }
}

impl ApiConfig {
    pub fn validate(&self, api_version: semver::Version) -> Result<(), ManifestError> {
        if !self.version.matches(&api_version) {
            return Err(ManifestError::UnsupportedApi);
        }

        Ok(())
    }
}

impl From<&LoadConfig> for LoadConfigType {
    fn from(config: &LoadConfig) -> Self {
        match config {
            LoadConfig::Extism { .. } => LoadConfigType::Extism,
            LoadConfig::Native { .. } => LoadConfigType::Native,
        }
    }
}
