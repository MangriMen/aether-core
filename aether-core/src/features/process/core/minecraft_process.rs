use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::process::Child;
use uuid::Uuid;

#[derive(Debug)]
pub struct MinecraftProcess {
    pub metadata: MinecraftProcessMetadata,
    pub child: Child,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftProcessMetadata {
    pub uuid: Uuid,
    pub id: String,
    pub start_time: DateTime<Utc>,
}
