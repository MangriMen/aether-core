use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::ContentTypeDto;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContentFileDto {
    pub content_path: String,
    pub content_type: ContentTypeDto,
    pub disabled: bool,
    pub filename: String,
    pub hash: String,
    pub instance_relative_path: String,
    pub name: Option<String>,
    pub size: u64,
    pub update: Option<HashMap<String, serde_json::Value>>,
}
