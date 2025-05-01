use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        java::{infra::AzulJreProvider, Java, JavaInstallationService, JavaStorage},
        settings::LocationInfo,
    },
    shared::{domain::AsyncUseCaseWithInputAndError, infra::ReqwestClient},
};

use super::InstallJreUseCase;

pub struct InstallJavaUseCase<JS: JavaStorage, JIS: JavaInstallationService> {
    storage: Arc<JS>,
    java_installation_service: JIS,
    install_jre_use_case: Arc<InstallJreUseCase<AzulJreProvider<ReqwestClient>>>,
    location_info: Arc<LocationInfo>,
}

impl<JS: JavaStorage, JIS: JavaInstallationService> InstallJavaUseCase<JS, JIS> {
    pub fn new(
        storage: Arc<JS>,
        java_installation_service: JIS,
        install_jre_use_case: Arc<InstallJreUseCase<AzulJreProvider<ReqwestClient>>>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            storage,
            java_installation_service,
            install_jre_use_case,
            location_info,
        }
    }
}

#[async_trait]
impl<JS, JIS> AsyncUseCaseWithInputAndError for InstallJavaUseCase<JS, JIS>
where
    JS: JavaStorage + Send + Sync,
    JIS: JavaInstallationService + Send + Sync,
{
    type Input = u32;
    type Output = Java;
    type Error = crate::Error;

    async fn execute(&self, version: Self::Input) -> Result<Self::Output, Self::Error> {
        let installed_jre_path = self
            .install_jre_use_case
            .execute((version, self.location_info.java_dir()))
            .await?;

        let java = self
            .java_installation_service
            .locate_java(&installed_jre_path)
            .await
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!("Java {} not found", version)).as_error()
            })?;

        self.storage.upsert(&java).await?;

        Ok(java)
    }
}
