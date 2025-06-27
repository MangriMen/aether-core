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

    pub async fn execute(&self, version: u32) -> Result<Java, JavaError> {
        let java = self
            .storage
            .get(version)
            .await?
            .ok_or(JavaError::JavaNotFound { version })?;

        self.java_installation_service
            .locate_java(Path::new(&java.path))
            .await
    }
}
