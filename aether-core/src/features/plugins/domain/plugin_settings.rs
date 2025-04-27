use serde::{Deserialize, Serialize};

use super::PathMapping;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PluginSettings {
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    #[serde(default)]
    pub allowed_paths: Vec<PathMapping>,
}
