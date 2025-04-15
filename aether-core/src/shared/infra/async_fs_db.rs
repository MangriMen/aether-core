use std::path::PathBuf;

use super::{read_json_async, write_json_async};

pub struct AsyncFsDb<T> {
    file: PathBuf,
    phantom: std::marker::PhantomData<T>,
}

impl<T> AsyncFsDb<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Default,
{
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            phantom: std::marker::PhantomData,
        }
    }

    async fn ensure_file_exists(&self) -> crate::Result<()> {
        if !self.file.exists() {
            log::info!(
                "Credentials file not found, creating new one at {}",
                self.file.display()
            );
            self.write_file_contents(T::default()).await?
        }
        Ok(())
    }

    pub async fn read_file_contents(&self) -> crate::Result<T> {
        self.ensure_file_exists().await?;
        read_json_async(&self.file).await
    }

    pub async fn write_file_contents(&self, data: T) -> crate::Result<()> {
        write_json_async(&self.file, data).await
    }
}
