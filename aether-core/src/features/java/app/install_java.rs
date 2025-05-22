use std::sync::Arc;

use crate::{
    features::{
        events::ProgressService,
        java::{infra::AzulJreProvider, Java, JavaInstallationService, JavaStorage},
        settings::LocationInfo,
    },
    shared::RequestClient,
};

use super::InstallJreUseCase;

pub struct InstallJavaUseCase<
    JS: JavaStorage,
    JIS: JavaInstallationService,
    PS: ProgressService,
    RC: RequestClient,
> {
    storage: Arc<JS>,
    java_installation_service: JIS,
    install_jre_use_case: Arc<InstallJreUseCase<AzulJreProvider<PS, RC>>>,
    location_info: Arc<LocationInfo>,
}

impl<JS: JavaStorage, JIS: JavaInstallationService, PS: ProgressService, RC: RequestClient>
    InstallJavaUseCase<JS, JIS, PS, RC>
{
    pub fn new(
        storage: Arc<JS>,
        java_installation_service: JIS,
        install_jre_use_case: Arc<InstallJreUseCase<AzulJreProvider<PS, RC>>>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            storage,
            java_installation_service,
            install_jre_use_case,
            location_info,
        }
    }

    pub async fn execute(&self, version: u32) -> crate::Result<Java> {
        let installed_jre_path = self
            .install_jre_use_case
            .execute(version, self.location_info.java_dir())
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
