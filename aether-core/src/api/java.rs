use std::sync::Arc;

use crate::{
    core::{domain::LazyLocator, LauncherState},
    features::java::{
        self,
        infra::{AzulJreProvider, FsJavaInstallationService},
        GetJavaUseCase, InstallJavaUseCase, InstallJreUseCase,
    },
};

#[tracing::instrument]
pub async fn install(version: u32) -> crate::Result<java::Java> {
    let state = LauncherState::get().await?;
    let lazy_locator = LazyLocator::get().await?;

    let jre_provider = Arc::new(AzulJreProvider::new(
        lazy_locator.get_progress_service().await,
        lazy_locator.get_request_client().await,
    ));

    let install_jre_use_case = Arc::new(InstallJreUseCase::new(jre_provider));

    let install_java_use_case = InstallJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
        install_jre_use_case,
        state.location_info.clone(),
    );

    Ok(install_java_use_case.execute(version).await?)
}

#[tracing::instrument]
pub async fn get(version: u32) -> crate::Result<java::Java> {
    let lazy_locator = LazyLocator::get().await?;

    let get_java_use_case = GetJavaUseCase::new(
        lazy_locator.get_java_storage().await,
        FsJavaInstallationService,
    );

    Ok(get_java_use_case.execute(version).await?)
}
