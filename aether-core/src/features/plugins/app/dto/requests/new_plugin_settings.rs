use serde::{Deserialize, Serialize};

use crate::features::plugins::PathMapping;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EditPluginSettings {
    pub allowed_hosts: Option<Vec<String>>,
    pub allowed_paths: Option<Vec<PathMapping>>,
}
