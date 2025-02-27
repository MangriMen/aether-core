use std::path::PathBuf;

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PluginMetadata {
    pub plugin: PluginInfo,
    pub wasm: WasmInfo,
    pub config: ConfigInfo,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct WasmInfo {
    pub file: String,
    pub allowed_hosts: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<(String, PathBuf)>>,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ConfigInfo {
    pub api_version: String,
}
