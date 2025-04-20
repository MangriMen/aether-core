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
    async fn get(&self, id: &str) -> Result<Instance, StorageError> {
        self.read(&self.location_info.instance_metadata_file(id))
            .await
    }
}
