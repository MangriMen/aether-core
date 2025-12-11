use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoaderVersionPreference {
    Latest,
    #[default]
    Stable,
    #[serde(untagged)]
    Exact(String),
}
