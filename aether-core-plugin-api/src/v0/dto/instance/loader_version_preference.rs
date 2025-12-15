use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoaderVersionPreferenceDto {
    Latest,
    #[default]
    Stable,
    #[serde(untagged)]
    Exact(String),
}
