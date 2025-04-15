use std::path::Path;

use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    features::auth::{Credentials, CredentialsStorage},
    shared::infra::{AsyncFileDb, AsyncJsonDb},
    ErrorKind,
};

pub struct FsCredentialsStorage {
    db: AsyncJsonDb<Vec<Credentials>>,
}

impl FsCredentialsStorage {
    pub fn new(settings_dir: &Path) -> Self {
        Self {
            db: AsyncJsonDb::new(settings_dir.join("credentials.json")),
        }
    }

    #[inline]
    fn get_default() -> Vec<Credentials> {
        Vec::default()
    }

    async fn update_active(credentials_list: &mut [Credentials], id: &Uuid) -> crate::Result<()> {
        if credentials_list.is_empty() {
            return Err(ErrorKind::NoCredentialsError.as_error());
        }

        let mut new_active_found = false;
        for credential in credentials_list.iter_mut() {
            if credential.id == *id {
                credential.active = true;
                new_active_found = true;
            } else if credential.active {
                credential.active = false;
            }
        }

        if !new_active_found {
            return Err(ErrorKind::NoCredentialsError.as_error());
        }

        Ok(())
    }
}

#[async_trait]
impl CredentialsStorage for FsCredentialsStorage {
    async fn list(&self) -> crate::Result<Vec<Credentials>> {
        self.db.ensure_read(Self::get_default).await
    }

    async fn get(&self, id: &Uuid) -> crate::Result<Credentials> {
        self.db
            .ensure_read(Self::get_default)
            .await?
            .iter()
            .find(|x| x.id == *id)
            .cloned()
            .ok_or_else(|| ErrorKind::NoCredentialsError.as_error())
    }

    async fn upsert(&self, credentials: &Credentials) -> crate::Result<Uuid> {
        let mut credentials_list = self.db.ensure_read(Self::get_default).await?;
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

        self.db.write(&credentials_list).await?;
        Ok(credentials.id)
    }

    async fn remove(&self, id: &Uuid) -> crate::Result<()> {
        let mut credentials_list = self.db.ensure_read(Self::get_default).await?;

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

        self.db.write(&credentials_list).await
    }

    async fn get_active(&self) -> crate::Result<Option<Credentials>> {
        Ok(self
            .db
            .ensure_read(Self::get_default)
            .await?
            .iter()
            .find(|x| x.active)
            .cloned())
    }

    async fn set_active(&self, id: &Uuid) -> crate::Result<()> {
        let mut credentials_list = self.db.ensure_read(Self::get_default).await?;
        Self::update_active(&mut credentials_list, id).await?;
        self.db.write(&credentials_list).await
    }
}
