use std::path::{Path, PathBuf};

use crate::{features::java::infra::azul_provider, shared::FetchSemaphore};

pub enum JreProvider {
    Azul,
}

pub async fn install_jre(
    version: u32,
    java_dir: &Path,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<PathBuf> {
    install_jre_with_provider(version, JreProvider::Azul, java_dir, fetch_semaphore).await
}

pub async fn install_jre_with_provider(
    version: u32,
    provider: JreProvider,
    java_dir: &Path,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<PathBuf> {
    match provider {
        JreProvider::Azul => azul_provider::install_jre(version, java_dir, fetch_semaphore).await,
    }
}
