use std::path::PathBuf;

use async_trait::async_trait;

use super::{read_json_async, read_toml_async, write_json_async, write_toml_async};

#[async_trait]
pub trait AsyncFileDb<T> {
    async fn read(&self) -> crate::Result<Option<T>>;
    async fn ensure_read(&self, get_default: fn() -> T) -> crate::Result<T>;
    async fn write(&self, data: &T) -> crate::Result<()>;
}

pub struct AsyncJsonDb<T> {
    file: PathBuf,
    phantom: std::marker::PhantomData<T>,
}

impl<T> AsyncJsonDb<T> {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> AsyncFileDb<T> for AsyncJsonDb<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    async fn read(&self) -> crate::Result<Option<T>> {
        if !self.file.exists() {
            return Ok(None);
        }

        read_json_async(&self.file).await
    }

    async fn ensure_read(&self, get_default: fn() -> T) -> crate::Result<T> {
        let result = self.read().await?;

        match result {
            Some(result) => Ok(result),
            None => {
                let default = get_default();
                self.write(&default).await?;
                Ok(default)
            }
        }
    }

    async fn write(&self, data: &T) -> crate::Result<()> {
        write_json_async(&self.file, data).await
    }
}

pub struct AsyncTomlDb<T> {
    file: PathBuf,
    phantom: std::marker::PhantomData<T>,
}

impl<T> AsyncTomlDb<T> {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> AsyncFileDb<T> for AsyncTomlDb<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    async fn read(&self) -> crate::Result<Option<T>> {
        if !self.file.exists() {
            return Ok(None);
        }

        read_toml_async(&self.file).await
    }

    async fn ensure_read(&self, get_default: fn() -> T) -> crate::Result<T> {
        Ok(self.read().await?.unwrap_or_else(get_default))
    }

    async fn write(&self, data: &T) -> crate::Result<()> {
        write_toml_async(&self.file, data).await
    }
}
