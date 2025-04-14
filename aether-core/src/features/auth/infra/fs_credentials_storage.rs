use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::auth::{Credentials, CredentialsStorage},
    shared::{read_json_async, write_json_async},
    ErrorKind,
};

pub struct FsCredentialsStorage {
    state: Arc<LauncherState>,
}

impl FsCredentialsStorage {
    pub fn new(state: Arc<LauncherState>) -> Self {
        Self { state }
    }

    fn get_credentials_file_path(&self) -> PathBuf {
        self.state.locations.settings_dir.join("credentials.json")
    }

    async fn ensure_credentials_file_exists(path: &Path) -> crate::Result<()> {
        if !path.exists() {
            log::info!(
                "Credentials file not found, creating new one at {}",
                path.display()
            );
            write_json_async(&path, Vec::<Credentials>::default()).await?
        }
        Ok(())
    }

    async fn read_file_contents(&self) -> crate::Result<Vec<Credentials>> {
        let path = self.get_credentials_file_path();
        Self::ensure_credentials_file_exists(&path).await?;
        read_json_async(&path).await
    }

    async fn write_file_contents(&self, credentials: Vec<Credentials>) -> crate::Result<()> {
        let path = self.get_credentials_file_path();
        write_json_async(&path, credentials).await
    }

    async fn update_active(credentials_list: &mut [Credentials], id: &Uuid) -> crate::Result<()> {
        if credentials_list.is_empty() {
            return Err(ErrorKind::NoCredentialsError.as_error());
        }

        let mut prev_active = None;
        let mut new_active = None;

        for credential in credentials_list.iter_mut() {
            if credential.active {
                prev_active = Some(credential);
            } else if credential.id == *id {
                new_active = Some(credential);
            }
        }

        if let Some(new_active) = new_active {
            if let Some(prev_active) = prev_active {
                prev_active.active = false;
            }
            new_active.active = true;
            Ok(())
        } else {
            Err(ErrorKind::NoCredentialsError.as_error())
        }
    }
}

#[async_trait]
impl CredentialsStorage for FsCredentialsStorage {
    async fn get(&self, id: &Uuid) -> crate::Result<Credentials> {
        self.read_file_contents()
            .await?
            .iter()
            .find(|x| x.id == *id)
            .cloned()
            .ok_or_else(|| ErrorKind::NoCredentialsError.as_error())
    }

    async fn get_active(&self) -> crate::Result<Option<Credentials>> {
        Ok(self
            .read_file_contents()
            .await?
            .iter()
            .find(|x| x.active)
            .cloned())
    }

    async fn get_all(&self) -> crate::Result<Vec<Credentials>> {
        self.read_file_contents().await
    }

    async fn upsert(&self, credentials: &Credentials) -> crate::Result<Uuid> {
        let mut credentials_list = self.read_file_contents().await?;
        let index = credentials_list.iter().position(|x| x.id == credentials.id);

        if let Some(index) = index {
            if credentials.active {
                credentials_list[index] = Credentials {
                    active: false,
                    ..credentials.clone()
                };
                Self::update_active(&mut credentials_list, &credentials.id).await?
            } else {
                credentials_list[index] = credentials.clone();
            }
        } else {
            credentials_list.push(credentials.clone());
        }

        self.write_file_contents(credentials_list).await?;
        Ok(credentials.id)
    }

    async fn set_active(&self, id: &Uuid) -> crate::Result<()> {
        let mut credentials_list = self.read_file_contents().await?;
        Self::update_active(&mut credentials_list, id).await?;
        self.write_file_contents(credentials_list).await
    }

    async fn upsert_all(&self, credentials_list: Vec<Credentials>) -> crate::Result<()> {
        self.write_file_contents(credentials_list).await
    }

    async fn remove(&self, id: &Uuid) -> crate::Result<()> {
        let mut credentials_list = self.read_file_contents().await?;

        let mut need_to_set_active = false;
        credentials_list.retain(|x| {
            if x.id == *id && x.active {
                need_to_set_active = true;
                return false;
            }

            true
        });

        if need_to_set_active {
            if let Some(first) = credentials_list.first_mut() {
                first.active = true;
            };
        }

        self.write_file_contents(credentials_list).await
    }
}
