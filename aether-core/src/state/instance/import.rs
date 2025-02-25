#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportHandler {
    pub pack_type: String,
    pub title: String,
    pub field_label: String,
    pub file_name: String,
    pub file_extensions: Vec<String>,
}
