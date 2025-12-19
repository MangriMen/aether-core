use std::sync::Arc;

use crate::features::{
    java::{Java, JavaInstallationService, JavaStorage, JreProvider},
    settings::LocationInfo,
};

use super::InstallJreUseCase;
use super::JavaApplicationError;

pub struct InstallJavaUseCase<JS: JavaStorage, JIS: JavaInstallationService, JP: JreProvider> {
    storage: Arc<JS>,
    java_installation_service: JIS,
    install_jre_use_case: Arc<InstallJreUseCase<JP>>,
    location_info: Arc<LocationInfo>,
}

impl<JS: JavaStorage, JIS: JavaInstallationService, JP: JreProvider>
    InstallJavaUseCase<JS, JIS, JP>
{
    pub fn new(
        storage: Arc<JS>,
        java_installation_service: JIS,
        install_jre_use_case: Arc<InstallJreUseCase<JP>>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            storage,
            java_installation_service,
            install_jre_use_case,
            location_info,
        }
    }

    pub async fn execute(&self, version: u32) -> Result<Java, JavaApplicationError> {
        let installed_jre_path = self
            .install_jre_use_case
            .execute(version, self.location_info.java_dir())
            .await?;

        let java = self
            .java_installation_service
            .locate_java(&installed_jre_path)
            .await?;

        self.storage.upsert(&java).await?;

        Ok(java)
    }
}
