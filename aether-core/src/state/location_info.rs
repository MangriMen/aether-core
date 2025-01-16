use std::path::PathBuf;

pub const METADATA_FOLDER_NAME: &str = ".minecraft";
pub const CACHE_FOLDER_NAME: &str = "cache";
pub const INSTANCES_FOLDER_NAME: &str = "instances";
pub const PLUGIN_FOLDER_NAME: &str = "plugins";

#[derive(Debug)]
pub struct LocationInfo {
    pub settings_dir: PathBuf, // Base settings directory - app database

    /// Changeable through settings
    pub config_dir: PathBuf, // Config directory - instances, minecraft files, etc.

    pub plugins_dir: PathBuf,
}

impl LocationInfo {
    /// Get the Minecraft instance metadata directory
    #[inline]
    pub fn metadata_dir(&self) -> PathBuf {
        self.config_dir.join(METADATA_FOLDER_NAME)
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

    /// Get the Minecraft libraries metadata directory
    #[inline]
    pub fn libraries_dir(&self) -> PathBuf {
        self.metadata_dir().join("libraries")
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

    /// Get the Minecraft legacy assets metadata directory
    #[inline]
    pub fn natives_dir(&self) -> PathBuf {
        self.metadata_dir().join("natives")
    }

    /// Get the natives directory for a version of Minecraft
    #[inline]
    pub fn version_natives_dir(&self, version: &str) -> PathBuf {
        self.natives_dir().join(version)
    }

    /// Get the instances directory for created instances
    #[inline]
    pub fn instances_dir(&self) -> PathBuf {
        self.config_dir.join(INSTANCES_FOLDER_NAME)
    }

    /// Get the directory for a specific instance
    #[inline]
    pub fn instance_dir(&self, id: &str) -> PathBuf {
        self.instances_dir().join(id)
    }

    /// Get the cache directory
    #[inline]
    pub fn cache_dir(&self) -> PathBuf {
        self.config_dir.join(CACHE_FOLDER_NAME)
    }

    /// Get the Minecraft java versions metadata directory
    #[inline]
    pub fn java_dir(&self) -> PathBuf {
        self.cache_dir().join("java")
    }

    /// Get the plugins directory
    #[inline]
    pub fn plugins_dir(&self) -> PathBuf {
        self.config_dir.join(PLUGIN_FOLDER_NAME)
    }

    /// Get the directory for a specific plugin
    #[inline]
    pub fn plugin_dir(&self, id: &str) -> PathBuf {
        self.plugins_dir().join(id)
    }

    /// Get the directory for a specific plugin inside an instance
    #[inline]
    pub fn instance_plugin_dir(&self, id: &str, plugin_id: &str) -> PathBuf {
        self.instance_dir(id)
            .join(PLUGIN_FOLDER_NAME)
            .join(plugin_id)
    }
}
