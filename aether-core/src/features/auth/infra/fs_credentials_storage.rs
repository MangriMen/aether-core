use std::path::{Path, PathBuf};

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    features::auth::{AuthError, Credentials, CredentialsStorage},
    shared::{ensure_read_json_async, write_json_async},
};

pub struct FsCredentialsStorage {
    credentials_file: PathBuf,
}

impl FsCredentialsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        Self {
            credentials_file: settings_dir.join("credentials.json"),
        }
    }

    async fn ensure_read(&self) -> Result<Vec<Credentials>, AuthError> {
        Ok(ensure_read_json_async(&self.credentials_file).await?)
    }

    async fn write(&self, data: &[Credentials]) -> Result<(), AuthError> {
        Ok(write_json_async(&self.credentials_file, data).await?)
    }
}

#[async_trait]
impl CredentialsStorage for FsCredentialsStorage {
    async fn list(&self) -> Result<Vec<Credentials>, AuthError> {
        self.ensure_read().await
    }

    async fn get(&self, id: Uuid) -> Result<Credentials, AuthError> {
        self.ensure_read()
            .await?
            .iter()
            .find(|x| x.id == id)
            .cloned()
            .ok_or(AuthError::CredentialsNotFound { id })
    }

    async fn upsert(&self, credentials: Credentials) -> Result<Credentials, AuthError> {
        let mut credentials_list = self.ensure_read().await?;

        if let Some(index) = credentials_list.iter().position(|c| c.id == credentials.id) {
            credentials_list[index] = credentials.clone();
        } else {
            credentials_list.push(credentials.clone());
        }

        self.write(&credentials_list).await?;
        Ok(credentials)
    }

    async fn remove(&self, id: Uuid) -> Result<(), AuthError> {
        let mut credentials_list = self.ensure_read().await?;
        let original_len = credentials_list.len();

        credentials_list.retain(|c| c.id != id);

        if credentials_list.len() == original_len {
            return Err(AuthError::CredentialsNotFound { id });
        }

        self.write(&credentials_list).await
    }

    async fn get_active(&self) -> Result<Credentials, AuthError> {
        self.ensure_read()
            .await?
            .iter()
            .find(|x| x.active)
            .cloned()
            .ok_or(AuthError::NoActiveCredentials)
    }

    async fn set_active(&self, id: Uuid) -> Result<Credentials, AuthError> {
        let mut credentials_list = self.ensure_read().await?;

        if let Some(cred) = credentials_list.iter_mut().find(|c| c.id == id) {
            cred.active = true;
            let cloned = cred.clone();
            self.write(&credentials_list).await?;
            Ok(cloned)
        } else {
            Err(AuthError::CredentialsNotFound { id })
        }
    }

    async fn deactivate_all(&self) -> Result<(), AuthError> {
        let mut credentials_list = self.ensure_read().await?;

        for cred in credentials_list.iter_mut() {
            cred.active = false;
        }

        self.write(&credentials_list).await
    }
}
