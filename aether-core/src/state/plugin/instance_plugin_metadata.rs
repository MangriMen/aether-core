#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct InstancePluginMetadata {
    pub is_loaded: bool,
}
