use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    features::{
        instance::{Instance, InstanceStorage},
        settings::LocationInfo,
    },
    shared::{read_json_async, write_json_async, StorageError},
};

pub struct FsInstanceStorage {
    location_info: Arc<LocationInfo>,
}

impl FsInstanceStorage {
    pub fn new(location_info: Arc<LocationInfo>) -> Self {
        Self { location_info }
    }

    pub async fn read<T>(&self, path: &Path) -> Result<T, StorageError>
    where
        T: DeserializeOwned,
    {
        if !path.exists() {
            return Err(StorageError::NotFound {
                path: path.to_path_buf(),
            });
        }

        let value = read_json_async::<T>(path)
            .await
            .map_err(|err| StorageError::ReadError(err.raw.to_string()))?;

        Ok(value)
    }

    pub async fn write<T>(&self, path: &Path, value: &T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        write_json_async(path, value)
            .await
            .map_err(|err| StorageError::WriteError(err.raw.to_string()))
    }
}

#[async_trait]
impl InstanceStorage for FsInstanceStorage {
    async fn list(&self) -> Result<Vec<Instance>, StorageError> {
        let instances_dir = self.location_info.instances_dir();

        if !instances_dir.exists() {
            return Ok(Vec::new());
        }

        if let Ok(entries) = instances_dir.read_dir() {
            let mut instances = Vec::new();

            for entry in entries.flatten() {
                let instance_json_path = self
                    .location_info
                    .instance_metadata_file_with_instance_dir(&entry.path());
                let instance = self.read(&instance_json_path).await?;
                instances.push(instance);
            }

            return Ok(instances);
        };

        Ok(Vec::new())
    }

    async fn get(&self, id: &str) -> Result<Instance, StorageError> {
        self.read(&self.location_info.instance_metadata_file(id))
            .await
    }

    async fn upsert(&self, instance: &Instance) -> Result<(), StorageError> {
        let path = self.location_info.instance_metadata_file(&instance.id);
        self.write(&path, instance).await
    }

    async fn remove(&self, id: &str) -> Result<(), StorageError> {
        let path = self.location_info.instance_metadata_dir(id);
        tokio::fs::remove_dir_all(&path)
            .await
            .map_err(|_| StorageError::WriteError(path.to_string_lossy().to_string()))?;

        Ok(())
    }
}
