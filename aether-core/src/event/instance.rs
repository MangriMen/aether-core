#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InstancePayloadType {
    Created,
    Synced,
    Edited,
    Removed,
}

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstancePayload {
    pub instance_path_id: String,
    pub event: InstancePayloadType,
}
