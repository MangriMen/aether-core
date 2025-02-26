use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct PluginSettings {
    pub allowed_hosts: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<(String, PathBuf)>>,
}

impl PluginSettings {
    pub async fn from_path(path: &PathBuf) -> crate::Result<Option<PluginSettings>> {
        if !path.exists() {
            return Ok(None);
        }

        crate::utils::io::read_toml_async(path).await
    }
}
