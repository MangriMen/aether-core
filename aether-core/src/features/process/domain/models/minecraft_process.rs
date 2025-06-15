use std::process::ExitStatus;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::process::Child;
use uuid::Uuid;

#[derive(Debug)]
pub struct MinecraftProcess {
    pub metadata: MinecraftProcessMetadata,
    child: Child,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftProcessMetadata {
    pub uuid: Uuid,
    pub instance_id: String,
    pub start_time: DateTime<Utc>,
}

impl MinecraftProcess {
    pub fn from_child(instance_id: String, child: Child) -> Self {
        Self {
            metadata: MinecraftProcessMetadata::from_instance_id(instance_id),
            child,
        }
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, std::io::Error> {
        self.child.try_wait()
    }

    pub async fn wait(&mut self) -> Result<ExitStatus, std::io::Error> {
        self.child.wait().await
    }

    pub async fn kill(&mut self) -> Result<(), std::io::Error> {
        self.child.kill().await
    }
}

impl MinecraftProcessMetadata {
    pub fn from_instance_id(instance_id: String) -> Self {
        MinecraftProcessMetadata {
            uuid: Uuid::new_v4(),
            instance_id,
            start_time: Utc::now(),
        }
    }
}
