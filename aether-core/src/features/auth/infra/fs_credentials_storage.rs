use std::path::{Path, PathBuf};

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    features::auth::{AuthError, Credentials, CredentialsStorage},
    shared::{ensure_read_json_async, write_json_async, StorageError},
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
        Ok(ensure_read_json_async(&self.credentials_file)
            .await
            .map_err(|e| StorageError::ReadError(e.to_string()))?)
    }

    async fn write(&self, data: &Vec<Credentials>) -> Result<(), AuthError> {
        Ok(write_json_async(&self.credentials_file, data)
            .await
            .map_err(|e| StorageError::ReadError(e.to_string()))?)
    }

    async fn update_active(
        credentials_list: &mut [Credentials],
        id: Uuid,
    ) -> Result<(), AuthError> {
        if credentials_list.is_empty() {
            return Err(AuthError::CredentialsNotFound { id });
        }

        let mut new_active_found = false;
        for credential in credentials_list.iter_mut() {
            if credential.id == id {
                credential.active = true;
                new_active_found = true;
            } else if credential.active {
                credential.active = false;
            }
        }

        if !new_active_found {
            return Err(AuthError::CredentialsNotFound { id });
        }

        Ok(())
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

    async fn upsert(&self, credentials: &Credentials) -> Result<Uuid, AuthError> {
        let mut credentials_list = self.ensure_read().await?;
        let index = credentials_list.iter().position(|x| x.id == credentials.id);

        if let Some(index) = index {
            if credentials.active {
                credentials_list[index] = Credentials {
                    active: false,
                    ..credentials.clone()
                };
            } else {
                credentials_list[index] = credentials.clone();
            }
        } else {
            credentials_list.push(credentials.clone());
        }

        Self::update_active(&mut credentials_list, credentials.id).await?;
        self.write(&credentials_list).await?;

        Ok(credentials.id)
    }

    async fn remove(&self, id: Uuid) -> Result<(), AuthError> {
        let mut credentials_list = self.ensure_read().await?;

        let mut need_to_set_active = false;
        credentials_list.retain(|x| {
            if x.id == id {
                if x.active {
                    need_to_set_active = true;
                }
                return false;
            }

            true
        });

        if need_to_set_active {
            if let Some(first) = credentials_list.first_mut() {
                first.active = true;
            };
        }

        self.write(&credentials_list).await
    }

    async fn get_active(&self) -> Result<Credentials, AuthError> {
        self.ensure_read()
            .await?
            .iter()
            .find(|x| x.active)
            .cloned()
            .ok_or(AuthError::ActiveCredentialsNotFound)
    }

    async fn set_active(&self, id: Uuid) -> Result<(), AuthError> {
        let mut credentials_list = self.ensure_read().await?;
        Self::update_active(&mut credentials_list, id).await?;
        self.write(&credentials_list).await
    }
}
