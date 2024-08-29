use std::path::PathBuf;

pub const METADATA_FOLDER_NAME: &str = ".minecraft";

#[derive(Debug)]
pub struct LocationInfo {
    pub settings_dir: PathBuf, // Base settings directory - app database

    /// Changeable through settings
    pub config_dir: PathBuf, // Config directory - instances, minecraft files, etc.
}

impl LocationInfo {
    /// Get the Minecraft instance metadata directory
    #[inline]
    pub fn metadata_dir(&self) -> PathBuf {
        self.config_dir.join(METADATA_FOLDER_NAME)
    }

    /// Get the Minecraft java versions metadata directory
    #[inline]
    pub fn java_versions_dir(&self) -> PathBuf {
        self.metadata_dir().join("java_versions")
    }

    /// Get the Minecraft versions metadata directory
    #[inline]
    pub fn versions_dir(&self) -> PathBuf {
        self.metadata_dir().join("versions")
    }

    /// Get the metadata directory for a given version
    #[inline]
    pub fn version_dir(&self, version: &str) -> PathBuf {
        self.versions_dir().join(version)
    }

    /// Get the Minecraft assets metadata directory
    #[inline]
    pub fn assets_dir(&self) -> PathBuf {
        self.metadata_dir().join("assets")
    }

    /// Get the assets index directory
    #[inline]
    pub fn assets_index_dir(&self) -> PathBuf {
        self.assets_dir().join("indexes")
    }

    /// Get the assets objects directory
    #[inline]
    pub fn objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    /// Get the directory for a specific object
    #[inline]
    pub fn object_dir(&self, hash: &str) -> PathBuf {
        self.objects_dir().join(&hash[..2]).join(hash)
    }

    /// Get the Minecraft legacy assets metadata directory
    #[inline]
    pub fn legacy_assets_dir(&self) -> PathBuf {
        self.metadata_dir().join("resources")
    }
}
