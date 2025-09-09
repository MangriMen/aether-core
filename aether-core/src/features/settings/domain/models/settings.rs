use std::{collections::HashSet, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub launcher_dir: PathBuf,
    pub metadata_dir: PathBuf,

    pub max_concurrent_downloads: usize,

    pub enabled_plugins: HashSet<String>,
}
