use std::{path::Path, sync::Arc};

use async_trait::async_trait;

use crate::{
    features::java::{Java, JavaError, JavaInstallationService, JavaStorage},
    shared::domain::AsyncUseCaseWithInputAndError,
};

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
}

#[async_trait]
impl<JS: JavaStorage, JIS: JavaInstallationService> AsyncUseCaseWithInputAndError
    for GetJavaUseCase<JS, JIS>
{
    type Input = u32;
    type Output = Java;
    type Error = crate::Error;

    async fn execute(&self, version: Self::Input) -> Result<Self::Output, Self::Error> {
        let java = self.storage.get(version).await?;

        let get_error = || JavaError::NotFound { version };

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
