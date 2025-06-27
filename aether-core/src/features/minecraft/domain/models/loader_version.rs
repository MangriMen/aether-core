use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(untagged, rename_all = "snake_case")]
pub enum LoaderVersionPreference {
    Latest,
    #[default]
    Stable,
    Exact(String),
}
