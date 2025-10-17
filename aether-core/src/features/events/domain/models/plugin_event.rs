use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginEvent {
    pub event: PluginEventType,
    pub plugin_id: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PluginEventType {
    Add,
    Edit,
    Remove,
}
