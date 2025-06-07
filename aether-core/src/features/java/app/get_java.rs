use std::{path::Path, sync::Arc};

use crate::features::java::{Java, JavaError, JavaInstallationService, JavaStorage};

pub struct GetJavaUseCase<JS: JavaStorage, JIS: JavaInstallationService> {
    storage: Arc<JS>,
    java_installation_service: JIS,
}

impl<JS: JavaStorage, JIS: JavaInstallationService> GetJavaUseCase<JS, JIS> {
    pub fn new(storage: Arc<JS>, java_installation_service: JIS) -> Self {
        Self {
            storage,
            java_installation_service,
        }
    }

    pub async fn execute(&self, version: u32) -> crate::Result<Java> {
        let java = self.storage.get(version).await?;

        let get_error = || JavaError::JreNotFound { version };

        if let Some(java) = java {
            Ok(self
                .java_installation_service
                .locate_java(Path::new(&java.path))
                .await
                .ok_or_else(get_error)?)
        } else {
            Err(get_error().into())
        }
    }
}
