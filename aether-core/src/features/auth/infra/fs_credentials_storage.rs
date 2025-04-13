use std::path::{Path, PathBuf};

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    core::LauncherState,
    features::auth::{Credentials, CredentialsStorage},
    shared::{read_json_async, write_json_async},
    ErrorKind,
};

pub struct FsCredentialsStorage;

impl FsCredentialsStorage {
    fn get_credentials_file_path(state: &LauncherState) -> PathBuf {
        state.locations.settings_dir.join("credentials.json")
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

    async fn read_file_contents(state: &LauncherState) -> crate::Result<Vec<Credentials>> {
        let path = Self::get_credentials_file_path(state);
        Self::ensure_credentials_file_exists(&path).await?;

        let credentials = read_json_async::<Vec<Credentials>>(&path).await?;

        Ok(credentials)
    }

    async fn write_file_contents(
        state: &LauncherState,
        credentials: Vec<Credentials>,
    ) -> crate::Result<()> {
        let path = Self::get_credentials_file_path(state);
        write_json_async(&path, credentials).await
    }

    async fn update_active(
        credentials_list: &mut Vec<Credentials>,
        id: &Uuid,
    ) -> crate::Result<()> {
        if credentials_list.is_empty() {
            return Err(ErrorKind::NoCredentialsError.as_error());
        }

        let mut prev_active = None;
        let mut new_active = None;

        for credential in credentials_list {
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
    async fn get(&self, state: &LauncherState, id: &Uuid) -> crate::Result<Credentials> {
        let credentials = Self::read_file_contents(state).await?;

        let found = credentials.iter().find(|x| x.id == *id);

        if let Some(credentials) = found {
            Ok(credentials.clone())
        } else {
            Err(ErrorKind::NoCredentialsError.as_error())
        }
    }

    async fn get_active(&self, state: &LauncherState) -> crate::Result<Option<Credentials>> {
        let credentials = Self::read_file_contents(state).await?;
        Ok(credentials.iter().find(|x| x.active).cloned())
    }

    async fn get_all(&self, state: &LauncherState) -> crate::Result<Vec<Credentials>> {
        Self::read_file_contents(state).await
    }

    async fn upsert(
        &self,
        state: &LauncherState,
        credentials: &Credentials,
    ) -> crate::Result<Uuid> {
        let mut credentials_list = Self::read_file_contents(state).await?;
        let index = credentials_list
            .iter()
            .position(|x| x.id == credentials.id)
            .ok_or_else(|| ErrorKind::NoCredentialsError.as_error())?;

        if credentials.active {
            credentials_list[index] = Credentials {
                active: false,
                ..credentials.clone()
            };
            Self::update_active(&mut credentials_list, &credentials.id).await?
        } else {
            credentials_list[index] = credentials.clone();
        }

        Self::write_file_contents(state, credentials_list).await?;

        Ok(credentials.id)
    }

    async fn set_active(&self, state: &LauncherState, id: &Uuid) -> crate::Result<()> {
        let mut credentials_list = Self::read_file_contents(state).await?;

        Self::update_active(&mut credentials_list, id).await?;

        Self::write_file_contents(state, credentials_list).await
    }

    async fn upsert_all(
        &self,
        state: &LauncherState,
        credentials_list: Vec<Credentials>,
    ) -> crate::Result<()> {
        Self::write_file_contents(state, credentials_list).await
    }

    async fn remove(&self, state: &LauncherState, id: &Uuid) -> crate::Result<()> {
        let mut credentials_list = Self::read_file_contents(state).await?;

        let mut need_to_set_active = false;
        credentials_list.retain(|x| {
            let need_retain = x.id != *id;

            if !need_retain && x.active {
                need_to_set_active = true
            }

            need_retain
        });

        if need_to_set_active {
            if let Some(first) = credentials_list.first_mut() {
                first.active = true;
            };
        }

        Self::write_file_contents(state, credentials_list).await
    }
}
