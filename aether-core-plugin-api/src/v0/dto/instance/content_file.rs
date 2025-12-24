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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v0::ContentTypeDto;

    #[test]
    fn test_content_file_camel_case() {
        let file = ContentFileDto {
            content_path: "path".into(),
            content_type: ContentTypeDto::Mod,
            disabled: false,
            filename: "mod.jar".into(),
            hash: "123".into(),
            instance_relative_path: "mods/".into(),
            name: None,
            size: 1024,
            update: None,
        };
        let json = serde_json::to_string(&file).unwrap();
        // Проверяем, что поле instance_relative_path превратилось в camelCase
        assert!(json.contains(r#""instanceRelativePath""#));
    }
}
