use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::process::Child;
use uuid::Uuid;

#[derive(Debug)]
pub struct MinecraftProcess {
    pub metadata: MinecraftProcessMetadata,
    pub child: Child,
}

impl MinecraftProcess {
    pub fn from_child(instance_id: &str, child: Child) -> Self {
        Self {
            metadata: MinecraftProcessMetadata {
                uuid: Uuid::new_v4(),
                start_time: Utc::now(),
                id: instance_id.to_string(),
            },
            child,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftProcessMetadata {
    pub uuid: Uuid,
    pub id: String,
    pub start_time: DateTime<Utc>,
}
