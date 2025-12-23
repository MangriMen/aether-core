use crate::features::auth::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Default)]
pub struct MockCredentialsStorage {
    store: Arc<Mutex<HashMap<Uuid, Credentials>>>,
}

impl MockCredentialsStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CredentialsStorage for MockCredentialsStorage {
    async fn list(&self) -> Result<Vec<Credentials>, AuthApplicationError> {
        Ok(self.store.lock().unwrap().values().cloned().collect())
    }

    async fn get(&self, id: Uuid) -> Result<Credentials, AuthApplicationError> {
        self.store
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or(AuthApplicationError::Domain(
                AuthDomainError::CredentialsNotFound { id },
            ))
    }

    async fn upsert(&self, credentials: Credentials) -> Result<Credentials, AuthApplicationError> {
        self.store
            .lock()
            .unwrap()
            .insert(credentials.id, credentials.clone());
        Ok(credentials)
    }

    async fn remove(&self, id: Uuid) -> Result<(), AuthApplicationError> {
        let mut store = self.store.lock().unwrap();
        if store.remove(&id).is_none() {
            return Err(AuthApplicationError::Domain(
                AuthDomainError::CredentialsNotFound { id },
            ));
        }
        Ok(())
    }

    async fn get_first(&self) -> Result<Credentials, AuthApplicationError> {
        self.store
            .lock()
            .unwrap()
            .values()
            .next()
            .cloned()
            .ok_or(AuthApplicationError::Domain(
                AuthDomainError::NoActiveCredentials,
            ))
    }

    async fn get_active(&self) -> Result<Credentials, AuthApplicationError> {
        self.store
            .lock()
            .unwrap()
            .values()
            .find(|c| c.active)
            .cloned()
            .ok_or(AuthApplicationError::Domain(
                AuthDomainError::NoActiveCredentials,
            ))
    }

    async fn set_active(&self, id: Uuid) -> Result<Credentials, AuthApplicationError> {
        let mut store = self.store.lock().unwrap();
        for c in store.values_mut() {
            if c.id == id {
                c.active = true;
                return Ok(c.clone());
            }
        }
        Err(AuthApplicationError::Domain(
            AuthDomainError::CredentialsNotFound { id },
        ))
    }

    async fn deactivate_all(&self) -> Result<(), AuthApplicationError> {
        for c in self.store.lock().unwrap().values_mut() {
            c.active = false;
        }
        Ok(())
    }
}
