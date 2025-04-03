use std::path::PathBuf;

use crate::state::LauncherState;

use super::JreProvider;

pub async fn install_jre(version: u32) -> crate::Result<PathBuf> {
    install_jre_with_provider(version, JreProvider::Azul).await
}

pub async fn install_jre_with_provider(
    version: u32,
    provider: JreProvider,
) -> crate::Result<PathBuf> {
    let state = LauncherState::get().await?;

    match provider {
        JreProvider::Azul => super::providers::azul::install_jre(&state, version).await,
    }
}
