use std::collections::HashMap;

use extism::ToBytes;
use extism_convert::{encoding, Json};
use register_schema_derive::RegisterSchema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ContentType;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, RegisterSchema, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct ContentFile {
    pub content_path: String,
    pub content_type: ContentType,
    pub disabled: bool,
    pub filename: String,
    pub hash: String,
    pub instance_relative_path: String,
    pub name: Option<String>,
    pub size: u64,
    pub update: Option<HashMap<String, serde_json::Value>>,
}
